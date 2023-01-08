pub mod ui;

use std::{
    io,
    time::{Duration, Instant},
};

use crossterm::event::{self, Event, KeyCode};
use planus_buffer_inspection::{object_mapping::ObjectMapping, InspectableFlatbuffer};
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
}

impl<'a> Inspector<'a> {
    pub fn new(buffer: InspectableFlatbuffer<'a>, root_table_index: DeclarationIndex) -> Self {
        Self {
            buffer,
            object_mapping: buffer.calculate_object_mapping(root_table_index),
            should_quit: false,
            cursor_pos: 0,
        }
    }
    pub fn on_key(&mut self, c: KeyCode) {
        match c {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.should_quit = true;
            }
            // Navigation
            KeyCode::Up => {
                self.cursor_pos = self.cursor_pos.saturating_sub(HEX_LINE_SIZE);
            }
            KeyCode::Down => {
                self.cursor_pos = self.cursor_pos.saturating_add(HEX_LINE_SIZE);
            }
            KeyCode::PageUp => {
                self.cursor_pos = self.cursor_pos.saturating_sub(8 * HEX_LINE_SIZE);
            }
            KeyCode::PageDown => {
                self.cursor_pos = self.cursor_pos.saturating_add(8 * HEX_LINE_SIZE);
            }

            KeyCode::Left => {
                self.cursor_pos = self.cursor_pos.saturating_sub(1);
            }
            KeyCode::Right => {
                self.cursor_pos = self.cursor_pos.saturating_add(1);
            }
            _ => {}
        }
        self.cursor_pos = self.cursor_pos.min(self.buffer.buffer.len() - 1);
    }

    pub fn on_tick(&mut self) {}
}

pub fn run_inspector<B: Backend>(
    terminal: &mut Terminal<B>,
    mut inspector: Inspector,
    tick_rate: Duration,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| ui::draw(f, &mut inspector))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                inspector.on_key(key.code);
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
