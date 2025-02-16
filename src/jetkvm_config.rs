use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct JetKvmConfig {
    pub host: String,
    pub port: String,
    pub api: String,
    pub password: String,
}

impl Default for JetKvmConfig {
    fn default() -> Self {
        Self {
            host: "".into(),
            port: "80".into(),
            api: "/webrtc/session".into(),
            password: "".into(),
        }
    }
}

impl JetKvmConfig {
    pub fn load() -> Result<Self, ConfigError> {
        let config = Config::builder()
            .add_source(File::with_name("config").required(false)) // Loads from config.toml (optional)
            .add_source(Environment::default().separator("_")) // Overrides with environment variables
            .build()?;

        config
            .try_deserialize()
            .or_else(|_| Ok(JetKvmConfig::default()))
    }

    /// Returns the full URL to use for the SDP exchange.
    pub fn session_url(&self) -> String {
        format!("http://{}:{}{}", self.host, self.port, self.api)
    }
}
