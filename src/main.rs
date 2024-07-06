use clap::Parser;
use serde::Deserialize;
use std::{
	error::Error,
	fs,
	path::{Path, PathBuf},
};

#[derive(Debug, Parser)]
struct Arguments {
	#[command(flatten)]
	verbose: clap_verbosity_flag::Verbosity,

	#[arg(short, long, help = "Path to configuration file")]
	configuration: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
struct Ntfy {
	url: String,
	username: Option<String>,
	password: Option<String>,
	topic: String,
}

impl Default for Ntfy {
	fn default() -> Self {
		Self {
			url: "https://ntfy.sh/".to_string(),
			username: None,
			password: None,
			topic: "srvchk".to_string(),
		}
	}
}

#[derive(Debug, Deserialize)]
enum HostKind {
	Any,
	IPv4,
	IPv6,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
struct Host {
	name: String,
	kind: HostKind,
	delay: f32,
	jitter: f32,
}

impl Default for Host {
	fn default() -> Self {
		Self {
			name: String::new(),
			kind: HostKind::Any,
			delay: 60.0,
			jitter: 10.0,
		}
	}
}

#[derive(Debug, Deserialize)]
#[serde(default)]
struct Configuration {
	ntfy: Ntfy,
	hosts: Vec<Host>,
}

impl Default for Configuration {
	fn default() -> Self {
		Configuration {
			ntfy: Ntfy::default(),
			hosts: Vec::new(),
		}
	}
}

impl Configuration {
	fn read(path: Option<&Path>) -> Result<Self, Box<dyn Error>> {
		match path {
			None => Ok(Self::default()),
			Some(path) => {
				let text = fs::read_to_string(path)?;
				let cfg = toml::from_str(&text)?;
				Ok(cfg)
			}
		}
	}
}

fn main() -> Result<(), Box<dyn Error>> {
	let args = Arguments::parse();
	let cfg = Configuration::read(args.configuration.as_deref())?;

	// TODO:
	println!("{:#?}", cfg);

	Ok(())
}
