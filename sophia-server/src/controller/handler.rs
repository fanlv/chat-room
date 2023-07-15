use async_trait::async_trait;

use sophia_core::{errno, errno_new};
use sophia_core::command::Command;
use sophia_core::consts::code;
use sophia_core::errors::Result;
use sophia_core::model::{Request, Response};

use crate::service::{message, push, user};

use super::server::Server;

/// Client -> Server
#[async_trait]
pub trait Handler {
    async fn login_handler(s: Server, request: Request) -> Result<Response>;
    async fn send_message_handler(s: Server, request: Request) -> Result<Response>;
}


#[async_trait]
impl Handler for Server {
    /// handle client login request
    async fn login_handler(s: Server, request: Request) -> Result<Response> {
        let remote = request.base.remote_add;

        if let Command::Login(login) = request.cmd {
            if login.user_name.len() == 0 {
                let response = Response::new(code::LOGIN_FAILED, "user name invalid".to_string());
                return Ok(response);
            }

            if !user::check_user_name(&s, &login.user_name, login.chat_id).await? {
                let msg = format!("username {} already exists, please choose a different username and try signing in again ", &login.user_name);
                let response = Response::new(code::USER_NAME_DUPLICATE_ERROR, msg);
                return Ok(response);
            }


            if !user::auth(&login).await? {
                let response = Response::new(code::LOGIN_FAILED, "password invalid".to_string());
                return Ok(response);
            }


            let user_info = user::login_handler(&s, login, remote.clone()).await?;
            let session_id = user_info.session_id.clone();

            push::user_online_event(&s, &user_info).await?;
            push::chat_message_list(&s, &user_info).await?;

            let resp = Response::success(session_id);
            return Ok(resp);
        }


        errno!("cmd invalid!")
    }


    /// handle client send text message request
    async fn send_message_handler(s: Server, request: Request) -> Result<Response> {
        if let Command::SendTextMessage { msg, chat_id } = request.cmd {
            let user = s.repo.session.get(&request.base.session_id).await?
                .ok_or(errno_new!("session_id invalid"))?;

            if chat_id != user.chat_id {
                return Ok(Response::new(code::CHAT_ID_INVALID, "chat_id invalid".to_string()));
            }

            message::send(&s, user, &msg).await?;


            let resp = Response::success("".to_string());
            return Ok(resp);
        }


        errno!("cmd invalid!")
    }
}

