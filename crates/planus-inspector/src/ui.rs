use std::fmt::Debug;

use planus_buffer_inspection::{InspectableFlatbuffer, Object};
use planus_types::intermediate::Declarations;
use tui::{
    backend::Backend,
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Widget, Wrap},
    Frame,
};

use crate::{
    vec_with_index::VecWithIndex, ActiveWindow, HexViewState, InfoViewData, Inspector, ModalState,
    RangeMatch, ViewState,
};

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

const OFFSET_STYLE: Style = Style {
    fg: Some(GREY),
    bg: None,
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
        inspector.view_state.draw_modal_view(
            f,
            modal_area,
            modal_state,
            &inspector.hex_view_state,
            &inspector.view_stack,
            &inspector.buffer.declarations,
        );
    }
}

impl<'a> ViewState<'a> {
    fn draw_main_ui<B: Backend>(
        &mut self,
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
            .constraints([Length(top_area.width.saturating_sub(45)), Min(0)].as_ref())
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
        let block = block(is_active, " Hex view ", DEFAULT_STYLE);
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

    pub fn draw_object_view<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect, is_active: bool) {
        let block = block(is_active, " Object view ", OUTER_AREA_STYLE);
        let inner_area = block.inner(area);

        if let Some(info_view_data) = &mut self.info_view_data {
            if area.height <= 4 {
                info_view_data.first_line_shown = info_view_data.first_line_shown.clamp(
                    (info_view_data.lines.index() + 1).saturating_sub(inner_area.height as usize),
                    info_view_data.lines.index(),
                );
            } else {
                info_view_data.first_line_shown = info_view_data.first_line_shown.clamp(
                    (info_view_data.lines.index() + 2).saturating_sub(inner_area.height as usize),
                    info_view_data.lines.index().saturating_sub(1),
                );
            }
        }

        let widget = ObjectViewWidget::new(self.info_view_data.as_ref(), is_active).block(block);
        f.render_widget(widget, area);
    }

    fn legend_view(&self, _active_window: ActiveWindow) -> Paragraph {
        let text = "?: help menu   arrow keys: move cursor   enter: follow pointer   tab: cycle view focus";
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
        let block = block(false, " Info view ", DEFAULT_STYLE);

        let paragraph = self
            .info_view(self.info_view_data.as_ref(), hex_view_state, declarations)
            .block(block);
        f.render_widget(paragraph, area);
    }

