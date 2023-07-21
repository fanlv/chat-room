use async_trait::async_trait;

use sophia_core::command::Command;
use sophia_core::errno;
use sophia_core::errors::Result;
use sophia_core::model::{Request, Response, User};

use crate::view_model::Message;
use crate::view_model::SomeUser;

use super::controller::Controller;

/// Server -> Client command handler func
#[async_trait]
pub(super) trait Handler {
    async fn receive_message(ctrl: Controller, request: Request) -> Result<Response>;
    async fn receive_message_list(ctrl: Controller, request: Request) -> Result<Response>;
    async fn user_online(ctrl: Controller, request: Request) -> Result<Response>;
    async fn user_offline(ctrl: Controller, request: Request) -> Result<Response>;
    async fn chat_user_list_to_user(ctrl: Controller, request: Request) -> Result<Response>;
}


pub struct HandlerImpl {}

#[async_trait]
impl Handler for HandlerImpl {
    async fn receive_message(ctrl: Controller, request: Request) -> Result<Response> {
        if let Command::NewMessage(message) = request.cmd {
            let msg = Message::from_message(message);

            tokio::spawn(async move {
                ctrl.push_message(msg).await;
            });

            let response = Response::success("ok".to_string());
            return Ok(response);
        }

        errno!("cmd {} invalid!", request.cmd_type)
    }

    async fn receive_message_list(ctrl: Controller, request: Request) -> Result<Response> {
        if let Command::ChatMessageList { message_list } = request.cmd {
            let message_list = message_list.iter().map(|msg| {
                Message::from_message(msg.clone())
            }).collect();

            // ctrl.log(Level::Info, format!("receive  message list: {:?}", message_list)).await;
            ctrl.set_message_list(message_list).await;


            let response = Response::success("ok".to_string());
            return Ok(response);
        }

        errno!("cmd {} invalid!", request.cmd_type)
    }


    async fn user_online(ctrl: Controller, request: Request) -> Result<Response> {
        if let Command::UserOnline { user, time } = request.cmd {
            return user_connection_change(ctrl, user, time, true).await;
        }

        errno!("cmd {} invalid!", request.cmd_type)
    }


    async fn user_offline(ctrl: Controller, request: Request) -> Result<Response> {
        if let Command::UserOffline { user, time } = request.cmd {
            return user_connection_change(ctrl, user, time, false).await;
        }

        errno!("cmd {} invalid!", request.cmd_type)
    }

    async fn chat_user_list_to_user(ctrl: Controller, request: Request) -> Result<Response> {
        if let Command::ChatUserList { user_list } = request.cmd {
            ctrl.update_user_list(user_list).await;

            let response = Response::success("ok".to_string());
            return Ok(response);
        }

        errno!("cmd {} invalid!", request.cmd_type)
    }

    // fn get_now_string() -> String {
    //     let system_time = SystemTime::now();
    //     let date_time: DateTime<Local> = system_time.into(); // 将 SystemTime 转换为 DateTime<Local>
    //     let time_string = date_time.format("%%m-%d %H:%M:%S").to_string(); // 格式化
    //
    // }
}

async fn user_connection_change(ctrl: Controller, user: User, time: i64, online: bool) -> Result<Response> {
    let mut action = "online";
    if !online {
        action = "offline";
    }

    let content = format!("{} {} is {} ", user.address, user.user_name, action);
    let msg = Message::new(time, content, SomeUser::System);

    tokio::spawn(async move {
        ctrl.push_message(msg).await;
    });

    let response = Response::success("ok".to_string());
    return Ok(response);
}

