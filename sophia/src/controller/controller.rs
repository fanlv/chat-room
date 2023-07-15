use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use crossterm::event::KeyCode;
use futures_util::future::BoxFuture;
use log::Level;
use tokio::sync::mpsc::Sender;
use tokio::sync::RwLock;

use sophia_core::command::CommandType;
use sophia_core::errno_new;
use sophia_core::errors::Result;
use sophia_core::model::{Request, Response, User};
use sophia_net::quic;

use crate::config;
use crate::controller::handler::HandlerImpl;
use crate::controller::MessageViewModel;
use crate::controller::view_model::AppViewModel;

use super::handler::Handler;

macro_rules! async_function {
    ($function:expr) => {
        Arc::new(|ctx: Controller, request: Request|  Box::pin($function(ctx, request)))
    };
}

type Callback = Arc<dyn Send + Sync + Fn(Controller, Request) -> BoxFuture<'static, Result<Response>>>;


#[derive(Clone)]
pub struct Controller {
    callbacks: HashMap<CommandType, Callback>,
    conn: Arc<RwLock<Option<quic::Connection>>>,
    pub session_id: Arc<RwLock<String>>,
    view_model: Arc<RwLock<AppViewModel>>,
    pub exit_app: Arc<RwLock<bool>>,
}

impl Controller {
    pub fn new(sender: Sender<AppViewModel>, conf: config::Config) -> Self {
        let callbacks = HashMap::new();
        let mut control = Self {
            callbacks,
            conn: Arc::new(RwLock::new(None)),
            session_id: Arc::new(RwLock::new(String::default())),
            view_model: Arc::new(RwLock::new(AppViewModel::new(sender, conf))),
            exit_app: Arc::new(RwLock::new(false)),
        };
        control.register_command();

        control
    }

    pub async fn set_conn(&mut self, conn: quic::Connection) {
        let mut curr_conn = self.conn.write().await;
        *curr_conn = Some(conn);
    }

    pub async fn opt_conn(&self) -> Option<quic::Connection> {
        let curr_conn = self.conn.read().await;
        let conn = curr_conn.clone();

        conn
    }

    pub async fn conn(&self) -> quic::Connection {
        let curr_conn = self.conn.read().await;
        let conn = curr_conn.as_ref().unwrap();

        conn.clone()
    }

    fn register_command(&mut self) {
        self.register(CommandType::NewMessage, async_function!(HandlerImpl::receive_message));
        self.register(CommandType::ChatMessageList, async_function!(HandlerImpl::receive_message_list));
        self.register(CommandType::UserOnline, async_function!(HandlerImpl::user_online));
        self.register(CommandType::UserOffline, async_function!(HandlerImpl::user_offline));
        self.register(CommandType::ChatUserList, async_function!(HandlerImpl::chat_user_list_to_user));
    }

    fn get(&self, cmd_type: CommandType) -> Result<Callback> {
        let res = self.callbacks.get(&cmd_type)
            .ok_or(errno_new!("cmd {:?} handler unsupported", cmd_type))?.clone();

        Ok(res)
    }

    fn register(&mut self, cmd_type: CommandType, callback: Callback) {
        self.callbacks.insert(cmd_type, callback);
    }

    pub async fn stop_accept_stream(&self) -> bool {
        self.exit_app.read().await.clone()
    }

    pub async fn log(&self, level: Level, content: String) {
        self.view_model.write().await.log(level, content).await;
    }

    pub async fn get_view_model(&self) -> AppViewModel {
        let vm = self.view_model.read().await.clone();
        vm
    }


    pub async fn update_user_list(&self, user_list: Vec<User>) {
        let mut state = self.view_model.write().await;
        state.user_list = user_list;
        state.refresh().await;
    }

    pub async fn refresh(&self) {
        self.view_model.read().await.refresh().await;
    }

    pub async fn push_message(&self, msg: MessageViewModel) {
        let mut state = self.view_model.write().await;
        state.message_list.push(msg);
        state.auto_set_scroll_messages().await;
        state.refresh().await;
    }

    pub async fn set_message_list(&self, msg_list: Vec<MessageViewModel>) {
        let mut state = self.view_model.write().await;
        state.message_list = msg_list;
        state.auto_set_scroll_messages().await;
        state.refresh().await;
    }


    pub async fn input_write(&self, character: char) {
        self.view_model.write().await.input_write(character);
    }

    pub async fn input_remove(&self) {
        self.view_model.write().await.input_remove();
    }

    pub async fn clean_input(&self) {
        self.view_model.write().await.clean_input();
    }


    pub async fn input_remove_previous(&self) {
        self.view_model.write().await.input_remove_previous();
    }


    pub async fn input_move_cursor(&self, movement: KeyCode) {
        self.view_model.write().await.input_move_cursor(movement);
    }

    pub async fn messages_scroll(&self, movement: KeyCode) {
        self.view_model.write().await.messages_scroll(movement).await;
    }
}


#[async_trait]
impl quic::RequestCallback for Controller {
    async fn handle_request(&self, request: Request) -> Result<Response> {
        let callback = self.get(request.cmd_type)?;
        let resp = callback(self.clone(), request).await;
        resp
    }
}
