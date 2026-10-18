use anyhow::{Context, Result};
use async_nats::jetstream::{self, consumer::AckPolicy, stream::StorageType};
use async_nats::Client;
use std::time::Duration;
use tracing::{info, warn};

pub use agent_body_core::{
    default_ack_wait, default_duplicate_window, STREAM_NAME, STREAM_SUBJECT_WILDCARD,
};

pub const USAGE_STREAM_NAME: &str = "AUTONOMIC_USAGE";
pub const USAGE_SUBJECT_WILDCARD: &str = "events.heart.>";
pub const USAGE_SUBJECT_EVENT: &str = "events.heart.usage";
pub const USAGE_SUBJECT_QUOTA: &str = "events.heart.quota";
pub const USAGE_SUBJECT_ALERT: &str = "events.heart.budget.alert";

/// Ensure the AUTONOMIC JetStream stream exists with durable file storage settings.
///
/// NATS server must be started with JetStream enabled, e.g.
/// `nats-server -js -sd ~/.autonomic/broker`
pub async fn ensure_autonomic_stream(client: &Client) -> Result<jetstream::Context> {
    let js = jetstream::new(client.clone());

    match js
        .get_or_create_stream(jetstream::stream::Config {
            name: STREAM_NAME.to_string(),
            subjects: vec![STREAM_SUBJECT_WILDCARD.to_string()],
            storage: StorageType::File,
            duplicate_window: default_duplicate_window(),
            max_age: Duration::from_secs(7 * 24 * 3600),
            ..Default::default()
        })
        .await
    {
        Ok(stream) => {
            info!(stream = %stream.cached_info().config.name, "jetstream stream ready");
            Ok(js)
        }
        Err(e) => {
            warn!(error = %e, "jetstream unavailable — is nats-server running with -js?");
            Err(e).context("ensure AUTONOMIC jetstream stream")
        }
    }
}

/// Ensure the AUTONOMIC_USAGE JetStream stream exists with 14-day file retention.
pub async fn ensure_usage_stream(client: &Client) -> Result<jetstream::Context> {
    let js = jetstream::new(client.clone());

    match js
        .get_or_create_stream(jetstream::stream::Config {
            name: USAGE_STREAM_NAME.to_string(),
            subjects: vec![USAGE_SUBJECT_WILDCARD.to_string()],
            storage: StorageType::File,
            duplicate_window: default_duplicate_window(),
            max_age: Duration::from_secs(14 * 24 * 3600),
            ..Default::default()
        })
        .await
    {
        Ok(stream) => {
            info!(stream = %stream.cached_info().config.name, "usage jetstream stream ready");
            Ok(js)
        }
        Err(e) => {
            warn!(error = %e, "usage jetstream unavailable — is nats-server running with -js?");
            Err(e).context("ensure AUTONOMIC_USAGE jetstream stream")
        }
    }
}

/// Create or update a durable pull consumer with explicit ACK and redelivery.
pub async fn ensure_durable_consumer(
    js: &jetstream::Context,
    durable_name: &str,
    filter_subject: &str,
) -> Result<()> {
    js.create_consumer_on_stream(
        jetstream::consumer::pull::Config {
            durable_name: Some(durable_name.to_string()),
            filter_subject: filter_subject.to_string(),
            ack_policy: AckPolicy::Explicit,
            ack_wait: default_ack_wait(),
            ..Default::default()
        },
        STREAM_NAME,
    )
    .await
    .with_context(|| format!("create durable consumer {durable_name}"))?;
    info!(
        consumer = durable_name,
        subject = filter_subject,
        "jetstream consumer ready"
    );
    Ok(())
}

/// Publish bytes to JetStream with Nats-Msg-Id deduplication.
pub async fn publish_dedup(
    js: &jetstream::Context,
    subject: &str,
    msg_id: &str,
    payload: &[u8],
) -> Result<()> {
    let mut headers = async_nats::HeaderMap::new();
    headers.insert("Nats-Msg-Id", msg_id);
    js.publish_with_headers(subject.to_string(), headers, payload.to_vec().into())
        .await?
        .await
        .context("jetstream publish ack")?;
    Ok(())
}

