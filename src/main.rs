mod arguments;
mod configuration;
mod logging;

use arguments::Arguments;
use clap::{crate_name, crate_version, Parser};
use configuration::{Configuration, Family, Host, Ntfy};
use log::{debug, error, info, warn};
use logging::Logger;
use rand::{rngs::OsRng, Rng};
use reqwest::{Client, ClientBuilder};
use std::{
	collections::HashMap,
	error::Error,
	path::{Path, PathBuf},
	process::Stdio,
	sync::Arc,
	time::Duration,
};
use tokio::{process::Command, runtime::Builder, task::JoinSet};
use which::which;

static USER_AGENT: &str = concat!(crate_name!(), "/", crate_version!());

fn main() -> Result<(), Box<dyn Error>> {
	let args = Arguments::parse();
	Logger::new(args.verbose.log_level_filter()).install()?;

	let cfg = Configuration::read(args.configuration.as_deref())?;

	let ping = which("ping")?;
	debug!("ping binary: {}", ping.to_string_lossy());

	let client = ClientBuilder::new().user_agent(USER_AGENT).build()?;

	let rt = Builder::new_current_thread().enable_all().build()?;
	rt.block_on(execute(cfg, ping, client));

	Ok(())
}

async fn execute(cfg: Configuration, ping: PathBuf, client: Client) {
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

async fn execute_one(ntfy: Arc<Ntfy>, host: Host, ping: Arc<PathBuf>, client: Client) {
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

		let mut cmd = build_command(ping.as_ref(), &host.dns, host.family);
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

fn build_command(ping: &Path, host: &str, family: Family) -> Command {
	let mut cmd = Command::new(ping);
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
