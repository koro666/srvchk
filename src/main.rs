mod arguments;
mod configuration;
mod logging;
mod pinger;

use arguments::Arguments;
use clap::{crate_name, crate_version, Parser};
use configuration::{Configuration, Host, Ntfy};
use log::{debug, error, info, warn};
use logging::Logger;
use pinger::Pinger;
use rand::{rngs::OsRng, Rng};
use reqwest::{Client, ClientBuilder};
use std::{collections::HashMap, error::Error, sync::Arc, time::Duration};
use tokio::{runtime::Builder, task::JoinSet};

static USER_AGENT: &str = concat!(crate_name!(), "/", crate_version!());

fn main() -> Result<(), Box<dyn Error>> {
	let args = Arguments::parse();
	Logger::new(args.verbose.log_level_filter()).install()?;

	let cfg = Configuration::read(args.configuration.as_deref())?;
	let ping = Pinger::new()?;

	let client = ClientBuilder::new().user_agent(USER_AGENT).build()?;

	let rt = Builder::new_current_thread().enable_all().build()?;
	rt.block_on(execute(cfg, ping, client));

	Ok(())
}

async fn execute(cfg: Configuration, ping: Pinger, client: Client) {
	let ntfy = Arc::new(cfg.ntfy);
	let ping = Arc::new(ping);

	let mut set = JoinSet::new();
	for host in cfg.hosts {
		set.spawn(execute_one(ntfy.clone(), host, ping.clone(), client.clone()));
	}

	if set.is_empty() {
		error!("no hosts defined")
	}

	while let Some(_) = set.join_next().await {}
}

async fn execute_one(ntfy: Arc<Ntfy>, host: Host, ping: Arc<Pinger>, client: Client) {
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

		let success = match ping.ping(&host.dns, host.family).await {
			Err(_) => false,
			Ok(status) => status,
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

		match notify_down(&ntfy, &host, &client).await {
			Err(error) => {
				error!(target: &target, "ntfy request failure: {}", error);
			}
			_ => {
				debug!(target: &target, "ntfy request sent");
			}
		}
	}
}

async fn notify_down(ntfy: &Ntfy, host: &Host, client: &Client) -> reqwest::Result<()> {
	let builder = client.post(ntfy.url.clone());

	let builder = match &ntfy.username {
		Some(username) => builder.basic_auth(username, ntfy.password.as_deref()),
		None => builder,
	};

	let mut json = HashMap::new();
	json.insert("topic", ntfy.topic.clone());
	json.insert(
		"title",
		format!("{} is down!", host.name.as_deref().unwrap_or(&host.dns)),
	);
	json.insert("message", format!("Host {:?} is not responding.", host.dns));

	if let Some(icon) = &ntfy.icon {
		json.insert("icon", icon.to_string());
	}

	let builder = builder.json(&json);

	let response = builder.send().await?;
	response.error_for_status()?;

	Ok(())
}
