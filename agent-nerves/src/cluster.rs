//! Multi-node cluster coordination and NATS route config generation.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_node_id")]
    pub node_id: String,
    #[serde(default)]
    pub wireguard_interface: Option<String>,
    #[serde(default)]
    pub leader_id: Option<String>,
    #[serde(default = "default_cluster_port")]
    pub cluster_port: u16,
    #[serde(default)]
    pub peers: Vec<ClusterPeer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterPeer {
    pub id: String,
    pub nats_url: String,
    #[serde(default)]
    pub route_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterState {
    pub term: u64,
    pub leader_id: String,
    pub node_id: String,
    pub initialized_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterStatus {
    pub enabled: bool,
    pub node_id: String,
    pub leader_id: String,
    pub term: u64,
    pub wireguard_interface: Option<String>,
    pub wireguard_up: bool,
    pub peers: Vec<PeerStatus>,
    pub state_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerStatus {
    pub id: String,
    pub nats_url: String,
    pub reachable: bool,
}

fn default_node_id() -> String {
    std::env::var("AUTONOMIC_NERVES_NODE_ID").unwrap_or_else(|_| "nerves-local".into())
}

fn default_cluster_port() -> u16 {
    6222
}

impl Default for ClusterConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            node_id: default_node_id(),
            wireguard_interface: None,
            leader_id: None,
            cluster_port: 6222,
            peers: Vec::new(),
        }
    }
}

pub fn state_path() -> PathBuf {
    agent_body_core::organ_state_dir("nerves").join("cluster.json")
}

pub fn init_cluster(config: &ClusterConfig) -> Result<ClusterState> {
    anyhow::ensure!(config.enabled, "cluster.enabled must be true in config");

    let leader = config
        .leader_id
        .clone()
        .unwrap_or_else(|| elect_leader_id(config));

    let state = ClusterState {
        term: 1,
        leader_id: leader,
        node_id: config.node_id.clone(),
        initialized_at: chrono::Utc::now().to_rfc3339(),
    };

    let path = state_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, serde_json::to_string_pretty(&state)?)?;
    Ok(state)
}

pub fn load_state() -> Result<Option<ClusterState>> {
    let path = state_path();
    if !path.exists() {
        return Ok(None);
    }
    let raw = std::fs::read_to_string(&path)?;
    Ok(Some(serde_json::from_str(&raw)?))
}

pub async fn cluster_status(config: &ClusterConfig) -> Result<ClusterStatus> {
    let state = load_state()?;
    let (term, leader_id) = state
        .as_ref()
        .map(|s| (s.term, s.leader_id.clone()))
        .unwrap_or_else(|| {
            (
                0,
                config
                    .leader_id
                    .clone()
                    .unwrap_or_else(|| config.node_id.clone()),
            )
        });

    let mut peers = Vec::new();
    for peer in &config.peers {
        let reachable = crate::nats::ping_url(&peer.nats_url)
            .await
            .map(|info| info.connected)
            .unwrap_or(false);
        peers.push(PeerStatus {
            id: peer.id.clone(),
            nats_url: peer.nats_url.clone(),
            reachable,
        });
    }

    Ok(ClusterStatus {
        enabled: config.enabled,
        node_id: config.node_id.clone(),
        leader_id,
        term,
        wireguard_interface: config.wireguard_interface.clone(),
        wireguard_up: wireguard_up(config.wireguard_interface.as_deref()),
        peers,
        state_path: state_path().display().to_string(),
    })
}

pub fn render_nats_cluster_config(
    config: &ClusterConfig,
    store_dir: &Path,
    client_port: u16,
) -> Result<String> {
    anyhow::ensure!(config.enabled, "cluster is disabled");

    let routes: Vec<String> = config
        .peers
        .iter()
        .filter_map(|p| p.route_url.clone().or_else(|| default_route_url(&p.nats_url)))
        .map(|r| format!("  nats-route://{r}"))
        .collect();

    let routes_block = if routes.is_empty() {
        String::new()
    } else {
        format!(
            "cluster {{\n  name: autonomic\n  listen: 0.0.0.0:{}\n  routes = [\n{}\n  ]\n}}\n",
            config.cluster_port,
            routes.join("\n")
        )
    };

    Ok(format!(
        "port: {client_port}\n\
         jetstream {{\n  store_dir: \"{}\"\n}}\n\
         {routes_block}",
        store_dir.display()
    ))
}

pub fn write_nats_cluster_config(
    config: &ClusterConfig,
    store_dir: &Path,
    client_port: u16,
) -> Result<PathBuf> {
    let body = render_nats_cluster_config(config, store_dir, client_port)?;
    let path = agent_body_core::organ_state_dir("nerves").join("nats-cluster.conf");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, body)?;
    Ok(path)
}

fn elect_leader_id(config: &ClusterConfig) -> String {
    let mut ids: Vec<String> = std::iter::once(config.node_id.clone())
        .chain(config.peers.iter().map(|p| p.id.clone()))
        .collect();
    ids.sort();
    ids.into_iter().next().unwrap_or_else(|| config.node_id.clone())
}

fn default_route_url(nats_url: &str) -> Option<String> {
    let host = nats_url
        .trim_start_matches("nats://")
        .split(':')
        .next()?;
    Some(format!("{host}:6222"))
}

fn wireguard_up(interface: Option<&str>) -> bool {
    let Some(iface) = interface else {
        return true;
    };
    if iface.is_empty() {
        return true;
    }
    std::process::Command::new("sh")
        .arg("-c")
        .arg(format!("ip link show {iface} 2>/dev/null || ifconfig {iface} 2>/dev/null"))
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_cluster_routes() {
        let config = ClusterConfig {
            enabled: true,
            node_id: "n1".into(),
            peers: vec![ClusterPeer {
                id: "n2".into(),
                nats_url: "nats://10.0.0.2:4222".into(),
                route_url: None,
            }],
            ..Default::default()
        };
        let conf = render_nats_cluster_config(&config, Path::new("/tmp/broker"), 4222).unwrap();
        assert!(conf.contains("cluster"));
        assert!(conf.contains("10.0.0.2:6222"));
    }
}
