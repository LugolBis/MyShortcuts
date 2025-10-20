use crate::database::AVAILABLE_SHEME;
use crate::objects::*;
use ratatui::{
    Frame,
    layout::Rect,
    prelude::Constraint,
    style::{Color, Modifier, Style, Stylize},
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Cell, HighlightSpacing, Paragraph, Row, Table, TableState},
};
use unicode_width::UnicodeWidthStr;

const ROW_BG: Color = Color::Black;
const ROW_SELECTED: Color = Color::Rgb(155, 175, 223);
const ROW_FONT: Color = Color::Rgb(227, 151, 143);
const COLUMN_SELECTED: Color = Color::Rgb(101, 175, 223);
const CELL_SELECTED: Color = Color::Rgb(155, 175, 223);
const HEADER: Color = Color::Rgb(218, 93, 72);

const ROW_WAS_SELECTED: Color = Color::Rgb(117, 146, 206);
const COLUMN_WAS_SELECTED: Color = Color::Rgb(117, 146, 206);
const CELL_EDITING: Color = Color::Rgb(151, 192, 80);

#[derive(Debug)]
pub struct WidgetShortcuts {
    values: Vec<Shortcut>,
    state: State,
}

#[derive(Debug)]
pub struct WidgetConfigurations {
    values: Vec<Configuration>,
    state: State,
    hidde: bool,
}

impl WidgetShortcuts {
    pub fn from(values: Vec<Shortcut>, state: State) -> Self {
        WidgetShortcuts { values, state }
    }

    pub fn get_values(&self) -> &Vec<Shortcut> {
        &self.values
    }

    pub fn get_mut_values(&mut self) -> &mut Vec<Shortcut> {
        &mut self.values
    }

    pub fn get_state(&self) -> State {
        State::clone(&self.state)
    }

    pub fn set_state(&mut self, state: State) {
        self.state = state
    }

    pub fn set_values(&mut self, values: Vec<Shortcut>) {
        if !values.is_empty() {
            self.values = values
        } else {
            self.values = vec![Shortcut::default()]
        }
    }
}

impl WidgetConfigurations {
    pub fn from(values: Vec<Configuration>, state: State) -> Self {
        WidgetConfigurations {
            values,
            state,
            hidde: true,
        }
    }

    pub fn get_values(&self) -> &Vec<Configuration> {
        &self.values
    }

    pub fn get_mut_values(&mut self) -> &mut Vec<Configuration> {
        &mut self.values
    }

    pub fn get_state(&self) -> State {
        State::clone(&self.state)
    }

    pub fn set_state(&mut self, state: State) {
        self.state = state
    }

    pub fn set_values(&mut self, values: Vec<Configuration>) {
        if !values.is_empty() {
            self.values = values
        } else {
            self.values = vec![Configuration::default()]
        }
    }

    pub fn hidde(&mut self) {
        self.hidde = !self.hidde
    }
}

impl Common for WidgetShortcuts {
    fn get_header(&self) -> [&str; 2] {
        [" Kind ", " Name "]
    }
    fn get_title(&self) -> &str {
        " Shortcuts "
    }
    fn get_common_state(&self) -> State {
        State::clone(&self.state)
    }

    fn constraint_len_calculator(&self) -> (u16, u16) {
        let name_len = self
            .values
            .iter()
            .map(|cnx| cnx.get_name().width())
            .max()
            .unwrap_or(0)
            + 1;
        let kind_len = self
            .values
            .iter()
            .map(|cnx| cnx.get_kind().width())
            .max()
            .unwrap_or(0)
            + 1;
        #[allow(clippy::cast_possible_truncation)]
        (kind_len.max(6) as u16, name_len.max(5) as u16)
    }

    fn get_rows(&self) -> Vec<ratatui::widgets::Row<'_>> {
        self.values
            .iter()
            .map(|shortcut| {
                let item = [shortcut.get_kind(), shortcut.get_name()];
                item.into_iter()
                    .map(|content| Cell::from(Text::from(format!("\n{content}\n"))))
                    .collect::<Row>()
                    .style(Style::new().fg(ROW_FONT).bg(ROW_BG))
                    .height(3)
            })
            .collect()
    }

    fn get_editing_value(&self, index: usize) -> [String; 2] {
        let config = &self.get_values()[index];
        [
            String::clone(config.get_kind()),
            String::clone(config.get_name()),
        ]
    }
}

