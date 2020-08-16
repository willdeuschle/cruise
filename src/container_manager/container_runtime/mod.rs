use crate::container::{RuncStatus, Status, ID};
use log::debug;
use std::error::Error;
use std::fmt;
use std::process::Command;
use std::string::FromUtf8Error;

#[derive(Debug)]
pub struct ContainerRuntime {
    runtime_path: String,
}

pub struct RuntimeSpecOptions {
    pub bundle_path: String,
    pub command: String,
    pub args: Vec<String>,
}

impl RuntimeSpecOptions {
    pub fn new(bundle_path: String, command: String, args: Vec<String>) -> RuntimeSpecOptions {
        RuntimeSpecOptions {
            bundle_path,
            command,
            args,
        }
    }
}

pub struct RuntimeCreateOptions {
    pub bundle_path: String,
    pub container_pidfile: String,
    pub container_id: String,
}

impl RuntimeCreateOptions {
    pub fn new(
        bundle_path: String,
        container_pidfile: String,
        container_id: String,
    ) -> RuntimeCreateOptions {
        RuntimeCreateOptions {
            bundle_path,
            container_pidfile,
            container_id,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum RuncMethod {
    Spec,
    Create,
    Start,
    Kill,
    Delete,
    State,
}

impl RuncMethod {
    fn to_string(&self) -> String {
        match self {
            Self::Spec => String::from("spec"),
            Self::Create => String::from("create"),
            Self::Start => String::from("start"),
            Self::Kill => String::from("kill"),
            Self::Delete => String::from("delete"),
            Self::State => String::from("state"),
        }
    }
}

#[derive(Debug)]
pub enum ContainerRuntimeError {
    // represents an error executing runc
    RuncError {
        method: RuncMethod,
        container_id: ID,
        source: std::io::Error,
    },
    // represents an error executing sed
    SedError {
        updating: String,
        source: std::io::Error,
    },
    // represents an error converting the status of a container from bytes to a string
    ConvertContainerStatusError(FromUtf8Error),
    // represents an error parsing the status of a container
    ParseContainerStatusError(serde_json::Error),
    // represents an error when the container is not found by the runtime
    ContainerNotFoundError {
        container_id: ID,
    },
}

impl fmt::Display for ContainerRuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::RuncError {
                ref method,
                ref container_id,
                ..
            } => {
                if *method == RuncMethod::Create || *method == RuncMethod::Spec {
                    write!(f, "failed to execute runc {:?}", method)
                } else {
                    write!(
                        f,
                        "failed to execute runc {:?} for container_id {}",
                        method, container_id
                    )
                }
            }
            Self::SedError { ref updating, .. } => {
                write!(f, "failed to use sed to update {}", updating)
            }
            Self::ConvertContainerStatusError(ref err) => err.fmt(f),
            Self::ParseContainerStatusError(ref err) => err.fmt(f),
            Self::ContainerNotFoundError { ref container_id } => {
                write!(f, "container with container_id {} not found", container_id)
            }
        }
    }
}

impl Error for ContainerRuntimeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match *self {
            Self::RuncError { ref source, .. } => Some(source),
            Self::SedError { ref source, .. } => Some(source),
            Self::ConvertContainerStatusError(_) => None,
            Self::ParseContainerStatusError(_) => None,
            Self::ContainerNotFoundError { .. } => None,
        }
    }
}

impl ContainerRuntime {
    pub fn new(runtime_path: String) -> ContainerRuntime {
        ContainerRuntime { runtime_path }
    }

    pub fn new_runtime_spec(
        self: &Self,
        opts: &RuntimeSpecOptions,
    ) -> Result<(), ContainerRuntimeError> {
        // generate generic spec
        let mut runc = Command::new(&self.runtime_path);
        runc.arg(RuncMethod::Spec.to_string())
            .arg("--bundle")
            .arg(&opts.bundle_path);
        runc.output()
            .map_err(|source| ContainerRuntimeError::RuncError {
                method: RuncMethod::Spec,
                container_id: String::from(""),
                source,
            })?;
        let config_path = format!("{}/config.json", &opts.bundle_path);
        // override `args` in spec with combination of command and args options
        let mut args = String::from(format!("\"{}\"", opts.command));
        for arg in opts.args.iter() {
            args.push_str(&format!(", \"{}\"", arg));
        }
        let mut args_sed = Command::new("sed");
        args_sed
            .arg("-i")
            .arg(format!("s/\"sh\"/{}/", args))
            .arg(&config_path);
        args_sed
            .output()
            .map_err(|source| ContainerRuntimeError::SedError {
                updating: String::from("args"),
                source,
            })?;
        // override `"terminal": true` to `"terminal": false` in spec
        let mut terminal_sed = Command::new("sed");
        terminal_sed
            .arg("-i")
            .arg("s/\"terminal\": true/\"terminal\": false/")
            .arg(&config_path);
        terminal_sed
            .output()
            .map_err(|source| ContainerRuntimeError::SedError {
                updating: String::from("terminal settings"),
                source,
            })?;
        Ok(())
    }

