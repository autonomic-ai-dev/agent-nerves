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

pub async fn ping(config: &Config) -> Result<NatsInfo> {
    let url = &config.nats.url;

    match async_nats::connect(url).await {
        Ok(client) => {
            let server_id = Some(client.server_info().server_id.clone());
            drop(client);
            Ok(NatsInfo {
                connected: true,
                url: url.clone(),
                server_id,
                error: None,
            })
        }
        Err(e) => Ok(NatsInfo {
            connected: false,
            url: url.clone(),
            server_id: None,
            error: Some(e.to_string()),
        }),
    }
}
