use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::sync::Mutex;
use std::time::SystemTime;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone)]
pub struct Container {
    pub id: ID,
    pub name: String,
    pub status: Status,
    pub exit_code: i32,

    pub created_at: Option<SystemTime>,
}

impl Container {
    pub fn id(self: &Self) -> &ID {
        &self.id
    }
}

pub fn new(id: &ID, name: String) -> Container {
    Container {
        id: id.clone(),
        name,
        status: Status::Initialized,
        exit_code: -1,
        created_at: None,
    }
}

#[derive(Serialize, Deserialize)]
pub struct RuncStatus {
    pub status: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub enum Status {
    Initialized,
    Created,
    Running,
    Paused,
    Stopped,
    Unknown,
}

impl Status {
    pub fn from_runc_status(runc_status: &RuncStatus) -> Status {
        match runc_status.status.as_str() {
            "created" => Status::Created,
            "running" => Status::Running,
            "pausing" => Status::Running,
            "paused" => Status::Paused,
            "stopped" => Status::Stopped,
            _ => Status::Unknown,
        }
    }
}

pub type ID = String;

pub fn rand_id() -> ID {
    Uuid::new_v4().to_string()
}

pub struct ContainerMap {
    map: Mutex<HashMap<ID, Container>>,
}

#[derive(Debug)]
pub struct ContainerMapError {
    reason: String,
}

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
            return Err(ContainerMapError {
                reason: "container already exists".into(),
            });
        }
        let container_id: String = container.id().clone();
        map.insert(container.id().clone(), container);
        Ok(container_id)
    }

    pub fn update_status(
        self: &Self,
        container_id: &ID,
        status: Status,
    ) -> Result<(), ContainerMapError> {
        let mut map = self.map.lock().unwrap();
        if !map.contains_key(container_id) {
            return Err(ContainerMapError {
                reason: format!("container with ID `{}` does not exist", container_id),
            });
        }
        let container = map.get_mut(container_id).unwrap();
        container.status = status;
        Ok(())
    }

    pub fn update_creation_time(
        self: &Self,
        container_id: &ID,
        created_at: SystemTime,
    ) -> Result<(), ContainerMapError> {
        let mut map = self.map.lock().unwrap();
        if !map.contains_key(container_id) {
            return Err(ContainerMapError {
                reason: format!("container with ID `{}` does not exist", container_id),
            });
        }
        let container = map.get_mut(container_id).unwrap();
        container.created_at = Some(created_at);
        Ok(())
    }

    pub fn get(self: &Self, container_id: &ID) -> Result<Box<Container>, ContainerMapError> {
        let map = self.map.lock().unwrap();
        if !map.contains_key(container_id) {
            return Err(ContainerMapError {
                reason: format!("container with ID `{}` does not exist", container_id),
            });
        }
        let container_clone = map.get(container_id).unwrap().clone();
        Ok(Box::new(container_clone))
    }

    pub fn remove(self: &Self, container_id: &ID) {
        let mut map = self.map.lock().unwrap();
        if !map.contains_key(container_id) {
            return;
        }
        map.remove(container_id);
    }
}
