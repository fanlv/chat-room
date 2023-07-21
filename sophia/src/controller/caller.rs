use async_trait::async_trait;

use sophia_core::{command, errno};
use sophia_core::command::Command;
use sophia_core::errors::Result;
use sophia_core::model::{Request, Response};

use super::controller::Controller;

#[async_trait]
pub trait Caller {
    async fn login(&self, cmd: command::Login) -> Result<String>;
    async fn send_msg(&self, msg: &str, chat_id: i64) -> Result<String>;
}


#[async_trait]
impl Caller for Controller {
    async fn login(&self, cmd: command::Login) -> Result<String> {
        if self.not_connect().await {
            return errno!("connect failed")
        }

        let req = Request::new(Command::Login(cmd));

        let resp = self.conn().await.send(req).await?;
        let _ = if_response_code_not_zero_return_err(&resp)?;
        let session_id = resp.msg;

        return Ok(session_id);
    }

    async fn send_msg(&self, msg: &str, chat_id: i64) -> Result<String> {
        if self.not_connect().await {
            return errno!("connect failed")
        }

        let mut req = Request::new(Command::SendTextMessage { msg: msg.to_string(), chat_id });
        req.base.session_id = self.session_id.read().await.to_string();

        let resp = self.conn().await.send(req).await?;
        let _ = if_response_code_not_zero_return_err(&resp)?;
        let session_id = resp.msg;

        return Ok(session_id);
    }
}

fn if_response_code_not_zero_return_err(resp: &Response) -> Result<()> {
    if resp.code != 0 {
        return errno!("code : {}, msg = {}", resp.code, resp.msg);
    }

    Ok(())
}

