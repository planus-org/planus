pub mod ui;

use std::{
    io,
    time::{Duration, Instant},
};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use planus_buffer_inspection::{
    object_mapping::{Interpretation, Line, ObjectIndex, ObjectMapping},
    InspectableFlatbuffer, Object,
};
use planus_types::intermediate::DeclarationIndex;
use tui::{backend::Backend, Terminal};

use crate::ui::HEX_LINE_SIZE;

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

impl ActiveWindow {
    #[must_use]
    pub fn toggle(self) -> Self {
        match self {
            ActiveWindow::ObjectView => ActiveWindow::HexView,
            ActiveWindow::HexView => ActiveWindow::ObjectView,
        }
    }
}

pub struct Inspector<'a> {
    pub object_mapping: ObjectMapping<'a>,
    pub buffer: InspectableFlatbuffer<'a>,
    pub should_quit: bool,
    pub view_stack: Vec<ViewState<'a>>,
    pub active_window: ActiveWindow,
    pub view_state: ViewState<'a>,
}

#[derive(Debug)]
pub struct ViewState<'a> {
    pub current_byte: usize,
    pub current_object_index: ObjectIndex,
    pub current_line: Option<usize>,
    pub lines: Vec<Line<'a>>,
    pub interpretations: Vec<Interpretation>,
    pub interpretation_index: usize,
}

impl<'a> ViewState<'a> {
    fn new_for_object(
        object_mapping: &ObjectMapping<'a>,
        buffer: &InspectableFlatbuffer<'a>,
        object: Object<'a>,
    ) -> Self {
        let object_index = object_mapping
            .root_objects
            .get_index_of(&object)
            .unwrap_or_else(|| panic!("{object:?}, {:?}", object_mapping.root_objects));
        let mut interpretations = Vec::new();
        let mut interpretation_index = None;
        object_mapping.get_interpretations_cb(object.offset(), buffer, |interpretation| {
            if interpretation.root_object_index == object_index {
                interpretation_index.get_or_insert(interpretations.len());
            }
            interpretations.push(interpretation);
        });

        Self {
            current_byte: object.offset() as usize,
            current_line: Some(0),
            current_object_index: object_index,
            lines: object_mapping
                .line_tree(object_index, buffer)
                .flatten(buffer),
            interpretations,
            interpretation_index: interpretation_index.unwrap(),
        }
    }

    fn set_byte_pos(
        &mut self,
        object_mapping: &ObjectMapping<'a>,
        buffer: &InspectableFlatbuffer<'a>,
        index: usize,
    ) {
        if self.current_byte != index {
            self.current_byte = index;
            self.interpretations =
                object_mapping.get_interpretations(self.current_byte as u32, buffer);
            if self.interpretations.is_empty() {
                self.interpretation_index = 0;
                self.current_line = None;
            } else {
                self.set_interpretation_index(
                    object_mapping,
                    buffer,
                    self.interpretations
                        .iter()
                        .enumerate()
                        .min_by_key(|(_, interpretation)| {
                            if interpretation.root_object_index == self.current_object_index {
                                (
                                    0,
                                    interpretation
                                        .lines
                                        .iter()
                                        .map(|line| line.abs_diff(self.current_line.unwrap_or(0)))
                                        .min()
                                        .unwrap_or(usize::MAX),
                                )
                            } else {
                                (1, 0)
                            }
                        })
                        .map(|(i, _)| i)
                        .unwrap(),
                );
            };
        }
    }

    fn set_line_pos(
        &mut self,
        object_mapping: &ObjectMapping<'a>,
        buffer: &InspectableFlatbuffer<'a>,
        line: usize,
    ) {
        if line < self.lines.len() && self.current_line != Some(line) {
            self.current_line = Some(line);
            self.current_byte = self.lines[line].start as usize;
            let start_line_index = self.lines[line].start_line_index;
            self.interpretations =
                object_mapping.get_interpretations(self.current_byte as u32, buffer);
            self.interpretation_index = self
                .interpretations
                .iter()
                .position(|interpretation| {
                    interpretation.root_object_index == self.current_object_index
                        && interpretation.lines.contains(&start_line_index)
                })
                .unwrap();
        }
    }

