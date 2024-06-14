use std::net::*;

use clap::Parser;
use trust_dns_resolver::config::*;
use trust_dns_resolver::TokioAsyncResolver;
use url::Url;

use config::Config;
use sophia_core::errno_new;
use sophia_core::errors::Result;

mod client;
mod config;
mod controller;
mod view_model;
mod ui;

#[derive(Parser, Debug)] // requires `derive` feature
#[clap(name = "sophia")]
pub struct Args {
    /// login user_name
    #[arg(short = 'u', long = "user", default_value = "")]
    user_name: String,
    /// login password
    #[arg(short = 'p', long = "password", default_value = "666666")]
    password: String,
    /// room id
    #[arg(short = 'c', long = "chat_id", default_value = "10086")]
    chat_id: i64,
    /// cert path
    #[arg(short = 'd', long = "der", default_value = "./sophia-core/cert/cert.der")]
    cert: String,
    /// server address
    #[arg(short = 's', long = "server", default_value = "localhost:5858")]
    server: String,
    /// theme
    #[arg(short = 't', long = "theme", default_value = "dark")]
    theme: String,
    /// e.g. www.example.com
    #[arg(default_value = "")]
    server_name: String,
    /// e.g. www.example.com:5588
    #[arg(default_value = "")]
    server_address: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    set_up_debug_log();


    let args = get_env_args().await?;
    let config = Config::from_args(args);

    client::run(config).await?;

    Ok(())
}

// use std::time::Duration;
fn set_up_debug_log() {
    // env_logger::builder()
    //     .filter_level(log::LevelFilter::Off)
    //     .parse_env("RUST_LOG")
    //     .init();
    //
    // return;

    // console_subscriber::ConsoleLayer::builder()
    //     // set how long the console will retain data from completed tasks
    //     .retention(Duration::from_secs(60))
    //     // set the address the server is bound to
    //     .server_addr(([127, 0, 0, 1], 8881))
    //     // ... other configurations ...
    //     .init();


    // use std::fs::File;
    // use std::io::Write;
    // use log::LevelFilter;

    // let log_file = "/Users/fanlv/Desktop/log_output.txt";
    //
    // let file = File::create(log_file).expect("Unable to create log file");
    //
    //
    // fern::Dispatch::new()
    //     .format(move |out, message, record| {
    //         out.finish(format_args!(
    //             "{date} {level} {message}",
    //             date = chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
    //             level = 1,
    //             message = message,
    //         ))
    //     })
    //     .level(LevelFilter::Info)
    //     .chain(fern::Output::call(move |data| {
    //         let mut file = file.try_clone().expect("Failed to clone log file handle");
    //
    //         if let Err(e) = writeln!(file, "{:?}", data.args()) {
    //             eprintln!("Error while writing logs: {:?}", e);
    //         }
    //     }))
    //     .apply()
    //     .expect("Failed to set up logger");
}


async fn get_env_args() -> Result<Args> {
    let mut args = Args::parse();

    let url_str = format!("https://{}", args.server);

    // get hostname
    let mut hostname = "localhost".to_string();
    let url = Url::parse(&url_str)
        .map_err(|e| errno_new!("parse server url failed , err =  {}", e))?;
    if let Some(host) = url.host_str() {
        hostname = host.to_string()
    }

    // get port
    let mut port: u16 = 5858;
    if let Some(p) = url.port() {
        port = p
    }

    // get ip from hostname
    let ip = lookup_ip(&hostname).await?;

    args.server_address = format!("{}:{}", ip.to_string(), port.to_string());
    args.server_name = hostname;


    Ok(args)
}

async fn lookup_ip(domain: &str) -> Result<IpAddr> {
    let response = TokioAsyncResolver::tokio(
        ResolverConfig::default(),
        ResolverOpts::default()).lookup_ip(domain).await
        .map_err(|e| errno_new!("failed lookup_ip {} , err =  {}",domain, e))?;


    response.iter().next().ok_or_else(|| errno_new!("no addresses returned"))
}

