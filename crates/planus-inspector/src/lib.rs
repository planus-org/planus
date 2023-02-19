pub mod ui;
pub mod vec_with_index;

use std::{io, time::Duration};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use planus_buffer_inspection::{
    object_info::ObjectName,
    object_mapping::{Interpretation, Line, ObjectIndex, ObjectMapping},
    InspectableFlatbuffer, Object,
};
use planus_types::intermediate::DeclarationIndex;
use tui::{backend::Backend, layout::Rect, Terminal};

use crate::{
    ui::{FoldState, Node, TreeState, TreeStateLine},
    vec_with_index::VecWithIndex,
};

#[derive(Copy, Clone, Debug)]
pub enum ActiveWindow {
    ObjectView,
    HexView,
}

#[derive(Debug)]
pub enum ModalState<'a> {
    GoToByte {
        input: String,
    },
    XRefs {
        xrefs: VecWithIndex<String>,
    },
    ViewHistory {
        index: usize,
    },
    HelpMenu,
    TreeView {
        header: &'static str,
        state: TreeState<'a>,
    },
}

pub struct Inspector<'a> {
    pub object_mapping: ObjectMapping<'a>,
    pub buffer: InspectableFlatbuffer<'a>,
    pub should_quit: bool,
    pub view_stack: Vec<ViewState<'a>>,
    pub active_window: ActiveWindow,
    pub view_state: ViewState<'a>,
    pub hex_view_state: HexViewState,
    pub modal: Option<ModalState<'a>>,
}

pub struct HexViewState {
    pub line_size: usize,
    pub line_pos: usize,
    pub max_offset_hex_digits: usize,
}

impl HexViewState {
    pub fn new(buffer_size: usize) -> Self {
        let max_offset = buffer_size.max(1) as u64;
        let max_offset_ilog2 = 63 - max_offset.leading_zeros();
        let max_offset_hex_digits = max_offset_ilog2 / 4 + 1;
        let max_offset_hex_digits = (max_offset_hex_digits + (max_offset_hex_digits & 1)) as usize;
        Self {
            line_size: 0,
            line_pos: 0,
            max_offset_hex_digits,
        }
    }

    pub fn update_for_area(&mut self, buffer_len: usize, byte_index: usize, area: Rect) {
        let remaining_width = area.width as usize - self.max_offset_hex_digits - 2;
        let eight_groups = (remaining_width + 1) / 25;
        self.line_size = eight_groups * 8;

        if area.height != 0 && self.line_size != 0 {
            let cursor_line = byte_index / self.line_size;
            let mut first_line = self.line_pos / self.line_size;
            let max_line_index = (buffer_len - 1) / self.line_size;

            if area.height <= 4 {
                first_line = first_line.clamp(
                    (cursor_line + 1).saturating_sub(area.height as usize),
                    cursor_line,
                );
            } else {
                first_line = first_line.clamp(
                    (cursor_line + 2).saturating_sub(area.height as usize),
                    cursor_line.saturating_sub(1),
                );
            }
            first_line = first_line.min((max_line_index + 1).saturating_sub(area.height as usize));
            self.line_pos = first_line * self.line_size;
        }
    }
}

pub struct Ranges {
    pub inner_range: Option<std::ops::Range<usize>>,
    pub outer_range: Option<std::ops::Range<usize>>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum RangeMatch {
    Inner,
    Outer,
}

impl Ranges {
    pub fn best_match(&self, value: usize) -> Option<RangeMatch> {
        if let Some(inner_range) = &self.inner_range {
            if inner_range.contains(&value) {
                return Some(RangeMatch::Inner);
            }
        }

        if let Some(outer_range) = &self.outer_range {
            if outer_range.contains(&value) {
                return Some(RangeMatch::Outer);
            }
        }

        None
    }
}

#[derive(Clone, Debug)]
pub struct ViewState<'a> {
    pub byte_index: usize,
    pub info_view_data: Option<InfoViewData<'a>>,
}

#[derive(Clone, Debug)]
pub struct InfoViewData<'a> {
    pub first_line_shown: usize,
    pub lines: VecWithIndex<Line<'a>>,
    pub interpretations: VecWithIndex<Interpretation>,
    pub root_object_index: ObjectIndex,
}

