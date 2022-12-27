pub mod ui;

use std::{
    collections::BTreeMap,
    io,
    time::{Duration, Instant},
};

use crossterm::event::{self, Event, KeyCode};
use planus_buffer_inspection::{ByteIndex, Object};
use tui::{backend::Backend, Terminal};

pub type ObjectIndex = usize;

pub struct TreeState<T> {
    data: T,
    unfolded: bool,
    children: Option<Vec<TreeState<T>>>,
}

#[derive(Default)]
struct ViewState<'a> {
    pub all_objects: Vec<Object<'a>>,
    pub current_gui_root_object: ObjectIndex,
    pub byte_mapping: BTreeMap<ByteIndex, Vec<ObjectIndex>>,
    pub parents: BTreeMap<ObjectIndex, ObjectIndex>,
}

#[derive(Default)]
pub struct Inspector<'a> {
    view_state: ViewState<'a>,
    should_quit: bool,
}

impl<'a> Inspector<'a> {
    pub fn on_key(&mut self, c: KeyCode) {
        match c {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.should_quit = true;
            }
            _ => {}
        }
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
