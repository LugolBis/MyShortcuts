use std::io;
use std::fs::OpenOptions;
use std::io::Write;

use crate::{objects::*, postgresql};
use crate::ui::{WidgetConfigurations,WidgetConnections};
use crate::database::{Database,CLASSIC_SHEME,CUSTOM_SHEME,AVAILABLE_SHEME};
use crate::utils::run_bash;
use crate::{neo4j, result_vec, format_config};

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
                let index = ts.selected().map_or(0, |i| (i.saturating_sub(1)) % self.connections.get_values().len());
                ts.select(Some(index));
                self.connections.set_state(State::Selected(ts));
                // ScrollBar interaction here 
            },
            (State::Selected(mut ts),State::WasSelected(_),KeyCode::Down) => {
                let index = ts.selected().map_or(0, |i| (i.saturating_add(1)) % self.connections.get_values().len());
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
                let index = ts.selected().map_or(0, |i| (i.saturating_sub(1)) % self.configurations.get_values().len());
                ts.select(Some(index));
                self.configurations.set_state(State::Selected(ts));
                // ScrollBar interaction here 
            },
            (State::WasSelected(_),State::Selected(mut ts),KeyCode::Down) => {
                let index = ts.selected().map_or(0, |i| (i.saturating_add(1)) % self.configurations.get_values().len());
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
            (State::WasSelected(index),State::WasSelected(_), KeyCode::Char('q') | KeyCode::Esc) => {
                self.show_pop_up = (false,0);
                self.connections.set_state(State::Selected(index));
            },
            (State::Selected(_),State::WasSelected(_), KeyCode::Char('q') | KeyCode::Esc) => self.exit(),
            (State::WasSelected(_),State::Selected(_), KeyCode::Char('q') | KeyCode::Esc) => self.exit(),
            (_,_,_) => {}
        }
    }

    /* 
    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match (self.widgets[0].get_state(), self.widgets[1].get_state(),key_event.code) {
            (State::WasSelected(index0),State::Selected(index1),KeyCode::Char('a')) => {
                self.widgets[1].get_mut_args().insert(index1, String::from("DefaultNewArgument"));
                let new_config = format_config!(self.widgets[1].get_args());
                if let Some(name) = result_vec!(self.widgets[0].get_arg(index0).unwrap()," ",false).get(1) {
                    let _ = Database::query_write(&format!("update connections set configuration='{}' where name='{}';",new_config,name));
                }
            },
            (State::WasSelected(index0),State::Selected(index1),KeyCode::Char('r')) => {
                self.widgets[1].get_mut_args().remove(index1);
                let new_config = format_config!(self.widgets[1].get_args());
                if let Some(name) = result_vec!(self.widgets[0].get_arg(index0).unwrap()," ",false).get(1) {
                    let _ = Database::query_write(&format!("update connections set configuration='{}' where name='{}';",new_config,name));
                }
            },
            (State::WasSelected(_),State::Selected(index),KeyCode::Char('e')) => {
                self.save = String::clone(self.widgets[1].get_arg(index).unwrap());
                self.widgets[1].set_state(State::Editing(index, 0usize));
            },
            (State::WasSelected(_),State::Editing(index_name, index_edit),KeyCode::Char(new_char)) => {
                if let Some(name) = self.widgets[1].get_mut_arg(index_name) {
                    name.insert(index_edit+1,new_char);
                    self.widgets[1].set_state(State::Editing(index_name, index_edit+1));
                }
            },
            (State::WasSelected(_),State::Editing(index_name,index_edit),KeyCode::Left) => {
                self.widgets[1].set_state(State::Editing(index_name, index_edit.saturating_sub(1usize)));
            },
            (State::WasSelected(_),State::Editing(index_name,index_edit),KeyCode::Right) => {
                if index_edit < self.widgets[1].get_arg(index_name).unwrap().len()-1 {
                    self.widgets[1].set_state(State::Editing(index_name, index_edit+1usize));
                }
            },
            (State::WasSelected(_),State::Editing(index_name,index_edit),KeyCode::Backspace) => {
                if let Some(name) = self.widgets[1].get_mut_arg(index_name) {
                    if index_edit<name.len() && name.len()>1 {
                        name.remove(index_edit);
                        self.widgets[1].set_state(State::Editing(index_name, index_edit.saturating_sub(1usize)));
                    }
                } 
            },
            (State::WasSelected(index_name),State::Editing(index,_),KeyCode::Enter) => {
                if let Some(name) = result_vec!(self.widgets[0].get_arg(index_name).unwrap()," ",false).get(1) {
                    match Database::query_write(&format!("update connections set configuration='{}' where name='{}'",name,format_config!(self.widgets[1].get_args()))) {
                        Ok(_) => {
                            self.widgets[1].set_state(State::Selected(index));
                            self.save = String::new();
                        },
                        Err(error) => {
                            if let Ok(mut file) = OpenOptions::new().create(true).write(true).append(true).open("logs.txt") {
                                let _ = file.write_all(format!("ERROR when try to Update the name '{}' to '{}' :\n  {}\n",self.save,name,error).as_bytes());
                            }
                        }
                    }
                }
            }
            (_,_,_) => {}
        }
    }*/

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
                        //let configurations = ;
                        let new_configurations: Vec<Configuration>;
                        if *&["Neo4j","PostgreSQL"].contains(&connection.get_kind().as_str()) {
                            new_configurations = CLASSIC_SHEME.iter().enumerate()
                            .map(|(index,kind)|
                                if let Some(value) = result_vec!(configurations,";").get(index) {Configuration::from(value, *kind)}
                                else {Configuration::from("",*kind)}).collect();
                        }
                        else {
                            configurations.pop();
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
            if kind == "Neo4j" {
                if let Ok(_) = file.write_all(neo4j!(self.configurations.get_values().iter().map(|c| c.get_value()).collect::<Vec<&String>>()).as_bytes()) {
                    let _ = run_bash();
                }
            }
            else if kind == "PostgreSQL" {
                if let Ok(_) = file.write_all(postgresql!(self.configurations.get_values().iter().map(|c| c.get_value()).collect::<Vec<&String>>()).as_bytes()) {
                    let _ = run_bash();
                }
            }
            else if kind == "Custom" {
                if let Ok(_) = file.write_all(self.configurations.get_values()[0].get_value().as_bytes()) {
                    let _ = run_bash();
                }
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
            "Custom" => query = format!("insert into connections values ('Default{}','Your shell command','Custom')",total_connections),
            _ => query = String::new()
        }
        let _ = Database::query_write(&query);
    }
}

fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}