impl<'a> InfoViewData<'a> {
    fn new_from_root_object(
        object_mapping: &ObjectMapping<'a>,
        buffer: &InspectableFlatbuffer<'a>,
        object: Object<'a>,
    ) -> Self {
        let root_object_index = object_mapping
            .root_objects
            .get_index_of(&object)
            .unwrap_or_else(|| panic!("{object:?}, {:?}", object_mapping.root_objects));
        let mut interpretations = Vec::new();
        let mut interpretation_index = None;
        object_mapping.get_interpretations_cb(object.offset(), buffer, |interpretation| {
            if interpretation.root_object_index == root_object_index {
                interpretation_index.get_or_insert(interpretations.len());
            }
            interpretations.push(interpretation);
        });
        Self {
            first_line_shown: 0,
            lines: VecWithIndex::new(
                object_mapping
                    .line_tree(root_object_index, buffer)
                    .flatten(buffer),
                0,
            ),
            root_object_index,
            interpretations: VecWithIndex::new(interpretations, interpretation_index.unwrap()),
        }
    }

    fn new_from_byte_index<Score: Ord>(
        object_mapping: &ObjectMapping<'a>,
        buffer: &InspectableFlatbuffer<'a>,
        byte_index: usize,
        interpretation_score: impl Fn(&Interpretation) -> Score,
    ) -> Option<Self> {
        let interpretations = object_mapping.get_interpretations(byte_index as u32, buffer);
        if interpretations.is_empty() {
            return None;
        }

        let (interpretation_index, interpretation) = interpretations
            .iter()
            .enumerate()
            .min_by_key(|(i, interpretation)| (interpretation_score(interpretation), *i))
            .unwrap();
        let root_object_index = interpretation.root_object_index;
        let line_index = *interpretation.lines.last().unwrap();
        Some(Self {
            first_line_shown: 0,
            lines: VecWithIndex::new(
                object_mapping
                    .line_tree(root_object_index, buffer)
                    .flatten(buffer),
                line_index,
            ),
            interpretations: VecWithIndex::new(interpretations, interpretation_index),
            root_object_index,
        })
    }

    fn set_interpretation_index(
        &mut self,
        object_mapping: &ObjectMapping<'a>,
        buffer: &InspectableFlatbuffer<'a>,
        interpretation_index: usize,
    ) {
        if self.interpretations.try_set_index(interpretation_index) {
            self.root_object_index = self.interpretations.cur().root_object_index;
            self.lines = VecWithIndex::new(
                object_mapping
                    .line_tree(self.root_object_index, buffer)
                    .flatten(buffer),
                *self.interpretations.cur().lines.last().unwrap(),
            );
        }
    }

    /// If anything was changed, then the new byte index is returned
    fn set_line_pos(
        &mut self,
        object_mapping: &ObjectMapping<'a>,
        buffer: &InspectableFlatbuffer<'a>,
        line_index: usize,
    ) -> Option<usize> {
        if self.lines.try_set_index(line_index) {
            let byte_index = self.lines.cur().start as usize;
            let start_line_index = self.lines.cur().start_line_index;
            let interpretations = object_mapping.get_interpretations(byte_index as u32, buffer);
            let interpretation_index = interpretations
                .iter()
                .position(|interpretation| {
                    interpretation.root_object_index == self.root_object_index
                        && interpretation.lines.contains(&start_line_index)
                })
                .unwrap();
            self.interpretations = VecWithIndex::new(interpretations, interpretation_index);
            Some(byte_index)
        } else {
            None
        }
    }
}

impl<'a> ViewState<'a> {
    fn new_for_object(
        object_mapping: &ObjectMapping<'a>,
        buffer: &InspectableFlatbuffer<'a>,
        object: Object<'a>,
    ) -> Self {
        Self {
            byte_index: object.offset() as usize,
            info_view_data: Some(InfoViewData::new_from_root_object(
                object_mapping,
                buffer,
                object,
            )),
        }
    }

    fn set_byte_pos(
        &mut self,
        object_mapping: &ObjectMapping<'a>,
        buffer: &InspectableFlatbuffer<'a>,
        byte_index: usize,
    ) {
        if self.byte_index != byte_index {
            self.byte_index = byte_index;

            let current_root_object_index =
                self.info_view_data.as_ref().map(|d| d.root_object_index);
            let current_line_index = self.info_view_data.as_ref().map_or(0, |d| d.lines.index());
            self.info_view_data = InfoViewData::new_from_byte_index(
                object_mapping,
                buffer,
                byte_index,
                |interpretation| {
                    if Some(interpretation.root_object_index) == current_root_object_index {
                        (
                            0,
                            interpretation
                                .lines
                                .iter()
                                .map(|line| line.abs_diff(current_line_index))
                                .min()
                                .unwrap_or(usize::MAX),
                        )
                    } else {
                        (1, 0)
                    }
                },
            );
        }
    }

