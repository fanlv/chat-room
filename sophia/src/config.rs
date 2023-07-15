use log::debug;
use rand::distributions::{Alphanumeric, DistString};

use crate::Args;

#[derive(Clone, Debug)]
pub struct Config {
    pub cert_path: String,
    pub server_addr: String,
    pub server_name: String,
    pub application_level_protocols: Vec<String>,
    pub user_name: String,
    pub theme: String,
    pub chat_id: i64,
    pub password: String,
}


impl Config {
    pub fn from_args(args: Args) -> Self {
        let mut config = Config {
            cert_path: args.cert,
            server_addr: args.server_address,
            server_name: args.server_name,
            application_level_protocols: vec!["quic-demo".to_string()],
            user_name: args.user_name,
            chat_id: args.chat_id,
            password: args.password,
            theme: args.theme,
        };

        if config.user_name.len() == 0 {
            let string = Alphanumeric.sample_string(&mut rand::thread_rng(), 8);
            debug!("set random user_name : {}", string);
            config.user_name = whoami::username();
        }


        config
    }

    // pub fn default() -> Self {
    //     Config {
    //         cert_path: String::default(),
    //         server_addr: String::default(),
    //         server_name: String::default(),
    //         application_level_protocols: vec![],
    //         user_name: String::default(),
    //         chat_id: 0,
    //         password: String::default(),
    //     }
    // }
}
