use std::sync::Arc;

use chrono::{DateTime, Local};
use crossterm::event::KeyCode;
use log::Level;
use tokio::sync::mpsc::Sender;
use tokio::sync::RwLock;

use sophia_core::model::Message as ModelMessage;
use sophia_core::model::User;

use crate::config::Config;

#[derive(Clone, Debug)]
pub struct AppViewModel {
    pub message_list: Vec<MessageViewModel>,
    pub logs: Vec<Log>,
    pub conf: Config,
    pub input: Vec<char>,
    pub input_cursor: usize,
    pub user_list: Vec<User>,
    pub scroll_messages_view_pos: Arc<RwLock<usize>>,
    pub reset_pos: usize,
    pub sender: Arc<Sender<AppViewModel>>,
    pub auto_set_scroll_messages_view_pos: Arc<RwLock<usize>>,
}

#[derive(Clone, Debug)]
pub struct Log {
    pub level: Level,
    pub content: String,
    pub time: DateTime<Local>,
}

#[derive(Clone, Debug)]
pub struct MessageViewModel {
    pub time: i64,
    pub content: String,
    pub user: SomeUser,
}

#[derive(Clone, Debug)]
pub enum SomeUser {
    User(User),
    System,
}

impl MessageViewModel {
    pub fn new(time: i64, content: String, user: SomeUser) -> Self {
        MessageViewModel { time, content, user }
    }

    pub fn from_message(msg: ModelMessage) -> Self {
        MessageViewModel { time: msg.time, content: msg.content, user: SomeUser::User(msg.user) }
    }
}


impl Log {
    pub fn new(level: Level, content: String, time: DateTime<Local>) -> Self {
        Self { level, content, time }
    }
}


impl AppViewModel {
    pub fn new(sender: Sender<AppViewModel>, conf: Config) -> Self {
        Self {
            message_list: Vec::new(),
            user_list: Vec::new(),
            logs: Vec::new(),
            input: Vec::new(),
            conf,
            input_cursor: 0,
            scroll_messages_view_pos: Arc::new(RwLock::new(0)),
            reset_pos: 0,
            sender: Arc::new(sender),
            auto_set_scroll_messages_view_pos: Arc::new(RwLock::new(0)),
        }
    }


    pub fn input(&self) -> &[char] {
        &self.input
    }

    pub fn input_write(&mut self, character: char) {
        self.input.insert(self.input_cursor, character);
        self.input_cursor += 1;
    }

    pub fn input_remove(&mut self) {
        if self.input_cursor < self.input.len() {
            self.input.remove(self.input_cursor);
        }
    }

    pub fn clean_input(&mut self) {
        self.input_cursor = 0;
        self.input = Vec::new();
    }

    pub async fn auto_set_scroll_messages(&mut self) {
        let mut auto_set_scroll_messages_view_pos = self.auto_set_scroll_messages_view_pos.write().await;
        *auto_set_scroll_messages_view_pos = self.message_list.len();
    }

    pub fn input_remove_previous(&mut self) {
        if self.input_cursor > 0 {
            self.input_cursor -= 1;
            self.input.remove(self.input_cursor);
        }
    }


    pub fn input_move_cursor(&mut self, movement: KeyCode) {
        match movement {
            KeyCode::Left => {
                if self.input_cursor > 0 {
                    self.input_cursor -= 1;
                }
            }
            KeyCode::Right => {
                if self.input_cursor < self.input.len() {
                    self.input_cursor += 1;
                }
            }
            KeyCode::Home => {
                self.input_cursor = 0;
            }
            KeyCode::End => {
                self.input_cursor = self.input.len();
            }
            _ => {}
        }
    }

    pub async fn messages_scroll(&mut self, movement: KeyCode) {
        let mut scroll_messages_view_pos = self.scroll_messages_view_pos.write().await;
        match movement {
            KeyCode::Up => {
                if *scroll_messages_view_pos > 0 {
                    *scroll_messages_view_pos -= 1;
                }
            }
            KeyCode::Down => {
                *scroll_messages_view_pos += 1;
            }
            KeyCode::PageUp => {
                *scroll_messages_view_pos += 0;
            }
            _ => {}
        }
    }


    pub async fn log(&mut self, level: Level, content: String) {
        self.logs.push(Log::new(level, content, Local::now()));
        self.refresh().await;
    }

    pub async fn refresh(&self) {
        let _ = self.sender.send(self.clone()).await;
    }
}
