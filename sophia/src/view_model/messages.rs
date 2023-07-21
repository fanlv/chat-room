use crossterm::event::KeyCode;

use sophia_core::model::Message as ModelMessage;
use sophia_core::model::User;

#[derive(Clone, Debug)]
pub struct ChatMessageViewModel {
    pub messages: Vec<Message>,
    pub scroll_pos: usize,
    pub scroll_to_pos: usize,
}


#[derive(Clone, Debug)]
pub struct Message {
    pub time: i64,
    pub content: String,
    pub user: SomeUser,
}


#[derive(Clone, Debug)]
pub enum SomeUser {
    User(User),
    System,
}

impl Message {
    pub fn new(time: i64, content: String, user: SomeUser) -> Self {
        Message { time, content, user }
    }

    pub fn from_message(msg: ModelMessage) -> Self {
        Message { time: msg.time, content: msg.content, user: SomeUser::User(msg.user) }
    }
}

impl ChatMessageViewModel {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            scroll_pos: 0,
            scroll_to_pos: 0,
        }
    }


    pub fn scroll_to_end(&mut self) {
        self.scroll_to_pos = self.messages.len();
    }

    pub fn messages_scroll(&mut self, movement: KeyCode) {
        match movement {
            KeyCode::Up => {
                if self.scroll_pos > 0 {
                    self.scroll_pos -= 1;
                }
            }
            KeyCode::Down => {
                self.scroll_pos += 1;
            }
            KeyCode::PageUp => {
                self.scroll_pos += 0;
            }
            _ => {}
        }
    }
}