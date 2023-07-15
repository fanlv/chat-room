use std::{
    fs, fs::File,
    io::BufReader, net::SocketAddr,
    path::PathBuf,
    str,
    sync::Arc,
};

use quinn::{Endpoint, ServerConfig};

use sophia_core::errno_new;
use sophia_core::errors::Result;

use crate::quic::connection;

#[derive(Clone)]
pub struct Server {
    cert_path: String,
    key_path: String,
    listen_addr: String,
    application_level_protocols: Vec<String>,
}


pub struct Listener {
    endpoint: Endpoint,
}

impl Listener {
    pub fn new(endpoint: Endpoint) -> Self {
        Listener {
            endpoint,
        }
    }

    pub async fn accept(&self) -> Result<connection::Connection> {
        let connecting = self.endpoint.accept().await
            .ok_or(errno_new!("accept nil con"))?;
        let conn = connecting.await?;

        Ok(connection::Connection::new(conn))
    }
}

impl Server {
    pub fn new() -> Server {
        Server {
            cert_path: String::new(),
            key_path: String::new(),
            listen_addr: String::new(),
            application_level_protocols: connection::ALPN_QUIC_HTTP.iter().map(|s| s.to_string()).collect(),
        }
    }

    pub fn address(&self) -> &str {
        &self.listen_addr
    }


    pub fn with_cert_path(&mut self, cert_path: String) -> &mut Self {
        self.cert_path = cert_path;
        self
    }

    pub fn with_key_path(&mut self, key_path: String) -> &mut Self {
        self.key_path = key_path;
        self
    }

    pub fn with_listen_addr(&mut self, listen_addr: String) -> &mut Self {
        self.listen_addr = listen_addr;
        self
    }

    pub fn with_application_level_protocols(&mut self, alp: Vec<String>) -> &mut Self {
        self.application_level_protocols = alp;
        self
    }


    /// 启动一个 Quic 服务端
    pub async fn listen(&mut self) -> Result<Listener> {
        let addr = self.listen_addr.parse::<SocketAddr>()?;
        let (certs, private_key) = read_certs_from_file(&self.cert_path, &self.key_path)
            .map_err(|e| errno_new!("read cert file failed, {} , {}, \n err = {}",&self.cert_path, &self.key_path, e))?;


        let mut server_crypto = rustls::ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(certs, private_key)?;
        server_crypto.alpn_protocols = self.application_level_protocols.
            iter().map(|x| x.as_bytes().to_vec()).collect();

        // server_crypto.key_log = Arc::new(rustls::KeyLogFile::new());
        let mut server_config = ServerConfig::with_crypto(Arc::new(server_crypto));
        let transport = connection::get_transport();
        server_config.transport_config(Arc::new(transport));
        // server_config.use_retry(true);


        // let server_config = ServerConfig::with_single_cert(certs, private_key)?;
        let endpoint = Endpoint::server(server_config, addr)?;

        Ok(Listener::new(endpoint))
    }
}

fn read_certs_from_file(cert_path: &str, key_path: &str) -> Result<(Vec<rustls::Certificate>, rustls::PrivateKey)> {
    let mut cert_chain_reader = BufReader::new(File::open(cert_path)?);
    let certs = rustls_pemfile::certs(&mut cert_chain_reader)?
        .into_iter()
        .map(rustls::Certificate)
        .collect();

    // let mut key_reader = BufReader::new(File::open(key_path)?);
    // // if the file starts with "BEGIN RSA PRIVATE KEY"
    // let mut keys = rustls_pemfile::rsa_private_keys(&mut key_reader)?;
    // // if the file starts with "BEGIN PRIVATE KEY"
    // // let mut keys = rustls_pemfile::pkcs8_private_keys(&mut key_reader)?;
    // assert_eq!(keys.len(), 1);
    // let key = rustls::PrivateKey(keys.remove(0));

    let key_path = PathBuf::from(key_path);

    let key = fs::read(key_path.clone()).map_err(|e| errno_new!("failed to read private key , {}", e.to_string()))?;
    let key = if key_path.extension().map_or(false, |x| x == "der") {
        rustls::PrivateKey(key)
    } else {
        let pkcs8 = rustls_pemfile::pkcs8_private_keys(&mut &*key)
            .map_err(|e| errno_new!("malformed PKCS #8 private key , {}", e.to_string()))?;
        match pkcs8.into_iter().next() {
            Some(x) => rustls::PrivateKey(x),
            None => {
                let rsa = rustls_pemfile::rsa_private_keys(&mut &*key)
                    .map_err(|e| errno_new!("malformed PKCS #1 private key , {}", e.to_string()))?;
                match rsa.into_iter().next() {
                    Some(x) => rustls::PrivateKey(x),
                    None => {
                        panic!("no private keys found");
                    }
                }
            }
        }
    };


    Ok((certs, key))
}
