use std::sync::Arc;

use clap::{crate_name, crate_version};
use log::{debug, error};
use reqwest::ClientBuilder;
use serde::{Deserialize, Serialize};

static USER_AGENT: &str = concat!(crate_name!(), "/", crate_version!());

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct Target {
	pub url: String,
	pub username: Option<String>,
	pub password: Option<String>,
	pub topic: String,
	pub icon: Option<String>,
}

impl Default for Target {
	fn default() -> Self {
		Self {
			url: "https://ntfy.sh/".to_string(),
			username: None,
			password: None,
			topic: crate_name!().to_string(),
			icon: None,
		}
	}
}

#[derive(Clone)]
pub struct Client {
	target: Arc<Target>,
	client: reqwest::Client,
}

#[derive(Debug, Serialize)]
struct Payload<'x> {
	topic: &'x str,
	title: String,
	message: String,
	icon: Option<&'x str>,
}

impl Client {
	pub fn new(target: Target) -> reqwest::Result<Self> {
		let target = Arc::new(target);
		let client = ClientBuilder::new().user_agent(USER_AGENT).build()?;
		Ok(Self {
			target: target,
			client: client,
		})
	}

	pub async fn notify(&self, name: Option<&str>, dns: &str) -> reqwest::Result<()> {
		let builder = self.client.post(self.target.url.clone());

		let builder = match &self.target.username {
			Some(username) => builder.basic_auth(username, self.target.password.as_deref()),
			None => builder,
		};

		let payload = Payload {
			topic: &self.target.topic,
			title: format!("{} is down!", name.unwrap_or(dns)),
			message: format!("Host {:?} is unreachable.", dns),
			icon: self.target.icon.as_deref(),
		};

		let builder = builder.json(&payload);

		match async move {
			let response = builder.send().await?;
			let response = response.error_for_status()?;
			Ok(response)
		}
		.await
		{
			Ok(_) => {
				debug!("request sent");
				Ok(())
			}
			Err(error) => {
				error!("request failure: {}", error);
				Err(error)
			}
		}
	}
}
