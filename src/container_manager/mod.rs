use crate::container::{new as new_container, rand_id, Container, ContainerMap, Status, ID};
use crate::container_runtime::{ContainerRuntime, RuntimeCreateOptions, RuntimeSpecOptions};
use crate::container_store::ContainerStore;
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

#[derive(Debug)]
pub struct ContainerManagerError {
    container_id: ID,
    pub reason: String,
}

impl ContainerManagerError {
    fn new(container_id: &ID, reason: String) -> ContainerManagerError {
        ContainerManagerError {
            container_id: container_id.clone(),
            reason,
        }
    }
}

impl ContainerManager {
    pub fn new(root_dir: String, runtime_path: String) -> Result<ContainerManager, std::io::Error> {
        let container_store = ContainerStore::new(root_dir)?;
        Ok(ContainerManager {
            container_map: ContainerMap::new(),
            container_store,
            container_runtime: ContainerRuntime::new(runtime_path),
        })
    }

    fn rollback_container_create(self: &Self, container_id: &ID) {
        self.container_map.remove(&container_id);
        self.container_store.remove_container(&container_id)
    }

    /// create_container does the following:
    /// - invoke create_container_helper to create the container
    /// - on an error, invoke rollback_container_create to clean up leftover
    ///   state, including in-memory container and container directory on disk
    pub fn create_container(
        self: &Self,
        opts: ContainerOptions,
    ) -> Result<String, ContainerManagerError> {
        match self.create_container_helper(opts) {
            Ok(container_id) => Ok(container_id),
            Err(err) => {
                // best effort rollback
                self.rollback_container_create(&err.container_id);
                Err(err)
            }
        }
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
    pub fn create_container_helper(
        self: &Self,
        opts: ContainerOptions,
    ) -> Result<String, ContainerManagerError> {
        // generate container id
        let container_id = rand_id();
        // create & store in-memory container structure
        let container: Container = new_container(&container_id, opts.name);
        let container_id = match self.container_map.add(container) {
            Ok(container_id) => container_id,
            Err(err) => {
                return Err(ContainerManagerError::new(
                    &container_id,
                    format!("{:?}", err),
                ))
            }
        };
        // create container directory on disk
        match self.container_store.create_container(&container_id) {
            Ok(_) => (),
            Err(err) => {
                return Err(ContainerManagerError::new(
                    &container_id,
                    format!("{:?}", err),
                ))
            }
        };
        // create container bundle on disk
        let container_bundle_dir = match self
            .container_store
            .create_container_bundle(&container_id, &opts.rootfs_path)
        {
            Ok(container_bundle_dir) => container_bundle_dir,
            Err(err) => {
                return Err(ContainerManagerError::new(
                    &container_id,
                    format!("{:?}", err),
                ))
            }
        };
        // create container runtime spec on disk
        let spec_opts =
            RuntimeSpecOptions::new(container_bundle_dir.clone(), opts.command, opts.args);
        match self.container_runtime.new_runtime_spec(&spec_opts) {
            Ok(()) => (),
            Err(err) => {
                return Err(ContainerManagerError::new(
                    &container_id,
                    format!("{:?}", err),
                ))
            }
        }
        // create container
        let create_opts = RuntimeCreateOptions::new(
            container_bundle_dir.clone(),
            "container.pidfile".into(),
            container_id.clone(),
        );
        match self.container_runtime.create_container(create_opts) {
            Ok(()) => (),
            Err(err) => {
                return Err(ContainerManagerError::new(
                    &container_id,
                    format!("{:?}", err),
                ))
            }
        }
        // update container status and persist to disk
        match self.update_and_persist_status(&container_id, Status::Created, SystemTime::now()) {
            Ok(_) => Ok(container_id),
            Err(err) => Err(err),
        }
    }

    pub fn start_container(self: &Self, container_id: &ID) -> Result<(), ContainerManagerError> {
        // ensure container exists and is in created state
        match self.container_map.get(container_id) {
            Ok(container) => {
                if container.status != Status::Created {
                    return Err(ContainerManagerError::new(
                        &container_id,
                        format!("container does not have `Created` status"),
                    ));
                }
            }
            Err(err) => {
                return Err(ContainerManagerError::new(
                    container_id,
                    format!("{:?}", err),
                ))
            }
        }
        // container start
        match self.container_runtime.start_container(container_id) {
            Ok(_) => (),
            Err(err) => {
                return Err(ContainerManagerError::new(
                    &container_id,
                    format!("{:?}", err),
                ))
            }
        }
        // update container status and persist to disk
        self.update_and_persist_status(&container_id, Status::Running, SystemTime::UNIX_EPOCH)
    }

    /// update_and_persist_status updates container status and persist to disk
    fn update_and_persist_status(
        self: &Self,
        container_id: &ID,
        status: Status,
        created_at: SystemTime,
    ) -> Result<(), ContainerManagerError> {
        // update container status
        match self.container_map.update(container_id, status, created_at) {
            Ok(_) => (),
            Err(err) => {
                return Err(ContainerManagerError::new(
                    &container_id,
                    format!("{:?}", err),
                ))
            }
        }
        // persist container state to disk
        let container = match self.container_map.get(&container_id) {
            Ok(container) => container,
            Err(err) => {
                return Err(ContainerManagerError::new(
                    &container_id,
                    format!("{:?}", err),
                ))
            }
        };
        match self.container_store.persist_container_state(&container) {
            Ok(_) => Ok(()),
            Err(err) => {
                return Err(ContainerManagerError::new(
                    &container_id,
                    format!("{:?}", err),
                ))
            }
        }
    }
}
