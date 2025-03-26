use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Stylize},
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
    crossterm::event::KeyCode
};

#[derive(Debug, Clone, Copy)]
pub enum State {
    Selected(usize),
    WasSelected(usize)
}

#[derive(Debug)]
pub struct MyWidget {
    id: u64,
    args: Vec<String>,
    state: State
}

impl MyWidget {
    pub fn new(id:u64, selected:bool) -> Self {
        match selected {
            true => MyWidget { id, args: vec![], state: State::Selected(0) },
            false => MyWidget { id, args: vec![], state: State::WasSelected(0) }
        }
    }

    pub fn from(id:u64,args:Vec<String>, state:State) -> Self {
        MyWidget { id, args, state }
    }

    pub fn get_state(&self) -> State {
        self.state
    }

    pub fn get_args(&self) -> &Vec<String> {
        &self.args
    }

    pub fn get_current_arg(&self) -> &String {
        match self.state {
            State::Selected(index) | State::WasSelected(index) => &self.args[index]
        }
    }

    pub fn set_args(&mut self, args:Vec<String>) {
        self.args = args
    }

    pub fn set_state(&mut self, state:State) {
        self.state = state
    }

    pub fn update_state(&mut self, key_pressed:KeyCode) {
        match self.id {
            0 => handle_event0(self, key_pressed),
            1 => handle_event1(self, key_pressed),
            _ => {}
        }
    }
}

impl Widget for &MyWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title:Line;
        let instructions:Line;
        match self.id {
            0 => {
                title = Line::from(" Connections ".bold());
                instructions = Line::from(vec![
                    " Select : ".into(),"<Up> <Down>".cyan().bold(),
                    " | Open : ".into(),"<o>".cyan().bold()," | Quit : ".into(),"<q> ".cyan().bold(),
                ]);
            },
            1 => {
                title = Line::from(" Informations ".bold());
                instructions = Line::from(vec![
                    " Save : ".into(),"<s>".cyan().bold(),
                    " | Edit : ".into(),"<e> ".cyan().bold()
                ]);
            },
            _ => {
                title = Line::from(" Default ".bold());
                instructions = Line::from(vec![" Default ".into()," Quit ".into(),"<Q> ".cyan().bold()]);
            },
        }
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::ROUNDED);
        
        let mut lines: Vec<Line> = self.args.iter().map(|a| Line::from(String::clone(a))).collect();
        if lines.len() > 0 {
            match self.state {
                State::Selected(index) => lines[index] = Line::clone(&lines[index]).patch_style(Color::LightCyan),
                State::WasSelected(index) => lines[index] = Line::clone(&lines[index]).patch_style(Color::Cyan),
            }
        }
        let content = Text::from(lines);

        Paragraph::new(content)
            .centered()
            .block(block)
            .render(area, buf);
    }
}

fn handle_event0(widget: &mut MyWidget, key_pressed:KeyCode) {
    match (key_pressed, widget.state) {
        (KeyCode::Right, State::Selected(index)) => widget.state = State::WasSelected(index),
        (KeyCode::Left, State::WasSelected(index)) => widget.state = State::Selected(index),
        (KeyCode::Up, State::Selected(index)) => {
            if widget.args.len()>0 {
                if let Some(result) = index.checked_sub(1usize) {
                    widget.state = State::Selected(result)
                } 
            }
        },
        (KeyCode::Down, State::Selected(index)) => {
            if widget.args.len()>0 && index+1<widget.args.len() {
                widget.state = State::Selected(index+1)
            }
        },
        (_,_) => {}
    }
}

fn handle_event1(widget: &mut MyWidget, key_pressed:KeyCode) {
    match (key_pressed, widget.state) {
        (KeyCode::Right, State::WasSelected(index)) => widget.state = State::Selected(index),
        (KeyCode::Left, State::Selected(_)) => widget.state = State::WasSelected(0usize),
        (KeyCode::Up, State::Selected(index)) => {
            if widget.args.len()>0 {
                if let Some(result) = index.checked_sub(1usize) {
                    widget.state = State::Selected(result)
                } 
            }
        },
        (KeyCode::Down, State::Selected(index)) => {
            if widget.args.len()>0 && index+1<widget.args.len() {
                widget.state = State::Selected(index+1)
            }
        },
        (_,_) => {}
    }
}