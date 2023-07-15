use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;

use sophia_core::errno_new;
use sophia_core::errors::Result;
use sophia_core::model::UserInfo;

#[derive(Clone)]
pub struct ChatMemoryImpl {
    chat_to_users: Arc<RwLock<HashMap<i64, HashMap<String, UserInfo>>>>,
}


impl ChatMemoryImpl {
    /// Creates a new instance of ChatMemoryImpl with an empty HashMap.
    pub fn new() -> Self {
        Self {
            chat_to_users: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl crate::service::ChatRepo for ChatMemoryImpl {
    async fn list(&self) -> Vec<(i64, HashMap<String, UserInfo>)> {
        let chat_to_users_read = self.chat_to_users.read().await;
        let chat_to_users_copy = chat_to_users_read.iter().map(|(&k, v)| (k, v.clone())).collect();
        chat_to_users_copy
    }


    async fn save(&self, chat_id: i64, user: UserInfo) -> Result<()> {
        let mut chat_to_users = self.chat_to_users.write().await;

        let remote = &user.address;
        chat_to_users.entry(chat_id)
            .or_insert_with(HashMap::new)
            .insert(remote.to_string(), user);

        Ok(())
    }

    async fn get(&self, chat_id: i64) -> Result<HashMap<String, UserInfo>> {
        let chat_to_users = self.chat_to_users.read().await;
        Ok(chat_to_users.get(&chat_id).cloned().unwrap_or_default())
    }


    async fn remove(&self, chat_id: i64, remote: &str) -> Result<()> {
        let mut chat_to_users = self.chat_to_users.write().await;
        let users = chat_to_users.get_mut(&chat_id)
            .ok_or(errno_new!("chat user list not found {}", chat_id))?;

        users.remove(remote)
            .ok_or(errno_new!("remote = {} user info not found ", remote))?;

        Ok(())
    }
}


