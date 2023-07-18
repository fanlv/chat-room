use chrono::prelude::*;
use log::error;

use sophia_core::command::Command;
use sophia_core::errors::Result;
use sophia_core::model::{Request, User, UserInfo};

use crate::controller::Server;

pub async fn user_offline_event(s: &Server, user_info: &UserInfo) -> Result<()> {
    push_user_conn_change_event(s, user_info, false).await
}


pub async fn user_online_event(s: &Server, user_info: &UserInfo) -> Result<()> {
    push_user_conn_change_event(s, user_info, true).await
}


pub async fn push_user_conn_change_event(s: &Server, user_info: &UserInfo, is_online: bool) -> Result<()> {
    let remote_to_user = s.repo.chat.get(user_info.chat_id).await?;
    let user_vec: Vec<UserInfo> = remote_to_user.values().cloned().collect();
    let now = Utc::now().timestamp();

    let req;
    let except_for_addr: &str;
    if is_online {
        except_for_addr = "";
        req = Request::new(Command::UserOnline {
            time: now,
            user: User::from_user_info(user_info),
        });
    } else {
        except_for_addr = &user_info.address;
        req = Request::new(Command::UserOffline {
            time: now,
            user: User::from_user_info(user_info),
        })
    }

    push_to_user(req, s, &user_info.address, &user_vec).await;
    chat_user_list(s, user_info.chat_id, except_for_addr, &user_vec).await?;

    Ok(())
}


pub async fn chat_user_list(s: &Server, chat_id: i64, except_for_addr: &str, to_users: &Vec<UserInfo>) -> Result<()> {
    let remote_to_user = s.repo.chat.get(chat_id).await?;

    let mut user_vec: Vec<UserInfo> = remote_to_user.values().cloned().collect();
    user_vec.sort_by(|a, b| {
        a.login_time.cmp(&b.login_time)
    });


    let users: Vec<User> = user_vec
        .iter()
        .map(|info| User::from_user_info(info))
        .collect();

    let req = Request::new(Command::ChatUserList {
        user_list: users,
    });

    push_to_user(req, s, except_for_addr, to_users).await;

    Ok(())
}


pub(super) async fn push_to_chat_user(req: Request, s: &Server, except_for_addr: &str, chat_id: i64) -> Result<()> {
    let remote_to_user = s.repo.chat.get(chat_id).await?;
    let user_vec: Vec<UserInfo> = remote_to_user.values().cloned().collect();
    push_to_user(req, s, except_for_addr, &user_vec).await;

    Ok(())
}

async fn push_to_user(req: Request, s: &Server, except_for_addr: &str, to_users: &Vec<UserInfo>) {
    for u in to_users {
        if u.address == except_for_addr {
            continue;
        }

        let user_remote = u.address.to_string();
        let chat_id = u.chat_id;

        let conn = s.cons.get(&user_remote).await;
        if conn.is_none() {
            error!("conn not found {}, req = {:?}", user_remote, req);
            continue;
        }

        let conn = conn.unwrap();
        let user_remote = user_remote.clone();
        let req = req.clone();
        tokio::spawn(async move {
            let res = conn.send(req.clone()).await;
            match res {
                Err(e) => error!("failed push {} online in chat {}  failed : {} , req {:?}", user_remote, chat_id, e,req),
                _ => (),//info!("success push to client {}, req = {:?}", user_remote, req )
            }
        });
    }
}


pub async fn chat_message_list(s: &Server, user_info: &UserInfo) -> Result<()> {
    let msg_list = s.repo.message.get(user_info.chat_id).await?;
    let req = Request::new(Command::ChatMessageList { message_list: msg_list });
    push_to_user(req, s, "", &vec![user_info.clone()]).await;

    Ok(())
}
