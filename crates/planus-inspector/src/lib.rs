pub mod ui;

use std::{
    io,
    time::{Duration, Instant},
};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use planus_buffer_inspection::{
    allocations::SearchResult, object_formatting::ObjectFormatting, object_mapping::ObjectMapping,
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

#[derive(Clone, Default, Debug)]
pub struct ViewState<'a> {
    pub current_byte: usize,
    pub current_line: usize,
    pub current_object_formatting: Option<ObjectFormatting<'a>>,
    pub search_results: Vec<SearchResult<'a>>,
    pub search_result_index: usize,
}

impl<'a> ViewState<'a> {
    fn update_view_data(&mut self, object_mapping: &ObjectMapping<'a>) {
        self.search_results = object_mapping.allocations.get(self.current_byte);
        if let Some(search_result) = self.search_results.get(self.search_result_index) {
            let allocation =
                &object_mapping.allocations.allocations[search_result.root_allocation_index];
            let object_formatting = allocation.to_formatting(&object_mapping);
            if self.current_line >= object_formatting.lines.len() {
                self.current_line = object_formatting.lines.len() - 1;
            }
            self.current_object_formatting = Some(object_formatting);
        }
    }

    fn set_byte_view(&mut self, object_mapping: &ObjectMapping<'a>, index: usize) {
        if self.current_byte != index {
            self.current_byte = index;
            self.update_search_results(object_mapping);
            self.find_closest_match(object_mapping);
        }
    }

    fn set_line_view(&mut self, object_mapping: &ObjectMapping<'a>, line: usize) {
        if self.current_line != line
            && line
                < self
                    .current_object_formatting
                    .as_ref()
                    .map(|o| o.lines.len())
                    .unwrap_or(0)
        {
            self.current_line = line;
            if let Some(current_object_formatting) = &self.current_object_formatting {
                self.current_byte = current_object_formatting.lines[line].byte_range.0;
                self.update_search_results(object_mapping);
                self.search_result_index = self
                    .search_results
                    .iter()
                    .enumerate()
                    .find(|(_i, _sr)| true)
                    .map(|(i, _)| i)
                    .unwrap_or(0);
            }
        }
    }

    fn update_search_results(&mut self, object_mapping: &ObjectMapping<'a>) {
        // TODO: handle empty
        self.search_results = object_mapping.allocations.get(self.current_byte);
    }

    fn find_closest_match(&mut self, object_mapping: &ObjectMapping<'a>) {
        // TODO: fix
        self.search_result_index = 0;
        self.current_line = 0;

        if let Some(search_result) = self.search_results.get(self.search_result_index) {
            let allocation =
                &object_mapping.allocations.allocations[search_result.root_allocation_index];
            let object_formatting = allocation.to_formatting(&object_mapping);
            self.current_object_formatting = Some(object_formatting);
        }
    }
}

impl<'a> Inspector<'a> {
    pub fn new(buffer: InspectableFlatbuffer<'a>, root_table_index: DeclarationIndex) -> Self {
        Self {
            buffer,
            object_mapping: buffer.calculate_object_mapping(root_table_index),
            should_quit: false,
            view_stack: Vec::new(),
            active_window: ActiveWindow::HexView,
            view_state: ViewState::default(),
        }
    }

    pub fn on_key(&mut self, key: KeyEvent) -> bool {
        let mut should_draw = match key.code {
            KeyCode::Tab => {
                self.active_window = self.active_window.toggle();
                true
            }
            KeyCode::Char('q') | KeyCode::Esc => {
                self.should_quit = true;
                false
            }
            KeyCode::Enter => {
                if let Some(search_result) = self
                    .view_state
                    .search_results
                    .get(self.view_state.search_result_index)
                {
                    let field_access = search_result.field_path.last().unwrap();
                    let allocation =
                        &self.object_mapping.allocations.allocations[field_access.allocation_index];
                    let (object, _) = self
                        .object_mapping
                        .all_objects
                        .get_index(allocation.object_index)
                        .unwrap();
                    if let Object::Offset(offset_object) = object {
                        self.view_stack.push(self.view_state.clone());
                        self.view_state = ViewState::default();
                        self.view_state.set_byte_view(
                            &self.object_mapping,
                            offset_object.get_byte_index(&self.buffer).unwrap(),
                        );
                    }
                }
                true
            }
            KeyCode::Backspace => {
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
        let mut current_line = self.view_state.current_line;
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
    inspector
        .view_state
        .set_byte_view(&inspector.object_mapping, 0);

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