    fn set_interpretation_index(
        &mut self,
        object_mapping: &ObjectMapping<'a>,
        buffer: &InspectableFlatbuffer<'a>,
        interpretation_index: usize,
    ) {
        if interpretation_index < self.interpretations.len() {
            self.interpretation_index = interpretation_index;
            if self.current_object_index
                != self.interpretations[self.interpretation_index].root_object_index
            {
                self.current_object_index =
                    self.interpretations[self.interpretation_index].root_object_index;
                self.lines = object_mapping
                    .line_tree(self.current_object_index, buffer)
                    .flatten(buffer)
            }
            self.current_line = Some(
                *self.interpretations[self.interpretation_index]
                    .lines
                    .last()
                    .unwrap(),
            );
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
        }
    }

    pub fn on_key(&mut self, key: KeyEvent) -> bool {
        let mut should_draw = match (key.code, key.modifiers) {
            (KeyCode::Tab, _) => {
                self.active_window = self.active_window.toggle();
                if matches!(self.active_window, ActiveWindow::ObjectView)
                    && self.view_state.current_line.is_none()
                {
                    self.view_state.current_line = Some(0);
                }
                true
            }
            (KeyCode::Char('g'), _) => {
                self.view_state = ViewState::new_for_object(
                    &self.object_mapping,
                    &self.buffer,
                    self.object_mapping.root_object,
                );
                true
            }
            (KeyCode::Char('c'), _) if self.view_state.interpretations.len() > 0 => {
                self.view_state.set_interpretation_index(
                    &self.object_mapping,
                    &self.buffer,
                    (self.view_state.interpretation_index + 1)
                        % self.view_state.interpretations.len(),
                );
                true
            }
            (KeyCode::Enter, _) => {
                if let Some(current_line) = self.view_state.current_line {
                    if let Some(offset_object) = self.view_state.lines[current_line].offset_object {
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
            (KeyCode::Char('q'), _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                self.should_quit = true;
                false
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

        should_draw = should_draw
            || match self.active_window {
                ActiveWindow::ObjectView => self.object_view_key(key),
                ActiveWindow::HexView => self.hex_view_key(key),
            };

        should_draw
    }

    pub fn hex_view_key(&mut self, key: KeyEvent) -> bool {
        let mut current_byte = self.view_state.current_byte;
        let should_draw = match key.code {
            // Navigation
            KeyCode::Up => {
                current_byte = current_byte.saturating_sub(HEX_LINE_SIZE);
                true
            }
            KeyCode::Down => {
                current_byte = current_byte.saturating_add(HEX_LINE_SIZE);
                true
            }
            KeyCode::PageUp => {
                current_byte = current_byte.saturating_sub(8 * HEX_LINE_SIZE);
                true
            }
            KeyCode::PageDown => {
                self.view_state.current_byte = self
                    .view_state
                    .current_byte
                    .saturating_add(8 * HEX_LINE_SIZE);
                true
            }

            KeyCode::Left => {
                current_byte = current_byte.saturating_sub(1);
                true
            }
            KeyCode::Right => {
                current_byte = current_byte.saturating_add(1);
                true
            }
            KeyCode::Home => {
                self.view_state.current_byte = 0;
                true
            }
            KeyCode::End => {
                self.view_state.current_byte = self.buffer.buffer.len() - 1;
                true
            }
            _ => false,
        };
        current_byte = current_byte.min(self.buffer.buffer.len() - 1);

        self.view_state
            .set_byte_pos(&self.object_mapping, &self.buffer, current_byte);

        should_draw
    }

    fn object_view_key(&mut self, key: KeyEvent) -> bool {
        let mut current_line = self.view_state.current_line.unwrap_or(0);
        match key.code {
            KeyCode::Up => current_line = current_line.saturating_sub(1),
            KeyCode::Down => current_line = current_line + 1,
            _ => (),
        }
        self.view_state
            .set_line_pos(&self.object_mapping, &self.buffer, current_line);
        true
    }

    pub fn on_tick(&mut self) {}
}

pub fn run_inspector<B: Backend>(
    terminal: &mut Terminal<B>,
    mut inspector: Inspector,
    tick_rate: Duration,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    let mut should_draw = true;
    loop {
        if should_draw {
            terminal.draw(|f| ui::draw(f, &mut inspector))?;
            should_draw = false;
        }

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if crossterm::event::poll(timeout)? {
            match event::read()? {
                Event::Key(key) => {
                    should_draw = inspector.on_key(key);
                }
                Event::Resize(_, _) => should_draw = true,
                _ => (),
            }
        }
        if last_tick.elapsed() >= tick_rate {
            inspector.on_tick();
            last_tick = Instant::now();
        }
        if inspector.should_quit {
            return Ok(());
        }
    }
}
