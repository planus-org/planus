use planus_buffer_inspection::{
    object_formatting::{ObjectFormatting, ObjectFormattingKind, ObjectFormattingLine},
    object_info::ObjectName,
};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
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
        .constraints([Percentage(40), Percentage(60)].as_ref())
        .split(vert[0]);

    interpretations_view(f, top[0], inspector);
    info_area(f, top[1], inspector);
    hex_view(f, vert[1], inspector);
}

fn interpretations_view<B: Backend>(f: &mut Frame<B>, area: Rect, inspector: &mut Inspector) {
    let mut text = vec![
        Spans::from(Span::styled(
            format!("Objects at 0x{:x}: ", inspector.view_state.current_byte),
            Style::default(),
        )),
        Spans::default(),
    ];
    for search_result in &inspector.view_state.search_results {
        for field_access in &search_result.field_path {
            let allocation =
                &inspector.object_mapping.allocations.allocations[field_access.allocation_index];

            let (object, _object_allocation_index) = inspector
                .object_mapping
                .all_objects
                .get_index(allocation.object_index)
                .unwrap_or_else(|| panic!("Cannot get object for allocation {field_access:?}"));
            text.extend_from_slice(&[Spans::from(Span::styled(
                format!(
                    "{}: {} @ 0x{:x}",
                    field_access.field_name,
                    object.resolve_name(&inspector.buffer),
                    allocation.start,
                ),
                Style::default(),
            ))]);
        }
    }

    let block = Block::default().borders(Borders::ALL);
    let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}

pub fn hex_view<B: Backend>(f: &mut Frame<B>, area: Rect, inspector: &mut Inspector) {
    let is_active = matches!(inspector.active_window, crate::ActiveWindow::HexView);

    let mut ranges: Vec<std::ops::Range<usize>> = Vec::new();
    let range_colors = [Color::Blue, Color::Cyan, Color::Gray];
    for search_result in &inspector.view_state.search_results {
        let Some(field_access) = search_result.field_path.last() else { continue; };
        let allocation =
            &inspector.object_mapping.allocations.allocations[field_access.allocation_index];
        ranges.push(allocation.start..allocation.end);
    }

    let mut view = Vec::new();
    let skipped_lines = inspector.view_state.current_byte / HEX_LINE_SIZE;
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
            let style = if pos == inspector.view_state.current_byte {
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

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(if is_active {
            Color::Green
        } else {
            Color::White
        }));
    let paragraph = Paragraph::new(view).block(block).wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}

fn info_area<B: Backend>(f: &mut Frame<B>, area: Rect, inspector: &mut Inspector) {
    let is_active = matches!(inspector.active_window, crate::ActiveWindow::ObjectView);

    let mut text = Vec::new();

    if let Some(obj_fmt) = inspector.view_state.current_object_formatting.as_ref() {
        let lines = obj_fmt.to_string(&inspector.buffer);

        for (i, line) in lines.lines().enumerate().skip(
            inspector
                .view_state
                .current_line
                .saturating_sub(usize::from(area.height).saturating_sub(8)),
        ) {
            let style = if i == inspector.view_state.current_line {
                Style::default().add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            text.push(Spans::from(Span::styled(format!("{line}"), style)));
        }
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(if is_active {
            Color::Green
        } else {
            Color::White
        }));

    let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: false });
    f.render_widget(paragraph, area);
}
