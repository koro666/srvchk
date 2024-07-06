mod arguments;
mod configuration;
mod logging;

use arguments::Arguments;
use clap::Parser;
use configuration::{Configuration, Host, HostKind, Ntfy};
use log::{debug, error, info, warn};
use logging::Logger;
use rand::{rngs::OsRng, Rng};
use std::{
	error::Error,
	path::{Path, PathBuf},
	process::Stdio,
	sync::Arc,
	time::Duration,
};
use tokio::{process::Command, runtime::Builder, task::JoinSet};
use which::which;

fn main() -> Result<(), Box<dyn Error>> {
	let args = Arguments::parse();
	Logger::new(args.verbose.log_level_filter()).install()?;

	let cfg = Configuration::read(args.configuration.as_deref())?;

	let ping = which("ping")?;
	debug!("ping binary: {}", ping.to_string_lossy());

	let rt = Builder::new_current_thread().enable_all().build()?;
	rt.block_on(execute(cfg, ping));

	Ok(())
}

async fn execute(cfg: Configuration, ping: PathBuf) {
	let ntfy = Arc::new(cfg.ntfy);
	let ping = Arc::new(ping);

	let mut set = JoinSet::new();
	for host in cfg.hosts {
		set.spawn(execute_one(ntfy.clone(), host, ping.clone()));
	}

	if set.is_empty() {
		error!("no hosts defined")
	}

	while let Some(_) = set.join_next().await {}
}

async fn execute_one(ntfy: Arc<Ntfy>, host: Host, ping: Arc<PathBuf>) {
	let target = format!("srvchk:{}", host.name);
	debug!(target: &target, "starting");

	let mut random = OsRng {};
	let mut reachable = true;

	loop {
		let delay = random.gen_range((host.delay - host.jitter)..(host.delay + host.jitter));
		if delay > 0.0 {
			debug!(target: &target, "sleeping for {:.2} seconds...", delay);
			tokio::time::sleep(Duration::from_secs_f32(delay)).await;
		}

		let mut cmd = build_command(ping.as_ref(), &host.name, host.kind);
		debug!(target: &target, "running {:?}", cmd.as_std());

		let success = match cmd.status().await {
			Err(error) => {
				error!(target: &target, "execution failure: {}", error);
				false
			}
			Ok(status) => status.success(),
		};

		if success == reachable {
			debug!(target: &target, "reachability status has not changed");
			continue;
		};

		reachable = success;
		if reachable {
			info!(target: &target, "host is reachable again");
			continue;
		}

		warn!(target: &target, "host is unreachable");

		// TODO:
	}
}

fn build_command(ping: &Path, host: &str, kind: HostKind) -> Command {
	let mut cmd = Command::new(ping);
	cmd.args(&["-n", "-c", "2"]);

	match kind {
		HostKind::IPv4 => {
			cmd.arg("-4");
		}
		HostKind::IPv6 => {
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
