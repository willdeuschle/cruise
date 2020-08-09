use log::{Metadata, Record};

pub struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        let target = metadata.target();
        target == "cruise::daemon" || target == "cruise::client"
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("{}: {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}
