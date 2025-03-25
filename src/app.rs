use std::io;

use crate::ui::{MyWidget,State};
use crate::database::Database;

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
            [Constraint::Percentage(30), Constraint::Percentage(100)],
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
        match key_event.code {
            KeyCode::Char('q') | KeyCode::Esc => self.exit(),
            key_code => {
                for widget in &mut self.widgets {
                    widget.update_state(key_code);
                }
            }
        }
    }

    fn update_widgets_args(&mut self) {
        if let Ok(names) = Database::query_read("select type,name from connections order by type;") {
            self.widgets[0].set_args(names.split("\n").map(|name| name.replace(";", " | ")).collect());
        }
        match self.widgets[0].get_state() {
            State::Selected(index) | State::WasSelected(index) => {
                if let Some(name) = self.widgets[0].get_args().get(index) {
                    if let Some(name) = name.split(" | ").collect::<Vec<&str>>().get(1) {
                        if let Ok(configurations) = Database::query_read(&format!("select configuration from connections where name={};",name)) {
                            self.widgets[1].set_args(configurations.split(";").map(|arg| String::from(arg)).collect());
                        }
                    }
                }
            },
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}