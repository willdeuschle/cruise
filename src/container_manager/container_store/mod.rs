use crate::container::{Container, ID};
use std::error::Error;
use std::fmt;
use std::fs::{
    copy, create_dir, create_dir_all, read_dir, read_to_string, remove_dir_all, rename, write,
};
use std::path::Path;

#[derive(Debug)]
pub struct ContainerStore {
    root_dir: String,
}

// this could stand to be in its own module, we're cheating a little here by
// wrapping extra information into std::io::Errors for the sake of not
// moving this into its own module
fn copy_dir<P, Q>(from: P, to: Q) -> Result<(), std::io::Error>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    let from = from.as_ref();
    if !from.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("{:?} does not exist", from),
        ));
    }

    let to = to.as_ref();
    if !to.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("{:?} does not exist", from),
        ));
    }

    for from_entry in read_dir(from)? {
        let from_entry_path = from_entry?.path();
        let from_entry_last_component = match from_entry_path.components().last() {
            Some(last_component) => last_component,
            None => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
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
                    return Err(std::io::Error::new(
                        err.kind(),
                        format!("failed to copy file: {}", err),
                    ));
                }
            }
        }
    }
    Ok(())
}

#[derive(Debug)]
pub enum ContainerStoreError {
    // represents an error creating the main containers dir
    CreateContainersDirError {
        source: std::io::Error,
    },
    // represents an error reading the main containers dir
    ReadContainersDirError {
        source: std::io::Error,
    },
    // represents an error creating a specific container directory
    CreateSpecificContainerDirError {
        container_id: ID,
        source: std::io::Error,
    },
    // represents an error creating the rootfs directory of a container
    CreateRootfsDirError {
        container_id: ID,
        source: std::io::Error,
    },
    // represents an error copying a containers rootfs directory
    CopyRootfsDirError {
        container_id: ID,
        source: std::io::Error,
    },
    // represents an error when a container directory already exists that
    // we're trying to create
    ContainerDirAlreadyExistsError {
        container_id: ID,
    },
    // represents an error serializing the state of a container
    SerializeContainerStateError {
        container_id: ID,
        source: serde_json::Error,
    },
    // represents an error persisting the state of a container
    PersistContainerStateError {
        container_id: ID,
        source: std::io::Error,
    },
    // represents an error renaming a container state file
    RenameContainerStateFileError {
        container_id: ID,
        source: std::io::Error,
    },
    // represents an error reading a container state file
    ReadContainerStateFileError {
        container_id: ID,
        source: std::io::Error,
    },
    // represents an error parsing container state
    ParseContainerStateError {
        container_id: ID,
        source: serde_json::Error,
    },
    // represents an error where the container id is not in the filesystem path
    ContainerIDNotInPathError {
        container_dir: String,
    },
    // represents a catchall stdio error
    IOError(std::io::Error),
}

impl From<std::io::Error> for ContainerStoreError {
    fn from(err: std::io::Error) -> ContainerStoreError {
        ContainerStoreError::IOError(err)
    }
}

impl fmt::Display for ContainerStoreError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::CreateContainersDirError { .. } => write!(f, "failed to create containers dir"),
            Self::ReadContainersDirError { .. } => write!(f, "failed to read containers dir"),
            Self::CreateSpecificContainerDirError {
                ref container_id, ..
            } => write!(
                f,
                "failed to create container dir for container_id {}",
                container_id
            ),
            Self::CreateRootfsDirError {
                ref container_id, ..
            } => write!(
                f,
                "failed to create rootfs dir for container_id {}",
                container_id
            ),
            Self::CopyRootfsDirError {
                ref container_id, ..
            } => write!(
                f,
                "failed to copy rootfs dir for container_id {}",
                container_id
            ),
            Self::ContainerDirAlreadyExistsError { ref container_id } => write!(
                f,
                "container directory already exists for container_id {}",
                container_id
            ),
            Self::SerializeContainerStateError {
                ref container_id, ..
            } => write!(
                f,
                "failed to serialize container for container_id {}",
                container_id
            ),
            Self::PersistContainerStateError {
                ref container_id, ..
            } => write!(
                f,
                "failed to perisst container state for container_id {}",
                container_id
            ),
            Self::RenameContainerStateFileError {
                ref container_id, ..
            } => write!(
                f,
                "failed to rename container state for container_id {}",
                container_id
            ),
            Self::ReadContainerStateFileError {
                ref container_id, ..
            } => write!(
                f,
                "failed to read container state file for container_id {}",
                container_id
            ),
            Self::ParseContainerStateError {
                ref container_id, ..
            } => write!(
                f,
                "failed to read parse container state for container_id {}",
                container_id
            ),
            Self::ContainerIDNotInPathError { ref container_dir } => write!(
                f,
                "container id was not at the end of the container_dir path: {}",
                container_dir
            ),
            Self::IOError(ref err) => err.fmt(f),
        }
    }
}

