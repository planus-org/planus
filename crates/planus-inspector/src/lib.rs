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

pub struct Inspector<'a> {
    pub object_mapping: ObjectMapping<'a>,
    pub buffer: InspectableFlatbuffer<'a>,
    pub should_quit: bool,
    pub cursor_pos: usize,
    pub offset_stack: Vec<usize>,
}

impl<'a> Inspector<'a> {
    pub fn new(buffer: InspectableFlatbuffer<'a>, root_table_index: DeclarationIndex) -> Self {
        Self {
            buffer,
            object_mapping: buffer.calculate_object_mapping(root_table_index),
            should_quit: false,
            cursor_pos: 0,
            offset_stack: Vec::new(),
        }
    }
    pub fn on_key(&mut self, key: KeyEvent) -> bool {
        let should_draw = match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.should_quit = true;
                false
            }
            // Navigation
            KeyCode::Up => {
                self.cursor_pos = self.cursor_pos.saturating_sub(HEX_LINE_SIZE);
                true
            }
            KeyCode::Down => {
                self.cursor_pos = self.cursor_pos.saturating_add(HEX_LINE_SIZE);
                true
            }
            KeyCode::PageUp => {
                self.cursor_pos = self.cursor_pos.saturating_sub(8 * HEX_LINE_SIZE);
                true
            }
            KeyCode::PageDown => {
                self.cursor_pos = self.cursor_pos.saturating_add(8 * HEX_LINE_SIZE);
                true
            }

            KeyCode::Left => {
                self.cursor_pos = self.cursor_pos.saturating_sub(1);
                true
            }
            KeyCode::Right => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    let search_results = self.object_mapping.allocations.get::<2>(self.cursor_pos);
                    if let Some(search_result) = search_results.first() {
                        let allocation = search_result.result.first().unwrap();
                        if let Some(object) = allocation.object {
                            let (object, _) =
                                self.object_mapping.all_objects.get_index(object).unwrap();
                            if let Object::Offset(offset_object) = object {
                                self.offset_stack.push(self.cursor_pos);
                                self.cursor_pos =
                                    offset_object.get_byte_index(&self.buffer).unwrap();
                            }
                        }
                    }
                } else {
                    self.cursor_pos = self.cursor_pos.saturating_add(1);
                }
                true
            }
            KeyCode::Enter => {
                let search_results = self.object_mapping.allocations.get::<1>(self.cursor_pos);
                if let Some(search_result) = search_results.first() {
                    let allocation = search_result.result.last().unwrap();
                    if let Some(object_index) = allocation.object {
                        let (object, _) = self
                            .object_mapping
                            .all_objects
                            .get_index(object_index)
                            .unwrap();
                        if let Object::Offset(offset_object) = object {
                            self.offset_stack.push(self.cursor_pos);
                            self.cursor_pos = offset_object.get_byte_index(&self.buffer).unwrap();
                        }
                    }
                }
                true
            }
            KeyCode::Backspace => {
                if let Some(pos) = self.offset_stack.pop() {
                    self.cursor_pos = pos;
                    true
                } else {
                    false
                }
            }
            KeyCode::Home => {
                self.cursor_pos = 0;
                true
            }
            KeyCode::End => {
                self.cursor_pos = self.buffer.buffer.len() - 1;
                true
            }
            _ => false,
        };
        self.cursor_pos = self.cursor_pos.min(self.buffer.buffer.len() - 1);

        should_draw
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
                Event::Key(key) => should_draw = inspector.on_key(key),
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
