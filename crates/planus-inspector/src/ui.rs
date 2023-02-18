use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::{Inspector, ModalState, RangeMatch};

pub fn draw<B: Backend>(f: &mut Frame<B>, inspector: &mut Inspector) {
    use Constraint::*;
    let areas = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Length(f.size().height.saturating_sub(1)), Min(0)].as_ref())
        .split(f.size());

    let main_area = areas[0];
    let hotkey_area = areas[1];

    let areas = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Percentage(60), Percentage(40)].as_ref())
        .split(main_area);

    let top_area = areas[0];
    let hex_area = areas[1];

    let areas = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Length(top_area.width.saturating_sub(20)), Min(0)].as_ref())
        .split(top_area);

    let object_area = areas[0];
    let info_area = areas[1];

    object_view(f, object_area, inspector);
    hex_view(f, hex_area, inspector);
    f.render_widget(Paragraph::new("Info!").block(block(false)), info_area);
    f.render_widget(Paragraph::new("wut"), hotkey_area);

    if let Some(modal_state) = inspector.modal.as_ref() {
        let modal_area = centered_rect(20, 20, f.size());
        modal_view(f, modal_area, modal_state);
    }
}

fn modal_view<B: Backend>(f: &mut Frame<B>, area: Rect, modal_state: &ModalState) {
    let paragraph = match modal_state {
        ModalState::GoToByte { input } => {
            let text = vec![
                Spans::from(Span::styled(format!("Go to position"), Style::default())),
                Spans::from(Span::styled(input.clone(), Style::default())),
                Spans::default(),
            ];
            let block = block(false);
            Paragraph::new(text).block(block).wrap(Wrap { trim: false })
        }
    };
    f.render_widget(Clear, area);
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
    if inner_area.height != 0 && inspector.hex_view_state.line_size != 0 {
        let cursor_line = inspector.view_state.byte_index / inspector.hex_view_state.line_size;
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
                let style = if is_active && pos == inspector.view_state.byte_index {
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

fn object_view<B: Backend>(f: &mut Frame<B>, area: Rect, inspector: &mut Inspector) {
    let is_active = matches!(inspector.active_window, crate::ActiveWindow::ObjectView);
    let block = block(is_active);

    let mut text = Vec::new();

    if let Some(info_view_data) = &mut inspector.view_state.info_view_data {
        for (i, line) in info_view_data
            .lines
            .iter()
            .enumerate()
            .skip(info_view_data.lines.index().saturating_sub(1))
        {
            let style = if i == info_view_data.lines.index() {
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

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}
