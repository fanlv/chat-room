use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;

use sophia_core::errors::Result;
use sophia_core::model::Message;

use crate::service::MessageRepo;

#[derive(Clone)]
pub struct MessageMemoryImpl {
    chat_id_to_messages: Arc<RwLock<HashMap<i64, Vec<Message>>>>,
}

impl MessageMemoryImpl {
    pub fn new() -> Self {
        Self {
            chat_id_to_messages: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}


#[async_trait]
impl MessageRepo for MessageMemoryImpl {
    async fn save(&self, msg: Message) -> Result<()> {
        let mut chat_id_to_messages = self.chat_id_to_messages.write().await;
        let messages = chat_id_to_messages.entry(msg.user.chat_id).or_insert(Vec::new());
        messages.push(msg);

        Ok(())
    }

    async fn get(&self, chat_id: i64) -> Result<Vec<Message>> {
        let chat_id_to_messages = self.chat_id_to_messages.read().await;
        let messages = chat_id_to_messages.get(&chat_id);

        match messages {
            Some(messages) => Ok(messages.clone()),
            None => Ok(Vec::new()), // Return an empty vec if chat_id is not found
        }
    }
}