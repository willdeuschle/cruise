use std::fs::create_dir_all;
use std::io::Error;

#[derive(Debug)]
pub struct ContainerStore {
    root_dir: String,
}

impl ContainerStore {
    pub fn new(root_dir: String) -> Result<ContainerStore, Error> {
        let _ = create_dir_all(root_dir.clone())?;
        Ok(ContainerStore { root_dir: root_dir })
    }

    // pass in a spec and a root fs path, put those in the bundle dir
    //pub fn create_container_bundle() {
    //}
}
