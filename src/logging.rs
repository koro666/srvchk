use log::{LevelFilter, Log, Metadata, Record, SetLoggerError};
use std::io::{stderr, Write};

pub struct Logger {
	level: LevelFilter,
}

impl Log for Logger {
	fn enabled(&self, metadata: &Metadata) -> bool {
		metadata.level() <= self.level
	}

	fn log(&self, record: &Record) {
		let mut handle = stderr().lock();
		let _ = writeln!(
			handle,
			"[{:.1}] [{}] {}",
			record.level(),
			record.target(),
			record.args()
		);
	}

	fn flush(&self) {
		let mut handle = stderr().lock();
		let _ = handle.flush();
	}
}

impl Logger {
	pub fn new(level: LevelFilter) -> Self {
		Logger { level: level }
	}

	pub fn install(self) -> Result<(), SetLoggerError> {
		log::set_max_level(self.level);
		log::set_boxed_logger(Box::new(self))
	}
}
