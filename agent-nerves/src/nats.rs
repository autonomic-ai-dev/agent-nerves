use std::collections::HashMap;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::config::Config;

const ORGAN: &str = "nerves";
const ORGAN_USER_ENV: &str = "AUTONOMIC_NATS_USER";

#[derive(Debug, Serialize)]
pub struct NatsInfo {
    pub connected: bool,
    pub url: String,
    pub server_id: Option<String>,
    pub error: Option<String>,
}

#[derive(Deserialize)]
struct BrokerFile {
    organs: HashMap<String, OrganCredentials>,
}

#[derive(Deserialize)]
struct OrganCredentials {
    user: String,
    password: String,
}

fn organ_credentials(raw: &[u8], organ: &str) -> Result<Option<(String, String)>> {
    let file: BrokerFile = serde_json::from_slice(raw)?;
    Ok(file
        .organs
        .get(organ)
        .map(|creds| (creds.user.clone(), creds.password.clone())))
}

fn broker_credentials(organ: &str) -> Result<Option<(String, String)>> {
    let path = agent_body_core::broker_dir().join("creds.json");
    if !path.exists() {
        return Ok(None);
    }
    organ_credentials(&std::fs::read(&path)?, organ)
}

pub async fn connect(url: &str) -> Result<async_nats::Client> {
    let env_user_set = std::env::var(ORGAN_USER_ENV).is_ok_and(|user| !user.is_empty());
    if env_user_set {
        return Ok(agent_body_core::connect_nats().await?);
    }
    let options = match broker_credentials(ORGAN)? {
        Some((user, password)) => {
            async_nats::ConnectOptions::new().user_and_password(user, password)
        }
        None => async_nats::ConnectOptions::new(),
    };
    Ok(options.connect(url).await?)
}

pub async fn ping(config: &Config) -> Result<NatsInfo> {
    ping_url(&config.nats.url).await
}

pub async fn ping_url(url: &str) -> Result<NatsInfo> {
    match connect(url).await {
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

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"{"version":1,"port":4222,"organs":{
        "nerves":{"user":"nerves-user","password":"not-a-real-secret","publish":[],"subscribe":[]},
        "heart":{"user":"heart-user","password":"x","publish":[],"subscribe":[]}}}"#;

    #[test]
    fn finds_the_named_organ_credentials() {
        let creds = organ_credentials(SAMPLE.as_bytes(), "nerves").unwrap();
        assert_eq!(
            creds,
            Some(("nerves-user".to_string(), "not-a-real-secret".to_string()))
        );
    }

    #[test]
    fn unknown_organ_has_no_credentials() {
        assert_eq!(
            organ_credentials(SAMPLE.as_bytes(), "muscle").unwrap(),
            None
        );
    }

    #[test]
    fn malformed_broker_file_is_an_error() {
        assert!(organ_credentials(b"{not json", "nerves").is_err());
    }
}
