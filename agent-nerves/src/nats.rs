use anyhow::Result;
use serde::Serialize;

use crate::config::Config;

#[derive(Debug, Serialize)]
pub struct NatsInfo {
    pub connected: bool,
    pub url: String,
    pub server_id: Option<String>,
    pub error: Option<String>,
}

pub async fn connect(url: &str) -> Result<async_nats::Client> {
    Ok(async_nats::connect(url).await?)
}

pub async fn ping(config: &Config) -> Result<NatsInfo> {
    ping_url(&config.nats.url).await
}

pub async fn ping_url(url: &str) -> Result<NatsInfo> {
    match async_nats::connect(url).await {
        Ok(client) => {
            let server_id = Some(client.server_info().server_id.clone());
            drop(client);
            Ok(NatsInfo {
                connected: true,
                url: url.to_string(),
                server_id,
                error: None,
            })
        }
        Err(e) => Ok(NatsInfo {
            connected: false,
            url: url.to_string(),
            server_id: None,
            error: Some(e.to_string()),
        }),
    }
}