    pub fn set_line_pos(
        &mut self,
        object_mapping: &ObjectMapping<'a>,
        buffer: &InspectableFlatbuffer<'a>,
        line_index: usize,
    ) {
        if let Some(info_view_date) = &mut self.info_view_data {
            if let Some(byte_index) =
                info_view_date.set_line_pos(object_mapping, buffer, line_index)
            {
                self.byte_index = byte_index;
            }
        }
    }

    pub fn hex_ranges(&self) -> Ranges {
        Ranges {
            inner_range: self
                .info_view_data
                .as_ref()
                .map(|d| d.lines.cur().start as usize..d.lines.cur().end as usize),
            outer_range: self.info_view_data.as_ref().and_then(|d| {
                d.lines
                    .first()
                    .map(|line| line.start as usize..line.end as usize)
            }),
        }
    }
}

impl<'a> Inspector<'a> {
    pub fn new(buffer: InspectableFlatbuffer<'a>, root_table_index: DeclarationIndex) -> Self {
        let object_mapping = buffer.calculate_object_mapping(root_table_index);
        Self {
            buffer,
            view_state: ViewState::new_for_object(
                &object_mapping,
                &buffer,
                object_mapping.root_object,
            ),
            object_mapping,
            should_quit: false,
            view_stack: Vec::new(),
            active_window: ActiveWindow::ObjectView,
            hex_view_state: HexViewState::new(buffer.buffer.len()),
            modal: None,
        }
    }

    pub fn on_key(&mut self, key: KeyEvent) -> bool {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let mut should_draw = match key.code {
            KeyCode::Tab => match self.active_window {
                ActiveWindow::HexView => {
                    if self.view_state.info_view_data.is_some() {
                        self.active_window = ActiveWindow::ObjectView;
                        true
                    } else {
                        false
                    }
                }
                ActiveWindow::ObjectView => {
                    self.active_window = ActiveWindow::HexView;
                    self.view_state.set_byte_pos(
                        &self.object_mapping,
                        &self.buffer,
                        self.view_state.byte_index,
                    );
                    true
                }
            },
            KeyCode::Char('g') => {
                self.toggle_modal(ModalState::GoToByte {
                    input: String::new(),
                });
                true
            }
            KeyCode::Char('x') => {
                self.toggle_modal(ModalState::XRefs {
                    xrefs: VecWithIndex::new(vec!["lol".to_owned()], 0),
                });
                true
            }
            KeyCode::Char('h') => {
                self.toggle_modal(ModalState::ViewHistory {
                    index: self.view_stack.len(),
                });
                true
            }
            KeyCode::Char('i') => {
                if let Some(info_view_data) = &self.view_state.info_view_data {
                    let state = TreeState {
                        lines: VecWithIndex::new(
                            info_view_data
                                .interpretations
                                .iter()
                                .enumerate()
                                .map(|(i, interpretation)| {
                                    let (object, _) = self
                                        .object_mapping
                                        .root_objects
                                        .get_index(interpretation.root_object_index)
                                        .unwrap();
                                    let mut view_state = self.view_state.clone();
                                    view_state
                                        .info_view_data
                                        .as_mut()
                                        .unwrap()
                                        .set_interpretation_index(
                                            &self.object_mapping,
                                            &self.buffer,
                                            i,
                                        );
                                    TreeStateLine {
                                        indent_level: 0,
                                        node: Node {
                                            text: object.print_object(&self.buffer),
                                            view_state: Some(view_state),
                                            children: None,
                                        },
                                        fold_state: FoldState::NoChildren,
                                    }
                                })
                                .collect(),
                            info_view_data.interpretations.index(),
                        ),
                    };
                    self.toggle_modal(ModalState::TreeView {
                        header: " Interpretations ",
                        state,
                    });

                    true
                } else {
                    false
                }
            }
            KeyCode::Char('?') => {
                self.toggle_modal(ModalState::HelpMenu);
                true
            }
            KeyCode::Char('q') => {
                self.should_quit = true;
                false
            }
            KeyCode::Char('c') if ctrl => {
                self.should_quit = true;
                false
            }
            KeyCode::Char('c') => {
                if let Some(info_view_data) = &mut self.view_state.info_view_data {
                    info_view_data.set_interpretation_index(
                        &self.object_mapping,
                        &self.buffer,
                        (info_view_data.interpretations.index() + 1)
                            % info_view_data.interpretations.len(),
                    );
                    true
                } else {
                    false
                }
            }
            KeyCode::Enter if self.modal.is_none() => {
                if let Some(info_view_data) = &mut self.view_state.info_view_data {
                    if let Object::Offset(offset_object) = &info_view_data.lines.cur().object {
                        if let Ok(inner) = offset_object.follow_offset(&self.buffer) {
                            let old_view_state = std::mem::replace(
                                &mut self.view_state,
                                ViewState::new_for_object(
                                    &self.object_mapping,
                                    &self.buffer,
                                    inner,
                                ),
                            );
                            self.view_stack.push(old_view_state);
                        }
                    }
                }
                true
            }
            KeyCode::Esc if self.modal.is_none() => {
                if let Some(view_state) = self.view_stack.pop() {
                    self.view_state = view_state;
                    true
                } else {
                    self.should_quit = true;
                    false
                }
            }
            KeyCode::Backspace if self.modal.is_none() => {
                if let Some(view_state) = self.view_stack.pop() {
                    self.view_state = view_state;
                    true
                } else {
                    false
                }
            }
            _ => false,
        };

        if let Some(modal_state) = self.modal.take() {
            self.modal = self.modal_view_key(key, modal_state);
            should_draw = true;
        }

        should_draw = should_draw
            || match self.active_window {
                ActiveWindow::ObjectView => self.object_view_key(key),
                ActiveWindow::HexView => self.hex_view_key(key),
            };

        should_draw
    }

