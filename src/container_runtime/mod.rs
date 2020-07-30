use crate::container::ID;
use std::process::Command;

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

#[derive(Debug)]
pub struct ContainerRuntimeError {
    reason: String,
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
        runc.arg("spec").arg("--bundle").arg(&opts.bundle_path);
        match runc.output() {
            Ok(_) => (),
            Err(err) => {
                return Err(ContainerRuntimeError {
                    reason: format!("failed to execute runc spec: {}", err),
                })
            }
        }
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
        match args_sed.output() {
            Ok(_) => (),
            Err(err) => {
                return Err(ContainerRuntimeError {
                    reason: format!("failed to update args with sed: {}", err),
                })
            }
        }
        // override `"terminal": true` to `"terminal": false` in spec
        let mut terminal_sed = Command::new("sed");
        terminal_sed
            .arg("-i")
            .arg("s/\"terminal\": true/\"terminal\": false/")
            .arg(&config_path);
        match terminal_sed.output() {
            Ok(_) => (),
            Err(err) => {
                return Err(ContainerRuntimeError {
                    reason: format!("failed to update terminal settings with sed: {}", err),
                })
            }
        }
        Ok(())
    }

    pub fn create_container(
        self: &Self,
        opts: RuntimeCreateOptions,
    ) -> Result<(), ContainerRuntimeError> {
        // command to execute: runc create --bundle bundle --pid-file container_pidfile container_id
        let mut runc_create = Command::new(&self.runtime_path);
        runc_create
            .arg("create")
            .arg("--bundle")
            .arg(&opts.bundle_path)
            .arg("--pid-file")
            .arg(format!("{}/{}", &opts.bundle_path, &opts.container_pidfile))
            .arg(opts.container_id);
        match runc_create.spawn() {
            // printing this for now so that we can see the result of the execution
            // TODO: clean this up (debug logging?)
            Ok(out) => println!("out: {:?}", out),
            Err(err) => {
                return Err(ContainerRuntimeError {
                    reason: format!("failed to spawn `runc create`: {}", err),
                })
            }
        }
        Ok(())
    }

    pub fn start_container(self: &Self, container_id: &ID) -> Result<(), ContainerRuntimeError> {
        let mut runc_start = Command::new(&self.runtime_path);
        runc_start.arg("start").arg(format!("{}", container_id));
        match runc_start.spawn() {
            // printing this for now so that we can see the result of the execution
            // TODO: clean this up (debug logging?)
            Ok(out) => println!("out: {:?}", out),
            Err(err) => {
                return Err(ContainerRuntimeError {
                    reason: format!("failed to spawn `runc start {}`: {}", container_id, err),
                })
            }
        }
        Ok(())
    }
}
