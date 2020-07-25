use clap::{App, Arg, SubCommand};

use cruise::container_manager;
use cruise::daemon;

const RUN_SUBCMD: &str = "run";
const PORT: &str = "port";

fn main() {
    let matches = App::new("Cruise daemon")
        .version("0.0")
        .author("Will D. <wjdeuschle@gmail.com>")
        .about("Cruise container manager daemon")
        .subcommand(
            SubCommand::with_name(RUN_SUBCMD).about("runs daemon").arg(
                Arg::with_name(PORT)
                    .help("port where daemon listens")
                    .long(PORT)
                    .default_value("50051"),
            ),
        )
        .get_matches();

    if let Some(matches) = matches.subcommand_matches(RUN_SUBCMD) {
        let port = matches.value_of(PORT).unwrap();
        daemon::new(container_manager::new())
            .run_server(port)
            .expect("Cruise daemon server failed");
    }
}
