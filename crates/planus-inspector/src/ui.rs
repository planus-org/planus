use tui::{
    backend::Backend,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::Inspector;

pub fn draw<B: Backend>(f: &mut Frame<B>, inspector: &mut Inspector) {
    let chunks = Layout::default()
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(f.size());

    hex_view(f, &[0x41, 0x42, 0x43, 0x44], chunks[0]);
}

pub fn hex_view<B: Backend>(f: &mut Frame<B>, data: &[u8], area: Rect) {
    let mut view = Vec::new();
    for chunk in data.chunks(16) {
        let mut line = Vec::new();
        for b in chunk {
            line.push(Span::styled(
                format!("{b:x} "),
                Style::default().fg(Color::Green),
            ));
        }
        view.push(Spans::from(line));
    }
    let block = Block::default().borders(Borders::ALL);
    let paragraph = Paragraph::new(view).block(block).wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}
