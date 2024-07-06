mod arguments;
mod configuration;
mod logging;

use arguments::Arguments;
use clap::Parser;
use configuration::Configuration;
use log::debug;
use logging::Logger;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
	let args = Arguments::parse();
	Logger::new(args.verbose.log_level_filter()).install()?;

	let cfg = Configuration::read(args.configuration.as_deref())?;

	// TODO:
	debug!("{:?}", cfg);

	Ok(())
}
