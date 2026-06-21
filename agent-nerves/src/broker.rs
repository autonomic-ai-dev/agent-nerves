use std::path::Path;
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;
use tracing::{info, warn};

/// Start an embedded `nats-server` with JetStream persistence when none is reachable.
pub async fn ensure_embedded_broker(
    store_dir: &Path,
    url: &str,
    cluster_config: Option<&crate::cluster::ClusterConfig>,
) -> anyhow::Result<Option<tokio::process::Child>> {
    if ping_nats().await {
        info!("NATS already reachable");
        return Ok(None);
    }

    let port = parse_port(url).unwrap_or(4222);
    info!(
        store = %store_dir.display(),
        port,
        "starting embedded nats-server (-js)"
    );

    let secure_conf = agent_body_core::server_config_path();
    let mut cmd = Command::new("nats-server");

    if secure_conf.exists() && !agent_body_core::nats_insecure_mode() {
        info!("using secure NATS config {}", secure_conf.display());
        cmd.arg("-c").arg(secure_conf);
    } else if let Some(cluster) = cluster_config.filter(|c| c.enabled) {
        if let Ok(conf_path) = crate::cluster::write_nats_cluster_config(cluster, store_dir, port) {
            info!("using NATS cluster config {}", conf_path.display());
            cmd.arg("-c").arg(conf_path);
        } else {
            cmd.arg("-js")
                .arg("-sd")
                .arg(store_dir)
                .arg("-p")
                .arg(port.to_string());
        }
    } else {
        cmd.arg("-js")
            .arg("-sd")
            .arg(store_dir)
            .arg("-p")
            .arg(port.to_string());
    }

    cmd.stdout(Stdio::null()).stderr(Stdio::null());

    let child = cmd
        .spawn()
        .map_err(|e| anyhow::anyhow!("failed to spawn nats-server: {e}"))?;

    for _ in 0..20 {
        tokio::time::sleep(Duration::from_millis(250)).await;
        if ping_nats().await {
            info!("embedded nats-server is ready");
            return Ok(Some(child));
        }
    }

    warn!("embedded nats-server did not become ready in time");
    Ok(Some(child))
}

async fn ping_nats() -> bool {
    agent_body_core::ping_nats().await
}

fn parse_port(url: &str) -> Option<u16> {
    url.rsplit(':').next()?.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_port_from_nats_url() {
        assert_eq!(parse_port("nats://localhost:4222"), Some(4222));
    }
}
