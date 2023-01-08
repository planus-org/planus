use planus_buffer_inspection::{children::Byterange, object_info::ObjectName};
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
    let objs = inspector
        .object_mapping
        .get_bytes_for_pos(inspector.cursor_pos);

    let mut ranges = Vec::new();
    let range_colors = [Color::Blue, Color::Cyan, Color::Gray];
    for obj in objs {
        if let Some(range) = obj.byterange(&inspector.buffer) {
            ranges.push(range);
        }
    }

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
            let pos = (line_no + skipped_lines) * HEX_LINE_SIZE + col_no;
            let style = if pos == inspector.cursor_pos {
                Style::default().bg(Color::White)
            } else {
                if let Some(i) = ranges.iter().position(|r| r.contains(&pos)) {
                    Style::default().bg(range_colors.get(i).cloned().unwrap_or(Color::Black))
                } else {
                    Style::default()
                }
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
    let mut text = vec![
        Spans::from(Span::styled(
            format!("offset: {}", inspector.cursor_pos),
            Style::default(),
        )),
        Spans::default(),
    ];
    for obj in objs {
        let range = obj
            .byterange(&inspector.buffer)
            .map(|r| format!("{}-{}", r.start, r.end))
            .unwrap_or_else(|| format!("none"));
        text.extend_from_slice(&[
            Spans::from(Span::raw(obj.resolve_name(&inspector.buffer))),
            Spans::from(Span::raw(format!("range: {range}"))),
            Spans::default(),
        ]);
    }
    let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}