/// Publish a model turn usage event to JetStream with deduplication.
pub async fn publish_usage_event(
    js: &jetstream::Context,
    event_id: &str,
    payload: &[u8],
) -> Result<()> {
    publish_dedup(js, USAGE_SUBJECT_EVENT, event_id, payload).await
}

/// Publish a subscription quota snapshot event to JetStream with deduplication.
pub async fn publish_quota_event(
    js: &jetstream::Context,
    quota_id: &str,
    payload: &[u8],
) -> Result<()> {
    publish_dedup(js, USAGE_SUBJECT_QUOTA, quota_id, payload).await
}

/// Publish a budget alert event to JetStream with deduplication.
pub async fn publish_budget_alert(
    js: &jetstream::Context,
    alert_id: &str,
    payload: &[u8],
) -> Result<()> {
    publish_dedup(js, USAGE_SUBJECT_ALERT, alert_id, payload).await
}

/// Cloud JetStream leaf bridge targeting telemetry events.
pub const CLOUD_BRIDGE_SUBJECTS: &[&str] = &[USAGE_SUBJECT_EVENT, USAGE_SUBJECT_QUOTA];

/// Check if a subject is targeted by the cloud telemetry leaf bridge.
#[must_use]
pub fn is_cloud_bridge_target(subject: &str) -> bool {
    CLOUD_BRIDGE_SUBJECTS.contains(&subject)
}

/// Options for configuring the remote cloud NATS connection.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CloudBridgeOptions {
    pub url: String,
    pub creds_file: Option<String>,
    pub jwt: Option<String>,
    pub nkey_seed: Option<String>,
    pub user: Option<String>,
    pub password: Option<String>,
}

impl CloudBridgeOptions {
    #[must_use]
    pub fn from_config(cfg: &crate::config::CloudBrokerConfig) -> Self {
        Self {
            url: cfg.url.clone(),
            creds_file: cfg.creds_file.clone(),
            jwt: cfg.jwt.clone(),
            nkey_seed: cfg.nkey_seed.clone(),
            user: cfg.user.clone(),
            password: cfg.password.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stream_constants_are_stable() {
        assert_eq!(STREAM_NAME, "AUTONOMIC");
        assert!(STREAM_SUBJECT_WILDCARD.contains("autonomic"));
    }

    #[test]
    fn usage_stream_constants_are_stable() {
        assert_eq!(USAGE_STREAM_NAME, "AUTONOMIC_USAGE");
        assert_eq!(USAGE_SUBJECT_WILDCARD, "events.heart.>");
        assert_eq!(USAGE_SUBJECT_EVENT, "events.heart.usage");
        assert_eq!(USAGE_SUBJECT_QUOTA, "events.heart.quota");
        assert_eq!(USAGE_SUBJECT_ALERT, "events.heart.budget.alert");
    }

    #[test]
    fn test_cloud_bridge_targets_usage_and_quota_events() {
        assert!(is_cloud_bridge_target("events.heart.usage"));
        assert!(is_cloud_bridge_target("events.heart.quota"));
        assert!(!is_cloud_bridge_target("events.heart.budget.alert"));
        assert!(!is_cloud_bridge_target("autonomic.spine.run"));
    }

    #[test]
    fn test_cloud_bridge_options_from_config() {
        let cfg = crate::config::CloudBrokerConfig {
            enabled: true,
            url: "tls://connect.ngs.global:4222".into(),
            creds_file: Some("/path/to/ngs.creds".into()),
            jwt: Some("jwt-token".into()),
            nkey_seed: Some("seed".into()),
            user: None,
            password: None,
            leaf_node: true,
        };

        let opts = CloudBridgeOptions::from_config(&cfg);
        assert_eq!(opts.url, "tls://connect.ngs.global:4222");
        assert_eq!(opts.creds_file.as_deref(), Some("/path/to/ngs.creds"));
        assert_eq!(opts.jwt.as_deref(), Some("jwt-token"));
    }
}
