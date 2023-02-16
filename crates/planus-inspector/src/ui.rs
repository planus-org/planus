use planus_buffer_inspection::object_info::ObjectName;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
    Frame,
};

use crate::{Inspector, RangeMatch};

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
    for (i, interpretation) in inspector.view_state.interpretations.iter().enumerate() {
        let object = inspector
            .object_mapping
            .root_objects
            .get_index(interpretation.root_object_index)
            .unwrap()
            .0;
        text.extend_from_slice(&[Spans::from(Span::styled(
            format!(
                "{}{}:{} @ {:x}",
                if i == inspector.view_state.interpretation_index {
                    "* "
                } else {
                    "  "
                },
                object.resolve_name(&inspector.buffer),
                interpretation.lines.last().unwrap(),
                object.offset(),
            ),
            Style::default(),
        ))]);
    }

    let block = block(false);
    let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: false });
    f.render_widget(paragraph, area);
}

pub fn hex_view<B: Backend>(f: &mut Frame<B>, area: Rect, inspector: &mut Inspector) {
    let is_active = matches!(inspector.active_window, crate::ActiveWindow::HexView);
    let block = block(is_active);
    let inner_area = block.inner(area);

    let ranges = inspector.view_state.hex_ranges();

    let max_offset = inspector.buffer.buffer.len().max(1) as u64;
    let max_offset_ilog2 = 63 - max_offset.leading_zeros();
    let max_offset_hex_digits = max_offset_ilog2 / 4 + 1;
    let max_offset_hex_digits = (max_offset_hex_digits + (max_offset_hex_digits & 1)) as usize;

    let remaining_width = inner_area.width as usize - max_offset_hex_digits - 2;
    let eight_groups = (remaining_width + 1) / 25;
    inspector.hex_view_state.line_size = eight_groups * 8;

    let mut view = Vec::new();
    if inner_area.height != 0 {
        let cursor_line = inspector.view_state.current_byte / inspector.hex_view_state.line_size;
        let mut first_line = inspector.hex_view_state.line_pos / inspector.hex_view_state.line_size;
        let total_lines = inspector.buffer.buffer.len() / inspector.hex_view_state.line_size;

        if inner_area.height <= 4 {
            first_line = first_line.clamp(
                (cursor_line + 1).saturating_sub(inner_area.height as usize),
                cursor_line,
            );
        } else {
            first_line = first_line.clamp(
                (cursor_line + 2).saturating_sub(inner_area.height as usize),
                cursor_line.saturating_sub(1),
            );
        }
        first_line = first_line.min((total_lines + 1).saturating_sub(inner_area.height as usize));
        inspector.hex_view_state.line_pos = first_line * inspector.hex_view_state.line_size;

        let text_style = Style::default();

        for (line_no, chunk) in inspector
            .buffer
            .buffer
            .chunks(inspector.hex_view_state.line_size)
            .skip(first_line)
            .take(inner_area.height as usize)
            .enumerate()
        {
            let mut line = vec![Span::styled(
                format!(
                    "{:0width$x}  ",
                    (first_line + line_no) * inspector.hex_view_state.line_size,
                    width = max_offset_hex_digits
                ),
                text_style.fg(Color::Magenta),
            )];
            for (col_no, b) in chunk.iter().enumerate() {
                let pos = (line_no + first_line) * inspector.hex_view_state.line_size + col_no;
                let style = if is_active && pos == inspector.view_state.current_byte {
                    text_style.add_modifier(Modifier::BOLD)
                } else {
                    text_style.add_modifier(Modifier::DIM)
                };

                let style = match ranges.best_match(pos) {
                    None => style.bg(Color::Black),
                    Some(RangeMatch::Outer) => style.bg(Color::DarkGray),
                    Some(RangeMatch::Inner) => style.bg(Color::Blue),
                };

                line.push(Span::styled(format!("{b:02x}"), style));
                if col_no + 1 < chunk.len() {
                    let style = match (ranges.best_match(pos), ranges.best_match(pos + 1)) {
                        (None, _) | (_, None) => style.bg(Color::Black),
                        (Some(RangeMatch::Outer), Some(_)) | (Some(_), Some(RangeMatch::Outer)) => {
                            style.bg(Color::DarkGray)
                        }
                        (Some(RangeMatch::Inner), Some(RangeMatch::Inner)) => style.bg(Color::Blue),
                    };
                    if col_no % 8 != 7 {
                        line.push(Span::styled(" ", style));
                    } else {
                        line.push(Span::styled("  ", style));
                    }
                }
            }
            view.push(Spans::from(line));
        }
    }

    let paragraph = Paragraph::new(view).block(block).wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}

fn info_area<B: Backend>(f: &mut Frame<B>, area: Rect, inspector: &mut Inspector) {
    let is_active = matches!(inspector.active_window, crate::ActiveWindow::ObjectView);
    let block = block(is_active);
    let inner_area = block.inner(area);

    let mut text = Vec::new();

    if let Some(current_line) = inspector.view_state.current_line {
        for (i, line) in inspector
            .view_state
            .lines
            .iter()
            .enumerate()
            .skip(current_line)
        {
            let style = if Some(i) == inspector.view_state.current_line {
                Style::default().add_modifier(Modifier::BOLD)
            } else {
                Style::default().add_modifier(Modifier::DIM)
            };
            text.push(Spans::from(Span::styled(line.line.clone(), style)));
        }
    }

    let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: false });
    f.render_widget(paragraph, area);
}

fn block(is_active: bool) -> Block<'static> {
    let res = Block::default().borders(Borders::ALL);
    if is_active {
        res.border_style(Style::default().fg(Color::White))
            .border_type(BorderType::Rounded)
    } else {
        res.border_style(Style::default().fg(Color::Rgb(128, 128, 128)))
            .border_type(BorderType::Rounded)
    }
}