impl Common for WidgetConfigurations {
    fn get_header(&self) -> [&str; 2] {
        [" Property ", " Value "]
    }
    fn get_title(&self) -> &str {
        " Configurations "
    }
    fn get_common_state(&self) -> State {
        State::clone(&self.state)
    }

    fn constraint_len_calculator(&self) -> (u16, u16) {
        let value_len = self
            .get_values()
            .iter()
            .map(|cnx| cnx.get_value().width())
            .max()
            .unwrap_or(4)
            + 1;
        let kind_len = self
            .get_values()
            .iter()
            .map(|cnx| cnx.get_kind().width())
            .max()
            .unwrap_or(4)
            + 1;
        #[allow(clippy::cast_possible_truncation)]
        (kind_len.max(9) as u16, value_len.max(6) as u16)
    }

    fn get_rows(&self) -> Vec<Row<'_>> {
        if self.hidde {
            self.values
                .iter()
                .map(|configuration| {
                    let mut item = [configuration.get_kind(), configuration.get_value()];
                    let binding = "*".repeat(item[1].len());
                    item[1] = &binding;
                    item.into_iter()
                        .map(|content| Cell::from(Text::from(format!("\n{content}\n"))))
                        .collect::<Row>()
                        .style(Style::new().fg(ROW_FONT).bg(ROW_BG))
                        .height(3)
                })
                .collect()
        } else {
            self.values
                .iter()
                .map(|configuration| {
                    let item = [configuration.get_kind(), configuration.get_value()];
                    item.into_iter()
                        .map(|content| Cell::from(Text::from(format!("\n{content}\n"))))
                        .collect::<Row>()
                        .style(Style::new().fg(ROW_FONT).bg(ROW_BG))
                        .height(3)
                })
                .collect()
        }
    }

    fn get_editing_value(&self, index: usize) -> [String; 2] {
        let config = &self.get_values()[index];
        [
            String::clone(config.get_kind()),
            String::clone(config.get_value()),
        ]
    }
}

pub trait Common {
    fn get_header(&self) -> [&str; 2];

    fn get_title(&self) -> &str;

    fn get_common_state(&self) -> State;

    fn get_rows(&self) -> Vec<ratatui::widgets::Row<'_>>;

    fn get_editing_value(&self, index: usize) -> [String; 2];

    fn constraint_len_calculator(&self) -> (u16, u16);

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let header = self
            .get_header()
            .into_iter()
            .map(Cell::from)
            .collect::<Row>()
            .style(Style::default().add_modifier(Modifier::BOLD).fg(HEADER))
            .height(2);

        let selected_row_style: Style;
        let selected_col_style: Style;
        let selected_cell_style: Style;

        let mut rows: Vec<ratatui::widgets::Row<'_>>;
        let len_constraints = self.constraint_len_calculator();

        match self.get_common_state() {
            State::Selected(_) => {
                selected_row_style = Style::default()
                    .add_modifier(Modifier::REVERSED)
                    .fg(ROW_SELECTED);
                selected_col_style = Style::default().fg(COLUMN_SELECTED);
                selected_cell_style = Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(CELL_SELECTED);
                rows = self.get_rows();
            }
            State::WasSelected(_) => {
                selected_row_style = Style::default()
                    .add_modifier(Modifier::REVERSED)
                    .fg(ROW_WAS_SELECTED);
                selected_col_style = Style::default().fg(COLUMN_WAS_SELECTED);
                selected_cell_style = Style::default();
                rows = self.get_rows();
            }
            State::Editing(ts, input) => {
                selected_row_style = Style::default()
                    .add_modifier(Modifier::REVERSED)
                    .fg(ROW_WAS_SELECTED);
                selected_col_style = Style::default().fg(COLUMN_SELECTED);
                selected_cell_style = Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(CELL_EDITING);
                rows = self.get_rows();

                if let Some(index) = ts.selected() {
                    // Updating the input value in the target row
                    let mut editing_value = self.get_editing_value(index);
                    editing_value[1] = input.value().into();
                    rows[index] = editing_value
                        .into_iter()
                        .map(|content| Cell::from(Text::from(format!("\n{content}\n"))))
                        .collect::<Row>()
                        .style(Style::new().bg(Color::Black))
                        .height(3);

                    let width = area.width.max(2) - 3;
                    let scroll = input.visual_scroll(width as usize);
                    let cursor_pos = (input.visual_cursor().max(scroll) - scroll + 1) as u16;

                    let (current_x, current_y) =
                        calculate_cursor_position(area, cursor_pos, len_constraints.0, index);
                    frame.set_cursor_position((current_x, current_y));
                }
            }
        }

