use std::fs::{copy, File};
use std::fs::{create_dir, create_dir_all, read_dir};
use std::io::prelude::*;
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
    pub fn create_container(self: &Self, container_id: &str) -> Result<String, Error> {
        let container_dir = self.container_dir(container_id);
        if Path::new(&container_dir).exists() {
            return Err(Error::new(
                ErrorKind::AlreadyExists,
                "directory already exists",
            ));
        }
        let _ = create_dir_all(&container_dir)?;
        Ok(container_dir)
    }

    // pass in a spec and a root fs path, put those in the bundle dir
    pub fn create_container_bundle(
        self: &Self,
        container_id: &str,
        rootfs: &str,
        spec: &[u8],
    ) -> Result<(), Error> {
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
        // write the spec to disk
        let spec_file_path = self.runtime_spec_file(container_id);
        let mut spec_file = File::create(spec_file_path)?;
        spec_file.write_all(spec)?;
        Ok(())
    }

    fn container_dir(self: &Self, container_id: &str) -> String {
        format!("{}/containers/{}", self.root_dir, container_id)
    }

    fn bundle_dir(self: &Self, container_id: &str) -> String {
        format!("{}/bundle", self.container_dir(container_id))
    }

    fn rootfs_dir(self: &Self, container_id: &str) -> String {
        format!("{}/rootfs", self.bundle_dir(container_id))
    }

    fn runtime_spec_file(self: &Self, container_id: &str) -> String {
        format!("{}/config.json", self.bundle_dir(container_id))
    }
}
