use std::{fs, net::SocketAddr, sync::Arc};
use std::string::ToString;

use quinn::{ClientConfig, Endpoint};

use sophia_core::errno_new;
use sophia_core::errors::Result;

use super::connection;

#[derive(Clone)]
pub struct Client {
    cert_path: String,
    server_addr: String,
    server_name: String,
    application_level_protocols: Vec<String>,
}


impl Client {
    pub fn new() -> Self {
        Client {
            cert_path: String::new(),
            server_addr: String::new(),
            server_name: String::new(),
            application_level_protocols: connection::ALPN_QUIC_HTTP.iter().map(|s| s.to_string()).collect(),
        }
    }

    pub fn with_cert_path(&mut self, cert_path: String) -> &mut Self {
        self.cert_path = cert_path;
        self
    }

    pub fn with_server_addr(&mut self, server_addr: String) -> &mut Self {
        self.server_addr = server_addr;
        self
    }

    pub fn with_server_name(&mut self, server_name: String) -> &mut Self {
        self.server_name = server_name;
        self
    }

    pub fn with_application_level_protocols(&mut self, alp: Vec<String>) -> &mut Self {
        self.application_level_protocols = alp;
        self
    }


    pub async fn connect(&self) -> Result<connection::Connection> {
        let addr = self.server_addr.parse::<SocketAddr>()?;

        // 1. set up cert and Client config
        let mut roots = rustls::RootCertStore::empty();
        roots.add(&rustls::Certificate(
            fs::read(&self.cert_path)
                .map_err(|e| errno_new!("read der cert file failed {} , \n err : {}",&self.cert_path, e))?
        ))?;

        let mut client_crypto = rustls::ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(roots)
            .with_no_client_auth();
        client_crypto.alpn_protocols = self.application_level_protocols
            .iter()
            .map(|x| x.as_bytes().to_vec())
            .collect();

        let mut client_config = ClientConfig::new(Arc::new(client_crypto));

        let transport = connection::get_transport();
        client_config.transport_config(Arc::new(transport));


        // 2. connect server
        let mut endpoint = Endpoint::client("[::]:0".parse().unwrap())?;
        endpoint.set_default_client_config(client_config);

        let conn = endpoint.connect(addr, &self.server_name)?.await?;

        let conn = connection::Connection::new(conn);

        Ok(conn)
    }
}


