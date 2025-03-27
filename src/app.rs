use std::io;
use std::fs::OpenOptions;
use std::io::Write;


use crate::ui::{MyWidget,State};
use crate::database::Database;
use crate::utils::run_bash;
use crate::{neo4j, result_string, result_vec, format_config};

use ratatui::{
    prelude::{Constraint, Layout, Direction},
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind}
};

pub fn main_app() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let app_result = App::new().run(&mut terminal);
    ratatui::restore();
    app_result
}

pub struct App {
    widgets: Vec<MyWidget>,
    exit: bool,
}

impl App {

    pub fn new() -> Self {
        App { widgets: vec![MyWidget::new(0,true),MyWidget::new(1,false)], exit: false }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            self.update_widgets_args();
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        let layout = Layout::new(
            Direction::Horizontal,
            [Constraint::Percentage(40), Constraint::Percentage(100)],
        )
        .split(frame.area());
        frame.render_widget(&self.widgets[0], layout[0]);
        frame.render_widget(&self.widgets[1], layout[1]);
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
        match (self.widgets[0].get_state(),self.widgets[1].get_state(),key_event.code) {
            (State::Selected(index),State::WasSelected(_),KeyCode::Up) => {
                if self.widgets[0].get_args().len()>0 { self.widgets[0].set_state(State::Selected(index.saturating_sub(1usize))); }
            },
            (State::Selected(index),State::WasSelected(_),KeyCode::Down) => {
                if index+1 < self.widgets[0].get_args().len() { self.widgets[0].set_state(State::Selected(index+1)); }
            },
            (State::Selected(index),State::WasSelected(_),KeyCode::Right) => {
                self.widgets[0].set_state(State::WasSelected(index));
                self.widgets[1].set_state(State::Selected(0usize));
            },
            (State::WasSelected(_),State::Selected(index),KeyCode::Up) => {
                if self.widgets[1].get_args().len()>0 { self.widgets[1].set_state(State::Selected(index.saturating_sub(1usize))); }
            },
            (State::WasSelected(_),State::Selected(index),KeyCode::Down) => {
                if index+1 < self.widgets[1].get_args().len() { self.widgets[1].set_state(State::Selected(index+1)); }
            },
            (State::WasSelected(index),State::Selected(_),KeyCode::Left) => {
                self.widgets[0].set_state(State::Selected(index));
                self.widgets[1].set_state(State::WasSelected(0usize));
            },
            (State::Selected(index)|State::WasSelected(index),State::WasSelected(_)|State::Selected(_), KeyCode::Char('o')) => {
                let connection = result_vec!(self.widgets[0].get_arg(index).unwrap()," ",false);
                let config = self.widgets[1].get_args();
                if connection[0] == "Neo4j" {
                    if let Ok(mut file) = OpenOptions::new().write(true).create(true).truncate(true).open("current_command.txt") {
                        if let Ok(_) = file.write_all(neo4j!(config).as_bytes()) {
                            let _ = run_bash();
                        }
                    }
                }
            },
            (State::Selected(_),State::WasSelected(_),KeyCode::Char('a')) => {
                let default_name = format!("DefaultConnection{}",self.widgets[0].get_args().len());
                let _ = Database::query_write(&format!("insert into connections values ('{}','Host;Port;Username;Password;','CustomType');",&default_name));
            },
            (State::Selected(index),State::WasSelected(_),KeyCode::Char('r')) => {
                if let Some(name) = result_vec!(self.widgets[0].get_args()[index]," ",false).get(1) {
                    let _ = Database::query_write(&format!("delete from connections where name='{}';",name));
                }
            },
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
            (State::Selected(_),State::WasSelected(_), KeyCode::Char('q') | KeyCode::Esc) => self.exit(),
            (State::WasSelected(_),State::Selected(_), KeyCode::Char('q') | KeyCode::Esc) => self.exit(),
            (_,_,_) => {}
        }
    }

    fn update_widgets_args(&mut self) {
        if let Ok(names) = Database::query_read("select type,name from connections order by type;") {
            self.widgets[0].set_args(result_vec!(names,"\n",true));
        }
        match self.widgets[0].get_state() {
            State::Selected(index) | State::WasSelected(index) => {
                if let Some(name) = self.widgets[0].get_args().get(index) {
                    if let Some(name) = result_vec!(name," ",false).get(1) {
                        if let Ok(configurations) = Database::query_read(&format!("select configuration from connections where name='{}';",name)) {
                            self.widgets[1].set_args(result_vec!(configurations,";",false));
                        }
                    }
                }
            },
            State::Editing(_, _) => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}