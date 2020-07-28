use std::process::Command;

#[derive(Debug)]
pub struct ContainerRuntime {
    runtime_path: String,
}

// TODO: see if all these fields are necessary based on the generation
pub struct RuntimeSpecOptions {
    //pub Command: String,
    //pub Args: Vec<String>,
    pub root_path: String,
    //pub RootReadonly: bool,
}

#[derive(Debug)]
pub struct ContainerRuntimeError {
    reason: String,
}

impl ContainerRuntime {
    pub fn new(runtime_path: String) -> ContainerRuntime {
        ContainerRuntime { runtime_path }
    }

    // TODO: see what needs to change from base "runc spec". likely at least the commands and the
    // terminal setting
    pub fn new_runtime_spec(
        self: &Self,
        opts: &RuntimeSpecOptions,
    ) -> Result<(), ContainerRuntimeError> {
        let mut runc = Command::new(&self.runtime_path);
        runc.arg("spec").arg("--bundle").arg(&opts.root_path);
        match runc.output() {
            Ok(_) => Ok(()),
            Err(err) => {
                return Err(ContainerRuntimeError {
                    reason: format!("failed to execute runc spec: {}", err),
                })
            }
        }
    }
}
