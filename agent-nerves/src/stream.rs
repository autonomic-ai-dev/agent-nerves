use anyhow::Result;
use futures::StreamExt;

pub async fn tail_stream(url: &str, subject: &str, raw: bool) -> Result<()> {
    let client = async_nats::connect(url).await?;
    println!("Connected to NATS at {}", url);
    println!("Subscribing to '{}'. Press Ctrl+C to stop.", subject);
    println!("---");

    let mut subscription = client.subscribe(subject.to_string()).await?;
    while let Some(msg) = subscription.next().await {
        if raw {
            println!("[{}] {}", msg.subject, String::from_utf8_lossy(&msg.payload));
        } else {
            let timestamp = chrono::Utc::now().format("%H:%M:%S%.3f");
            let payload_str = String::from_utf8_lossy(&msg.payload);
            let pretty = serde_json::from_str::<serde_json::Value>(&payload_str)
                .map(|v| serde_json::to_string_pretty(&v).unwrap_or_else(|_| payload_str.to_string()))
                .unwrap_or_else(|_| payload_str.to_string());
            println!("\n[{}] Subject: {}", timestamp, msg.subject);
            for line in pretty.lines() {
                println!("  {}", line);
            }
        }
    }
    Ok(())
}
