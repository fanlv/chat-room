use crate::config::Config;
use crate::view_model::input::Input;
use crate::view_model::log::LogList;
use crate::view_model::messages::ChatMessageList;
use crate::view_model::user_list::UserList;

#[derive(Clone, Debug)]
pub struct App {
    pub log_vm: LogList,
    pub input_vm: Input,
    pub user_vm: UserList,
    pub msg_vm: ChatMessageList,
    pub conf: Config,
}


impl App {
    pub fn new(conf: Config) -> Self {
        Self {
            msg_vm: ChatMessageList::new(),
            user_vm: UserList::new(),
            log_vm: LogList::new(),
            input_vm: Input::new(),
            conf,
        }
    }
}
