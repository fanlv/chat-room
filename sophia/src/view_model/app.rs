use crate::config::Config;
use crate::view_model::input::InputViewModel;
use crate::view_model::log::LogViewModel;
use crate::view_model::messages::ChatMessageViewModel;
use crate::view_model::user_list::UserViewModel;

#[derive(Clone, Debug)]
pub struct AppViewModel {
    pub log_vm: LogViewModel,
    pub input_vm: InputViewModel,
    pub user_vm: UserViewModel,
    pub msg_vm: ChatMessageViewModel,
    pub conf: Config,
}


impl AppViewModel {
    pub fn new(conf: Config) -> Self {
        Self {
            msg_vm: ChatMessageViewModel::new(),
            user_vm: UserViewModel::new(),
            log_vm: LogViewModel::new(),
            input_vm: InputViewModel::new(),
            conf,
        }
    }
}
