use clap::{App, AppSettings, Arg, SubCommand};

use cruise::client;

const CONTAINER_SUBCMD: &str = "container";
const PORT: &str = "port";

const CONTAINER_CREATE: &str = "create";
const CONTAINER_START: &str = "start";
const CONTAINER_STOP: &str = "stop";
const CONTAINER_GET: &str = "get";
const CONTAINER_LIST: &str = "list";
const CONTAINER_DELETE: &str = "delete";

const CONTAINER_ROOTFS_PATH: &str = "rootfs";
const CONTAINER_NAME: &str = "CONTAINER_NAME";
const CONTAINER_ID: &str = "CONTAINER_ID";
const CONTAINER_CMD: &str = "CONTAINER_CMD";
const CONTAINER_ARGS: &str = "CONTAINER_ARGS";

fn main() {
    let matches = App::new("Cruise client")
        .version("0.0")
        .author("Will D. <wjdeuschle@gmail.com>")
        .about("Cruise container manager client")
        .subcommand(
            SubCommand::with_name(CONTAINER_SUBCMD)
                .about("controls containers")
                .arg(
                    Arg::with_name(PORT)
                        .help("port where client connects to daemon")
                        .long(PORT)
                        .default_value("50051"),
                )
                .subcommand(
                    SubCommand::with_name(CONTAINER_CREATE)
                        .setting(AppSettings::TrailingVarArg)
                        .about("creates container")
                        .arg(
                            Arg::with_name(CONTAINER_ROOTFS_PATH)
                                .help("container rootfs path")
                                .long(CONTAINER_ROOTFS_PATH)
                                .takes_value(true)
                                .required(true),
                        )
                        .arg(
                            Arg::with_name(CONTAINER_NAME)
                                .help("container name")
                                .required(true)
                                .index(1),
                        )
                        .arg(
                            Arg::with_name(CONTAINER_CMD)
                                .help("container command")
                                .required(true)
                                .index(2),
                        )
                        .arg(
                            Arg::with_name(CONTAINER_ARGS)
                                .help("container args")
                                .multiple(true),
                        ),
                )
                .subcommand(
                    SubCommand::with_name(CONTAINER_START)
                        .about("starts container")
                        .arg(
                            Arg::with_name(CONTAINER_ID)
                                .help("container id")
                                .required(true)
                                .index(1),
                        ),
                )
                .subcommand(SubCommand::with_name(CONTAINER_STOP).about("stops container"))
                .subcommand(
                    SubCommand::with_name(CONTAINER_GET)
                        .about("gets container")
                        .arg(
                            Arg::with_name(CONTAINER_ID)
                                .help("container id")
                                .required(true)
                                .index(1),
                        ),
                )
                .subcommand(SubCommand::with_name(CONTAINER_LIST).about("lists container"))
                .subcommand(SubCommand::with_name(CONTAINER_DELETE).about("deletes container")),
        )
        .get_matches();

    if let Some(matches) = matches.subcommand_matches(CONTAINER_SUBCMD) {
        let port = matches.value_of(PORT).unwrap();
        if let Some(matches) = matches.subcommand_matches(CONTAINER_CREATE) {
            let container_name = matches.value_of(CONTAINER_NAME).unwrap();
            let container_cmd = matches.value_of(CONTAINER_CMD).unwrap();
            let container_rootfs_path = matches.value_of(CONTAINER_ROOTFS_PATH).unwrap();
            let container_args = matches
                .values_of(CONTAINER_ARGS)
                .unwrap()
                .map(|s| s.to_string())
                .collect();
            client::create_container(
                port,
                container_name,
                container_cmd,
                container_args,
                container_rootfs_path,
            )
            .expect("create container failed");
        }
        if let Some(matches) = matches.subcommand_matches(CONTAINER_START) {
            let container_id = matches.value_of(CONTAINER_ID).unwrap();
            client::start_container(port, container_id.into()).expect("start container failed");
        }
        if let Some(matches) = matches.subcommand_matches(CONTAINER_STOP) {
            println!("NOT IMPLEMENTED: container stop {:?}", matches)
        }
        if let Some(matches) = matches.subcommand_matches(CONTAINER_GET) {
            let container_id = matches.value_of(CONTAINER_ID).unwrap();
            client::get_container(port, container_id.into()).expect("get container failed");
        }
        if let Some(_) = matches.subcommand_matches(CONTAINER_LIST) {
            client::list_containers(port).expect("list containers failed");
        }
        if let Some(matches) = matches.subcommand_matches(CONTAINER_DELETE) {
            println!("NOT IMPLEMENTED: container delete {:?}", matches)
        }
    }
}
