use std::io;
use std::fs::OpenOptions;
use std::io::Write;

use crate::ui::{WidgetConfigurations,WidgetConnections,Common,render_pop_up};

use crate::objects::*;
use crate::database::{Database, AVAILABLE_SHEME, CLASSIC_SHEME, CUSTOM_SHEME, FILE_SCHEME, MONGODB_SCHEME, REDIS_SCHEME, SOCKET_SCHEME};
use crate::utils::*;
use crate::{format_config, filter_config};

use ratatui::{
    widgets::TableState,prelude::{Constraint, Layout, Direction},
    DefaultTerminal, Frame,crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind}
};
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;

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
            self.update_widgets_args();
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
            render_pop_up(frame, self.show_pop_up.1);
        }
    }

    fn handle_events(&mut self) -> io::Result<()> {
        let event = event::read()?;
        if let Event::Key(key) = event {
            match (self.connections.get_state(),self.configurations.get_state()) {
                (State::Editing(ts0, input),State::WasSelected(_)) if key.code == KeyCode::Enter => {
                    self.save_editing(input.value().into(), true);
                    self.connections.set_state(State::Selected(ts0));
                },
                (State::WasSelected(_),State::Editing(ts1, input)) if key.code == KeyCode::Enter => {
                    self.save_editing(input.value().into(), false);
                    self.configurations.set_state(State::Selected(ts1));
                },
                (State::Editing(ts0, input),_) => {
                    let mut new_input = input;
                    new_input.handle_event(&event);
                    self.connections.set_state(State::Editing(ts0, new_input));
                }
                (_, State::Editing(ts1, input)) => {
                    let mut new_input = input;
                    new_input.handle_event(&event);
                    self.configurations.set_state(State::Editing(ts1, new_input));
                },
                _ if key.kind == KeyEventKind::Press => {
                    self.handle_key_event(key);
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match (self.connections.get_state(),self.configurations.get_state(),key_event.code) {
            (State::Selected(mut ts),State::WasSelected(_),KeyCode::Up) => {
                let index = ts.selected().map_or(0, |i| if i>0 { i-1 } else { self.connections.get_values().len()-1 });
                ts.select(Some(index));
                self.connections.set_state(State::Selected(ts));
                // ScrollBar interaction here 
            },
            (State::Selected(mut ts),State::WasSelected(_),KeyCode::Down) => {
                let index = ts.selected().map_or(0, |i| if i<self.connections.get_values().len()-1 { i+1 } else { 0 });
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
                let index = ts.selected().map_or(0, |i| if i>0 { i-1 } else { self.configurations.get_values().len()-1 });
                ts.select(Some(index));
                self.configurations.set_state(State::Selected(ts));
                // ScrollBar interaction here 
            },
            (State::WasSelected(_),State::Selected(mut ts),KeyCode::Down) => {
                let index = ts.selected().map_or(0, |i| if i<self.configurations.get_values().len()-1 { i+1 } else { 0 });
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
            (State::WasSelected(index), State::WasSelected(_), KeyCode::Char('q') | KeyCode::Esc) => {
                self.show_pop_up = (false,0);
                self.connections.set_state(State::Selected(index));
            },
            (State::Selected(mut ts0), State::WasSelected(_), KeyCode::Char('e')) => {
                if let Some(index) = ts0.selected() {
                    if let Some(connection) =  self.connections.get_values().get(index) {
                        self.save = String::clone(connection.get_name());
                        ts0.select_column(Some(1));
                        self.connections.set_state(State::Editing(ts0, Input::with_value(Input::default(), String::clone(connection.get_name()))));
                    }
                }
            },
            (State::WasSelected(ts0), State::Selected(mut ts1), KeyCode::Char('e')) => {
                match (ts0.selected(), ts1.selected()) {
                    (Some(index0),Some(index1)) => {
                        match (self.connections.get_values().get(index0), self.configurations.get_values().get(index1)) {
                            (Some(connection), Some(configuration)) => {
                                self.save = String::clone(connection.get_name());
                                ts1.select_column(Some(1));
                                self.configurations.set_state(State::Editing(ts1, Input::with_value(
                                    Input::default(), String::clone(configuration.get_value()))));
                            }
                            (Some(connection), None) => {
                                if let Some(configuration) = self.configurations.get_values().get(0usize) {
                                    self.save = String::clone(connection.get_name());
                                    ts1.select_column(Some(1));
                                    ts1.select(Some(0));
                                    self.configurations.set_state(State::Editing(ts1, Input::with_value(
                                        Input::default(), String::clone(configuration.get_value()))));
                                }
                            }
                            _ => {}
                        }
                    },
                    _ => {}
                }
            },
            (State::Selected(_),State::WasSelected(_), KeyCode::Char('q') | KeyCode::Esc) => self.exit(),
            (State::WasSelected(_),State::Selected(_), KeyCode::Char('q') | KeyCode::Esc) => self.exit(),
            (_,_,_) => {}
        }
    }

    fn update_widgets_args(&mut self) {
        match self.connections.get_state() {
            State::Editing(ts0, input) => {
                if let Some(index) = ts0.selected() {
                    if let Some(connection) = self.connections.get_mut_values().get_mut(index) {
                        connection.get_mut_name().clone_from(&String::from(input.value()));
                        return;
                    }
                }
            },
            _ => {
                if let Ok(connections) = Database::query_read("select name,type from connections order by type;") {
                    self.connections.set_values(connections.split("\n").filter(|e| *e!="" && *e!="\n")
                        .map(|cnx| if let Ok(cnx)  = Connection::parse(cnx)  {cnx} else {Connection::default()}).collect::<Vec<Connection>>());
                }
            }
        }
        
        match (self.connections.get_state(), self.configurations.get_state()) {
            (State::WasSelected(_), State::Editing(ts1, input)) => {
                if let Some(index) = ts1.selected() {
                    if let Some(configuration) = self.configurations.get_mut_values().get_mut(index) {
                        configuration.get_mut_value().clone_from(&String::from(input.value()));
                    }
                }
            },
            (State::Selected(ts) | State::WasSelected(ts), State::Selected(_) | State::WasSelected(_)) => {
                if let Some(connection) = self.connections.get_values().get(ts.selected().unwrap_or(0)) {
                    if let Ok(configurations) = Database::query_read(
                        &format!("select configuration from connections where name='{}';",connection.get_name()))
                    {
                        let configurations = configurations.split(";").map(|c| c).collect::<Vec<&str>>();
                        let new_configurations = get_current_config(configurations, connection.get_kind());
                        self.configurations.set_values(new_configurations);
                    }
                }
            },
            _ => {}
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
            let command: String;
            if kind == "Oracle" {
                command = oracle(filter_config!(self.configurations.get_values().iter().map(|c| c.get_value()).collect::<Vec<&String>>()))
            }
            else if kind == "MySQL" {
                command = mysql(filter_config!(self.configurations.get_values().iter().map(|c| c.get_value()).collect::<Vec<&String>>()))
            }
            else if kind == "MariaDB" {
                command = mariadb(filter_config!(self.configurations.get_values().iter().map(|c| c.get_value()).collect::<Vec<&String>>()))
            }
            else if kind == "PostgreSQL" {
                command = postgresql(filter_config!(self.configurations.get_values().iter().map(|c| c.get_value()).collect::<Vec<&String>>()));
            }
            else if kind == "SQLite" {
                command = sqlite(filter_config!(self.configurations.get_values().iter().map(|c| c.get_value()).collect::<Vec<&String>>()));
            }
            else if kind == "Redis" {
                command = redis(filter_config!(self.configurations.get_values().iter().map(|c| c.get_value()).collect::<Vec<&String>>()));
            }
            else if kind == "MongoDB" {
                command = mongodb(filter_config!(self.configurations.get_values().iter().map(|c| c.get_value()).collect::<Vec<&String>>()));
            }
            else if kind == "Neo4j" {
                command = neo4j(filter_config!(self.configurations.get_values().iter().map(|c| c.get_value()).collect::<Vec<&String>>()));
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
            "Oracle" => query = format!("insert into connections values ('Default{}','Required;Required;Required;Required','Oracle')",total_connections),
            "MySQL" => query = format!("insert into connections values ('Default{}','Required;Required;Required;Required','MySQL')",total_connections),
            "MariaDB" => query = format!("insert into connections values ('Default{}','Required;Required;Required;Required','MariaDB')",total_connections),
            "PostgreSQL" => query = format!("insert into connections values ('Default{}','Required;Required;Required;Required','PostgreSQL')",total_connections),
            "SQLite" => query = format!("insert into connections values ('Default{}','Required;','SQLite')",total_connections),
            "Redis" => query = format!("insert into connections values ('Default{}','Required;Required','Redis')",total_connections),
            "MongoDB" => query = format!("insert into connections values ('Default{}','Required;Required','Redis')",total_connections),
            "Neo4j" => query = format!("insert into connections values ('Default{}','Required;Required;Required;Required','Neo4j')",total_connections),
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

fn get_current_config(configurations: Vec<&str>, kind: &str) -> Vec<Configuration> {
    let scheme: Vec<&str>;
    match kind {
        "MySQL" | "MariaDB" => { scheme = SOCKET_SCHEME.to_vec(); },
        "Oracle" | "PostgreSQL" | "Neo4j" => { scheme = CLASSIC_SHEME.to_vec(); },
        "SQLite" => { scheme = FILE_SCHEME.to_vec(); },
        "Redis" => { scheme = REDIS_SCHEME.to_vec(); },
        "MongoDB" => { scheme = MONGODB_SCHEME.to_vec(); },
        "Custom" => { scheme = CUSTOM_SHEME.to_vec(); },
        _ => { scheme = vec!["Unknow"] }
    }
    scheme.iter().enumerate().map(|(index,kind)|
        if let Some(value) = configurations.get(index) {Configuration::from(value, *kind)}
        else {Configuration::from("",*kind)}).collect()
}