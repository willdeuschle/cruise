use crate::container::{Container, ID};
use std::fs::{
    copy, create_dir, create_dir_all, read_dir, read_to_string, remove_dir_all, rename, write,
};
use std::io::{Error, ErrorKind};
use std::path::Path;

#[derive(Debug)]
pub struct ContainerStore {
    root_dir: String,
}

fn copy_dir<P, Q>(from: P, to: Q) -> Result<(), Error>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    let from = from.as_ref();
    if !from.exists() {
        return Err(Error::new(
            ErrorKind::NotFound,
            format!("{:?} does not exist", from),
        ));
    }

    let to = to.as_ref();
    if !to.exists() {
        return Err(Error::new(
            ErrorKind::NotFound,
            format!("{:?} does not exist", from),
        ));
    }

    for from_entry in read_dir(from)? {
        let from_entry_path = from_entry?.path();
        let from_entry_last_component = match from_entry_path.components().last() {
            Some(last_component) => last_component,
            None => {
                return Err(Error::new(
                    ErrorKind::NotFound,
                    format!(
                        "last component not found in `from` path: {:?}",
                        from_entry_path
                    ),
                ))
            }
        };

        let mut to_entry_path = to.to_path_buf();
        to_entry_path.push(from_entry_last_component);

        if from_entry_path.is_dir() {
            create_dir(&to_entry_path)?;
            copy_dir(from_entry_path, to_entry_path)?;
        } else {
            match copy(from_entry_path, to_entry_path) {
                Ok(_) => (),
                Err(err) => {
                    return Err(Error::new(
                        ErrorKind::Other,
                        format!("failed to copy file: {}", err),
                    ));
                }
            }
        }
    }
    Ok(())
}

impl ContainerStore {
    pub fn new(root_dir: String) -> Result<ContainerStore, Error> {
        let _ = create_dir_all(root_dir.clone())?;
        Ok(ContainerStore { root_dir: root_dir })
    }

    // create_container creates the container directory on disk and returns the directory
    pub fn create_container(self: &Self, container_id: &ID) -> Result<(), Error> {
        let container_dir = self.specific_container_dir(container_id);
        if Path::new(&container_dir).exists() {
            return Err(Error::new(
                ErrorKind::AlreadyExists,
                "directory already exists",
            ));
        }
        let _ = create_dir_all(&container_dir)?;
        Ok(())
    }

    pub fn remove_container(self: &Self, container_id: &ID) {
        let container_dir = self.specific_container_dir(container_id);
        let _ = remove_dir_all(&container_dir);
    }

    // pass in a spec and a root fs path, put those in the bundle dir
    pub fn create_container_bundle(
        self: &Self,
        container_id: &ID,
        rootfs: &str,
    ) -> Result<String, Error> {
        // copy the rootfs of the container
        let rootfs_dir = self.rootfs_dir(container_id);
        let _ = create_dir_all(&rootfs_dir)?;
        match copy_dir(rootfs, rootfs_dir) {
            Ok(_) => (),
            Err(err) => {
                return Err(Error::new(
                    ErrorKind::Other,
                    format!("failed to copy rootfs: {}", err),
                ));
            }
        }
        Ok(self.bundle_dir(container_id))
    }

    pub fn atomic_persist_container_state(self: &Self, container: &Container) -> Result<(), Error> {
        let serialized_container = serde_json::to_string(&container)?;
        let temp_container_state_file = self.temp_container_state_file(container.id());
        write(&temp_container_state_file, serialized_container)?;
        rename(
            &temp_container_state_file,
            self.container_state_file(container.id()),
        )?;
        Ok(())
    }

    pub fn read_container_state(self: &Self, container_id: &ID) -> Result<Container, Error> {
        let container_state_file = self.container_state_file(container_id);
        let container_state_string = read_to_string(container_state_file)?;
        let container = serde_json::from_str(&container_state_string)?;
        Ok(container)
    }

    pub fn list_container_ids(self: &Self) -> Result<Vec<ID>, Error> {
        let mut container_ids = vec![];
        let container_dirs = match read_dir(self.container_dir()) {
            Ok(container_dirs) => container_dirs,
            Err(err) => {
                return Err(Error::new(
                    ErrorKind::Other,
                    format!(
                        "not able to read container_dir `{}`: {}",
                        self.container_dir(),
                        err
                    ),
                ))
            }
        };
        for container_dir in container_dirs {
            let container_dir = container_dir?.path();
            let container_id = match container_dir.components().last() {
                Some(container_id) => container_id,
                None => {
                    return Err(Error::new(
                        ErrorKind::NotFound,
                        format!("last component not found in path: {:?}", container_dir),
                    ))
                }
            };
            container_ids.push(container_id.as_os_str().to_string_lossy().to_string())
        }
        Ok(container_ids)
    }

    fn container_state_file(self: &Self, container_id: &ID) -> String {
        format!(
            "{}/container.state",
            self.specific_container_dir(container_id)
        )
    }

    fn temp_container_state_file(self: &Self, container_id: &ID) -> String {
        format!("{}.temp", self.container_state_file(container_id))
    }

    fn specific_container_dir(self: &Self, container_id: &ID) -> String {
        format!("{}/{}", self.container_dir(), container_id)
    }

    fn container_dir(self: &Self) -> String {
        format!("{}/containers", self.root_dir)
    }

    fn bundle_dir(self: &Self, container_id: &ID) -> String {
        format!("{}/bundle", self.specific_container_dir(container_id))
    }

    fn rootfs_dir(self: &Self, container_id: &ID) -> String {
        format!("{}/rootfs", self.bundle_dir(container_id))
    }
}
