use chrono::prelude::*;
use rand::distributions::{Alphanumeric, DistString};

use sophia_core::{errno, errno_new};
use sophia_core::command::Login;
use sophia_core::errors::Result;
use sophia_core::model::UserInfo;

use crate::controller::Server;

pub async fn login_handler(s: &Server, login: Login, remote: String) -> Result<UserInfo> {
    // 1. make session
    let session_id = Alphanumeric.sample_string(&mut rand::thread_rng(), 32);

    let now = Utc::now().timestamp();
    let user_info = UserInfo::new(login.user_name, remote,
                                  session_id.clone(), login.chat_id, now);

    // 2. save session_id -> user_info
    s.repo.session.save(session_id.clone(), user_info.clone()).await?;

    // 3. save user to chat user list
    s.repo.chat.save(login.chat_id, user_info.clone()).await?;

    return Ok(user_info);
}


pub async fn auth(request: &Login) -> Result<bool> {
    Ok(request.password == "666666")
}

pub async fn check_user_name(s: &Server, user_name: &str, chat_id: i64) -> Result<bool> {
    let user = s.repo.chat.get(chat_id).await?;

    for (_, user) in user.iter() {
        if user.name == user_name {
            return Ok(false);
        }
    }

    Ok(true)
}


pub async fn check_session(s: &Server, session_id: &str, remote: &str) -> Result<()> {
    let user = s.repo.session.get(session_id).await?
        .ok_or(errno_new!("session_id invalid"))?;

    if user.address != remote {
        return errno!("address {} invalid",remote);
    }

    Ok(())
}
