use log::*;

static LOGGER: SimpleLogger = SimpleLogger;

pub fn init(level: &str) -> Result<(), SetLoggerError> {
    use core::str::FromStr;

    set_logger(&LOGGER)
        .map(|()| set_max_level(LevelFilter::from_str(level)
                                .unwrap_or(LevelFilter::Debug)))
}

struct SimpleLogger;

impl Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Trace
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("\x1b[{}m {:5} - {} \x1b[0m", level_to_color_code(record.level()), record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

// (16进制) \x1b == \033 (8进制)

fn level_to_color_code(level: Level) -> u8 {
    match level {
        Level::Error => 31, // Red
        Level::Warn => 33, // Yellow
        Level::Info => 32, // Green
        Level::Debug => 36, // SkyBlue
        Level::Trace => 90, // BrightBlack
    }
}

