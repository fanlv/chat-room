use std::collections::HashMap;
use std::sync::Arc;

use sophia_net::quic::Connection;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct ConnectionManager {
    connections: Arc<RwLock<HashMap<String, Connection>>>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        ConnectionManager {
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn get(&self, remote_address: &str) -> Option<Connection> {
        let connections = self.connections.read().await;
        connections.get(remote_address).map(|conn| conn.clone())
    }

    pub async fn put(&self, conn: Connection) {
        let mut connections = self.connections.write().await;
        connections.insert(conn.remote_address(), conn);
    }

    pub async fn remove(&self, remote_address: &str) {
        let mut connections = self.connections.write().await;
        connections.remove(remote_address);
    }
}