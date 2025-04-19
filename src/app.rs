use std::fs::OpenOptions;
use std::io::{self,Write};

use crate::ui::{WidgetConfigurations,WidgetShortcuts,Common,render_pop_up,render_help};
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
    let mut app = App::new();
    let app_result = app.run(&mut terminal);
    ratatui::restore();
    app_result
}

pub struct App {
    shortcuts:WidgetShortcuts,
    configurations:WidgetConfigurations,
    /// The name of the Shortcut before any modification
    save: String,
    show_pop_up: (bool,usize),
    exit: bool,
}

impl App {

    pub fn new() -> Self {
        App {
            shortcuts: WidgetShortcuts::from(vec![],State::Selected(TableState::new().with_selected(0).with_selected_column(0))),
            configurations: WidgetConfigurations::from(vec![],State::WasSelected(TableState::new().with_selected(0))),
            save: String::new(),
            show_pop_up: (false,0usize),
            exit: false,
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
        let layout0 = Layout::new(Direction::Horizontal,[Constraint::Percentage(30), Constraint::Percentage(100)])
            .split(frame.area());

        let layout1 = Layout::new(Direction::Vertical,[Constraint::Percentage(80), Constraint::Percentage(20)])
            .split(layout0[0]);

        render_help(frame, layout1[1]);
        self.shortcuts.render(frame, layout1[0]);

        if self.show_pop_up.0 {
            let layout2 = Layout::new(Direction::Horizontal,[Constraint::Percentage(50),Constraint::Percentage(50)])
                .split(layout0[1]);
            render_pop_up(frame, self.show_pop_up.1, layout2[0]);
            self.configurations.render(frame, layout2[1]);
        }
        else {
            self.configurations.render(frame, layout0[1]);
        }
    }

    fn handle_events(&mut self) -> io::Result<()> {
        let event = event::read()?;
        if let Event::Key(key) = event {
            match (self.shortcuts.get_state(),self.configurations.get_state()) {
                (State::Editing(ts0, input),State::WasSelected(_)) if key.code == KeyCode::Enter => {
                    self.save_editing(input.value().into(), true);
                    self.shortcuts.set_state(State::Selected(ts0));
                },
                (State::WasSelected(_),State::Editing(ts1, input)) if key.code == KeyCode::Enter => {
                    self.save_editing(input.value().into(), false);
                    self.configurations.set_state(State::Selected(ts1));
                },
                (State::Editing(ts0, input),_) => {
                    let mut new_input = input;
                    new_input.handle_event(&event);
                    self.shortcuts.set_state(State::Editing(ts0, new_input));
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
        match (self.shortcuts.get_state(),self.configurations.get_state(),key_event.code) {
            (State::Selected(mut ts),State::WasSelected(_),KeyCode::Up) => {
                let index = ts.selected().map_or(0, |i| if i>0 { i-1 } else { self.shortcuts.get_values().len()-1 });
                ts.select(Some(index));
                self.shortcuts.set_state(State::Selected(ts));
                // ScrollBar interaction here 
            },
            (State::Selected(mut ts),State::WasSelected(_),KeyCode::Down) => {
                let index = ts.selected().map_or(0, |i| if i<self.shortcuts.get_values().len()-1 { i+1 } else { 0 });
                ts.select(Some(index));
                self.shortcuts.set_state(State::Selected(ts));
                // ScrollBar interaction here 
            },
            (State::Selected(ts0),State::WasSelected(ts1),KeyCode::Right) => {
                self.switch_selected_widget(ts0, ts1, true);
            },
            (State::Selected(mut ts0),State::WasSelected(_),KeyCode::Left) => {
                if let Some(index) = ts0.selected_column() {
                    ts0.select_column(Some(index.saturating_sub(1)));
                    self.shortcuts.set_state(State::Selected(ts0));
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
            (State::Selected(ts0)|State::WasSelected(ts0),State::WasSelected(_)|State::Selected(_), KeyCode::Char('o')|KeyCode::Char('O')) => {
                if let Some(shortcut) = self.shortcuts.get_values().get(ts0.selected().unwrap_or(0)) {
                    self.execute_shortcut(String::clone(shortcut.get_kind()));
                }
            }
            (State::Selected(index),State::WasSelected(_), KeyCode::Char('a')|KeyCode::Char('A')) => {
                self.shortcuts.set_state(State::WasSelected(index));
                self.show_pop_up = (true,0);
            },
            (State::WasSelected(_),State::Selected(index), KeyCode::Char('a')|KeyCode::Char('A')) => {
                self.configurations.set_state(State::WasSelected(index));
                self.show_pop_up = (true,0);
            },
            (State::WasSelected(_),State::WasSelected(_),KeyCode::Up) =>{
                if self.show_pop_up.1 == 0 { self.show_pop_up.1 = AVAILABLE_SHEME.len()-1; }
                else { self.show_pop_up.1 = self.show_pop_up.1-1; }
            },
            (State::WasSelected(_),State::WasSelected(_),KeyCode::Down) => self.show_pop_up = (true, (self.show_pop_up.1+1) % AVAILABLE_SHEME.len()),
            (State::WasSelected(index),State::WasSelected(_),KeyCode::Enter) => {
                self.add_new_shortcut();
                self.show_pop_up = (false,0);
                self.shortcuts.set_state(State::Selected(index));
            }
            (State::Selected(mut ts0),State::WasSelected(_),KeyCode::Char('r')|KeyCode::Char('R')) => {
                if let Some(index0) = ts0.selected() {
                    if let Some(shortcut) = self.shortcuts.get_values().get(index0) {
                        let _ = Database::query_write(&format!("delete from shortcuts where name='{}';", shortcut.get_name()));
                        if self.shortcuts.get_values().len() == 1 {
                            if let Err(error) = Database::query_write("insert into shortcuts values ('Default0', 'echo Welcome on MyShortcuts !', 'Custom');") {
                                Logs::write(format!("ERROR : app.rs - handle_key_event() -1st :\n{}",error));
                            }
                        }
                        ts0.select(Some(index0.saturating_sub(1)));
                        self.shortcuts.set_state(State::Selected(ts0));
                    }
                }
            },
            (State::WasSelected(ts0),State::Selected(ts1),KeyCode::Char('r')|KeyCode::Char('R')) => {
                if let Some(index0) = ts0.selected() {
                    if let Some(index1) = ts1.selected() {
                        self.configurations.get_mut_values()[index1] = Configuration::from("","");
                        let query = format!("update shortcuts set configuration='{}' where name='{}';",
                            format_config!(self.configurations.get_values()), self.shortcuts.get_values()[index0].get_name());
                        let _ = Database::query_write(&query);
                    }
                }
            },
            (State::WasSelected(index), State::WasSelected(_), KeyCode::Char('q') |KeyCode::Char('Q') | KeyCode::Esc) => {
                self.show_pop_up = (false,0);
                self.shortcuts.set_state(State::Selected(index));
            },
            (State::Selected(mut ts0), State::WasSelected(_), KeyCode::Char('e')|KeyCode::Char('E')) => {
                if let Some(index) = ts0.selected() {
                    if let Some(shortcut) =  self.shortcuts.get_values().get(index) {
                        self.save = String::clone(shortcut.get_name());
                        ts0.select_column(Some(1));
                        self.shortcuts.set_state(State::Editing(ts0, Input::with_value(Input::default(), String::clone(shortcut.get_name()))));
                    }
                }
            },
            (State::WasSelected(ts0), State::Selected(mut ts1), KeyCode::Char('e')|KeyCode::Char('E')) => {
                match (ts0.selected(), ts1.selected()) {
                    (Some(index0),Some(index1)) => {
                        match (self.shortcuts.get_values().get(index0), self.configurations.get_values().get(index1)) {
                            (Some(shortcut), Some(configuration)) => {
                                self.save = String::clone(shortcut.get_name());
                                ts1.select_column(Some(1));
                                self.configurations.set_state(State::Editing(ts1, Input::with_value(
                                    Input::default(), String::clone(configuration.get_value()))));
                            }
                            (Some(shortcut), None) => {
                                if let Some(configuration) = self.configurations.get_values().get(0usize) {
                                    self.save = String::clone(shortcut.get_name());
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
            (State::Selected(_)|State::WasSelected(_),State::Selected(_)|State::WasSelected(_),KeyCode::Char('h')|KeyCode::Char('H')) => {
                self.configurations.hidde()
            },
            (State::Selected(_),State::WasSelected(_), KeyCode::Char('q')| KeyCode::Char('Q') | KeyCode::Esc) => self.exit(),
            (State::WasSelected(_),State::Selected(_), KeyCode::Char('q')| KeyCode::Char('Q') | KeyCode::Esc) => self.exit(),
            (_,_,_) => {}
        }
    }

    fn update_widgets_args(&mut self) {
        match self.shortcuts.get_state() {
            State::Editing(ts0, input) => {
                if let Some(index) = ts0.selected() {
                    if let Some(shortcut) = self.shortcuts.get_mut_values().get_mut(index) {
                        shortcut.get_mut_name().clone_from(&String::from(input.value()));
                        return;
                    }
                }
            },
            _ => {
                if let Ok(shortcuts) = Database::query_read("select name,type from shortcuts order by type;") {
                    self.shortcuts.set_values(shortcuts.split("\n").filter(|e| *e!="" && *e!="\n")
                        .map(|cnx| if let Ok(cnx)  = Shortcut::parse(cnx)  {cnx} else {Shortcut::default()}).collect::<Vec<Shortcut>>());
                }
            }
        }
        
        match (self.shortcuts.get_state(), self.configurations.get_state()) {
            (State::WasSelected(_), State::Editing(ts1, input)) => {
                if let Some(index) = ts1.selected() {
                    if let Some(configuration) = self.configurations.get_mut_values().get_mut(index) {
                        configuration.get_mut_value().clone_from(&String::from(input.value()));
                    }
                }
            },
            (State::Selected(ts0) | State::WasSelected(ts0), State::Selected(_) | State::WasSelected(_)) => {
                let index0 = ts0.selected().unwrap_or(0);
                if let Some(shortcut) = self.shortcuts.get_values().get(index0) {
                    match Database::query_read(
                        &format!("select configuration from shortcuts where name='{}';",shortcut.get_name()))
                    {
                        Ok(configurations) => {
                            let configurations = configurations.split(";").map(|c| c.trim_end()).collect::<Vec<&str>>();
                            let new_configurations = get_current_config(configurations, shortcut.get_kind());
                            self.configurations.set_values(new_configurations);
                        }
                        Err(error) => {
                            Logs::write(format!("\nERROR : app.rs - update_widgets_args() -1st {}",error));
                        }
                    }
                }
                else {
                    Logs::write(format!("\nERROR : app.rs - update_widgets_args() -2nd :\n   app.shortcuts is empty : {:?}",self.shortcuts.get_values()));
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
                        self.shortcuts.set_state(State::Selected(ts0));
                    }
                    else {
                        ts0.select_column(Some(index));
                        ts1.select_column(Some(0));
                        self.shortcuts.set_state(State::WasSelected(ts0));
                        self.configurations.set_state(State::Selected(ts1));
                    }
                }
                None => {
                    ts0.select_column(Some(0));
                    self.shortcuts.set_state(State::Selected(ts0));
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
                        self.shortcuts.set_state(State::Selected(ts0));
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

    fn execute_shortcut(&self, kind:String) {
        if let Ok(mut file) = OpenOptions::new().write(true).create(true).truncate(true).open("current_command.txt") {
            let command: String;
            let current_configuration = self.configurations.get_values().iter().map(|c| c.get_value()).collect::<Vec<&String>>();
            if kind == "Oracle" {
                command = oracle(filter_config!(current_configuration))
            }
            else if kind == "MySQL" {
                command = mysql(filter_config!(current_configuration))
            }
            else if kind == "MariaDB" {
                command = mariadb(filter_config!(current_configuration))
            }
            else if kind == "PostgreSQL" {
                command = postgresql(filter_config!(current_configuration));
            }
            else if kind == "SQLite" {
                command = sqlite(filter_config!(current_configuration));
            }
            else if kind == "Redis" {
                command = redis(filter_config!(current_configuration));
            }
            else if kind == "MongoDB" {
                command = mongodb(filter_config!(current_configuration));
            }
            else if kind == "Neo4j" {
                command = neo4j(filter_config!(current_configuration));
            }
            else if kind == "Custom" {
                command = String::clone(self.configurations.get_values()[0].get_value());
            }
            else {
                command = String::from("echo 'Welcome on MyShortcuts !'");
            }

            if let Ok(_) = file.write_all(command.as_bytes()) {
                let _ = run_command();
            }
        }
    }

    fn add_new_shortcut(&self) {
        let kind = AVAILABLE_SHEME[self.show_pop_up.1];
        let names = self.shortcuts.get_values().iter().map(|s| String::clone(s.get_name())).collect::<Vec<String>>();
        let new_name = generate_name(names);
        let query: String;
        match kind {
            "Oracle" => query = format!("insert into shortcuts values ('{}','Required;Required;Required;Required','Oracle');",new_name),
            "MySQL" => query = format!("insert into shortcuts values ('{}','Required;Required;Required;Required','MySQL');",new_name),
            "MariaDB" => query = format!("insert into shortcuts values ('{}','Required;Required;Required;Required','MariaDB');",new_name),
            "PostgreSQL" => query = format!("insert into shortcuts values ('{}','Required;Required;Required;Required','PostgreSQL');",new_name),
            "SQLite" => query = format!("insert into shortcuts values ('{}','Required;','SQLite');",new_name),
            "Redis" => query = format!("insert into shortcuts values ('{}','Required;Required','Redis');",new_name),
            "MongoDB" => query = format!("insert into shortcuts values ('{}','Required;Required','MongoDB');",new_name),
            "Neo4j" => query = format!("insert into shortcuts values ('{}','Required;Required;Required;Required','Neo4j');",new_name),
            "Custom" => query = format!("insert into shortcuts values ('{}','echo Welcome on MyShortcuts','Custom');",new_name),
            _ => query = String::new()
        }
        if let Err(error) = Database::query_write(&query) {
            Logs::write(format!("\nERROR : app.rs - add_new_shortcut() :\n{}\n|-> Name generated : '{}'",error,new_name));
        }
    }

    fn save_editing(&mut self,new_value:String,is_shortcut:bool) {
        let query: String;
        if is_shortcut {
            query = format!("update shortcuts set name='{}' where name='{}';",new_value, self.save);
        }
        else {
            query = format!("update shortcuts set configuration='{}' where name='{}';",format_config!(self.configurations.get_values()), self.save);
        }
        let _ = Database::query_write(&query);
        self.save = String::new();
    }
}

fn generate_name(current_names: Vec<String>) -> String {
    let nb_max = current_names.iter()
        .filter(|x| x.contains("Default"))
        .map(|x| (*x).replace("Default","").parse::<u64>().unwrap_or(0))
        .max().unwrap_or(0);
    format!("Default{}",nb_max+1)
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