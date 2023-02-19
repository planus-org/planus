use planus_buffer_inspection::InspectableFlatbuffer;
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::{ActiveWindow, HexViewState, Inspector, ModalState, RangeMatch, ViewState};

const DARK_BLUE: Color = Color::Rgb(62, 103, 113);
const DARK_GREEN: Color = Color::Rgb(100, 88, 55);
const DARK_MAGENTA: Color = Color::Rgb(114, 77, 106);
const WHITE: Color = Color::Rgb(241, 242, 216);
const GREY: Color = Color::Rgb(145, 145, 133);
const BLACK: Color = Color::Rgb(0, 0, 0);

const CURSOR_STYLE: Style = Style {
    fg: Some(WHITE),
    bg: None,
    add_modifier: Modifier::UNDERLINED,
    sub_modifier: Modifier::DIM,
};

const INNER_AREA_STYLE: Style = Style {
    fg: Some(WHITE),
    bg: Some(DARK_MAGENTA),
    add_modifier: Modifier::empty(),
    sub_modifier: Modifier::empty(),
};

const OUTER_AREA_STYLE: Style = Style {
    fg: Some(WHITE),
    bg: Some(DARK_BLUE),
    add_modifier: Modifier::empty(),
    sub_modifier: Modifier::empty(),
};

const ADDRESS_STYLE: Style = Style {
    fg: Some(DARK_GREEN),
    bg: None,
    add_modifier: Modifier::empty(),
    sub_modifier: Modifier::empty(),
};

const DEFAULT_STYLE: Style = Style {
    fg: Some(GREY),
    bg: Some(BLACK),
    add_modifier: Modifier::DIM,
    sub_modifier: Modifier::empty(),
};

const EMPTY_STYLE: Style = Style {
    fg: None,
    bg: None,
    add_modifier: Modifier::empty(),
    sub_modifier: Modifier::empty(),
};

const ACTIVE_STYLE: Style = Style {
    fg: Some(WHITE),
    bg: None,
    add_modifier: Modifier::empty(),
    sub_modifier: Modifier::empty(),
};

pub fn draw<B: Backend>(f: &mut Frame<B>, inspector: &mut Inspector) {
    inspector.view_state.draw_main_ui(
        f,
        inspector.active_window,
        inspector.modal.is_some(),
        &inspector.buffer,
        &mut inspector.hex_view_state,
    );

    if let Some(modal_state) = inspector.modal.as_ref() {
        let modal_area = if let ModalState::GoToByte { .. } = modal_state {
            let mut area = centered_rect(20, 20, f.size());
            area.height = 3;
            area.width = 21;
            area
        } else {
            centered_rect(60, 60, f.size())
        };
        modal_view(f, modal_area, modal_state, inspector);
    }
}

