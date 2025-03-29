use ratatui::{
    prelude::Constraint, buffer::Buffer, crossterm::event::KeyCode, layout::Rect, style::{Color, Stylize, Style,Modifier},Frame,
    symbols::border, text::{Line, Span, Text}, widgets::{Block, Paragraph, Widget, Cell, Row, Table, TableState,HighlightSpacing}
};
use unicode_width::UnicodeWidthStr;
use crate::objects::*;

const ROW_PEER: Color = Color::DarkGray;
const ROW_ODD: Color = Color::Black;
const ROW_SELECTED: Color = Color::Blue;
const ROW_FONT: Color = Color::Cyan;
const COLUMN_SELECTED: Color = Color::Blue;
const CELL_SELECTED: Color = Color::LightBlue;
const HEADER: Color = Color::LightRed;

#[derive(Debug)]
pub struct WidgetConnections {
    values: Vec<Connection>,
    state: State
}

#[derive(Debug)]
pub struct WidgetConfigurations {
    values: Vec<Configuration>,
    state: State
}

impl WidgetConnections {
    pub fn from(values:Vec<Connection>,state:State) -> Self {
        WidgetConnections { values, state }
    }

    pub fn get_values(&self) -> &Vec<Connection> {
        &self.values
    }

    pub fn get_mut_values(&mut self) -> &mut Vec<Connection> {
        &mut self.values
    }

    pub fn get_state(&self) -> State {
        State::clone(&self.state)
    }

    pub fn set_state(&mut self, state:State) {
        self.state = state
    }

    pub fn set_values(&mut self,values:Vec<Connection>) {
        if values.len()>0 { self.values = values }
        else { self.values = vec![Connection::default()] }
    }

    fn constraint_len_calculator(&self) -> (u16, u16) {
        let name_len = self.values.iter().map(|cnx| cnx.get_name().width()).max().unwrap_or(0);
        let kind_len = self.values.iter().map(|cnx| cnx.get_kind().width()).max().unwrap_or(0);
        #[allow(clippy::cast_possible_truncation)]
        (name_len.max(5) as u16, kind_len.max(5) as u16)
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let header_style = Style::default()
            .add_modifier(Modifier::BOLD)
            .fg(HEADER);
        let selected_row_style = Style::default()
            .add_modifier(Modifier::REVERSED)
            .fg(ROW_SELECTED);
        let selected_col_style = Style::default().fg(COLUMN_SELECTED);

        let selected_cell_style = Style::default()
            .add_modifier(Modifier::BOLD)
            .fg(CELL_SELECTED);
        let header = ["Kind", "Name"]
            .into_iter()
            .map(Cell::from)
            .collect::<Row>()
            .style(header_style)
            .height(1);
        let rows = self.values.iter().enumerate().map(|(i, connection)| {
            let color = match i % 2 {
                0 => ROW_PEER,
                _ => ROW_ODD,
            };
            let item = [connection.get_kind(), connection.get_name()];
            item.into_iter()
                .map(|content| Cell::from(Text::from(format!("\n{content}\n"))))
                .collect::<Row>()
                .style(Style::new().fg(ROW_FONT).bg(color))
                .height(4)
        });
        let bar = " █ ";
        let len_constraints = self.constraint_len_calculator();
        let block = Block::bordered().border_set(border::ROUNDED);
        let t = Table::new(
            rows,
            [
                Constraint::Length(len_constraints.0 + 1),
                Constraint::Min(len_constraints.1 + 1),
                Constraint::Min(len_constraints.1),
            ],)
            .header(header)
            .row_highlight_style(selected_row_style)
            .column_highlight_style(selected_col_style)
            .cell_highlight_style(selected_cell_style)
            .highlight_symbol(Text::from(vec![
                "".into(),
                bar.into(),
                bar.into(),
                "".into(),
            ]))
            .bg(Color::Black)
            .highlight_spacing(HighlightSpacing::Always)
            .block(block);
        match self.get_state() {
            State::Selected(mut ts) | State::WasSelected(mut ts) => frame.render_stateful_widget(t, area, &mut ts),
            State::Editing(mut ts, _) => frame.render_stateful_widget(t, area, &mut ts)
        }
    }
}

impl WidgetConfigurations {
    pub fn from(values:Vec<Configuration>,state:State) -> Self {
        WidgetConfigurations { values, state }
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

    pub fn set_state(&mut self, state:State) {
        self.state = state
    }

    pub fn set_values(&mut self, values:Vec<Configuration>) {
        if values.len()>0 { self.values = values }
        else { self.values = vec![Configuration::default()] }
    }

    fn constraint_len_calculator(&self) -> (u16, u16) {
        let value_len = self.get_values().iter().map(|cnx| cnx.get_value().width()).max().unwrap_or(4);
        let kind_len = self.get_values().iter().map(|cnx| cnx.get_kind().width()).max().unwrap_or(4);
        #[allow(clippy::cast_possible_truncation)]
        (value_len as u16, kind_len as u16)
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let header_style = Style::default()
            .add_modifier(Modifier::BOLD)
            .fg(HEADER);
        let selected_row_style = Style::default()
            .add_modifier(Modifier::REVERSED)
            .fg(ROW_SELECTED);
        let selected_col_style = Style::default().fg(COLUMN_SELECTED);

        let selected_cell_style = Style::default()
            .add_modifier(Modifier::BOLD)
            .fg(CELL_SELECTED);
        let header = ["Kind", "Value"]
            .into_iter()
            .map(Cell::from)
            .collect::<Row>()
            .style(header_style)
            .height(1);
        let rows = self.values.iter().enumerate().map(|(i, configuration)| {
            let color = match i % 2 {
                0 => ROW_PEER,
                _ => ROW_ODD,
            };
            let item = [configuration.get_kind(), configuration.get_value()];
            item.into_iter()
                .map(|content| Cell::from(Text::from(format!("\n{content}\n"))))
                .collect::<Row>()
                .style(Style::new().fg(ROW_FONT).bg(color))
                .height(4)
        });
        let bar = " █ ";
        let len_constraints = self.constraint_len_calculator();
        let block = Block::bordered().border_set(border::ROUNDED);
        let t = Table::new(
            rows,
            [
                Constraint::Length(len_constraints.0 + 1),
                Constraint::Min(len_constraints.1 + 1),
                Constraint::Min(len_constraints.1),
            ],)
            .header(header)
            .row_highlight_style(selected_row_style)
            .column_highlight_style(selected_col_style)
            .cell_highlight_style(selected_cell_style)
            .highlight_symbol(Text::from(vec![
                "".into(),
                bar.into(),
                bar.into(),
                "".into(),
            ]))
            .bg(Color::Black)
            .highlight_spacing(HighlightSpacing::Always)
            .block(block);
        match self.get_state() {
            State::Selected(mut ts) | State::WasSelected(mut ts) => frame.render_stateful_widget(t, area, &mut ts),
            State::Editing(mut ts, _) => frame.render_stateful_widget(t, area, &mut ts)
        }
    }
}