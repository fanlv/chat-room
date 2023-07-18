use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use futures_util::future::BoxFuture;
use log::error;

use sophia_core::command::CommandType;
use sophia_core::consts::code;
use sophia_core::errno_new;
use sophia_core::errors::Result;
use sophia_core::model::{Request, Response};
use sophia_net::quic;

use crate::controller::conn_manager::ConnectionManager;
use crate::service::{ChatRepo, MessageRepo, SessionRepo, user};
use crate::service::push;

use super::handler::Handler;

#[derive(Clone)]
pub struct Server {
    callbacks: HashMap<CommandType, Callback>,
    pub cons: ConnectionManager,
    pub repo: Repository,
}

#[derive(Clone)]
pub struct Repository {
    pub session: Arc<dyn SessionRepo>,
    pub chat: Arc<dyn ChatRepo>,
    pub message: Arc<dyn MessageRepo>,
}

type Callback = Arc<dyn Send + Sync + Fn(Server, Request) -> BoxFuture<'static, Result<Response>>>;

macro_rules! async_function {
    ($function:expr) => {
        Arc::new(|s :Server , request: Request|  Box::pin($function(s,request)))
    };
}

impl Server {
    pub fn new(repo: Repository) -> Self {
        let callbacks = HashMap::new();
        let cons = ConnectionManager::new();
        let mut s = Self { callbacks, cons, repo };
        s.register_command();

        s
    }


    fn register_command(&mut self) {
        self.register(CommandType::Login, async_function!(Server::login_handler));
        self.register(CommandType::SendMessage, async_function!(Server::send_message_handler));
    }

    pub fn get_callback(&self, cmd_type: CommandType) -> Result<Callback> {
        let res = self.callbacks.get(&cmd_type)
            .ok_or(errno_new!("server receive cmd {:?} handler unsupported", cmd_type))?.clone();

        Ok(res)
    }

    pub fn register(&mut self, cmd_type: CommandType, callback: Callback) {
        self.callbacks.insert(cmd_type, callback);
    }


    pub async fn kick_out(&self, remote_addr: &str) -> Result<()> {
        // 1. remove client connection
        self.cons.remove(remote_addr).await;

        // 2. find user info
        let user = self.repo.session
            .get_with_remote_addr(remote_addr).await?
            .ok_or(errno_new!("not found {} user info", remote_addr))?;

        // 3. remove the user from the group user list
        self.repo.chat.remove(user.chat_id, remote_addr).await?;

        // 4. remove user session
        self.repo.session.remove(remote_addr).await?;

        // 5. notification user login out
        let res = push::user_offline_event(self, &user).await;
        if let Err(e) = res {
            error!("push::user_offline_event failed = {}", e);
        }

        Ok(())
    }


    async fn auth_session(&self, request: &Request) -> Result<Response> {
        if request.cmd_type == CommandType::Login {
            return Ok(Response::success("ok".to_string()));
        }

        let remote = &request.base.remote_add;
        let session_id = &request.base.session_id;
        let res = user::check_session(self, session_id, remote).await;

        match res {
            Ok(()) => { Ok(Response::success("ok".to_string())) }
            Err(e) => {
                Ok(Response::new(code::SESSION_ID_INVALID, e.to_string()))
            }
        }
    }
}


#[async_trait]
impl quic::RequestCallback for Server {
    async fn handle_request(&self, request: Request) -> Result<Response> {
        let result = self.auth_session(&request).await?;
        if result.code != 0 {
            error!("[{}]: receive invalid session request = {:?} ",
                    request.base.remote_add, request);

            return Ok(result);
        }

        let callback = self.get_callback(request.cmd_type)?;
        let resp = callback(self.clone(), request.clone()).await;
        resp
    }
}