    pub fn create_container(
        self: &Self,
        opts: RuntimeCreateOptions,
    ) -> Result<(), ContainerRuntimeError> {
        // command to execute: runc create --bundle bundle --pid-file container_pidfile container_id
        let mut runc_create = Command::new(&self.runtime_path);
        runc_create
            .arg(RuncMethod::Create.to_string())
            .arg("--bundle")
            .arg(&opts.bundle_path)
            .arg("--pid-file")
            .arg(format!("{}/{}", &opts.bundle_path, &opts.container_pidfile))
            .arg(opts.container_id);
        match runc_create.spawn() {
            Ok(out) => debug!("runc create output: {:?}", out),
            Err(source) => {
                return Err(ContainerRuntimeError::RuncError {
                    method: RuncMethod::Create,
                    container_id: String::from(""),
                    source,
                });
            }
        }
        Ok(())
    }

    pub fn start_container(self: &Self, container_id: &ID) -> Result<(), ContainerRuntimeError> {
        let mut runc_start = Command::new(&self.runtime_path);
        runc_start
            .arg(RuncMethod::Start.to_string())
            .arg(format!("{}", container_id));
        match runc_start.spawn() {
            Ok(out) => debug!("runc start output: {:?}", out),
            Err(source) => {
                return Err(ContainerRuntimeError::RuncError {
                    method: RuncMethod::Start,
                    container_id: container_id.clone(),
                    source,
                });
            }
        }
        Ok(())
    }

    pub fn kill_container(self: &Self, container_id: &ID) -> Result<(), ContainerRuntimeError> {
        let mut runc_kill = Command::new(&self.runtime_path);
        runc_kill
            .arg(RuncMethod::Kill.to_string())
            .arg(format!("{}", container_id))
            .arg("9");
        match runc_kill.output() {
            Ok(out) => debug!("runc kill output: {:?}", out),
            Err(source) => {
                return Err(ContainerRuntimeError::RuncError {
                    method: RuncMethod::Kill,
                    container_id: container_id.clone(),
                    source,
                });
            }
        }
        Ok(())
    }

    pub fn delete_container(self: &Self, container_id: &ID) -> Result<(), ContainerRuntimeError> {
        let mut runc_delete = Command::new(&self.runtime_path);
        runc_delete
            .arg(RuncMethod::Delete.to_string())
            .arg(format!("{}", container_id));
        match runc_delete.output() {
            Ok(out) => debug!("runc delete output: {:?}", out),
            Err(source) => {
                return Err(ContainerRuntimeError::RuncError {
                    method: RuncMethod::Delete,
                    container_id: container_id.clone(),
                    source,
                });
            }
        }
        Ok(())
    }

    pub fn get_container_status(
        self: &Self,
        container_id: &ID,
    ) -> Result<Status, ContainerRuntimeError> {
        let mut runc_status_cmd = Command::new(&self.runtime_path);
        runc_status_cmd
            .arg(RuncMethod::State.to_string())
            .arg(format!("{}", container_id));
        let status = match runc_status_cmd.output() {
            Ok(out) => out.stdout,
            Err(source) => {
                return Err(ContainerRuntimeError::RuncError {
                    method: RuncMethod::State,
                    container_id: container_id.clone(),
                    source,
                });
            }
        };
        let runc_status_str = String::from_utf8(status)
            .map_err(|source| ContainerRuntimeError::ConvertContainerStatusError(source))?;
        if runc_status_str == "" {
            // runc does not know about this container
            return Err(ContainerRuntimeError::ContainerNotFoundError {
                container_id: container_id.clone(),
            });
        }
        let runc_status: RuncStatus = serde_json::from_str(&runc_status_str)
            .map_err(|source| ContainerRuntimeError::ParseContainerStatusError(source))?;
        Ok(Status::from_runc_status(&runc_status))
    }
}
