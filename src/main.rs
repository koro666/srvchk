use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser)]
struct Arguments {
	#[command(flatten)]
	verbose: clap_verbosity_flag::Verbosity,

	#[arg(short, long, help = "Path to configuration file")]
	configuration: Option<PathBuf>
}

fn main() {
	let args = Arguments::parse();
	println!("{:#?}", args);
	// TODO:
}
