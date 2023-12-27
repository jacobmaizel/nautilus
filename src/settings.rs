use config::{Config, ConfigError, Environment, File};
// use serde::Deserialize;
use serde_derive::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[allow(unused)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(unused)]
pub struct TracingConfig {
    pub exporter_url: String,
    pub resource_name: String,
    pub rust_log: String,
    pub enabled: bool,
    pub gcloud_project_id: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(unused)]
pub struct ServerConfig {
    pub address: String,
    // pub port: String,
    pub allowed_origins: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(unused)]
pub struct Settings {
    pub server: ServerConfig,
    pub tracing: TracingConfig,
    pub environment: String,
    pub auth_domain: String,
    pub auth_audience: String,
    pub auth_management_audience: String,
    pub auth_management_client_id: String,
    pub auth_management_secret: String,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let environment =
            std::env::var("NAUTILUS_ENVIRONMENT").unwrap_or_else(|_| "development".into());
        let conf = Config::builder()
            // .add_source(File::with_name("config/default"))
            .add_source(File::with_name(&format!("config/{}", environment)))
            .add_source(File::with_name("config/local").required(false))
            // Eg.. `NAUTILUS_DEBUG=1 ./target/app` would set the `debug` key
            .add_source(
                Environment::with_prefix("NAUTILUS")
                    .try_parsing(true)
                    .separator("_"),
            )
            .add_source(
                Environment::with_prefix("AUTH")
                    .separator("__")
                    .try_parsing(true),
            )
            .build()?;

        conf.try_deserialize()
    }
}
