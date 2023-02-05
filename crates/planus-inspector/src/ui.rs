use planus_buffer_inspection::object_info::ObjectName;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
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

    let range_colors = [Color::Blue, Color::Cyan];
    let mut ranges = Vec::new();
    if let Some(current_line) = inspector.view_state.current_line {
        ranges.push(
            inspector.view_state.lines[current_line].start as usize
                ..inspector.view_state.lines[current_line].end as usize,
        );
    }
    ranges.push(
        inspector.view_state.lines[0].start as usize..inspector.view_state.lines[0].end as usize,
    );

    // TODO: make lines fill entire box instead of being 16 bytes
    let mut view = Vec::new();
    let skipped_lines = inspector.view_state.current_byte / HEX_LINE_SIZE;
    let text_style = Style::default();

    for (line_no, chunk) in inspector
        .buffer
        .buffer
        .chunks(HEX_LINE_SIZE)
        .skip(skipped_lines)
        .take(100)
        .enumerate()
    {
        let mut line = vec![Span::styled(
            format!("{:06x}  ", (skipped_lines + line_no) * 16),
            text_style.fg(Color::Rgb(128, 128, 128)),
        )];
        for (col_no, b) in chunk.iter().enumerate() {
            let pos = (line_no + skipped_lines) * HEX_LINE_SIZE + col_no;
            let style = if is_active && pos == inspector.view_state.current_byte {
                text_style.bg(Color::White).fg(Color::Black)
            } else {
                if let Some(i) = ranges.iter().position(|r| r.contains(&pos)) {
                    text_style.bg(range_colors.get(i).cloned().unwrap_or(Color::Black))
                } else {
                    text_style
                }
            };
            line.push(Span::styled(format!("{b:02x} "), style));
        }
        view.push(Spans::from(line));
    }

    let block = block(is_active);
    let paragraph = Paragraph::new(view).block(block).wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}

fn info_area<B: Backend>(f: &mut Frame<B>, area: Rect, inspector: &mut Inspector) {
    let is_active = matches!(inspector.active_window, crate::ActiveWindow::ObjectView);

    let mut text = Vec::new();

    for (i, line) in inspector.view_state.lines.iter().enumerate().skip(
        inspector
            .view_state
            .current_line
            .unwrap_or(0)
            .saturating_sub(usize::from(area.height).saturating_sub(8)),
    ) {
        let style = if Some(i) == inspector.view_state.current_line {
            Style::default().add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        text.push(Spans::from(Span::styled(line.line.clone(), style)));
    }

    let block = block(is_active);
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
