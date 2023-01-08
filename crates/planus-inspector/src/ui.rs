use planus_buffer_inspection::object_info::ObjectName;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::Inspector;

pub const HEX_LINE_SIZE: usize = 16;

pub fn draw<B: Backend>(f: &mut Frame<B>, inspector: &mut Inspector) {
    use Constraint::*;
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Percentage(20), Percentage(50), Percentage(30)].as_ref())
        .split(f.size());

    hex_view(f, chunks[1], inspector);
    info_area(f, chunks[2], inspector);
}

pub fn hex_view<B: Backend>(f: &mut Frame<B>, area: Rect, inspector: &mut Inspector) {
    let mut view = Vec::new();
    let skipped_lines = inspector.cursor_pos / HEX_LINE_SIZE;
    for (line_no, chunk) in inspector
        .buffer
        .buffer
        .chunks(HEX_LINE_SIZE)
        .skip(skipped_lines)
        .enumerate()
    {
        let mut line = Vec::new();
        for (col_no, b) in chunk.iter().enumerate() {
            let style =
                if (line_no + skipped_lines) * HEX_LINE_SIZE + col_no == inspector.cursor_pos {
                    Style::default().bg(Color::White)
                } else {
                    Style::default()
                };
            line.push(Span::styled(format!("{b:02x} "), style.fg(Color::Green)));
        }
        view.push(Spans::from(line));
    }
    let block = Block::default().borders(Borders::ALL);
    let paragraph = Paragraph::new(view).block(block).wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}

fn info_area<B: Backend>(f: &mut Frame<B>, area: Rect, inspector: &mut Inspector) {
    let objs = inspector
        .object_mapping
        .get_bytes_for_pos(inspector.cursor_pos);
    let block = Block::default().borders(Borders::ALL);
    let mut text = Vec::new();
    for obj in objs {
        text.push(Span::raw(obj.resolve_name(&inspector.buffer)));
    }
    let paragraph = Paragraph::new(Spans::from(text))
        .block(block)
        .wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}
