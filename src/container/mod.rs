use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::SystemTime;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone)]
pub struct Container {
    pub id: ID,
    pub name: String,
    pub status: Status,
    // TODO: will require the shim integration to properly update this
    pub exit_code: i32,

    pub created_at: Option<SystemTime>,
    pub started_at: Option<SystemTime>,
    // TODO: will require the shim integration to properly update this
    pub finished_at: Option<SystemTime>,
    pub command: String,
    pub args: Vec<String>,
}

impl Container {
    pub fn id(&self) -> &ID {
        &self.id
    }
}

pub fn new(id: &ID, name: &String, command: &String, args: &Vec<String>) -> Container {
    Container {
        id: id.clone(),
        name: name.clone(),
        status: Status::Initialized,
        exit_code: -1,
        created_at: None,
        started_at: None,
        finished_at: None,
        command: command.clone(),
        args: args.clone(),
    }
}

#[derive(Serialize, Deserialize)]
pub struct RuncStatus {
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub type ID = String;

pub fn rand_id() -> ID {
    Uuid::new_v4().to_string()
}
