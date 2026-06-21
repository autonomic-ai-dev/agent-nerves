use anyhow::{Context, Result};
use async_nats::jetstream::{self, consumer::DeliverPolicy};
use futures::StreamExt;

use crate::jetstream::ensure_autonomic_stream;
use agent_body_core::STREAM_NAME;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TailFrom {
    New,
    All,
}

impl TailFrom {
    pub fn parse(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "new" => Ok(Self::New),
            "all" | "start" | "beginning" => Ok(Self::All),
            other => anyhow::bail!("invalid --from value '{other}' (use 'new' or 'all')"),
        }
    }
}

pub async fn tail_stream(
    url: &str,
    subject: &str,
    raw: bool,
    from: TailFrom,
    jetstream_only: bool,
    filters_dir: Option<&std::path::Path>,
) -> Result<()> {
    if jetstream_only {
        return tail_jetstream(url, subject, raw, from, filters_dir).await;
    }

    match tail_jetstream(url, subject, raw, from, filters_dir).await {
        Ok(()) => Ok(()),
        Err(e) => {
            eprintln!("JetStream tail unavailable ({e}); falling back to core NATS subscribe.");
            tail_core_nats(url, subject, raw, filters_dir).await
        }
    }
}

async fn tail_jetstream(
    url: &str,
    subject: &str,
    raw: bool,
    from: TailFrom,
    filters_dir: Option<&std::path::Path>,
) -> Result<()> {
    let client = agent_body_core::connect_nats()
        .await
        .with_context(|| format!("connect to NATS at {url}"))?;
    let js = ensure_autonomic_stream(&client).await?;
    let mut stream = js
        .get_stream(STREAM_NAME)
        .await
        .context("get AUTONOMIC JetStream stream")?;
    let info = stream.info().await.context("read stream info")?;

    println!("JetStream stream: {}", STREAM_NAME);
    println!(
        "  messages: {}  bytes: {}  consumers: {}",
        info.state.messages, info.state.bytes, info.state.consumer_count
    );
    println!(
        "  tail subject: '{}'  from: {}  (Ctrl+C to stop)",
        subject,
        if from == TailFrom::All { "all" } else { "new" }
    );
    println!("---");

    let consumer = stream
        .create_consumer(jetstream::consumer::pull::Config {
            filter_subject: subject.to_string(),
            deliver_policy: if from == TailFrom::All {
                DeliverPolicy::All
            } else {
                DeliverPolicy::New
            },
            ..Default::default()
        })
        .await
        .context("create ephemeral JetStream tail consumer")?;

    let mut messages = consumer
        .messages()
        .await
        .context("subscribe to JetStream tail consumer")?;

    while let Some(msg) = messages.next().await {
        let msg = msg.context("JetStream tail message")?;
        if should_drop(filters_dir, &msg.subject, &msg.payload)? {
            let _ = msg.ack().await;
            continue;
        }
        let sequence = msg.info().ok().map(|info| info.stream_sequence);
        print_message(&msg.subject, &msg.payload, raw, sequence);
        if let Err(e) = msg.ack().await {
            eprintln!("ack warning: {e}");
        }
    }

    Ok(())
}

async fn tail_core_nats(
    url: &str,
    subject: &str,
    raw: bool,
    filters_dir: Option<&std::path::Path>,
) -> Result<()> {
    let client = agent_body_core::connect_nats()
        .await
        .with_context(|| format!("connect to NATS at {url}"))?;
    println!("Core NATS subscribe on '{subject}' (Ctrl+C to stop).");
    println!("---");

    let mut subscription = client.subscribe(subject.to_string()).await?;
    while let Some(msg) = subscription.next().await {
        if should_drop(filters_dir, &msg.subject, &msg.payload)? {
            continue;
        }
        print_message(&msg.subject, &msg.payload, raw, None);
    }
    Ok(())
}

fn should_drop(
    filters_dir: Option<&std::path::Path>,
    subject: &str,
    payload: &[u8],
) -> Result<bool> {
    let Some(dir) = filters_dir else {
        return Ok(false);
    };
    let decision = crate::filter::evaluate_event(dir, subject, payload)?;
    if !decision.allowed {
        eprintln!(
            "filtered [{subject}] rule={:?} reason={}",
            decision.matched_rule, decision.reason
        );
        return Ok(true);
    }
    Ok(false)
}

fn print_message(subject: &str, payload: &[u8], raw: bool, sequence: Option<u64>) {
    if raw {
        if let Some(seq) = sequence {
            println!(
                "[seq={seq}] [{subject}] {}",
                String::from_utf8_lossy(payload)
            );
        } else {
            println!("[{subject}] {}", String::from_utf8_lossy(payload));
        }
        return;
    }

    let timestamp = chrono::Utc::now().format("%H:%M:%S%.3f");
    let payload_str = String::from_utf8_lossy(payload);
    let pretty = serde_json::from_str::<serde_json::Value>(&payload_str)
        .map(|v| serde_json::to_string_pretty(&v).unwrap_or_else(|_| payload_str.to_string()))
        .unwrap_or_else(|_| payload_str.to_string());

    if let Some(seq) = sequence {
        println!("\n[{timestamp}] seq={seq} Subject: {subject}");
    } else {
        println!("\n[{timestamp}] Subject: {subject}");
    }
    for line in pretty.lines() {
        println!("  {line}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tail_from_parses_aliases() {
        assert_eq!(TailFrom::parse("new").unwrap(), TailFrom::New);
        assert_eq!(TailFrom::parse("ALL").unwrap(), TailFrom::All);
        assert_eq!(TailFrom::parse("beginning").unwrap(), TailFrom::All);
        assert!(TailFrom::parse("invalid").is_err());
    }
}
