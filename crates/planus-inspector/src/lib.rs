pub mod ui;

use std::{
    io,
    time::{Duration, Instant},
};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use planus_buffer_inspection::{
    allocations::SearchResult,
    object_formatting::{ObjectFormatting, ObjectFormattingKind},
    object_mapping::ObjectMapping,
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

#[derive(Clone, Debug)]
pub struct ViewState<'a> {
    pub current_byte: usize,
    pub current_line: Option<usize>,
    pub current_object_formatting: ObjectFormatting<'a>,
    pub search_results: Vec<SearchResult<'a>>,
    pub search_result_index: usize,
}

impl<'a> ViewState<'a> {
    fn new_for_object(object_mapping: &ObjectMapping<'a>, object: Object<'a>) -> Self {
        let (object_index, _object, &allocation_index) =
            object_mapping.all_objects.get_full(&object).unwrap();
        let search_results = object_mapping.allocations.get(object.offset());

        // TODO: should be unwrappable
        let search_result_index = search_results
            .iter()
            .position(|r| r.root_object_index == object_index)
            .unwrap_or_else(|| panic!("{object:?}\n{search_results:?}"));

        Self {
            current_byte: object.offset(),
            current_line: Some(0),
            current_object_formatting: object_mapping.allocations.allocations[allocation_index]
                .to_formatting(object_mapping),
            search_results,
            search_result_index,
        }
    }

    fn set_byte_view(&mut self, object_mapping: &ObjectMapping<'a>, index: usize) {
        if self.current_byte != index {
            self.current_byte = index;
            self.search_results = object_mapping.allocations.get(self.current_byte);
            self.find_closest_match(object_mapping);
        }
    }

    fn set_line_view(&mut self, object_mapping: &ObjectMapping<'a>, line: usize) {
        if line < self.current_object_formatting.lines.len()
            && self.current_line.unwrap_or(usize::MAX) != line
        {
            self.current_line = Some(line);
            let current_object = &self.current_object_formatting.lines[line];

            let current_field_path = if let ObjectFormattingKind::Object {
                allocation_path_index,
                ..
            } = current_object.kind
            {
                self.current_object_formatting
                    .allocation_paths
                    .get_index(allocation_path_index)
                    .unwrap()
                    .0
                    .as_slice()
            } else {
                &[]
            };
            self.current_byte = current_object.byte_range.0;
            self.search_results = object_mapping.allocations.get(self.current_byte);
            self.search_result_index = self
                .search_results
                .iter()
                .enumerate()
                .find(|(_i, sr)| {
                    sr.root_object_index == self.current_object_formatting.root_object_index
                        && sr.field_path == current_field_path
                })
                .map(|(i, _)| i)
                .unwrap_or(0);
        }
    }

    fn find_closest_match(&mut self, object_mapping: &ObjectMapping<'a>) {
        if self.search_results.is_empty() {
            self.current_line = None;
        } else {
            self.search_result_index = self
                .search_results
                .iter()
                .enumerate()
                .min_by_key(|(_, sr)| {
                    if sr.root_object_index == self.current_object_formatting.root_object_index {
                        if let Some(&line_index) = self
                            .current_object_formatting
                            .allocation_paths
                            .get(&sr.field_path)
                        {
                            (0, line_index.abs_diff(self.current_line.unwrap_or(0)))
                        } else {
                            (0, usize::MAX)
                        }
                    } else {
                        (1, 0)
                    }
                })
                .map(|(i, _sr)| i)
                .unwrap();
            let search_result = &self.search_results[self.search_result_index];
            if search_result.root_object_index != self.current_object_formatting.root_object_index {
                self.current_object_formatting = object_mapping.allocations.allocations
                    [search_result.root_allocation_index]
                    .to_formatting(object_mapping);
            }
            self.current_line = self
                .current_object_formatting
                .allocation_paths
                .get(&search_result.field_path)
                .copied();
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
                *object_mapping
                    .all_objects
                    .get_index(object_mapping.root_object.offset)
                    .unwrap()
                    .0,
            ),
            object_mapping: buffer.calculate_object_mapping(root_table_index),
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
            (KeyCode::Char('q') | KeyCode::Esc, _)
            | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                self.should_quit = true;
                false
            }
            (KeyCode::Enter, _) => {
                if let Some(current_line) = self.view_state.current_line {
                    if let ObjectFormattingKind::Object {
                        object: Object::Offset(offset_object),
                        ..
                    } = self.view_state.current_object_formatting.lines[current_line].kind
                    {
                        let inner = offset_object.get_inner(&self.buffer).unwrap();
                        let old_view_state = std::mem::replace(
                            &mut self.view_state,
                            ViewState::new_for_object(&self.object_mapping, inner),
                        );
                        self.view_stack.push(old_view_state);
                    }
                }
                true
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
            .set_byte_view(&self.object_mapping, current_byte);

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
            .set_line_view(&self.object_mapping, current_line);
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
