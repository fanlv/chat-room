use derive_more::Display;
use serde::{Deserialize, Serialize};

use crate::model::{Message, User};

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq, Display, Deserialize, Serialize)]
pub enum CommandType {
    // server handle cmd
    Login,
    SendMessage,

    // client handler cmd
    ChatMessageList,
    UserOnline,
    UserOffline,
    ChatUserList,
    NewMessage,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Command {
    Login(Login),
    SendTextMessage {
        msg: String,
        chat_id: i64,
    },
    UserOnline {
        time: i64,
        user: User,
    },
    UserOffline {
        time: i64,
        user: User,
    },
    ChatUserList {
        user_list: Vec<User>,
    },
    NewMessage(Message),
    ChatMessageList {
        message_list: Vec<Message>,
    },
}


#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Login {
    pub user_name: String,
    pub password: String,
    pub chat_id: i64,
}


impl Command {
    pub fn command_type(&self) -> CommandType {
        match self {
            Command::Login { 0: _ } => CommandType::Login,
            Command::SendTextMessage { msg: _, chat_id: _ } => CommandType::SendMessage,
            Command::UserOffline { time: _, user: _, } => CommandType::UserOffline,
            Command::UserOnline { time: _, user: _, } => CommandType::UserOnline,
            Command::ChatUserList { user_list: _ } => CommandType::ChatUserList,
            Command::NewMessage { 0: _ } => CommandType::NewMessage,
            Command::ChatMessageList { message_list: _ } => CommandType::ChatMessageList,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum CommandResult {
    DataStr(String),
    Abc,
}