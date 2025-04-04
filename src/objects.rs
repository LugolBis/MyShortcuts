use ratatui::{widgets::TableState,crossterm::event::{self, KeyCode}};
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;

#[derive(Debug, Clone)]
pub enum State {
    Selected(TableState),
    WasSelected(TableState),
    Editing(TableState, Input)
}

#[derive(Debug)]
pub struct Connection {
    name: String,
    kind: String
}

#[derive(Debug)]
pub struct Configuration {
    value: String,
    kind: String
}

impl Connection {
    pub fn default() -> Self {
        Connection { name: String::from("Default0"), kind: String::from("Custom") }
    }

    pub fn from(name:&str, kind:&str) -> Self {
        Connection { name: String::from(name), kind: String::from(kind) }
    }

    pub fn parse(value:&str) -> Result<Self,String> {
        let vector = value.split(";").collect::<Vec<&str>>();
        if let (Some(name),Some(kind)) = (vector.get(0),vector.get(1)) {
            Ok(Connection {name: String::from(*name), kind: String::from(*kind)})
        }
        else {
            Err(format!("ERROR : when try to parse the following connection : '{}'",value))
        }
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_mut_name(&mut self) -> &mut String {
        &mut self.name
    }

    pub fn get_kind(&self) -> &String {
        &self.kind
    }

    pub fn get_mut_kind(&mut self) -> &mut String {
        &mut self.kind
    }
}

impl Configuration {
    pub fn default() -> Self {
        Configuration { value: String::from("echo Welcome on MyShortcuts"), kind: String::from("DefaultProperty") }
    }

    pub fn from(value:&str, kind:&str) -> Self {
        Configuration { value: String::from(value), kind: String::from(kind) }
    }

    pub fn get_value(&self) -> &String {
        &self.value
    }

    pub fn get_mut_value(&mut self) -> &mut String {
        &mut self.value
    }

    pub fn get_kind(&self) -> &String {
        &self.kind
    }

    pub fn get_mut_kind(&mut self) -> &mut String {
        &mut self.kind
    }
}