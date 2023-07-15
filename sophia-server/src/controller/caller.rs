use async_trait::async_trait;

use super::server::Server;

#[async_trait]
pub trait Caller {}


#[async_trait]
impl Caller for Server {}