fn modal_view<B: Backend>(
    f: &mut Frame<B>,
    area: Rect,
    modal_state: &ModalState,
    inspector: &Inspector,
) {
    f.render_widget(Clear, area);
    let paragraph = match modal_state {
        ModalState::GoToByte { input } => {
            let text = vec![
                Spans::from(vec![
                    Span::styled("0x", DEFAULT_STYLE),
                    Span::styled(input, ACTIVE_STYLE),
                ]),
                Spans::default(),
            ];
            let block = block(true, " Go to offset ");
            f.set_cursor(
                // Put cursor past the end of the input text
                area.x + input.len() as u16 + 3,
                // Move one line down, from the border to the input line
                area.y + 1,
            );
            Paragraph::new(text).block(block).wrap(Wrap { trim: false })
        }
        ModalState::XRefs { .. } => {
            let text = vec![
                Spans::from(vec![Span::styled("0x", DEFAULT_STYLE)]),
                Spans::default(),
            ];
            let block = block(true, " XRefs ");
            Paragraph::new(text).block(block).wrap(Wrap { trim: false })
        }
        ModalState::ViewHistory { index } => {
            let mut text = Vec::new();

            for (line_no, view) in inspector
                .view_stack
                .iter()
                .chain(std::iter::once(&inspector.view_state))
                .enumerate()
            {
                let byte_index = view.byte_index;
                let name = if let Some(info) = &view.info_view_data {
                    &info.lines.cur().name
                } else {
                    "no object"
                };

                let style = if *index == line_no {
                    CURSOR_STYLE
                } else {
                    DEFAULT_STYLE
                };
                text.push(Spans::from(Span::styled(
                    format!("{line_no:02} 0x{byte_index:0>8x} {name}"),
                    style,
                )));
            }

            let block = block(true, " History ");
            Paragraph::new(text).block(block).wrap(Wrap { trim: false })
        }
        ModalState::Interpretations { index } => {
            let mut text = Vec::new();

            if let Some(info) = &inspector.view_state.info_view_data {
                for (line_no, interpretation) in info.interpretations.iter().enumerate() {
                    let line_index = interpretation.lines[0];
                    let line = info.lines.get(line_index).unwrap();
                    let name = &line.name;

                    let style = if *index == line_no {
                        CURSOR_STYLE
                    } else {
                        DEFAULT_STYLE
                    };
                    text.push(Spans::from(Span::styled(
                        format!("{line_no:2} {name}"),
                        style,
                    )));
                }
            }

            let block = block(true, " Interpretations ");
            Paragraph::new(text).block(block).wrap(Wrap { trim: false })
        }
        ModalState::HelpMenu => {
            let text = vec![
                Spans::from(Span::styled("Hotkeys", DEFAULT_STYLE)),
                Spans::from(Span::styled("Arrow keys: move cursor", DEFAULT_STYLE)),
            ];

            let block = block(true, " Help ");
            Paragraph::new(text).block(block).wrap(Wrap { trim: false })
        }
    };
    f.render_widget(paragraph, area);
}

