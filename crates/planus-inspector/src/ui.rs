use planus_buffer_inspection::InspectableFlatbuffer;
use planus_types::intermediate::Declarations;
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
const RED: Color = Color::Red;

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

const ALERT_STYLE: Style = Style {
    fg: Some(RED),
    bg: None,
    add_modifier: Modifier::BOLD,
    sub_modifier: Modifier::DIM,
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
        modal_view(
            f,
            modal_area,
            modal_state,
            inspector,
            &inspector.hex_view_state,
        );
    }
}

fn modal_view<B: Backend>(
    f: &mut Frame<B>,
    area: Rect,
    modal_state: &ModalState,
    inspector: &Inspector,
    hex_view_state: &HexViewState,
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
                    format!(
                        "{byte_index:0width$x} {name}",
                        width = hex_view_state.max_offset_hex_digits
                    ),
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
                    let root_object_index = interpretation.root_object_index;
                    let root_object = inspector
                        .object_mapping
                        .root_objects
                        .get_index(root_object_index);
                    let name = root_object
                        .unwrap()
                        .0
                        .type_name(&inspector.buffer.declarations);

                    let style = if *index == line_no {
                        CURSOR_STYLE
                    } else {
                        DEFAULT_STYLE
                    };
                    text.push(Spans::from(Span::styled(
                        format!("{line_no:2<} {name}"),
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
            .constraints([Length(top_area.width.saturating_sub(40)), Min(0)].as_ref())
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
        self.draw_info_view(f, info_area, hex_view_state, &buffer.declarations);
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
        let inner_area = block.inner(area);

        hex_view_state.update_for_area(buffer.buffer.len(), self.byte_index, inner_area);

        let paragraph = self
            .hex_view(buffer, inner_area, is_active, hex_view_state)
            .block(block);
        f.render_widget(paragraph, area);
    }

    fn hex_view(
        &self,
        buffer: &InspectableFlatbuffer<'a>,
        area: Rect,
        is_active: bool,
        hex_view_state: &HexViewState,
    ) -> Paragraph<'_> {
        let mut view = Vec::new();
        if area.height != 0 && hex_view_state.line_size != 0 {
            let first_line = hex_view_state.line_pos / hex_view_state.line_size;
            let ranges = self.hex_ranges();

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
                            width = hex_view_state.max_offset_hex_digits
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

        Paragraph::new(view)
    }

    pub fn draw_object_view<B: Backend>(&self, f: &mut Frame<B>, area: Rect, is_active: bool) {
        let block = block(is_active, " Object view ");

        let paragraph = self.object_view().block(block);
        f.render_widget(paragraph, area);
    }

    fn object_view(&self) -> Paragraph<'_> {
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
                text.push(Spans::from(Span::styled(line.line.as_str(), style)));
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

    pub fn draw_info_view<B: Backend>(
        &self,
        f: &mut Frame<B>,
        area: Rect,
        hex_view_state: &HexViewState,
        declarations: &Declarations,
    ) {
        let block = block(false, " Info view ");

        let paragraph = self.info_view(hex_view_state, declarations).block(block);
        f.render_widget(paragraph, area);
    }

    fn info_view(
        &self,
        hex_view_state: &HexViewState,
        declarations: &Declarations,
    ) -> Paragraph<'_> {
        let mut text = Vec::new();

        text.push(Spans::from(vec![
            Span::styled("Cursor", CURSOR_STYLE),
            Span::raw(format!(
                " {:0width$x}",
                self.byte_index,
                width = hex_view_state.max_offset_hex_digits
            )),
        ]));
        if let Some(info_view_area) = &self.info_view_data {
            let line = info_view_area.lines.cur();
            text.push(Spans::from(vec![
                Span::styled("Inner", INNER_AREA_STYLE),
                Span::raw(format!(
                    "  {:0width$x}-{:0width$x}",
                    line.start,
                    line.end - 1,
                    width = hex_view_state.max_offset_hex_digits
                )),
            ]));

            text.push(Spans::from(Span::raw(format!(
                "  {}",
                line.object.type_name(declarations)
            ))));
        } else {
            text.push(Spans::from(Span::styled("Inner", INNER_AREA_STYLE)));
            text.push(Spans::from(Span::raw("  -")));
        }
        if let Some(info_view_area) = &self.info_view_data {
            let line = &info_view_area.lines[0];
            text.push(Spans::from(vec![
                Span::styled("Outer", OUTER_AREA_STYLE),
                Span::raw(format!(
                    "  {:0width$x}-{:0width$x}",
                    line.start,
                    line.end - 1,
                    width = hex_view_state.max_offset_hex_digits
                )),
            ]));
            text.push(Spans::from(Span::raw(format!(
                "  {}",
                line.object.type_name(declarations)
            ))));
        } else {
            text.push(Spans::from(Span::styled("Outer", OUTER_AREA_STYLE)));
            text.push(Spans::from(Span::raw("  -")));
        }

        if let Some(info_view_data) = &self.info_view_data {
            if info_view_data.interpretations.len() > 1 {
                text.push(Spans::from(Span::raw("")));
                text.push(Spans::from(Span::styled(
                    format!(
                        "Interpretation: {}/{}",
                        info_view_data.interpretations.index() + 1,
                        info_view_data.interpretations.len()
                    ),
                    ALERT_STYLE,
                )));
                text.push(Spans::from(Span::raw("[c]: Cycle interpretations")));
                text.push(Spans::from(Span::raw("[i]: Pick interpretation")));
            }
        }

        Paragraph::new(text).wrap(Wrap { trim: false })
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
