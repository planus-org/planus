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
    let vert = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Percentage(50), Percentage(50)].as_ref())
        .split(f.size());
    let top = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Percentage(30), Percentage(70)].as_ref())
        .split(vert[0]);

    interpretations_view(f, top[0], inspector);
    info_area(f, top[1], inspector);
    hex_view(f, vert[1], inspector);
}

fn interpretations_view<B: Backend>(f: &mut Frame<B>, area: Rect, inspector: &mut Inspector) {
    let text = vec![
        Spans::from(Span::styled(
            format!("Objects at {}: ", inspector.hex_cursor_pos),
            Style::default(),
        )),
        Spans::default(),
    ];
    let block = Block::default().borders(Borders::ALL);
    let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}

pub fn hex_view<B: Backend>(f: &mut Frame<B>, area: Rect, inspector: &mut Inspector) {
    let mut ranges: Vec<std::ops::Range<usize>> = Vec::new();
    let range_colors = [Color::Blue, Color::Cyan, Color::Gray];
    let search_results = inspector
        .object_mapping
        .allocations
        .get(inspector.hex_cursor_pos);
    for search_result in search_results {
        let Some(allocation) = search_result.result.last() else { continue; };
        if allocation.object.is_none() {
            continue;
        }
        ranges.push(allocation.start..allocation.end);
    }

    let mut view = Vec::new();
    let skipped_lines = inspector.hex_cursor_pos / HEX_LINE_SIZE;
    for (line_no, chunk) in inspector
        .buffer
        .buffer
        .chunks(HEX_LINE_SIZE)
        .skip(skipped_lines)
        .take(100)
        .enumerate()
    {
        let mut line = Vec::new();
        for (col_no, b) in chunk.iter().enumerate() {
            let pos = (line_no + skipped_lines) * HEX_LINE_SIZE + col_no;
            let style = if pos == inspector.hex_cursor_pos {
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

    let is_active = matches!(inspector.active_window, crate::ActiveWindow::HexView);
    let block = Block::default().borders(Borders::ALL);
    let paragraph = Paragraph::new(view).block(block).wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}

fn info_area<B: Backend>(f: &mut Frame<B>, area: Rect, inspector: &mut Inspector) {
    let search_results = inspector
        .object_mapping
        .allocations
        .get(inspector.hex_cursor_pos);
    let block = Block::default().borders(Borders::ALL);
    let mut text = vec![
        Spans::from(Span::styled(
            format!("offset: {}", inspector.hex_cursor_pos),
            Style::default(),
        )),
        Spans::default(),
    ];

    for (i, search_result) in search_results.iter().enumerate() {
        for allocation in &search_result.result[search_result.result.len().saturating_sub(3)..] {
            let Some(object_index) = allocation.object
            else {
                continue;
            };
            let range = format!("{}-{}", allocation.start, allocation.end);
            let (object, _object_allocation_index) = inspector
                .object_mapping
                .all_objects
                .get_index(object_index)
                .unwrap_or_else(|| panic!("Cannot get object for allocation {allocation:?}"));
            text.extend_from_slice(&[
                Spans::from(Span::styled(
                    object.resolve_name(&inspector.buffer),
                    Style::default().fg(if i % 2 == 0 { Color::Red } else { Color::Blue }),
                )),
                Spans::from(Span::styled(
                    format!("range: {range}"),
                    Style::default().fg(if i % 2 == 0 {
                        Color::Gray
                    } else {
                        Color::White
                    }),
                )),
                Spans::default(),
            ]);
        }
    }
    let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}
