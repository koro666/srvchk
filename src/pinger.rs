use std::{path::PathBuf, process::Stdio};

use log::{debug, error};
use serde::Deserialize;
use tokio::process::Command;
use which::which;

#[derive(Debug)]
pub struct Pinger {
	binary: PathBuf,
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub enum Family {
	#[serde(rename = "*")]
	Any,
	#[serde(rename = "ipv4")]
	IPv4,
	#[serde(rename = "ipv6")]
	IPv6,
}

impl Pinger {
	pub fn new() -> Result<Self, which::Error> {
		let binary = which("ping")?;
		debug!("ping binary: {}", binary.to_string_lossy());
		Ok(Self { binary: binary })
	}

	pub async fn ping(&self, host: &str, family: Family) -> std::io::Result<bool> {
		let mut cmd = self.build(host, family);
		debug!("running {:?}", cmd.as_std());

		match cmd.status().await {
			Err(error) => {
				error!("execution failure: {}", error);
				Err(error)
			}
			Ok(status) => Ok(status.success()),
		}
	}

	#[cfg(unix)]
	fn build(&self, host: &str, family: Family) -> Command {
		let mut cmd = Command::new(&self.binary);
		cmd.args(&["-n", "-c", "2"]);

		match family {
			Family::IPv4 => {
				cmd.arg("-4");
			}
			Family::IPv6 => {
				cmd.arg("-6");
			}
			_ => {}
		};

		cmd.args(&["--", host]);

		cmd.stdin(Stdio::null());
		cmd.stdout(Stdio::null());
		cmd.stderr(Stdio::null());

		cmd.kill_on_drop(true);
		cmd
	}

	#[cfg(windows)]
	fn build(&self, host: &str, family: Family) -> Command {
		let mut cmd = Command::new(&self.binary);
		cmd.args(&["-n", "2"]);

		match family {
			Family::IPv4 => {
				cmd.arg("-4");
			}
			Family::IPv6 => {
				cmd.arg("-6");
			}
			_ => {}
		};

		cmd.arg(host);

		cmd.stdin(Stdio::null());
		cmd.stdout(Stdio::null());
		cmd.stderr(Stdio::null());

		cmd.creation_flags(0x08000000u32);
		cmd.kill_on_drop(true);
		cmd
	}
}
