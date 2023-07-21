use crossterm::event::KeyCode;

#[derive(Clone, Debug)]
pub struct InputViewModel {
    pub text: Vec<char>,
    pub cursor: usize,
}


impl InputViewModel {
    pub fn new() -> Self {
        Self {
            text: Vec::new(),
            cursor: 0,
        }
    }

    pub fn input(&self) -> &[char] {
        &self.text
    }

    pub fn input_write(&mut self, character: char) {
        self.text.insert(self.cursor, character);
        self.cursor += 1;
    }

    pub fn input_remove(&mut self) {
        if self.cursor < self.text.len() {
            self.text.remove(self.cursor);
        }
    }

    pub fn clean_input(&mut self) {
        self.cursor = 0;
        self.text = Vec::new();
    }


    pub fn input_remove_previous(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.text.remove(self.cursor);
        }
    }


    pub fn input_move_cursor(&mut self, movement: KeyCode) {
        match movement {
            KeyCode::Left => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                }
            }
            KeyCode::Right => {
                if self.cursor < self.text.len() {
                    self.cursor += 1;
                }
            }
            KeyCode::Home => {
                self.cursor = 0;
            }
            KeyCode::End => {
                self.cursor = self.text.len();
            }
            _ => {}
        }
    }
}

