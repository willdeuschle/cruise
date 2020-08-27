mod container_map;
mod container_runtime;
mod container_store;

use crate::container::{new as new_container, rand_id, Container, Status, ID};
use container_map::{ContainerMap, ContainerMapError};
use container_runtime::{
    ContainerRuntime, ContainerRuntimeError, RuntimeCreateOptions, RuntimeSpecOptions,
};
use container_store::{ContainerStore, ContainerStoreError};
use log::error;
use std::error::Error;
use std::fmt;
use std::time::SystemTime;

#[derive(Debug)]
pub struct ContainerManager {
    container_map: ContainerMap,
    container_store: ContainerStore,
    container_runtime: ContainerRuntime,
}

pub struct ContainerOptions {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub rootfs_path: String,
}

struct InternalCreateContainerError {
    container_id: ID,
    source: ContainerManagerError,
}

#[derive(Debug)]
pub enum ContainerManagerError {
    // represents an error creating the container store
    CreateContainerStoreError { source: ContainerStoreError },
    // represents an error reloading the container manager
    ReloadError { source: ContainerStoreError },
    // represents an error from the container store
    ContainerStoreError { source: ContainerStoreError },
    // represents an error when a container is not found
    ContainerNotFoundError { container_id: ID },
    // represents an error from the container map
    ContainerMapError { source: ContainerMapError },
    // represents an error from the container runtime
    ContainerRuntimeError { source: ContainerRuntimeError },
    // represents an error trying to create a container that's not in a created state
    StartContainerNotInCreatedStateError { container_id: ID },
    // represents an error trying to stop a container that's not in a running state
    StopContainerNotInRunningStateError { container_id: ID },
    // represents an error trying to delete a container that's not in a deleteable (created or
    // stopped) state
    DeleteContainerNotInDeleteableStateError { container_id: ID },
}

impl fmt::Display for ContainerManagerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::CreateContainerStoreError { .. } => write!(f, "failed to create container store"),
            Self::ReloadError { .. } => write!(f, "failed to reload container manager"),
            Self::ContainerStoreError { ref source } => source.fmt(f),
            Self::ContainerNotFoundError {
                ref container_id, ..
            } => write!(f, "container with container_id {} not found", container_id),
            Self::ContainerMapError { ref source } => source.fmt(f),
            Self::ContainerRuntimeError { ref source } => source.fmt(f),
            Self::StartContainerNotInCreatedStateError { ref container_id } => write!(
                f,
                "container with container_id {} is not in a created state",
                container_id
            ),
            Self::StopContainerNotInRunningStateError { ref container_id } => write!(
                f,
                "container with container_id {} is not in a running state",
                container_id
            ),
            Self::DeleteContainerNotInDeleteableStateError { ref container_id } => write!(
                f,
                "container with container_id {} is not in a deleteable (created or stopped) state",
                container_id
            ),
        }
    }
}

impl Error for ContainerManagerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match *self {
            Self::CreateContainerStoreError { ref source } => Some(source),
            Self::ReloadError { ref source } => Some(source),
            Self::ContainerStoreError { ref source } => source.source(),
            Self::ContainerNotFoundError { .. } => None,
            Self::ContainerMapError { ref source } => source.source(),
            Self::ContainerRuntimeError { ref source } => source.source(),
            Self::StartContainerNotInCreatedStateError { .. } => None,
            Self::StopContainerNotInRunningStateError { .. } => None,
            Self::DeleteContainerNotInDeleteableStateError { .. } => None,
        }
    }
}

impl From<ContainerMapError> for ContainerManagerError {
    fn from(err: ContainerMapError) -> ContainerManagerError {
        match err {
            ContainerMapError::ContainerNotFoundError { container_id } => {
                ContainerManagerError::ContainerNotFoundError { container_id }
            }
            _ => ContainerManagerError::ContainerMapError { source: err },
        }
    }
}

impl From<ContainerRuntimeError> for ContainerManagerError {
    fn from(err: ContainerRuntimeError) -> ContainerManagerError {
        match err {
            ContainerRuntimeError::ContainerNotFoundError { container_id } => {
                ContainerManagerError::ContainerNotFoundError { container_id }
            }
            _ => ContainerManagerError::ContainerRuntimeError { source: err },
        }
    }
}

impl From<ContainerStoreError> for ContainerManagerError {
    fn from(err: ContainerStoreError) -> ContainerManagerError {
        match err {
            _ => ContainerManagerError::ContainerStoreError { source: err },
        }
    }
}

// TODO: add locking per container_id to ensure operations between container_map and
// container_store are consistent
impl ContainerManager {
    pub fn new(
        root_dir: String,
        runtime_path: String,
    ) -> Result<ContainerManager, ContainerManagerError> {
        let container_store = ContainerStore::new(root_dir)?;
        let container_manager = ContainerManager {
            container_map: ContainerMap::new(),
            container_store,
            container_runtime: ContainerRuntime::new(runtime_path),
        };
        container_manager.reload()?;
        Ok(container_manager)
    }

