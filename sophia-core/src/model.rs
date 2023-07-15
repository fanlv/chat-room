use serde::{Deserialize, Serialize};

use crate::command::{Command, CommandResult, CommandType};
use crate::consts;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UserInfo {
    pub name: String,
    pub session_id: String,
    pub address: String,
    pub chat_id: i64,
    pub login_time: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct User {
    pub user_name: String,
    pub address: String,
    pub chat_id: i64,
    pub login_time: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Message {
    pub user: User,
    pub time: i64,
    pub content: String,
}


#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Base {
    pub session_id: String,
    pub remote_add: String,
    pub user_info: Option<UserInfo>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Request {
    pub base: Base,
    pub cmd: Command,
    pub cmd_type: CommandType,
}


#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Response {
    pub msg: String,
    pub code: usize,
    pub data: Option<CommandResult>,
}

impl Base {
    pub fn new() -> Self {
        Base {
            session_id: String::default(),
            remote_add: String::new(),
            user_info: None,
        }
    }
}

impl Request {
    pub fn new(cmd: Command) -> Self {
        let cmd_type = cmd.command_type();
        let base = Base::new();
        Request { base, cmd, cmd_type }
    }
}


impl Response {
    pub fn new(code: usize, msg: String) -> Self {
        Response { code, msg, data: None }
    }

    pub fn success(msg: String) -> Self {
        Response { code: consts::code::SUCCESS, msg, data: None }
    }
}


impl User {
    pub fn new(user_name: String, address: String, chat_id: i64, login_time: i64) -> Self {
        User { user_name, address, chat_id, login_time }
    }

    pub fn from_user_info(u: &UserInfo) -> Self {
        User {
            user_name: u.name.to_string(),
            address: u.address.to_string(),
            chat_id: u.chat_id,
            login_time: u.login_time,
        }
    }
}

impl UserInfo {
    pub fn new(user_name: String, remote_address: String, session_id: String, chat_id: i64, login_time: i64) -> Self {
        Self {
            name: user_name,
            address: remote_address,
            session_id,
            chat_id,
            login_time,
        }
    }
}