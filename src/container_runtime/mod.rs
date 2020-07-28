use std::process::Command;

#[derive(Debug)]
pub struct ContainerRuntime {
    runtime_path: String,
}

pub struct RuntimeSpecOptions {
    pub command: String,
    pub args: Vec<String>,
    pub root_path: String,
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
        runc.arg("spec").arg("--bundle").arg(&opts.root_path);
        match runc.output() {
            Ok(_) => (),
            Err(err) => {
                return Err(ContainerRuntimeError {
                    reason: format!("failed to execute runc spec: {}", err),
                })
            }
        }
        let config_path = format!("{}/config.json", &opts.root_path);
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
}
