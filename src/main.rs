mod arguments;
mod configuration;

use arguments::Arguments;
use clap::Parser;
use configuration::Configuration;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
	let args = Arguments::parse();
	let cfg = Configuration::read(args.configuration.as_deref())?;

	// TODO:
	println!("{:#?}", cfg);

	Ok(())
}
