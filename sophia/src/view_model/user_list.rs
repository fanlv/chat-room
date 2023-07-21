use sophia_core::model::User;

#[derive(Clone, Debug)]
pub struct UserViewModel {
    pub users: Vec<User>,
}


impl UserViewModel {
    pub fn new() -> Self {
        Self { users: Vec::new() }
    }
}