    /// reload does the following:
    /// - reads all container state files off disk
    ///     - if any of these state files fail to be parsed, we assume the
    ///       container is corrupted and remove it
    /// - adds the container to the in-memory store
    /// - syncs the container state with the container runtime
    fn reload(&self) -> Result<(), ContainerManagerError> {
        // get container ids off disk
        let container_ids = self
            .container_store
            .list_container_ids()
            .map_err(|source| ContainerManagerError::ReloadError { source })?;
        for container_id in container_ids {
            // parse container state file
            let container = match self.container_store.read_container_state(&container_id) {
                Ok(container) => container,
                Err(err) => {
                    error!(
                        "unable to parse state of container `{}`, err: `{}`. Removing container.",
                        container_id, err
                    );
                    self.container_store
                        .remove_container_directory(&container_id);
                    continue;
                }
            };
            // add container to in-memory store
            match self.container_map.add(container) {
                Ok(_) => (),
                Err(err) => {
                    error!(
                        "unable to add container `{}` to in-memory state, err: `{:?}`. Continuing.",
                        container_id, err
                    );
                    continue;
                }
            }
            // sync container with container runtime
            match self.sync_container_status_with_runtime(&container_id) {
                Ok(_) => (),
                Err(err) => {
                    error!(
                        "unable to sync state of container `{}`, err: `{:?}`. Removing container.",
                        container_id, err
                    );
                    self.container_store
                        .remove_container_directory(&container_id);
                    self.container_map.remove(&container_id);
                    continue;
                }
            }
        }
        Ok(())
    }

    fn rollback_container_create(&self, container_id: &ID) {
        self.container_map.remove(&container_id);
        self.container_store
            .remove_container_directory(&container_id)
    }

    /// create_container does the following:
    /// - invoke create_container_helper to create the container
    /// - on an error, invoke rollback_container_create to clean up leftover
    ///   state, including in-memory container and container directory on disk
    pub fn create_container(
        &self,
        opts: ContainerOptions,
    ) -> Result<String, ContainerManagerError> {
        self.create_container_helper(opts).or_else(|err| {
            // best effort rollback
            self.rollback_container_create(&err.container_id);
            return Err(err.source);
        })
    }

    /// create_container_helper does the following:
    /// - generate container id
    /// - create and store the in-memory container structure
    /// - create the container directory on disk
    /// - create the container bundle:
    ///     - copy the rootfs into the container bundle
    ///     - generate the runc spec for the container
    /// - create the container (runc exec)
    /// - update container status, write those to disk
    fn create_container_helper(
        &self,
        opts: ContainerOptions,
    ) -> Result<String, InternalCreateContainerError> {
        // generate container id
        let container_id = rand_id();
        // create & store in-memory container structure
        let container: Container =
            new_container(&container_id, &opts.name, &opts.command, &opts.args);
        let container_id =
            self.container_map
                .add(container)
                .map_err(|err| InternalCreateContainerError {
                    container_id: container_id.clone(),
                    source: err.into(),
                })?;
        // create container directory on disk
        self.container_store
            .create_container_directory(&container_id)
            .map_err(|err| InternalCreateContainerError {
                container_id: container_id.clone(),
                source: err.into(),
            })?;
        // create container bundle on disk
        let container_bundle_dir = self
            .container_store
            .create_container_bundle(&container_id, &opts.rootfs_path)
            .map_err(|err| InternalCreateContainerError {
                container_id: container_id.clone(),
                source: err.into(),
            })?;
        // create container runtime spec on disk
        let spec_opts =
            RuntimeSpecOptions::new(container_bundle_dir.clone(), opts.command, opts.args);
        self.container_runtime
            .new_runtime_spec(&spec_opts)
            .map_err(|err| InternalCreateContainerError {
                container_id: container_id.clone(),
                source: err.into(),
            })?;
        // create container
        let create_opts = RuntimeCreateOptions::new(
            container_bundle_dir.clone(),
            "container.pidfile".into(),
            container_id.clone(),
        );
        self.container_runtime
            .create_container(create_opts)
            .map_err(|err| InternalCreateContainerError {
                container_id: container_id.clone(),
                source: err.into(),
            })?;
        // update container creation time, status, and persist to disk
        self.update_container_created_at(&container_id, SystemTime::now())
            .map_err(|source| InternalCreateContainerError {
                container_id: container_id.clone(),
                source,
            })?;
        self.update_container_status(&container_id, Status::Created)
            .map_err(|source| InternalCreateContainerError {
                container_id: container_id.clone(),
                source,
            })?;
        self.atomic_persist_container_state(&container_id)
            .map_err(|source| InternalCreateContainerError {
                container_id: container_id.clone(),
                source,
            })?;
        Ok(container_id)
    }

