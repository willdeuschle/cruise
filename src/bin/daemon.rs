use clap::{App, Arg, SubCommand};
use log::{error, LevelFilter};
use std::process;

use cruise::container_manager;
use cruise::daemon;
use cruise::logging::SimpleLogger;

static LOGGER: SimpleLogger = SimpleLogger;

const RUN_SUBCMD: &str = "run";
const PORT: &str = "port";
const LIB_ROOT: &str = "lib_root";
const RUNTIME_PATH: &str = "runtime_path";
const DEBUG_ARG: &str = "debug";

fn main() {
    let matches = App::new("Cruise daemon")
        .version("0.0")
        .author("Will D. <wjdeuschle@gmail.com>")
        .about("Cruise container manager daemon")
        .arg(
            Arg::with_name(DEBUG_ARG)
                .help("enable debug logging")
                .long("debug")
                .short("v")
        )
        .subcommand(
            SubCommand::with_name(RUN_SUBCMD).about("runs daemon").arg(
                Arg::with_name(PORT)
                    .help("port where daemon listens")
                    .long(PORT)
                    .default_value("50051"))
            .arg(
                Arg::with_name(LIB_ROOT)
                    .help(
                        "root directory for persistent data, like container bundles, status, etc.",
                    )
                    .long(LIB_ROOT)
                    .default_value("/var/lib/cruise"),
            )
            .arg(
                Arg::with_name(RUNTIME_PATH)
                    .help(
                        "path to runtime (runc) executable",
                    )
                    .long(RUNTIME_PATH)
                    .default_value("/usr/bin/runc"),
            ),
        )
        .get_matches();

    let log_level = if matches.is_present(DEBUG_ARG) {
        LevelFilter::Debug
    } else {
        LevelFilter::Error
    };
    let _ = log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(log_level))
        .unwrap_or_else(|err| {
            eprintln!("failed to set logger: `{}`", err);
        });

    if let Some(matches) = matches.subcommand_matches(RUN_SUBCMD) {
        let port = matches.value_of(PORT).unwrap();
        let root_dir = matches.value_of(LIB_ROOT).unwrap();
        let runtime_path = matches.value_of(RUNTIME_PATH).unwrap();
        let container_manager =
            match container_manager::ContainerManager::new(root_dir.into(), runtime_path.into()) {
                Ok(container_manager) => container_manager,
                Err(err) => {
                    error!("failed to create container manager: {}", err);
                    process::exit(1);
                }
            };
        daemon::new(container_manager)
            .run_server(port)
            .expect("Cruise daemon server failed");
    }
}
