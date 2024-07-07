mod arguments;
mod configuration;
mod logging;
mod notifier;
mod pinger;

use arguments::Arguments;
use clap::Parser;
use configuration::{Configuration, Host};
use log::{debug, error, info, warn};
use logging::Logger;
use notifier::Client;
use pinger::Pinger;
use rand::{rngs::OsRng, Rng};
use std::{error::Error, sync::Arc, time::Duration};
use tokio::{runtime::Builder, task::JoinSet};

fn main() -> Result<(), Box<dyn Error>> {
	let args = Arguments::parse();
	Logger::new(args.verbose.log_level_filter()).install()?;

	let cfg = Configuration::read(args.configuration.as_deref())?;

	let ntfy = Client::new(cfg.ntfy)?;
	let ping = Pinger::new()?;

	let rt = Builder::new_current_thread().enable_all().build()?;
	rt.block_on(execute(ntfy, ping, cfg.hosts));

	Ok(())
}

async fn execute(ntfy: Client, ping: Pinger, hosts: Vec<Host>) {
	let ping = Arc::new(ping);

	let mut set = JoinSet::new();
	for host in hosts {
		set.spawn(execute_one(ntfy.clone(), ping.clone(), host));
	}

	if set.is_empty() {
		error!("no hosts defined")
	}

	while let Some(_) = set.join_next().await {}
}

async fn execute_one(ntfy: Client, ping: Arc<Pinger>, host: Host) {
	let target = format!(concat!(module_path!(), ":{}"), host.dns);
	debug!(target: &target, "starting");

	let mut random = OsRng {};
	let mut reachable = true;

	loop {
		let delay = random.gen_range((host.delay - host.jitter)..(host.delay + host.jitter));
		if delay > 0.0 {
			debug!(target: &target, "sleeping for {:.2} seconds...", delay);
			tokio::time::sleep(Duration::from_secs_f32(delay)).await;
		}

		let success = ping.ping(&host.dns, host.family).await.unwrap_or(false);

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

		let _ = ntfy.notify(host.name.as_deref(), &host.dns).await;
	}
}
