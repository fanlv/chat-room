use chrono::Utc;

use sophia_core::command::Command;
use sophia_core::errors::Result;
use sophia_core::model::{Message, Request, User, UserInfo};

use crate::controller::Server;
use crate::service::push;

pub async fn send(s: &Server, user: UserInfo, msg: &str) -> Result<()> {
    let u = User::from_user_info(&user);
    let now = Utc::now().timestamp();
    let message = Message {
        user: u,
        time: now,
        content: msg.to_string(),
    };

    let _ = s.repo.message.save(message.clone()).await?;
    let req = Request::new(Command::NewMessage(message));

    push::push_to_chat_user(req, s, "", user.chat_id).await?;

    Ok(())
}