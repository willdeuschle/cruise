use crate::container::{new as new_container, rand_id, Container, ContainerMap, Status};
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
pub struct ContainerCreateError {
    pub reason: String,
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

    pub fn create_container(
        self: &Self,
        opts: ContainerOptions,
    ) -> Result<String, ContainerCreateError> {
        // container manager creates TODOs:
        // - generate container id [DONE]
        // - create the in-memory container structure [DONE]
        // - store the in-memory container structure [DONE]
        // - create the container directory [DONE]
        // - create the runc spec for the container [DONE]
        // - create the container bundle: copy the rootfs in as well as the spec into the bundle
        // dir of the container [DONE]
        // - create the container (runc/or shim exec). on success, update the container status to
        // created, update the created timestamp, and write the updates to disk
        // on any failure in any of these steps - rollback. this means removing the container
        // directory from disk and removing the in-memory container map

        // generate container id
        let container_id = rand_id();

        // create & store in-memory container structure
        let container: Container = new_container(container_id, opts.name);
        let container_id = match self.container_map.add(container) {
            Ok(container_id) => container_id,
            Err(err) => {
                return Err(ContainerCreateError {
                    reason: format!("{:?}", err),
                })
            }
        };

        // create container directory on disk
        match self.container_store.create_container(&container_id) {
            Ok(_) => (),
            Err(err) => {
                return Err(ContainerCreateError {
                    reason: format!("{}", err),
                });
            }
        };
        // create container bundle on disk
        let container_bundle_dir = match self
            .container_store
            .create_container_bundle(&container_id, &opts.rootfs_path)
        {
            Ok(container_bundle_dir) => container_bundle_dir,
            Err(err) => {
                return Err(ContainerCreateError {
                    reason: format!("{}", err),
                })
            }
        };
        // create container runtime spec on disk
        let spec_opts =
            RuntimeSpecOptions::new(container_bundle_dir.clone(), opts.command, opts.args);
        match self.container_runtime.new_runtime_spec(&spec_opts) {
            Ok(()) => (),
            Err(err) => {
                return Err(ContainerCreateError {
                    reason: format!("{:?}", err),
                })
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
                return Err(ContainerCreateError {
                    reason: format!("{:?}", err),
                })
            }
        }
        // update container status and creation time
        match self
            .container_map
            .update(&container_id, Status::Created, SystemTime::now())
        {
            Ok(_) => (),
            Err(err) => {
                return Err(ContainerCreateError {
                    reason: format!("{:?}", err),
                })
            }
        }
        // TODO: persist container state to disk
        let container = match self.container_map.get(&container_id) {
            Ok(container) => container,
            Err(err) => {
                return Err(ContainerCreateError {
                    reason: format!("{:?}", err),
                })
            }
        };
        match self.container_store.persist_container_state(&container) {
            Ok(_) => (),
            Err(err) => {
                return Err(ContainerCreateError {
                    reason: format!("{:?}", err),
                })
            }
        }
        Ok(container_id)
    }
}