    /// start_container does the following:
    /// - ensure container exists and is in created state
    /// - start the container via the container runtime
    /// - update container start time and status, then persist
    pub fn start_container(&self, container_id: &ID) -> Result<(), ContainerManagerError> {
        // ensure container exists and is in created state
        match self.container_map.get(container_id) {
            Ok(container) => {
                if container.status != Status::Created {
                    return Err(
                        ContainerManagerError::StartContainerNotInCreatedStateError {
                            container_id: container_id.clone(),
                        },
                    );
                }
            }
            Err(err) => return Err(err.into()),
        }
        // container start
        self.container_runtime.start_container(container_id)?;
        // update container start time and status in memory, then persist to disk
        //     this current approach just optimistically sets the container to
        //     running and allows future calls to get/list to synchronize with runc.
        //     one other way we could consider doing this is polling runc until we
        //     see that the container is running and then updating.
        self.update_container_started_at(&container_id, SystemTime::now())?;
        self.update_container_status(&container_id, Status::Running)?;
        self.atomic_persist_container_state(&container_id)
    }

    /// stop_container does the following:
    /// - ensure container exists and is in running state
    /// - send a SIGKILL to the container via the container runtime
    /// - update container status, then persist
    pub fn stop_container(&self, container_id: &ID) -> Result<(), ContainerManagerError> {
        // ensure container exists and is in running state
        match self.container_map.get(container_id) {
            Ok(container) => {
                if container.status != Status::Running {
                    return Err(ContainerManagerError::StopContainerNotInRunningStateError {
                        container_id: container_id.clone(),
                    });
                }
            }
            Err(err) => return Err(err.into()),
        }
        // send SIGKILL to container via the container runtime
        self.container_runtime.kill_container(container_id)?;
        // update container status and persist to disk
        self.update_container_status(&container_id, Status::Stopped)?;
        self.atomic_persist_container_state(&container_id)
    }

    /// delete_container does the following:
    /// - ensure container exists and is in stopped state
    /// - tell the container runtime to delete the container
    /// - remove remnants of container in memory and on disk
    pub fn delete_container(&self, container_id: &ID) -> Result<(), ContainerManagerError> {
        // ensure container exists and is in stopped state
        match self.container_map.get(container_id) {
            Ok(container) => {
                if container.status != Status::Stopped && container.status != Status::Created {
                    return Err(
                        ContainerManagerError::DeleteContainerNotInDeleteableStateError {
                            container_id: container_id.clone(),
                        },
                    );
                }
            }
            Err(err) => return Err(err.into()),
        }
        // instruct container runtime to delete container
        self.container_runtime.delete_container(container_id)?;
        // remove container from memory and disk
        self.container_map.remove(&container_id);
        self.container_store
            .remove_container_directory(&container_id);
        Ok(())
    }

    /// get_container does the following:
    /// - synchronize container state with the container runtime, which fails
    ///   if the container does not exist
    /// - return container state from memory
    pub fn get_container(
        &self,
        container_id: &ID,
    ) -> Result<Box<Container>, ContainerManagerError> {
        self.sync_container_status_with_runtime(container_id)?;
        self.container_map
            .get(container_id)
            .map_err(|err| err.into())
    }

    /// list_containers does the following:
    /// - for every known container, synchronize container state with the
    ///   container runtime, which fails if any of the containers do not exist
    /// - return container states from memory
    pub fn list_containers(&self) -> Result<Vec<Container>, ContainerManagerError> {
        match self.container_map.list() {
            Ok(containers) => {
                for container in containers.iter() {
                    self.sync_container_status_with_runtime(container.id())?;
                }
            }
            Err(err) => return Err(err.into()),
        };
        self.container_map.list().map_err(|err| err.into())
    }

    /// sync_container_status_with_runtime does the following:
    /// - get container state from the container runtime
    /// - persist in memory and to disk
    fn sync_container_status_with_runtime(
        &self,
        container_id: &ID,
    ) -> Result<(), ContainerManagerError> {
        let status = self.container_runtime.get_container_status(container_id)?;
        // update container status in memory and persist to disk
        self.update_container_status(&container_id, status)?;
        self.atomic_persist_container_state(&container_id)
    }

    /// update_container_status updates container status in memory
    fn update_container_status(
        &self,
        container_id: &ID,
        status: Status,
    ) -> Result<(), ContainerManagerError> {
        self.container_map
            .update_status(container_id, status)
            .map_err(|err| err.into())
    }

    /// update_container_created_at updates container creation time in memory
    fn update_container_created_at(
        &self,
        container_id: &ID,
        created_at: SystemTime,
    ) -> Result<(), ContainerManagerError> {
        self.container_map
            .update_creation_time(container_id, created_at)
            .map_err(|err| err.into())
    }

    /// update_container_started_at updates container start time in memory
    fn update_container_started_at(
        &self,
        container_id: &ID,
        started_at: SystemTime,
    ) -> Result<(), ContainerManagerError> {
        self.container_map
            .update_start_time(container_id, started_at)
            .map_err(|err| err.into())
    }

    /// atomic_persist_container_state persists container state to disk
    fn atomic_persist_container_state(
        &self,
        container_id: &ID,
    ) -> Result<(), ContainerManagerError> {
        let container = self
            .container_map
            .get(&container_id)
            .map_err(|source| ContainerManagerError::ContainerMapError { source })?;
        self.container_store
            .atomic_persist_container_state(&container)
            .map_err(|err| err.into())
    }
}
