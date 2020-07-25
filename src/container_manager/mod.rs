use crate::container::{new as new_container, rand_id, Container};

#[derive(Debug)]
pub struct ContainerManager {}

pub fn new() -> ContainerManager {
    ContainerManager {}
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
    pub fn create_container(
        self: &Self,
        opts: ContainerOptions,
    ) -> Result<Container, ContainerCreateError> {
        // TODO: handle container creation, return container ID
        let c = new_container(rand_id(), opts.name);
        Ok(c)
    }
}
