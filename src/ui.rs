use ratatui::{
    buffer::Buffer, crossterm::event::KeyCode, layout::Rect, style::{Color, Stylize}, symbols::border, text::{Line, Span, Text}, widgets::{Block, Paragraph, Widget}
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Selected(usize),
    WasSelected(usize),
    Editing(usize, usize)
}

#[derive(Debug)]
pub struct MyWidget {
    id: u64,
    args: Vec<String>,
    args_name: Vec<String>,
    state: State,
}

impl MyWidget {
    pub fn new(id:u64, selected:bool) -> Self {
        match selected {
            true => MyWidget { id, args: vec![], args_name: vec![], state: State::Selected(0) },
            false => MyWidget { id, args: vec![], args_name: vec![], state: State::WasSelected(0) }
        }
    }

    pub fn from(id:u64, args:Vec<String>, args_name:Vec<String>, state:State) -> Self {
        MyWidget { id, args, args_name, state }
    }

    pub fn get_state(&self) -> State {
        self.state
    }

    pub fn get_args(&self) -> &Vec<String> {
        &self.args
    }

    pub fn get_mut_args(&mut self) -> &mut Vec<String> {
        &mut self.args
    }

    pub fn get_arg(&self, index:usize) -> Option<&String> {
        self.args.get(index)
    }

    pub fn get_mut_arg(&mut self, index:usize) -> Option<&mut String> {
        self.args.get_mut(index)
    }

    pub fn set_args(&mut self, args:Vec<String>) {
        self.args = args
    }

    pub fn set_args_name(&mut self, args_name:Vec<String>) {
        self.args_name = args_name
    }

    pub fn set_state(&mut self, state:State) {
        self.state = state
    }
}

impl Widget for &MyWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title:Line;
        let instructions:Line;
        let mut lines: Vec<Line>;
        match self.id {
            0 => {
                title = Line::from(" Connections ".bold());
                instructions = Line::from(vec![
                    " Select : ".into(),"<Up> <Down>".cyan().bold(),
                    " | Open : ".into(),"<o>".cyan().bold()," | Add : ".into(),"<a>".cyan().bold(),
                    " | Remove : ".into(),"<r> ".cyan().bold()
                ]);
                lines = self.args.iter().map(|a| Line::from(String::clone(a))).collect();
            },
            1 => {
                title = Line::from(" Informations ".bold());
                instructions = Line::from(vec![
                    " Edit : ".into(),"<e>".cyan().bold()," | Save : ".into(),"<Enter>".cyan().bold()," | Quit : ".into(),"<q> ".cyan().bold(),
                ]);
                match self.state {
                    State::Selected(_) | State::WasSelected(_) => lines = self.args.iter().map(|a| Line::from(String::clone(a))).collect(),
                    State::Editing(index_name, index_char) => {
                        lines = self.args_name.iter().map(|arg| Line::from(String::clone(arg))).collect();
                        for index in 0..lines.len() {
                            if let Some(arg) = self.args.get(index) {
                                if index == index_name {
                                    let mut cursor = String::clone(arg);
                                    cursor.insert_str(index_char+1, "|");
                                    lines.insert((index*2)+1, Line::from(cursor))
                                }
                                else {
                                    lines.insert((index*2)+1, Line::from(String::clone(arg))); 
                                }
                            }
                            else {
                                lines.insert((index*2)+1, Line::from(String::from("...")));
                            }
                        }
                    }
                }
            },
            _ => {
                title = Line::from(" Default ".bold());
                instructions = Line::from(vec![" Default ".into()," Quit ".into(),"<Q> ".cyan().bold()]);
                lines = self.args.iter().map(|a| Line::from(String::clone(a))).collect();
            },
        }
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::ROUNDED);
        
        if lines.len() > 0 {
            match self.state {
                State::Selected(index) => lines[index] = Line::clone(&lines[index]).patch_style(Color::LightCyan),
                State::WasSelected(index) => lines[index] = Line::clone(&lines[index]).patch_style(Color::Cyan),
                State::Editing(index_name, index_char) => {
                    lines[index_name+1] = Line::clone(&lines[index_name+1]).patch_style(Color::LightYellow);
                }
            }
        }
        let content = Text::from(lines);

        Paragraph::new(content)
            .centered()
            .block(block)
            .render(area, buf);
    }
}