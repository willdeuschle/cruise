use uuid::Uuid;

pub struct Container {
    pub id: ID,
    pub name: String,
    pub status: Status,
    pub exit_code: i32,
}

impl Container {
    pub fn id(self: &Self) -> &ID {
        &self.id
    }
}

pub fn new(id: ID, name: String) -> Container {
    Container {
        id,
        name,
        status: Status::Initialized,
        exit_code: -1,
    }
}

pub enum Status {
    Initialized,
    Created,
    Running,
    Stopped,
    Unknown,
}

pub type ID = String;

pub fn rand_id() -> ID {
    Uuid::new_v4().to_string()
}
