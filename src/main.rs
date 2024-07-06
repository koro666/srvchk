mod arguments;
mod configuration;
mod logging;

use arguments::Arguments;
use clap::Parser;
use configuration::{Configuration, Host, Ntfy};
use log::{debug, warn};
use logging::Logger;
use std::{error::Error, sync::Arc};
use tokio::{runtime::Builder, task::JoinSet};

fn main() -> Result<(), Box<dyn Error>> {
	let args = Arguments::parse();
	Logger::new(args.verbose.log_level_filter()).install()?;

	let cfg = Configuration::read(args.configuration.as_deref())?;

	let rt = Builder::new_current_thread().build()?;
	rt.block_on(execute(cfg));

	Ok(())
}

async fn execute(cfg: Configuration) {
	let ntfy = Arc::new(cfg.ntfy);

	let mut set = JoinSet::new();
	for host in cfg.hosts {
		set.spawn(execute_one(ntfy.clone(), host));
	}

	if set.is_empty() {
		warn!("no hosts defined")
	}

	while let Some(_) = set.join_next().await {}
}

async fn execute_one(ntfy: Arc<Ntfy>, host: Host) {
	debug!("handling host {:?}", host.name);

	// TODO:
}
