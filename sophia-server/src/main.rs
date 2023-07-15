use clap::Parser;

use sophia_core;
use sophia_core::errors::Result;

mod service;
mod repository;
mod controller;
mod server;


#[derive(Parser, Debug)] // requires `derive` feature
#[clap(name = "sophia-server")]
pub struct Args {
    #[arg(short = 'a', long = "addr", default_value = "0.0.0.0:5858")]
    address: String,
    #[arg(short = 'c', long = "crt", default_value = "./sophia-core/cert/cert.crt")]
    cert: String,
    #[arg(short = 'k', long = "key", default_value = "./sophia-core/cert/cert.key")]
    key: String,
    #[arg(default_value = "quic-demo")]
    application_level_protocol: String,
}


#[tokio::main]
async fn main() -> Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .parse_env("RUST_LOG")
        .init();

    let args = Args::parse();

    server::run(args).await?;

    Ok(())
}