        let block = Block::bordered()
            .border_set(border::ROUNDED)
            .title_top(Line::from(self.get_title()).centered())
            .title_style(Style::default().add_modifier(Modifier::BOLD).fg(HEADER));
        let t = Table::new(
            rows,
            [
                Constraint::Length(len_constraints.0 + 1),
                Constraint::Length(len_constraints.1 + 1),
            ],
        )
        .header(header)
        .row_highlight_style(selected_row_style)
        .column_highlight_style(selected_col_style)
        .cell_highlight_style(selected_cell_style)
        .highlight_symbol(Text::from(vec!["".into(), " █ ".into()]))
        .bg(Color::Black)
        .fg(ROW_FONT)
        .highlight_spacing(HighlightSpacing::Always)
        .block(block);
        match self.get_common_state() {
            State::Selected(mut ts) | State::WasSelected(mut ts) | State::Editing(mut ts, _) => {
                frame.render_stateful_widget(t, area, &mut ts)
            }
        }
    }
}

fn calculate_cursor_position(
    area: Rect,
    cursor: u16,
    left_constraint: u16,
    index: usize,
) -> (u16, u16) {
    let x = area.x + cursor + left_constraint + 5u16;
    let index = if index > 11 { 11_u16 } else { index as u16 };
    let y = area.y + 2u16 * (index + 1) + (index) + 2u16;
    (x, y)
}

pub fn render_pop_up(frame: &mut Frame, index: usize, area: Rect) {
    let mut rows: Vec<Row<'_>> = AVAILABLE_SHEME
        .iter()
        .map(|kind| {
            let item = (*kind).to_string();
            Row::default()
                .cells(vec![Line::from(item).centered()])
                .style(Style::new().fg(ROW_FONT).bg(ROW_BG))
                .height(2)
        })
        .collect();

    rows[index] = Row::clone(&rows[index])
        .style(Style::new().bg(Color::Black).fg(CELL_SELECTED))
        .height(2);

    let block = Block::bordered()
        .border_set(border::ROUNDED)
        .title(Line::from(" Select a kind of Shortcut ").centered())
        .title_style(Style::default().add_modifier(Modifier::BOLD).fg(HEADER))
        .style(Style::default().fg(ROW_FONT).bg(Color::Black));

    let t = Table::new(rows, [Constraint::Percentage(100), Constraint::Length(0)])
        .bg(Color::Black)
        .fg(ROW_FONT)
        .block(block);

    let mut ts = TableState::default();
    ts.select(Some(index));
    frame.render_stateful_widget(t, area, &mut ts)
}

pub fn render_help(frame: &mut Frame, area: Rect) {
    let title = Line::from(" Help command ".bold());
    let lines = vec![
        Line::from(vec![
            " Select : ".into(),
            "[Up]".light_cyan(),
            " | ".into(),
            "[Down]".light_cyan(),
            " | ".into(),
            "[Left]".light_cyan(),
            " | ".into(),
            "[Right]".light_cyan(),
        ]),
        Line::from(vec![" Add new shortcut : ".into(), "[a] ".light_cyan()]),
        Line::from(vec![
            " Remove shortcut/config : ".into(),
            "[r] ".light_cyan(),
        ]),
        Line::from(vec![
            " Open the selected shortcut : ".into(),
            "[o] ".light_cyan(),
        ]),
        Line::from(vec![
            " Edit shortcut/config : ".into(),
            "[e] ".light_cyan(),
        ]),
        Line::from(vec![" Save changes : ".into(), "[Enter] ".light_cyan()]),
        Line::from(vec![
            " Hidde/Show configs : ".into(),
            "[h] ".light_cyan(),
        ]),
        Line::from(vec![
            " Quit : ".into(),
            "[q]".light_cyan(),
            " | ".into(),
            "[Esc] ".light_cyan(),
        ]),
    ];

    let block = Block::bordered()
        .title(title.centered())
        .title_style(Style::default().add_modifier(Modifier::BOLD).fg(HEADER))
        .bg(Color::Black)
        .fg(ROW_FONT)
        .border_set(border::ROUNDED);

    let p = Paragraph::new(Text::from(lines).style(CELL_SELECTED)).block(block);
    frame.render_widget(p, area);
}
