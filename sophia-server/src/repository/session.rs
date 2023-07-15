use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;

use sophia_core::errors::Result;
use sophia_core::model::UserInfo;

#[derive(Clone)]
pub struct SessionMemoryImpl {
    session_id_to_user: Arc<RwLock<HashMap<String, UserInfo>>>,
    remote_to_user_info: Arc<RwLock<HashMap<String, UserInfo>>>,
}

impl SessionMemoryImpl {
    /// Creates a new instance of ChatMemoryImpl with an empty HashMap.
    pub fn new() -> Self {
        Self {
            session_id_to_user: Arc::new(RwLock::new(HashMap::new())),
            remote_to_user_info: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}


#[async_trait]
impl crate::service::SessionRepo for SessionMemoryImpl {
    async fn list_sessions(&self) -> HashMap<String, UserInfo> {
        let sessions = self.session_id_to_user.read().await;
        sessions.clone()
    }

    async fn list_remote(&self) -> HashMap<String, UserInfo> {
        let sessions = self.remote_to_user_info.read().await;
        sessions.clone()
    }

    async fn save(&self, session_id: String, user: UserInfo) -> Result<()> {
        let mut connections = self.session_id_to_user.write().await;
        connections.insert(session_id, user.clone());


        let remote = user.address.clone();
        let mut remote_to_user_info = self.remote_to_user_info.write().await;
        remote_to_user_info.insert(remote, user);

        Ok(())
    }

    async fn get(&self, session_id: &str) -> Result<Option<UserInfo>> {
        let sessions = self.session_id_to_user.read().await;

        Ok(sessions.get(session_id).map(|u| u.clone()))
    }


    async fn get_with_remote_addr(&self, remote: &str) -> Result<Option<UserInfo>> {
        let remote_to_user_info = self.remote_to_user_info.read().await;

        Ok(remote_to_user_info.get(remote).map(|u| u.clone()))
    }

    async fn remove(&self, remote: &str) -> Result<()> {
        let mut sessions = self.remote_to_user_info.write().await;
        let user = sessions.remove(remote);

        if user.is_none() {
            return Ok(());
        }

        let mut connections = self.session_id_to_user.write().await;
        connections.remove(user.unwrap().session_id.as_str());
        Ok(())
    }
}