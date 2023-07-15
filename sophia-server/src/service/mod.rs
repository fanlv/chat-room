use std::collections::HashMap;

use async_trait::async_trait;

use sophia_core::errors::Result;
use sophia_core::model::{Message, UserInfo};

pub mod user;
pub mod push;
pub mod message;

#[async_trait]
pub trait SessionRepo: Send + Sync {
    async fn list_sessions(&self) -> HashMap<String, UserInfo>;
    async fn list_remote(&self) -> HashMap<String, UserInfo>;
    async fn save(&self, session_id: String, user: UserInfo) -> Result<()>;
    async fn get(&self, session_id: &str) -> Result<Option<UserInfo>>;
    async fn get_with_remote_addr(&self, remote: &str) -> Result<Option<UserInfo>>;
    async fn remove(&self, remote: &str) -> Result<()>;
}


#[async_trait]
pub trait ChatRepo: Send + Sync {
    async fn list(&self) -> Vec<(i64, HashMap<String, UserInfo>)>;
    async fn save(&self, chat_id: i64, user: UserInfo) -> Result<()>;
    async fn get(&self, chat_id: i64) -> Result<HashMap<String, UserInfo>>;
    async fn remove(&self, chat_id: i64, remote: &str) -> Result<()>;
}

#[async_trait]
pub trait MessageRepo: Send + Sync {
    async fn save(&self, msg: Message) -> Result<()>;
    async fn get(&self, chat_id: i64) -> Result<Vec<Message>>;
}