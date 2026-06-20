use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use std::sync::Arc;

use crate::config::Config;
use crate::jetstream;
use crate::nats;
use crate::spine::SpineClient;

pub struct AppState {
    pub config: Config,
    pub spine: SpineClient,
    pub jetstream_ready: bool,
}

pub async fn start(config: Config) -> anyhow::Result<()> {
    tracing::info!("Starting agent-nerves daemon...");
    let broker_dir = config.broker_store_dir();
    std::fs::create_dir_all(&broker_dir)?;
    tracing::info!("Broker store dir: {}", broker_dir.display());

    if config.nats.embedded {
        let _embedded = crate::broker::ensure_embedded_broker(
            &broker_dir,
            &config.nats.url,
            Some(&config.cluster),
        )
        .await?;
    }

    let jetstream_ready = match nats::connect(&config.nats.url).await {
        Ok(client) => jetstream::ensure_autonomic_stream(&client).await.is_ok(),
        Err(e) => {
            tracing::warn!("NATS connect failed: {e}");
            false
        }
    };

    let spine = SpineClient::new(&config.spine.url, "agent-nerves", env!("CARGO_PKG_VERSION"));
    spine.register().await?;
    let spine_clone = spine.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(30)).await;
            let _ = spine_clone.heartbeat().await;
        }
    });
    let port = config.server.port;
    let state = Arc::new(AppState {
        config,
        spine,
        jetstream_ready,
    });
    let app = Router::new()
        .route("/health", get(health))
        .route("/nats/ping", post(nats_ping))
        .route("/jetstream/status", get(jetstream_status))
        .route("/cluster/status", get(cluster_status))
        .route("/filter/test", post(filter_test))
        .route("/filter/rules", get(filter_rules))
        .with_state(state);
    let addr = format!("0.0.0.0:{}", port);
    tracing::info!("HTTP server listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn health(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "broker_dir": state.config.broker_store_dir().display().to_string(),
        "jetstream_ready": state.jetstream_ready,
    }))
}

async fn jetstream_status(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "stream": agent_body_core::STREAM_NAME,
        "subjects": agent_body_core::STREAM_SUBJECT_WILDCARD,
        "broker_dir": state.config.broker_store_dir().display().to_string(),
        "ready": state.jetstream_ready,
        "ack_wait_secs": agent_body_core::default_ack_wait().as_secs(),
        "duplicate_window_secs": agent_body_core::default_duplicate_window().as_secs(),
    }))
}

async fn nats_ping(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    match nats::ping(&state.config).await {
        Ok(info) => Json(serde_json::to_value(&info).unwrap_or_default()),
        Err(e) => Json(serde_json::json!({"connected": false, "error": e.to_string()})),
    }
}

async fn cluster_status(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    match crate::cluster::cluster_status(&state.config.cluster).await {
        Ok(status) => Json(serde_json::json!({ "ok": true, "cluster": status })),
        Err(e) => Json(serde_json::json!({ "ok": false, "error": e.to_string() })),
    }
}

#[derive(serde::Deserialize)]
struct FilterTestRequest {
    subject: String,
    payload: String,
}

async fn filter_test(
    State(state): State<Arc<AppState>>,
    Json(req): Json<FilterTestRequest>,
) -> Json<serde_json::Value> {
    let dir = state
        .config
        .filters
        .directory
        .as_deref()
        .map(std::path::Path::new);
    let filters_dir = crate::filter::filters_dir(dir);
    match crate::filter::evaluate_event(&filters_dir, &req.subject, req.payload.as_bytes()) {
        Ok(decision) => Json(serde_json::json!({ "ok": true, "decision": decision })),
        Err(e) => Json(serde_json::json!({ "ok": false, "error": e.to_string() })),
    }
}

async fn filter_rules(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let dir = state
        .config
        .filters
        .directory
        .as_deref()
        .map(std::path::Path::new);
    let filters_dir = crate::filter::filters_dir(dir);
    match crate::filter::load_rules(&filters_dir) {
        Ok(rules) => Json(
            serde_json::json!({ "ok": true, "rules": rules, "directory": filters_dir.display().to_string() }),
        ),
        Err(e) => Json(serde_json::json!({ "ok": false, "error": e.to_string() })),
    }
}
