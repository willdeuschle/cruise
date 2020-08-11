use log::{Metadata, Record};

pub struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        let target = metadata.target();
        // this matches any of our library modules, daemon binary, or client binary
        target.starts_with("cruise::") || target == "daemon" || target == "client"
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("{}: {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}
