use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser)]
pub struct Arguments {
	#[command(flatten)]
	pub verbose: clap_verbosity_flag::Verbosity,

	#[arg(short, long, help = "Path to configuration file")]
	pub configuration: Option<PathBuf>,
}