    fn info_view(
        &self,
        info_view_data: Option<&InfoViewData>,
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
        if let Some(info_view_area) = &info_view_data {
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
        if let Some(info_view_area) = &info_view_data {
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

        if let Some(info_view_data) = &info_view_data {
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

    fn draw_modal_view<B: Backend>(
        &self,
        f: &mut Frame<B>,
        area: Rect,
        modal_state: &ModalState,
        hex_view_state: &HexViewState,
        view_stack: &[ViewState<'a>],
        declarations: &Declarations,
    ) {
        f.render_widget(Clear, area);
        let subareas = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);
        let left = subareas[0];
        let right = subareas[1];

        let mut info_view_data = None;

        match modal_state {
            ModalState::GoToByte { input } => {
                let text = vec![
                    Spans::from(vec![
                        Span::styled("0x", DEFAULT_STYLE),
                        Span::styled(input, ACTIVE_STYLE),
                    ]),
                    Spans::default(),
                ];
                let block = block(true, " Go to offset ", DEFAULT_STYLE);
                f.set_cursor(
                    // Put cursor past the end of the input text
                    area.x + input.len() as u16 + 3,
                    // Move one line down, from the border to the input line
                    area.y + 1,
                );
                f.render_widget(
                    Paragraph::new(text).block(block).wrap(Wrap { trim: false }),
                    area,
                );
            }
            ModalState::XRefs { .. } => {
                let text = vec![
                    Spans::from(vec![Span::styled("0x", DEFAULT_STYLE)]),
                    Spans::default(),
                ];
                let block = block(true, " XRefs ", DEFAULT_STYLE);
                f.render_widget(
                    Paragraph::new(text).block(block).wrap(Wrap { trim: false }),
                    left,
                );
            }
            ModalState::ViewHistory { index } => {
                let mut text = Vec::new();

                for (line_no, view) in view_stack.iter().chain(std::iter::once(self)).enumerate() {
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
                if *index < view_stack.len() {
                    info_view_data = view_stack.get(*index);
                } else {
                    info_view_data = Some(self);
                }

                let block = block(true, " History ", DEFAULT_STYLE);
                f.render_widget(
                    Paragraph::new(text).block(block).wrap(Wrap { trim: false }),
                    left,
                );
            }
            ModalState::HelpMenu => {
                let text = vec![
                    Spans::from(Span::styled(
                        "Tab: cycle Object/Hex view focus",
                        DEFAULT_STYLE,
                    )),
                    Spans::from(Span::styled("Arrow keys: move cursor", DEFAULT_STYLE)),
                    Spans::from(Span::styled(
                        "Enter: go to entry / follow pointer",
                        DEFAULT_STYLE,
                    )),
                    Spans::from(Span::styled(
                        "Backspace: return from entry / pointer",
                        DEFAULT_STYLE,
                    )),
                    Spans::from(Span::styled("H: open history modal", DEFAULT_STYLE)),
                    Spans::from(Span::styled("G: go to offset", DEFAULT_STYLE)),
                    Spans::from(Span::styled("I: open interpretations modal", DEFAULT_STYLE)),
                    Spans::from(Span::styled(
                        "C: cycle between interpretations",
                        DEFAULT_STYLE,
                    )),
                    Spans::from(Span::styled("Q: quit", DEFAULT_STYLE)),
                ];

                let block = block(true, " Help ", DEFAULT_STYLE);
                f.render_widget(
                    Paragraph::new(text).block(block).wrap(Wrap { trim: false }),
                    area,
                );
            }
            ModalState::TreeView { state, header } => {
                f.render_widget(
                    TreeStateWidget {
                        tree_state: state,
                        block: Some(block(true, header, DEFAULT_STYLE)),
                    },
                    left,
                );

                f.render_widget(
                    ObjectViewWidget {
                        info_view_data: state
                            .lines
                            .cur()
                            .node
                            .view_state
                            .as_ref()
                            .and_then(|v| v.info_view_data.as_ref()),
                        block: Some(block(true, " Preview ", OUTER_AREA_STYLE)),
                        is_active: false,
                    },
                    right,
                );
            }
        };

        if let Some(info_view_data) = info_view_data {
            let info_view = self.info_view(
                info_view_data.info_view_data.as_ref(),
                hex_view_state,
                declarations,
            );
            f.render_widget(info_view, right);
        }
    }
}

fn block(is_active: bool, title: &'static str, inner_style: Style) -> Block<'static> {
    let res = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title_alignment(Alignment::Center)
        .title(title)
        .style(inner_style);
    if is_active {
        res.border_style(ACTIVE_STYLE)
    } else {
        let mut style = DEFAULT_STYLE;
        if let Some(bg) = inner_style.bg {
            style = style.bg(bg);
        }
        res.border_style(style)
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

struct ObjectViewWidget<'a> {
    info_view_data: Option<&'a InfoViewData<'a>>,
    block: Option<Block<'a>>,
    is_active: bool,
}

impl<'a> ObjectViewWidget<'a> {
    pub fn new(info_view_data: Option<&'a InfoViewData<'a>>, is_active: bool) -> Self {
        Self {
            info_view_data,
            block: None,
            is_active,
        }
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
}

impl<'a> Widget for ObjectViewWidget<'a> {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        let area = match self.block.take() {
            Some(b) => {
                let inner_area = b.inner(area);
                b.render(area, buf);
                inner_area
            }
            None => area,
        };
        if area.height == 0 {
            return;
        }

        if let Some(info_view_data) = &self.info_view_data {
            let selected_line = info_view_data.lines.cur();
            let mut inner_area = area;

            inner_area.x += selected_line.indentation as u16;
            inner_area.width =
                (inner_area.width as usize).saturating_sub(selected_line.indentation) as u16;

            let first_selected_line = selected_line
                .start_line_index
                .max(info_view_data.first_line_shown)
                - info_view_data.first_line_shown;
            let last_selected_line = selected_line
                .end_line_index
                .min(info_view_data.first_line_shown + area.height as usize - 1)
                - info_view_data.first_line_shown;
            inner_area.y += first_selected_line as u16;
            inner_area.height =
                (inner_area.height as usize).saturating_sub(first_selected_line) as u16;

            inner_area.width = (inner_area.width as usize).min(selected_line.object_width) as u16;
            inner_area.height = (inner_area.height as usize)
                .min(last_selected_line - first_selected_line + 1)
                as u16;

            buf.set_style(inner_area, INNER_AREA_STYLE);

            for (line_index, (i, line)) in info_view_data
                .lines
                .iter()
                .enumerate()
                .skip(info_view_data.first_line_shown)
                .enumerate()
                .take(area.height as usize)
            {
                let style = if i == info_view_data.lines.index() && self.is_active {
                    CURSOR_STYLE
                } else {
                    EMPTY_STYLE
                };

                buf.set_stringn(
                    area.left() + line.indentation as u16,
                    area.top() + line_index as u16,
                    &line.line,
                    area.width as usize - line.indentation,
                    style,
                );

                if matches!(line.object, Object::Offset(_)) {
                    buf.set_string(
                        area.left(),
                        area.top() + line_index as u16,
                        "*",
                        OFFSET_STYLE,
                    );
                }
            }
        }
    }
}

pub struct Node<'a> {
    pub text: String,
    pub view_state: Option<ViewState<'a>>,
    pub children: Option<Box<dyn Fn() -> Vec<Node<'a>>>>,
}

impl<'a> Debug for Node<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Node")
            .field("text", &self.text)
            .field("view_state", &self.view_state)
            .finish_non_exhaustive()
    }
}

#[derive(Debug)]
pub struct TreeState<'a> {
    pub lines: VecWithIndex<TreeStateLine<'a>>,
}

impl<'a> TreeState<'a> {
    pub fn toggle_fold(&mut self) {
        let line = self.lines.cur_mut();
        match line.fold_state {
            FoldState::NoChildren => (),
            FoldState::Folded => {
                if let Some(children_gen) = &line.node.children {
                    let nodes = children_gen();
                    let new_lines = nodes
                        .into_iter()
                        .map(|node| TreeStateLine {
                            indent_level: line.indent_level + 2,
                            fold_state: if node.children.is_some() {
                                FoldState::Folded
                            } else {
                                FoldState::NoChildren
                            },
                            node,
                        })
                        .collect::<Vec<_>>();
                    line.fold_state = FoldState::Unfolded;
                    self.lines.insert(new_lines);
                } else {
                    // This should never happen, but we just ignore it
                    line.fold_state = FoldState::NoChildren;
                }
            }
            FoldState::Unfolded => {
                line.fold_state = FoldState::Folded;
                let indent_level = line.indent_level;
                self.lines.remove_while(|l| indent_level < l.indent_level);
            }
        }
    }
}

#[derive(Debug)]
pub struct TreeStateLine<'a> {
    pub indent_level: usize,
    pub node: Node<'a>,
    pub fold_state: FoldState,
}

#[derive(Copy, Clone, Debug)]
pub enum FoldState {
    NoChildren,
    Folded,
    Unfolded,
}

struct TreeStateWidget<'a, 'b> {
    pub tree_state: &'b TreeState<'a>,
    pub block: Option<Block<'a>>,
}

impl<'a, 'b> Widget for TreeStateWidget<'a, 'b> {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        let area = match self.block.take() {
            Some(b) => {
                let inner_area = b.inner(area);
                b.render(area, buf);
                inner_area
            }
            None => area,
        };
        if area.height == 0 {
            return;
        }

        for (i, line) in self.tree_state.lines.iter().enumerate() {
            let style = if i == self.tree_state.lines.index() {
                CURSOR_STYLE
            } else {
                EMPTY_STYLE
            };

            let suffix = match line.fold_state {
                FoldState::NoChildren => "",
                FoldState::Folded => " [+]",
                FoldState::Unfolded => " [-]",
            };

            buf.set_stringn(
                area.left() + 2 * line.indent_level as u16,
                area.top() + i as u16,
                format!("{}{suffix}", line.node.text),
                area.width as usize - line.indent_level,
                style,
            );
        }
    }
}
