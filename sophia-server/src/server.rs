use std::sync::Arc;

use log::{error, info};

use sophia_core;
use sophia_core::errors::Errno::ConnectionClosed;
use sophia_core::errors::Result;
use sophia_net::quic;

use crate::Args;
use crate::controller::{Repository, Server};
use crate::repository::chat::ChatMemoryImpl;
use crate::repository::message::MessageMemoryImpl;
use crate::repository::session::SessionMemoryImpl;

pub async fn run(args: Args) -> Result<()> {
    let mut quic_server = quic::Server::new();
    let quic_server = quic_server
        .with_cert_path(args.cert)
        .with_key_path(args.key)
        .with_application_level_protocols(vec![args.application_level_protocol])
        .with_listen_addr(args.address);


    let listen = quic_server.listen().await?;
    info!("listen add = {}", quic_server.address());

    let repo = setup_repo_impl();
    let server = Server::new(repo);


    loop {
        let conn = listen.accept().await;
        if let Err(e) = conn {
            error!("accept error = {:?}", e);
            continue;
        }

        let conn = conn.unwrap();
        server.cons.put(conn.clone()).await;


        let server = server.clone();
        let remote = conn.remote_address();

        tokio::spawn(async move {
            let res = conn.accept_request(server.clone()).await;
            if let Err(e) = res {
                match e {
                    ConnectionClosed => info!("remote {} connection lost", remote),
                    _ => error!("server lost client {} , reason = {} ",remote, e),
                }

                let res = server.clone().kick_out(&remote).await;
                if let Err(e) = res {
                    error!("kick_out client {}  failed = {}",remote, e);
                }
            } else {
                info!("remote {} connection lost", remote);
            }
        });
    }
}


fn setup_repo_impl() -> Repository {
    Repository {
        chat: Arc::new(ChatMemoryImpl::new()),
        session: Arc::new(SessionMemoryImpl::new()),
        message: Arc::new(MessageMemoryImpl::new()),
    }
}

