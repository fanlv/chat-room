use std::str;
use std::sync::Arc;

use async_trait::async_trait;
use futures_util::future::BoxFuture;
use log::error;
use tokio::time::Duration;

use sophia_core::{errno, errno_new};
use sophia_core::consts::code;
use sophia_core::errors::Errno::ConnectionClosed;
use sophia_core::errors::Result;
use sophia_core::model::{Request, Response};

const MAX_SIZE: usize = 1024 * 1024;
const KEEP_ALIVE_INTERVAL: u64 = 1;
const IDLE_TIMEOUT: u64 = 3;

pub(super) const ALPN_QUIC_HTTP: &[&'static str] = &["hq-29", "quic-demo"];

#[derive(Clone)]
pub struct Connection {
    conn: Arc<quinn::Connection>,
}

#[async_trait]
pub trait RequestCallback: Send + Sync + Clone + 'static {
    async fn handle_request(&self, request: Request) -> Result<Response>;
}

#[async_trait]
impl<F> RequestCallback for F
    where
        F: Fn(Request) -> BoxFuture<'static, Result<Response>> + Send + Sync + Clone + 'static,
{
    async fn handle_request(&self, request: Request) -> Result<Response> {
        (self)(request).await
    }
}

impl Connection {
    pub(super) fn new(conn: quinn::Connection) -> Self {
        Connection { conn: Arc::new(conn) }
    }

    pub fn remote_address(&self) -> String {
        self.conn.remote_address().to_string()
    }

    pub async fn closed(&self) {
        let _ = self.conn.close(0u32.into(), b"");
    }

    pub async fn accept_stream(&self) -> Result<(quinn::SendStream, quinn::RecvStream)> {
        let stream = self.conn.accept_bi().await;
        let stream = match stream {
            Err(quinn::ConnectionError::ApplicationClosed { .. }) => {
                return Err(ConnectionClosed);
            }
            Err(e) => {
                return errno!("accept_stream failed: {}", e.to_string());
            }
            Ok(s) => s,
        };

        Ok(stream)
    }

    pub async fn read_request(&self, mut recv: quinn::RecvStream) -> Result<Request> {
        let vec_u8 = recv.read_to_end(MAX_SIZE).await
            .map_err(|e| errno_new!("read_to_end err = {}",e))?;

        let mut request: Request = serde_json::from_slice(&vec_u8).unwrap();
        request.base.remote_add = self.remote_address().to_string();

        Ok(request)
    }

    pub async fn write_response(&self, resp: Response, mut send: quinn::SendStream) -> Result<()> {
        let serialized = serde_json::to_vec(&resp)
            .map_err(|e| errno_new!("serde_json::to_vec err = {}",e))?;
        send.write_all(&serialized).await
            .map_err(|e| errno_new!("failed to send request: {}", e))?;
        send.finish().await
            .map_err(|e| errno_new!("failed to shutdown stream: {}", e))?;

        Ok(())
    }

    pub async fn accept_request(&self, callback: impl RequestCallback) -> Result<()>
    {
        loop {
            let callback = callback.clone();
            let stream = self.accept_stream().await?;
            let res = Self::handle(self.clone(), stream.0,
                                   stream.1, callback);

            tokio::spawn(async move {
                if let Err(e) = res.await {
                    error!("handle_request failed = {}", e);
                }
            });
        }
    }


    async fn handle(conn: Connection, send: quinn::SendStream,
                    recv: quinn::RecvStream, callback: impl RequestCallback) -> Result<()>
    {
        let request = conn.read_request(recv).await?;
        let resp = callback.handle_request(request).await;
        match resp {
            Err(e) => conn.write_response(Response::new(code::SESSION_ID_INVALID, e.to_string()), send).await?,
            Ok(resp) => conn.write_response(resp, send).await?,
        }

        Ok(())
    }


    pub async fn send(&self, request: Request) -> Result<Response> {
        let (mut send, mut recv) = self.conn.open_bi().await
            .map_err(|e| errno_new!("conn.open_bi failed =  {}",e))?;

        // 1. read request -> json
        let serialized = serde_json::to_vec(&request)
            .map_err(|e| errno_new!("encode req failed =  {}",e))?;

        // 2. send json data
        send.write_all(&serialized).await
            .map_err(|e| errno_new!("failed to send request = {:?} , err = {}", request, e))?;
        send.finish().await
            .map_err(|e| errno_new!("failed to send finish, err =  {}", e))?;

        // 3. read json data
        let vec_u8 = recv.read_to_end(MAX_SIZE).await
            .map_err(|e| errno_new!("read data failed, err =  {}", e))?;

        // 4. json data -> Response
        let resp: Response = serde_json::from_slice(&vec_u8)
            .map_err(|e| errno_new!("decode data failed, err =  {}", e))?;


        Ok(resp)
    }
}


pub(super) fn get_transport() -> quinn::TransportConfig {
    let mut transport = quinn::TransportConfig::default();
    transport.keep_alive_interval(Some(Duration::from_secs(KEEP_ALIVE_INTERVAL)))
        .max_idle_timeout(Some(Duration::from_secs(IDLE_TIMEOUT).try_into().unwrap()));

    transport
}
