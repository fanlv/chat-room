use sophia_core::model::User;

#[derive(Clone, Debug)]
pub struct UserList {
    pub users: Vec<User>,
}


impl UserList {
    pub fn new() -> Self {
        Self { users: Vec::new() }
    }
}