use std::{error::Error, fs, path::Path};

use crate::notifier::Target;
use crate::pinger::Family;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct Host {
	pub name: Option<String>,
	pub dns: String,
	pub family: Family,
	pub delay: f32,
	pub jitter: f32,
}

impl Default for Host {
	fn default() -> Self {
		Self {
			name: None,
			dns: String::new(),
			family: Family::Any,
			delay: 60.0,
			jitter: 10.0,
		}
	}
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct Configuration {
	pub ntfy: Target,
	pub hosts: Vec<Host>,
}

impl Default for Configuration {
	fn default() -> Self {
		Configuration {
			ntfy: Target::default(),
			hosts: Vec::new(),
		}
	}
}

impl Configuration {
	pub fn read(path: Option<&Path>) -> Result<Self, Box<dyn Error>> {
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
