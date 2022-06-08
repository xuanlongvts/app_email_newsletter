use config::{Config, ConfigError, File};
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct DatabaseSettings {
	pub username: String,
	pub password: Secret<String>,
	pub port: u16,
	pub host: String,
	pub database_name: String,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
	pub database: DatabaseSettings,
	pub application_port: u16,
}

pub fn get_configuration() -> Result<Settings, ConfigError> {
	let settings = Config::builder()
		.add_source(File::with_name("configuration"))
		.build()?;

	settings.try_deserialize()
}

impl DatabaseSettings {
	pub fn connection_string(&self) -> Secret<String> {
		Secret::new(format!(
			"postgres://{}:{}@{}:{}/{}",
			self.username,
			self.password.expose_secret(),
			self.host,
			self.port,
			self.database_name
		))
	}

	pub fn connection_string_without_db(&self) -> Secret<String> {
		Secret::new(format!(
			"postgres://{}:{}@{}:{}",
			self.username,
			self.password.expose_secret(),
			self.host,
			self.port
		))
	}
}
