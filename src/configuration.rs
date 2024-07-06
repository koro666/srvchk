use serde::Deserialize;
use std::{error::Error, fs, path::Path};

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct Ntfy {
	pub url: String,
	pub username: Option<String>,
	pub password: Option<String>,
	pub topic: String,
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
pub enum HostKind {
	Any,
	IPv4,
	IPv6,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct Host {
	pub name: String,
	pub kind: HostKind,
	pub delay: f32,
	pub jitter: f32,
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
pub struct Configuration {
	pub ntfy: Ntfy,
	pub hosts: Vec<Host>,
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
