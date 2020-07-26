use crate::container::{new as new_container, rand_id, Container, ContainerMap};

#[derive(Debug)]
pub struct ContainerManager {
    container_map: ContainerMap,
}

pub struct ContainerOptions {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub rootfs_path: String,
}

#[derive(Debug)]
pub struct ContainerCreateError;

impl ContainerManager {
    pub fn new() -> ContainerManager {
        ContainerManager {
            container_map: ContainerMap::new(),
        }
    }

    pub fn create_container(
        self: &Self,
        opts: ContainerOptions,
    ) -> Result<String, ContainerCreateError> {
        // container manager creates TODOs:
        // - generate container id [DONE]
        // - create the in-memory container structure [DONE]
        // - store the in-memory container structure [DONE]
        // - create the container directory
        // - create the runc spec for the container
        // - create the container bundle: copy the rootfs in as well as the spec into the bundle
        // dir of the container
        // - create the container (runc/or shim exec). on success, update the container status to
        // created, update the created timestamp, and write the updates to disk
        // on any failure in any of these steps - rollback. this means removing the container
        // directory from disk and removing the in-memory container map
        let container_id = rand_id();
        let container: Container = new_container(container_id, opts.name);
        match self.container_map.add(container) {
            Ok(container_id) => Ok(container_id),
            Err(_) => Err(ContainerCreateError),
        }
    }
}
