use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::sync::Mutex;
use std::time::SystemTime;

use crate::container::{Container, Status, ID};

#[derive(Debug)]
pub enum ContainerMapError {
    ContainerAlreadyExistsError { container_id: ID },
    ContainerNotFoundError { container_id: ID },
}

impl fmt::Display for ContainerMapError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::ContainerAlreadyExistsError { ref container_id } => write!(
                f,
                "container with container_id {} already exists",
                container_id
            ),
            Self::ContainerNotFoundError { ref container_id } => {
                write!(f, "container with container_id {} not found", container_id)
            }
        }
    }
}

impl Error for ContainerMapError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match *self {
            Self::ContainerAlreadyExistsError { .. } => None,
            Self::ContainerNotFoundError { .. } => None,
        }
    }
}

pub struct ContainerMap {
    map: Mutex<HashMap<ID, Container>>,
}

impl fmt::Debug for ContainerMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

    /// add adds a container to the map
    pub fn add(&self, container: Container) -> Result<ID, ContainerMapError> {
        let mut map = self.map.lock().unwrap();
        if map.contains_key(container.id()) {
            return Err(ContainerMapError::ContainerAlreadyExistsError {
                container_id: container.id().clone(),
            });
        }
        let container_id: String = container.id().clone();
        map.insert(container.id().clone(), container);
        Ok(container_id)
    }

    /// get gets a container from the map
    pub fn get(&self, container_id: &ID) -> Result<Box<Container>, ContainerMapError> {
        let map = self.map.lock().unwrap();
        if !map.contains_key(container_id) {
            return Err(ContainerMapError::ContainerNotFoundError {
                container_id: container_id.clone(),
            });
        }
        let container_clone = map.get(container_id).unwrap().clone();
        Ok(Box::new(container_clone))
    }

    /// list lists containers in the map
    pub fn list(&self) -> Result<Vec<Container>, ContainerMapError> {
        let map = self.map.lock().unwrap();
        Ok(map.values().map(|c| c.clone()).collect())
    }

    /// remove removes a container from the map
    pub fn remove(&self, container_id: &ID) {
        let mut map = self.map.lock().unwrap();
        if !map.contains_key(container_id) {
            return;
        }
        map.remove(container_id);
    }

    /// update_status updates the status of a container
    pub fn update_status(
        &self,
        container_id: &ID,
        status: Status,
    ) -> Result<(), ContainerMapError> {
        let mut map = self.map.lock().unwrap();
        if !map.contains_key(container_id) {
            return Err(ContainerMapError::ContainerNotFoundError {
                container_id: container_id.clone(),
            });
        }
        let container = map.get_mut(container_id).unwrap();
        container.status = status;
        Ok(())
    }

    /// update_creation_time updates the creation time of a container
    pub fn update_creation_time(
        &self,
        container_id: &ID,
        created_at: SystemTime,
    ) -> Result<(), ContainerMapError> {
        let mut map = self.map.lock().unwrap();
        if !map.contains_key(container_id) {
            return Err(ContainerMapError::ContainerNotFoundError {
                container_id: container_id.clone(),
            });
        }
        let container = map.get_mut(container_id).unwrap();
        container.created_at = Some(created_at);
        Ok(())
    }

    // update_start_time updates the start time of a container
    pub fn update_start_time(
        &self,
        container_id: &ID,
        started_at: SystemTime,
    ) -> Result<(), ContainerMapError> {
        let mut map = self.map.lock().unwrap();
        if !map.contains_key(container_id) {
            return Err(ContainerMapError::ContainerNotFoundError {
                container_id: container_id.clone(),
            });
        }
        let container = map.get_mut(container_id).unwrap();
        container.started_at = Some(started_at);
        Ok(())
    }
}
