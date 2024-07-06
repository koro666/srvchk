use clap::Parser;
use clap_verbosity_flag::{InfoLevel, Verbosity};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(version)]
pub struct Arguments {
	#[command(flatten)]
	pub verbose: Verbosity<InfoLevel>,

	#[arg(short, long, help = "Path to configuration file")]
	pub configuration: Option<PathBuf>,
}
