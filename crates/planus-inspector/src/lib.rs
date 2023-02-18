pub mod ui;
pub mod vec_with_index;

use std::{io, time::Duration};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use planus_buffer_inspection::{
    object_mapping::{Interpretation, Line, ObjectIndex, ObjectMapping},
    InspectableFlatbuffer, Object,
};
use planus_types::intermediate::DeclarationIndex;
use tui::{backend::Backend, Terminal};

use crate::vec_with_index::VecWithIndex;

pub struct TreeState<T> {
    pub data: T,
    pub unfolded: bool,
    pub children: Option<Vec<TreeState<T>>>,
}

#[derive(Copy, Clone, Debug)]
pub enum ActiveWindow {
    ObjectView,
    HexView,
}

#[derive(Clone, Debug)]
pub enum ModalState {
    GoToByte { input: String },
}

pub struct Inspector<'a> {
    pub object_mapping: ObjectMapping<'a>,
    pub buffer: InspectableFlatbuffer<'a>,
    pub should_quit: bool,
    pub view_stack: Vec<ViewState<'a>>,
    pub active_window: ActiveWindow,
    pub view_state: ViewState<'a>,
    pub hex_view_state: HexViewState,
    pub modal: Option<ModalState>,
}

pub struct HexViewState {
    pub line_size: usize,
    pub height: usize,
    pub line_pos: usize,
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

#[derive(Debug)]
pub struct ViewState<'a> {
    pub byte_index: usize,
    pub info_view_data: Option<InfoViewData<'a>>,
}

#[derive(Debug)]
pub struct InfoViewData<'a> {
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
            hex_view_state: HexViewState {
                line_size: 0,
                height: 0,
                line_pos: 0,
            },
            modal: None,
        }
    }

    pub fn on_key(&mut self, key: KeyEvent) -> bool {
        let mut should_draw = match (key.code, key.modifiers) {
            (KeyCode::Tab, _) => match self.active_window {
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
            (KeyCode::Char('g'), _) => {
                self.modal = Some(ModalState::GoToByte {
                    input: String::new(),
                });
                true
            }
            (KeyCode::Char('q'), _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                self.should_quit = true;
                false
            }
            (KeyCode::Char('c'), _) => {
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
            (KeyCode::Enter, _) => {
                if let Some(info_view_data) = &mut self.view_state.info_view_data {
                    if let Some(offset_object) = info_view_data.lines.cur().offset_object {
                        let inner = offset_object.get_inner(&self.buffer).unwrap();
                        let old_view_state = std::mem::replace(
                            &mut self.view_state,
                            ViewState::new_for_object(&self.object_mapping, &self.buffer, inner),
                        );
                        self.view_stack.push(old_view_state);
                    }
                }
                true
            }
            (KeyCode::Esc, _) => {
                if let Some(view_state) = self.view_stack.pop() {
                    self.view_state = view_state;
                    true
                } else {
                    self.should_quit = true;
                    false
                }
            }
            (KeyCode::Backspace, _) => {
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

            KeyCode::Left => {
                current_byte = current_byte.saturating_sub(1);
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
        if let Some(info_view_data) = &mut self.view_state.info_view_data {
            let line = match key.code {
                KeyCode::Up => info_view_data.lines.index().saturating_sub(1),
                KeyCode::Down => info_view_data.lines.index() + 1,
                _ => return false,
            };
            self.view_state
                .set_line_pos(&self.object_mapping, &self.buffer, line);
        }
        true
    }

    fn modal_view_key(&mut self, key: KeyEvent, mut modal_state: ModalState) -> Option<ModalState> {
        match &mut modal_state {
            ModalState::GoToByte { input } => match key.code {
                KeyCode::Char(c @ '0'..='9') => {
                    input.push(c);
                }
                KeyCode::Enter => {
                    let addr = usize::from_str_radix(&input, 16).unwrap();
                    self.update_byte_pos(addr);
                    return None;
                }
                _ => (),
            },
        }

        Some(modal_state)
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

        if crossterm::event::poll(Duration::MAX)? {
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