impl Error for ContainerStoreError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match *self {
            Self::CreateContainersDirError { ref source } => Some(source),
            Self::ReadContainersDirError { ref source } => Some(source),
            Self::CreateSpecificContainerDirError { ref source, .. } => Some(source),
            Self::CreateRootfsDirError { ref source, .. } => Some(source),
            Self::CopyRootfsDirError { ref source, .. } => Some(source),
            Self::ContainerDirAlreadyExistsError { .. } => None,
            Self::SerializeContainerStateError { ref source, .. } => Some(source),
            Self::PersistContainerStateError { ref source, .. } => Some(source),
            Self::RenameContainerStateFileError { ref source, .. } => Some(source),
            Self::ReadContainerStateFileError { ref source, .. } => Some(source),
            Self::ParseContainerStateError { ref source, .. } => Some(source),
            Self::ContainerIDNotInPathError { .. } => None,
            Self::IOError(_) => None,
        }
    }
}

impl ContainerStore {
    pub fn new(root_dir: String) -> Result<ContainerStore, ContainerStoreError> {
        let cs = ContainerStore { root_dir: root_dir };
        let _ = create_dir_all(cs.containers_dir())
            .map_err(|source| ContainerStoreError::CreateContainersDirError { source })?;
        Ok(cs)
    }

    // create_container creates the container directory on disk and returns the directory
    pub fn create_container(self: &Self, container_id: &ID) -> Result<(), ContainerStoreError> {
        let container_dir = self.specific_container_dir(container_id);
        if Path::new(&container_dir).exists() {
            return Err(ContainerStoreError::ContainerDirAlreadyExistsError {
                container_id: container_id.clone(),
            });
        }
        create_dir_all(&container_dir).map_err(|source| {
            ContainerStoreError::CreateSpecificContainerDirError {
                container_id: container_id.clone(),
                source,
            }
        })
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
    ) -> Result<String, ContainerStoreError> {
        // copy the rootfs of the container
        let rootfs_dir = self.rootfs_dir(container_id);
        let _ = create_dir_all(&rootfs_dir).map_err(|source| {
            ContainerStoreError::CreateRootfsDirError {
                container_id: container_id.clone(),
                source,
            }
        })?;
        copy_dir(rootfs, rootfs_dir).map_err(|source| ContainerStoreError::CopyRootfsDirError {
            container_id: container_id.clone(),
            source,
        })?;
        Ok(self.bundle_dir(container_id))
    }

    pub fn atomic_persist_container_state(
        self: &Self,
        container: &Container,
    ) -> Result<(), ContainerStoreError> {
        let serialized_container = serde_json::to_string(&container).map_err(|source| {
            ContainerStoreError::SerializeContainerStateError {
                container_id: container.id().clone(),
                source,
            }
        })?;
        let temp_container_state_file = self.temp_container_state_file(container.id());
        write(&temp_container_state_file, serialized_container).map_err(|source| {
            ContainerStoreError::PersistContainerStateError {
                container_id: container.id().clone(),
                source,
            }
        })?;
        rename(
            &temp_container_state_file,
            self.container_state_file(container.id()),
        )
        .map_err(
            |source| ContainerStoreError::RenameContainerStateFileError {
                container_id: container.id().clone(),
                source,
            },
        )?;
        Ok(())
    }

    pub fn read_container_state(
        self: &Self,
        container_id: &ID,
    ) -> Result<Container, ContainerStoreError> {
        let container_state_file = self.container_state_file(container_id);
        let container_state_string = read_to_string(container_state_file).map_err(|source| {
            ContainerStoreError::ReadContainerStateFileError {
                container_id: container_id.clone(),
                source,
            }
        })?;
        serde_json::from_str(&container_state_string).map_err(|source| {
            ContainerStoreError::ParseContainerStateError {
                container_id: container_id.clone(),
                source,
            }
        })
    }

    pub fn list_container_ids(self: &Self) -> Result<Vec<ID>, ContainerStoreError> {
        let mut container_ids = vec![];
        let container_dirs = read_dir(self.containers_dir())
            .map_err(|source| ContainerStoreError::ReadContainersDirError { source })?;
        for container_dir in container_dirs {
            let container_dir = container_dir?.path();
            let container_id = container_dir.components().last().ok_or(
                ContainerStoreError::ContainerIDNotInPathError {
                    container_dir: container_dir.to_string_lossy().to_string(),
                },
            )?;
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
        format!("{}/{}", self.containers_dir(), container_id)
    }

    fn containers_dir(self: &Self) -> String {
        format!("{}/containers", self.root_dir)
    }

    fn bundle_dir(self: &Self, container_id: &ID) -> String {
        format!("{}/bundle", self.specific_container_dir(container_id))
    }

    fn rootfs_dir(self: &Self, container_id: &ID) -> String {
        format!("{}/rootfs", self.bundle_dir(container_id))
    }
}
