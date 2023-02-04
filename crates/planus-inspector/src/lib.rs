pub mod ui;

use std::{
    io,
    time::{Duration, Instant},
};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use planus_buffer_inspection::{object_mapping::ObjectMapping, InspectableFlatbuffer, Object};
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
    pub hex_cursor_pos: usize,
    pub object_line_pos: usize,
    pub offset_stack: Vec<usize>,
    pub active_window: ActiveWindow,
}

impl<'a> Inspector<'a> {
    pub fn new(buffer: InspectableFlatbuffer<'a>, root_table_index: DeclarationIndex) -> Self {
        Self {
            buffer,
            object_mapping: buffer.calculate_object_mapping(root_table_index),
            should_quit: false,
            hex_cursor_pos: 0,
            object_line_pos: 0,
            offset_stack: Vec::new(),
            active_window: ActiveWindow::HexView,
        }
    }

    pub fn on_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Tab => self.active_window = self.active_window.toggle(),
            KeyCode::Char('q') | KeyCode::Esc => {
                self.should_quit = true;
            }
            _ => (),
        }

        match self.active_window {
            ActiveWindow::ObjectView => self.object_view_key(key),
            ActiveWindow::HexView => self.hex_view_key(key),
        }
    }

    pub fn hex_view_key(&mut self, key: KeyEvent) -> bool {
        let should_draw = match key.code {
            // Navigation
            KeyCode::Up => {
                self.hex_cursor_pos = self.hex_cursor_pos.saturating_sub(HEX_LINE_SIZE);
                true
            }
            KeyCode::Down => {
                self.hex_cursor_pos = self.hex_cursor_pos.saturating_add(HEX_LINE_SIZE);
                true
            }
            KeyCode::PageUp => {
                self.hex_cursor_pos = self.hex_cursor_pos.saturating_sub(8 * HEX_LINE_SIZE);
                true
            }
            KeyCode::PageDown => {
                self.hex_cursor_pos = self.hex_cursor_pos.saturating_add(8 * HEX_LINE_SIZE);
                true
            }

            KeyCode::Left => {
                self.hex_cursor_pos = self.hex_cursor_pos.saturating_sub(1);
                true
            }
            KeyCode::Right => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    let search_results = self.object_mapping.allocations.get(self.hex_cursor_pos);
                    if let Some(search_result) = search_results.first() {
                        if let Some(index) = search_result.field_path.len().checked_sub(2) {
                            let field_access = search_result.field_path[index];
                            let (object, _) = self
                                .object_mapping
                                .all_objects
                                .get_index(field_access.allocation.object_index)
                                .unwrap();
                            if let Object::Offset(offset_object) = object {
                                self.offset_stack.push(self.hex_cursor_pos);
                                self.hex_cursor_pos =
                                    offset_object.get_byte_index(&self.buffer).unwrap();
                            }
                        }
                    }
                } else {
                    self.hex_cursor_pos = self.hex_cursor_pos.saturating_add(1);
                }
                true
            }
            KeyCode::Enter => {
                let search_results = self.object_mapping.allocations.get(self.hex_cursor_pos);
                if let Some(search_result) = search_results.first() {
                    let field_access = search_result.field_path.last().unwrap();
                    let (object, _) = self
                        .object_mapping
                        .all_objects
                        .get_index(field_access.allocation.object_index)
                        .unwrap();
                    if let Object::Offset(offset_object) = object {
                        self.offset_stack.push(self.hex_cursor_pos);
                        self.hex_cursor_pos = offset_object.get_byte_index(&self.buffer).unwrap();
                    }
                }
                true
            }
            KeyCode::Backspace => {
                if let Some(pos) = self.offset_stack.pop() {
                    self.hex_cursor_pos = pos;
                    true
                } else {
                    false
                }
            }
            KeyCode::Home => {
                self.hex_cursor_pos = 0;
                true
            }
            KeyCode::End => {
                self.hex_cursor_pos = self.buffer.buffer.len() - 1;
                true
            }
            _ => false,
        };
        self.hex_cursor_pos = self.hex_cursor_pos.min(self.buffer.buffer.len() - 1);

        should_draw
    }

    fn object_view_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Up => self.object_line_pos = self.object_line_pos.saturating_sub(1),
            KeyCode::Down => self.object_line_pos = self.object_line_pos + 1,
            _ => (),
        }
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
