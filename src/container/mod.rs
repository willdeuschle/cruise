use std::collections::HashMap;
use std::fmt;
use std::sync::Mutex;
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

pub struct ContainerMap {
    map: Mutex<HashMap<ID, Container>>,
}

#[derive(Debug)]
pub struct ContainerMapError;

impl fmt::Debug for ContainerMap {
    fn fmt(self: &Self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ContainerMap")
    }
}

impl ContainerMap {
    pub fn new() -> ContainerMap {
        let mutex_guarded_map = Mutex::new(HashMap::<ID, Container>::new());
        ContainerMap {
            map: mutex_guarded_map,
        }
    }

    pub fn add(self: &Self, container: Container) -> Result<ID, ContainerMapError> {
        let mut map = self.map.lock().unwrap();
        if map.contains_key(container.id()) {
            return Err(ContainerMapError);
        }
        let container_id: String = container.id().clone();
        map.insert(container.id().clone(), container);
        Ok(container_id)
    }
}
