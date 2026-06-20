use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub nats: NatsConfig,
    pub spine: SpineConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NatsConfig {
    pub url: String,
    pub store_dir: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SpineConfig {
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig { port: 3102 },
            nats: NatsConfig {
                url: "nats://localhost:4222".into(),
                store_dir: None,
            },
            spine: SpineConfig {
                url: "http://localhost:3100".into(),
            },
            logging: LoggingConfig {
                level: "info".into(),
            },
        }
    }
}

impl Config {
    pub fn config_path() -> PathBuf {
        agent_body_core::config_path()
    }

    pub fn broker_store_dir(&self) -> PathBuf {
        self.nats
            .store_dir
            .as_ref()
            .map(PathBuf::from)
            .unwrap_or_else(agent_body_core::broker_dir)
    }

    pub fn load() -> Result<Self> {
        agent_body_core::organ_config::load("nerves")
    }
}