impl<'a> ViewState<'a> {
    fn draw_main_ui<B: Backend>(
        &self,
        f: &mut Frame<B>,
        active_window: ActiveWindow,
        modal_is_active: bool,
        buffer: &InspectableFlatbuffer<'a>,
        hex_view_state: &mut HexViewState,
    ) {
        use Constraint::*;

        let areas = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Length(f.size().height.saturating_sub(1)), Min(0)].as_ref())
            .split(f.size());

        let main_area = areas[0];
        let legend_area = areas[1];

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

        self.draw_object_view(
            f,
            object_area,
            matches!(active_window, ActiveWindow::ObjectView) && !modal_is_active,
        );
        self.draw_hex_view(
            f,
            hex_area,
            matches!(active_window, ActiveWindow::HexView) && !modal_is_active,
            buffer,
            hex_view_state,
        );

        f.render_widget(
            Paragraph::new("Info!").block(block(false, " Info view ")),
            info_area,
        );
        self.draw_legend_view(f, legend_area, active_window);
    }

    pub fn draw_hex_view<B: Backend>(
        &self,
        f: &mut Frame<B>,
        area: Rect,
        is_active: bool,
        buffer: &InspectableFlatbuffer<'a>,
        hex_view_state: &mut HexViewState,
    ) {
        let block = block(is_active, " Hex view ");

        let paragraph = self
            .hex_view(buffer, block.inner(area), is_active, hex_view_state)
            .block(block);
        f.render_widget(paragraph, area);
    }

    fn hex_view(
        &self,
        buffer: &InspectableFlatbuffer<'a>,
        area: Rect,
        is_active: bool,
        hex_view_state: &mut HexViewState,
    ) -> Paragraph {
        let ranges = self.hex_ranges();

        let max_offset = buffer.buffer.len().max(1) as u64;
        let max_offset_ilog2 = 63 - max_offset.leading_zeros();
        let max_offset_hex_digits = max_offset_ilog2 / 4 + 1;
        let max_offset_hex_digits = (max_offset_hex_digits + (max_offset_hex_digits & 1)) as usize;

        let remaining_width = area.width as usize - max_offset_hex_digits - 2;
        let eight_groups = (remaining_width + 1) / 25;
        hex_view_state.line_size = eight_groups * 8;

        let mut view = Vec::new();
        if area.height != 0 && hex_view_state.line_size != 0 {
            let cursor_line = self.byte_index / hex_view_state.line_size;
            let mut first_line = hex_view_state.line_pos / hex_view_state.line_size;
            let total_lines = buffer.buffer.len() / hex_view_state.line_size;

            if area.height <= 4 {
                first_line = first_line.clamp(
                    (cursor_line + 1).saturating_sub(area.height as usize),
                    cursor_line,
                );
            } else {
                first_line = first_line.clamp(
                    (cursor_line + 2).saturating_sub(area.height as usize),
                    cursor_line.saturating_sub(1),
                );
            }
            first_line = first_line.min((total_lines + 1).saturating_sub(area.height as usize));
            hex_view_state.line_pos = first_line * hex_view_state.line_size;

            for (line_no, chunk) in buffer
                .buffer
                .chunks(hex_view_state.line_size)
                .skip(first_line)
                .take(area.height as usize)
                .enumerate()
            {
                let mut line = vec![
                    Span::styled(
                        format!(
                            "{:0width$x}",
                            (first_line + line_no) * hex_view_state.line_size,
                            width = max_offset_hex_digits
                        ),
                        ADDRESS_STYLE,
                    ),
                    Span::styled("  ", EMPTY_STYLE),
                ];
                for (col_no, b) in chunk.iter().enumerate() {
                    let pos = (line_no + first_line) * hex_view_state.line_size + col_no;

                    let mut style = EMPTY_STYLE;

                    if is_active && pos == self.byte_index {
                        style = style.patch(CURSOR_STYLE);
                    }

                    match ranges.best_match(pos) {
                        Some(RangeMatch::Outer) => {
                            style = style.patch(OUTER_AREA_STYLE);
                        }
                        Some(RangeMatch::Inner) => {
                            style = style.patch(INNER_AREA_STYLE);
                        }
                        None => (),
                    }

                    line.push(Span::styled(format!("{b:02x}"), style));
                    if col_no + 1 < chunk.len() {
                        let style = match (ranges.best_match(pos), ranges.best_match(pos + 1)) {
                            (None, _) | (_, None) => EMPTY_STYLE,
                            (Some(RangeMatch::Outer), Some(_))
                            | (Some(_), Some(RangeMatch::Outer)) => OUTER_AREA_STYLE,
                            (Some(RangeMatch::Inner), Some(RangeMatch::Inner)) => INNER_AREA_STYLE,
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

        Paragraph::new(view).wrap(Wrap { trim: true })
    }

    pub fn draw_object_view<B: Backend>(&self, f: &mut Frame<B>, area: Rect, is_active: bool) {
        let block = block(is_active, " Object view ");

        let paragraph = self.object_view().block(block);
        f.render_widget(paragraph, area);
    }

    fn object_view(&self) -> Paragraph {
        let mut text = Vec::new();

        if let Some(info_view_data) = &self.info_view_data {
            for (i, line) in info_view_data
                .lines
                .iter()
                .enumerate()
                .skip(info_view_data.lines.index().saturating_sub(1))
            {
                let style = if i == info_view_data.lines.index() {
                    ACTIVE_STYLE
                } else {
                    DEFAULT_STYLE
                };
                text.push(Spans::from(Span::styled(line.line.clone(), style)));
            }
        }

        Paragraph::new(text).wrap(Wrap { trim: false })
    }

    fn legend_view(&self, active_window: ActiveWindow) -> Paragraph {
        let text = match active_window {
            ActiveWindow::ObjectView => "up/down: move cursor  ?: help menu",
            ActiveWindow::HexView => "arrow keys: move cursor",
        };
        Paragraph::new(Spans::from(Span::styled(text, DEFAULT_STYLE)))
    }

    fn draw_legend_view<B: Backend>(
        &self,
        f: &mut Frame<B>,
        area: Rect,
        active_window: ActiveWindow,
    ) {
        let paragraph = self.legend_view(active_window);
        f.render_widget(paragraph, area);
    }
}

fn block(is_active: bool, title: &'static str) -> Block<'static> {
    let res = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title_alignment(Alignment::Center)
        .title(title)
        .style(DEFAULT_STYLE);
    if is_active {
        res.border_style(ACTIVE_STYLE)
    } else {
        res.border_style(DEFAULT_STYLE)
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
