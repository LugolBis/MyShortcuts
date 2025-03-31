use std::io;
use std::fs::OpenOptions;
use std::io::Write;

use crate::{objects::*, postgresql};
use crate::ui::{WidgetConfigurations,WidgetConnections,Common};
use crate::database::{Database,CLASSIC_SHEME,CUSTOM_SHEME,AVAILABLE_SHEME};
use crate::utils::run_bash;
use crate::{neo4j, format_config, filter_config};

use ratatui::{
    widgets::{TableState,Clear,Block,Paragraph},layout::Flex,prelude::{Constraint, Layout, Direction, Rect},
    DefaultTerminal, Frame,crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},text::{Line,Text},
    style::Color
};

pub fn main_app() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let app_result = App::new().run(&mut terminal);
    ratatui::restore();
    app_result
}

pub struct App {
    connections:WidgetConnections,
    configurations:WidgetConfigurations,
    /// The name of the Shortcut before any modification
    save: String,
    show_pop_up: (bool,usize),
    exit: bool,
}

impl App {

    pub fn new() -> Self {
        App {
            connections: WidgetConnections::from(vec![],State::Selected(TableState::new().with_selected(0).with_selected_column(0))),
            configurations: WidgetConfigurations::from(vec![],State::WasSelected(TableState::new().with_selected(0))),
            save: String::new(),
            show_pop_up: (false,0usize),
            exit: false
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            if self.save == "" {
                self.update_widgets_args();
            }
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        let layout = Layout::new(
            Direction::Horizontal,
            [Constraint::Percentage(30), Constraint::Percentage(100)],
        )
        .split(frame.area());
        self.connections.render(frame, layout[0]);
        self.configurations.render(frame, layout[1]);

        if self.show_pop_up.0 {
            let mut lines = AVAILABLE_SHEME.iter().map(|kind| Line::from(*kind)).collect::<Vec<Line>>();
            lines[self.show_pop_up.1] = Line::clone(&lines[self.show_pop_up.1]).patch_style(Color::LightCyan);
            let content = Text::from(lines);
            let block = Block::bordered().title(Line::from(" Select a kind of Shortcut " ).centered());
            let area = popup_area(frame.area(), 50, 50);
            let paragraph = Paragraph::new(content).centered().block(block);
            frame.render_widget(Clear, area);
            frame.render_widget(paragraph, area);
        }
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match (self.connections.get_state(),self.configurations.get_state(),key_event.code) {
            (State::Selected(mut ts),State::WasSelected(_),KeyCode::Up) => {
                let index = ts.selected().map_or(0, |i| if i>0 { i-1 } else { self.connections.get_values().len() });
                ts.select(Some(index));
                self.connections.set_state(State::Selected(ts));
                // ScrollBar interaction here 
            },
            (State::Selected(mut ts),State::WasSelected(_),KeyCode::Down) => {
                let index = ts.selected().map_or(0, |i| if i<self.connections.get_values().len() { i+1 } else { 0 });
                ts.select(Some(index));
                self.connections.set_state(State::Selected(ts));
                // ScrollBar interaction here 
            },
            (State::Selected(ts0),State::WasSelected(ts1),KeyCode::Right) => {
                self.switch_selected_widget(ts0, ts1, true);
            },
            (State::Selected(mut ts0),State::WasSelected(_),KeyCode::Left) => {
                if let Some(index) = ts0.selected_column() {
                    ts0.select_column(Some(index.saturating_sub(1)));
                    self.connections.set_state(State::Selected(ts0));
                }
            },
            (State::WasSelected(_),State::Selected(mut ts),KeyCode::Up) => {
                let index = ts.selected().map_or(0, |i| if i>0 { i-1 } else { self.configurations.get_values().len() });
                ts.select(Some(index));
                self.configurations.set_state(State::Selected(ts));
                // ScrollBar interaction here 
            },
            (State::WasSelected(_),State::Selected(mut ts),KeyCode::Down) => {
                let index = ts.selected().map_or(0, |i| if i<self.configurations.get_values().len() { i+1 } else { 0 });
                ts.select(Some(index));
                self.configurations.set_state(State::Selected(ts));
                // ScrollBar interaction here 
            },
            (State::WasSelected(ts0),State::Selected(ts1),KeyCode::Left) => {
                self.switch_selected_widget(ts0, ts1, false);
            }
            (State::WasSelected(_),State::Selected(mut ts1),KeyCode::Right) => {
                if let Some(index) = ts1.selected_column() {
                    if index<self.configurations.get_values().len() { ts1.select_column(Some(index.saturating_add(1))) };
                    self.configurations.set_state(State::Selected(ts1));
                }
                else {
                    ts1.select_column(Some(0));
                    self.configurations.set_state(State::Selected(ts1))
                }
            },
            (State::Selected(ts0)|State::WasSelected(ts0),State::WasSelected(_)|State::Selected(_), KeyCode::Char('o')) => {
                if let Some(connection) = self.connections.get_values().get(ts0.selected().unwrap_or(0)) {
                    self.execute_shortcut(connection.get_kind());
                }
            }
            (State::Selected(index),State::WasSelected(_), KeyCode::Char('a')) => {
                self.connections.set_state(State::WasSelected(index));
                self.show_pop_up = (true,0);
            },
            (State::WasSelected(_),State::WasSelected(_),KeyCode::Up) => self.show_pop_up = (true, self.show_pop_up.1.saturating_sub(1)),
            (State::WasSelected(_),State::WasSelected(_),KeyCode::Down) => self.show_pop_up = (true, (self.show_pop_up.1+1) % AVAILABLE_SHEME.len()),
            (State::WasSelected(index),State::WasSelected(_),KeyCode::Enter) => {
                self.add_new_shortcut();
                self.show_pop_up = (false,0);
                self.connections.set_state(State::Selected(index));
            }
            (State::Selected(index),State::WasSelected(_),KeyCode::Char('r')) => {
                if let Some(index) = index.selected() {
                    if let Some(connection) = self.connections.get_values().get(index) {
                        let _ = Database::query_write(&format!("delete from connections where name='{}';", connection.get_name()));
                    }
                }
            },
            (State::WasSelected(ts0),State::Selected(ts1),KeyCode::Char('r')) => {
                if let Some(index0) = ts0.selected() {
                    if let Some(index1) = ts1.selected() {
                        self.configurations.get_mut_values()[index1] = Configuration::from("","");
                        let query = format!("update connections set configuration='{}' where name='{}';",
                            format_config!(self.configurations.get_values()), self.connections.get_values()[index0].get_name());
                        let _ = Database::query_write(&query);
                    }
                }
            },
            (State::WasSelected(index),State::WasSelected(_), KeyCode::Char('q') | KeyCode::Esc) => {
                self.show_pop_up = (false,0);
                self.connections.set_state(State::Selected(index));
            },
            (State::Selected(mut ts0),State::WasSelected(_),KeyCode::Char('e')) => {
                if let Some(index) = ts0.selected() {
                    if let Some(connection) =  self.connections.get_values().get(index) {
                        self.save = String::clone(connection.get_name());
                        ts0.select_column(Some(1));
                        self.connections.set_state(State::Editing(ts0, 0));
                    }
                }
            },
            (State::Editing(ts0, cursor),State::WasSelected(_),KeyCode::Left) => {
                self.connections.set_state(State::Editing(ts0, cursor.saturating_sub(1)));
            },
            (State::Editing(ts0, cursor),State::WasSelected(_),KeyCode::Right) => {
                if let Some(index) = ts0.selected() {
                    if cursor<self.connections.get_values()[index].get_name().len() {
                        self.connections.set_state(State::Editing(ts0, cursor+1));
                    }
                }
            },
            (State::Editing(ts0, cursor),State::WasSelected(_),KeyCode::Char(new_char)) => {
                if let Some(index) = ts0.selected() {
                    if let Some(connection) = self.connections.get_mut_values().get_mut(index) {
                        connection.get_mut_name().insert(cursor,new_char);
                        self.connections.set_state(State::Editing(ts0, cursor+1));
                    }
                }
            },
            (State::Editing(ts0, cursor),State::WasSelected(_),KeyCode::Backspace) => {
                if let Some(index) = ts0.selected() {
                    if let Some(connection) = self.connections.get_mut_values().get_mut(index) {
                        let len_word = connection.get_name().len();
                        if cursor>0 && len_word>1 {
                            connection.get_mut_name().remove(cursor-1);
                            self.connections.set_state(State::Editing(ts0, cursor.saturating_sub(1usize)));
                        }
                    } 
                }
            },
            (State::Editing(ts0, _),State::WasSelected(_),KeyCode::Enter) => {
                if let Some(index) = ts0.selected() {
                    if let Some(connection) = self.connections.get_values().get(index) {
                        self.save_editing(String::clone(connection.get_name()),true);
                    }
                }
                self.connections.set_state(State::Selected(ts0))
            },
            (State::WasSelected(ts0),State::Selected(mut ts1),KeyCode::Char('e')) => {
                if let Some(index) = ts0.selected() {
                    if let Some(connection) = self.connections.get_values().get(index) {
                        self.save = String::clone(connection.get_name());
                        ts1.select_column(Some(1));
                        self.configurations.set_state(State::Editing(ts1, 0));
                    }
                }
            },
            (State::WasSelected(_),State::Editing(ts1, cursor),KeyCode::Left) => {
                self.configurations.set_state(State::Editing(ts1, cursor.saturating_sub(1)));
            },
            (State::WasSelected(_),State::Editing(ts1, cursor),KeyCode::Right) => {
                if let Some(index) = ts1.selected() {
                    if cursor<self.configurations.get_values()[index].get_value().len() {
                        self.configurations.set_state(State::Editing(ts1, cursor+1));
                    }
                }
            },
            (_,_,KeyCode::Char('é')) => {}
            (State::WasSelected(_),State::Editing(ts1, cursor),KeyCode::Char(new_char)) => {
                if let Some(index) = ts1.selected() {
                    if let Some(configuration) = self.configurations.get_mut_values().get_mut(index) {
                        configuration.get_mut_value().insert(cursor,new_char);
                        self.configurations.set_state(State::Editing(ts1, cursor+1));
                    }
                }
            },
            (State::WasSelected(_),State::Editing(ts1, cursor),KeyCode::Backspace) => {
                if let Some(index) = ts1.selected() {
                    if let Some(configuration) = self.configurations.get_mut_values().get_mut(index) {
                        let len_word = configuration.get_value().len();
                        if cursor>0 && len_word>1 {
                            configuration.get_mut_value().remove(cursor-1);
                            self.configurations.set_state(State::Editing(ts1, cursor.saturating_sub(1usize)));
                        }
                    } 
                }
            },
            (State::WasSelected(_),State::Editing(ts1, cursor),KeyCode::Enter) => {
                if let Some(index) = ts1.selected() {
                    if let Some(configuration) = self.configurations.get_values().get(index) {
                        self.save_editing(String::clone(configuration.get_value()),false);
                    }
                }
                self.configurations.set_state(State::Selected(ts1))
            },
            (State::Selected(_),State::WasSelected(_), KeyCode::Char('q') | KeyCode::Esc) => self.exit(),
            (State::WasSelected(_),State::Selected(_), KeyCode::Char('q') | KeyCode::Esc) => self.exit(),
            (_,_,_) => {}
        }
    }

    fn update_widgets_args(&mut self) {
        if let Ok(connections) = Database::query_read("select name,type from connections order by type;") {
            self.connections.set_values(connections.split("\n").filter(|e| *e!="" && *e!="\n")
                .map(|cnx| if let Ok(cnx)  = Connection::parse(cnx)  {cnx} else {Connection::default()}).collect::<Vec<Connection>>());
        }
        match self.connections.get_state() {
            State::Selected(ts) | State::WasSelected(ts) => {
                if let Some(connection) = self.connections.get_values().get(ts.selected().unwrap_or(0)) {
                    if let Ok(mut configurations) = Database::query_read(
                        &format!("select configuration from connections where name='{}';",connection.get_name()))
                    {
                        let new_configurations: Vec<Configuration>;
                        if *&["Neo4j","PostgreSQL"].contains(&connection.get_kind().as_str()) {
                            let configurations = configurations.split(";").map(|c| c).collect::<Vec<&str>>();
                            new_configurations = CLASSIC_SHEME.iter().enumerate()
                            .map(|(index,kind)|
                                if let Some(value) = configurations.get(index) {Configuration::from(value, *kind)}
                                else {Configuration::from("",*kind)}).collect();
                        }
                        else {
                            configurations.pop(); configurations.pop();
                            new_configurations = vec![Configuration::from(&configurations, CUSTOM_SHEME[0])];
                        }
                        self.configurations.set_values(new_configurations);
                    }
                }
            },
            State::Editing(_, _) => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn switch_selected_widget(&mut self, mut ts0: TableState, mut ts1: TableState, from_widget0: bool) {
        if from_widget0 {
            // Widget0 -> Selected -> Key Right Pressed
            match ts0.selected_column() {
                Some(index) => {
                    if index == 0 {
                        ts0.select_column(Some(index+1));
                        self.connections.set_state(State::Selected(ts0));
                    }
                    else {
                        ts0.select_column(Some(index));
                        ts1.select_column(Some(0));
                        self.connections.set_state(State::WasSelected(ts0));
                        self.configurations.set_state(State::Selected(ts1));
                    }
                }
                None => {
                    ts0.select_column(Some(0));
                    self.connections.set_state(State::Selected(ts0));
                }
            }
        }
        else {
            // Widget1 -> Selected -> Key Left Pressed
            match ts1.selected_column() {
                Some(index) => {
                    if index == 0 {
                        ts1.select_column(Some(index));
                        ts0.select_column(Some(1));
                        self.connections.set_state(State::Selected(ts0));
                        self.configurations.set_state(State::WasSelected(ts1));
                        
                    }
                    else {
                        ts1.select_column(Some(index-1));
                        self.configurations.set_state(State::Selected(ts1));
                    }
                }
                None => {
                    ts1.select_column(Some(0));
                    self.configurations.set_state(State::Selected(ts1));
                }
            }
        }
    }

    fn execute_shortcut(&self, kind:&String) {
        if let Ok(mut file) = OpenOptions::new().write(true).create(true).truncate(true).open("current_command.txt") {
            let mut command: String;
            if kind == "Neo4j" {
                command = neo4j!(filter_config!(self.configurations.get_values().iter().map(|c| c.get_value()).collect::<Vec<&String>>()));
            }
            else if kind == "PostgreSQL" {
                command = postgresql!(filter_config!(self.configurations.get_values().iter().map(|c| c.get_value()).collect::<Vec<&String>>()));
            }
            else if kind == "Custom" {
                command = String::clone(self.configurations.get_values()[0].get_value());
            }
            else {
                command = String::from("echo 'Welcome on MyShortcuts !'");
            }
            if let Ok(_) = file.write_all(command.as_bytes()) {
                let _ = run_bash();
            }
        }
    }

    fn add_new_shortcut(&self) {
        let kind = AVAILABLE_SHEME[self.show_pop_up.1];
        let total_connections = self.connections.get_values().len();
        let query: String;
        match kind {
            "Neo4j" => query = format!("insert into connections values ('Default{}','Required;Required;Required;Required','Neo4j')",total_connections),
            "PostgreSQL" => query = format!("insert into connections values ('Default{}','Required;Required;Required;Required','PostgreSQL')",total_connections),
            "Custom" => query = format!("insert into connections values ('Default{}','echo Welcome on MyShortcuts','Custom')",total_connections),
            _ => query = String::new()
        }
        let _ = Database::query_write(&query);
    }

    fn save_editing(&mut self,new_value:String,is_connection:bool) {
        let query: String;
        if is_connection {
            query = format!("update connections set name='{}' where name='{}';",new_value, self.save);
        }
        else {
            query = format!("update connections set configuration='{}' where name='{}';",format_config!(self.configurations.get_values()), self.save);
        }
        let _ = Database::query_write(&query);
        self.save = String::new();
    }
}

fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}