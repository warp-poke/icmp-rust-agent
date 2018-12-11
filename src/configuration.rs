use serde_derive::Deserialize;
use config::{Config, ConfigError, Environment, File};

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub broker: String,
    pub topic: String,
    pub consumer_group: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub host: String,
    pub zone: String,
    pub iteration_number: i64,
}

impl Settings {
    pub fn from(path: String) -> Result<Self, ConfigError> {
        let mut s = Config::new();

        s.merge(File::with_name(&path).required(false))?;
        s.merge(Environment::with_prefix("AGENT"))?;

        // You can deserialize (and thus freeze) the entire configuration as
        s.try_into()
    }
}
