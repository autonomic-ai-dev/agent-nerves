use anyhow::{Context, Result};
use async_nats::jetstream::{self, consumer::AckPolicy, stream::StorageType};
use async_nats::Client;
use std::time::Duration;
use tracing::{info, warn};

pub use agent_body_core::{
    default_ack_wait, default_duplicate_window, STREAM_NAME, STREAM_SUBJECT_WILDCARD,
};

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
    info!(consumer = durable_name, subject = filter_subject, "jetstream consumer ready");
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stream_constants_are_stable() {
        assert_eq!(STREAM_NAME, "AUTONOMIC");
        assert!(STREAM_SUBJECT_WILDCARD.contains("autonomic"));
    }
}
