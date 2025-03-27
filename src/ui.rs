use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Stylize},
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
    crossterm::event::KeyCode
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

    pub fn set_args(&mut self, args:Vec<String>) {
        self.args = args
    }

    pub fn set_state(&mut self, state:State) {
        self.state = state
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
                    " | Open : ".into(),"<o>".cyan().bold()," | Add : ".into(),"<a>".cyan().bold(),
                    " | Remove : ".into(),"<r> ".cyan().bold()
                ]);
            },
            1 => {
                title = Line::from(" Informations ".bold());
                instructions = Line::from(vec![
                    " Edit : ".into(),"<e>".cyan().bold()," | Save : ".into(),"<Enter>".cyan().bold()," | Quit : ".into(),"<q> ".cyan().bold(),
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
                State::Editing(_, _) => todo!()
            }
        }
        let content = Text::from(lines);

        Paragraph::new(content)
            .centered()
            .block(block)
            .render(area, buf);
    }
}