    pub fn hex_view_key(&mut self, key: KeyEvent) -> bool {
        let mut current_byte = self.view_state.byte_index;
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        match key.code {
            // Navigation
            KeyCode::Up => {
                if let Some(next) = current_byte.checked_sub(self.hex_view_state.line_size) {
                    current_byte = next;
                }
            }
            KeyCode::Down => {
                let next = current_byte.saturating_add(self.hex_view_state.line_size);
                if next < self.buffer.buffer.len() {
                    current_byte = next;
                }
            }
            KeyCode::PageUp => {
                current_byte = current_byte.saturating_sub(8 * self.hex_view_state.line_size);
            }
            KeyCode::PageDown => {
                current_byte = self
                    .view_state
                    .byte_index
                    .saturating_add(8 * self.hex_view_state.line_size);
            }
            KeyCode::Left if ctrl => {
                if let Some(range) = self.view_state.hex_ranges().inner_range {
                    current_byte = range.start.saturating_sub(1);
                } else {
                    current_byte = current_byte.saturating_sub(1);
                }
            }
            KeyCode::Left => {
                current_byte = current_byte.saturating_sub(1);
            }
            KeyCode::Right if ctrl => {
                if let Some(range) = self.view_state.hex_ranges().inner_range {
                    current_byte = range.end;
                } else {
                    current_byte = current_byte.saturating_add(1);
                }
            }
            KeyCode::Right => {
                current_byte = current_byte.saturating_add(1);
            }
            KeyCode::Home => {
                current_byte = 0;
            }
            KeyCode::End => {
                current_byte = self.buffer.buffer.len() - 1;
            }
            _ => (),
        };
        self.update_byte_pos(current_byte)
    }

    fn object_view_key(&mut self, key: KeyEvent) -> bool {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        if let Some(info_view_data) = &mut self.view_state.info_view_data {
            let line_step = 5;
            let cur_index = info_view_data.lines.index();
            let last_index = info_view_data.lines.len() - 1;
            let line = match key.code {
                KeyCode::Left => info_view_data.lines.cur().parent_line_index,
                KeyCode::Right => {
                    let cur = info_view_data.lines.cur();
                    if cur.start_line_index != cur.end_line_index {
                        cur.start_line_index + 1
                    } else {
                        return false;
                    }
                }
                KeyCode::Up if ctrl => {
                    let start_line_index = info_view_data.lines.cur().start_line_index;
                    if start_line_index != cur_index {
                        start_line_index
                    } else {
                        cur_index.saturating_sub(1) as usize
                    }
                }
                KeyCode::Up => cur_index.saturating_sub(1),
                KeyCode::PageUp => cur_index.saturating_sub(line_step),
                KeyCode::Down if ctrl => {
                    let end_line_index = info_view_data.lines.cur().end_line_index;
                    if end_line_index != cur_index {
                        end_line_index
                    } else {
                        cur_index.saturating_add(1) as usize
                    }
                }
                KeyCode::Down => cur_index + 1,
                KeyCode::PageDown => last_index.min(cur_index + line_step),
                KeyCode::Home => 0,
                KeyCode::End => last_index,
                _ => return false,
            };
            self.view_state
                .set_line_pos(&self.object_mapping, &self.buffer, line);
        }
        true
    }

