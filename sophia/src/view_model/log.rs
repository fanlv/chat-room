use chrono::{DateTime, Local};
use log::Level;

#[derive(Clone, Debug)]
pub struct LogViewModel {
    pub contents: Vec<Log>,
}

impl LogViewModel {
    pub fn new() -> Self {
        Self { contents: Vec::new() }
    }
}


#[derive(Clone, Debug)]
pub struct Log {
    pub level: Level,
    pub content: String,
    pub time: DateTime<Local>,
}


impl Log {
    pub fn new(level: Level, content: String, time: DateTime<Local>) -> Self {
        Self { level, content, time }
    }
}

impl LogViewModel {
    pub fn log(&mut self, level: Level, content: String) {
        self.contents.push(Log::new(level, content, Local::now()));
    }
}