    fn modal_view_key(
        &mut self,
        key: KeyEvent,
        mut modal_state: ModalState<'a>,
    ) -> Option<ModalState<'a>> {
        if let KeyCode::Esc = key.code {
            return None;
        }

        match &mut modal_state {
            ModalState::GoToByte { input } => match key.code {
                KeyCode::Char(c @ '0'..='9')
                | KeyCode::Char(c @ 'a'..='f')
                | KeyCode::Char(c @ 'A'..='F') => {
                    if input.len() < 16 {
                        input.push(c.to_ascii_lowercase());
                    }
                }
                KeyCode::Enter => {
                    let addr = usize::from_str_radix(&input, 16).unwrap();
                    self.view_stack.push(self.view_state.clone());
                    self.update_byte_pos(addr);
                    self.active_window = ActiveWindow::HexView;
                    return None;
                }
                KeyCode::Backspace => {
                    input.pop();
                }
                _ => (),
            },
            ModalState::XRefs { .. } => match key.code {
                _ => (),
            },
            ModalState::ViewHistory { index } => match key.code {
                KeyCode::Up => {
                    *index = index.saturating_sub(1);
                }
                KeyCode::Down => {
                    *index = index.saturating_add(1).min(self.view_stack.len());
                }
                KeyCode::Enter => {
                    if *index < self.view_stack.len() {
                        self.view_stack.truncate(*index + 1);
                        self.view_state = self.view_stack.pop().unwrap();
                        return None;
                    } else {
                        // The last element in the list is current view state, so we do nothing
                        return None;
                    }
                }
                _ => (),
            },
            ModalState::HelpMenu => (),
            ModalState::TreeView { state, .. } => match key.code {
                KeyCode::Up => {
                    state
                        .lines
                        .try_set_index(state.lines.index().saturating_sub(1));
                }
                KeyCode::Down => {
                    state
                        .lines
                        .try_set_index(state.lines.index().saturating_add(1));
                }
                KeyCode::Right => state.toggle_fold(),
                KeyCode::Enter => {
                    let line = state.lines.cur();
                    if let Some(view_state) = &line.node.view_state {
                        let old_view_state =
                            std::mem::replace(&mut self.view_state, view_state.clone());
                        self.view_stack.push(old_view_state);
                        return None;
                    }
                }
                _ => (),
            },
        }

        Some(modal_state)
    }

    fn toggle_modal(&mut self, modal: ModalState<'a>) {
        if self.modal.as_ref().map(|m| std::mem::discriminant(m))
            == Some(std::mem::discriminant(&modal))
        {
            self.modal = None;
        } else {
            self.modal = Some(modal);
        }
    }

    fn update_byte_pos(&mut self, current_byte: usize) -> bool {
        let current_byte = current_byte.min(self.buffer.buffer.len() - 1);
        if current_byte != self.view_state.byte_index {
            self.view_state
                .set_byte_pos(&self.object_mapping, &self.buffer, current_byte);

            true
        } else {
            false
        }
    }
}

pub fn run_inspector<B: Backend>(
    terminal: &mut Terminal<B>,
    mut inspector: Inspector,
) -> io::Result<()> {
    let mut should_draw = true;
    loop {
        if should_draw {
            terminal.draw(|f| ui::draw(f, &mut inspector))?;
            should_draw = false;
        }

        if crossterm::event::poll(Duration::MAX - Duration::from_secs(5))? {
            match event::read()? {
                Event::Key(key) => {
                    should_draw = inspector.on_key(key);
                }
                Event::Resize(_, _) => should_draw = true,
                _ => (),
            }
        }
        if inspector.should_quit {
            return Ok(());
        }
    }
}
