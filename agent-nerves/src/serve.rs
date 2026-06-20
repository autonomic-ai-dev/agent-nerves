use axum::{Json, Router, extract::State, routing::{get, post}};
use std::sync::Arc;

use crate::config::Config;
use crate::nats;
use crate::spine::SpineClient;

pub struct AppState {
    pub config: Config,
    pub spine: SpineClient,
}

pub async fn start(config: Config) -> anyhow::Result<()> {
    tracing::info!("Starting agent-nerves daemon...");
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
    let state = Arc::new(AppState { config, spine });
    let app = Router::new()
        .route("/health", get(health))
        .route("/nats/ping", post(nats_ping))
        .with_state(state);
    let addr = format!("0.0.0.0:{}", port);
    tracing::info!("HTTP server listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn health(State(_): State<Arc<AppState>>) -> Json<serde_json::Value> {
    Json(serde_json::json!({"status": "ok"}))
}

async fn nats_ping(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    match nats::ping(&state.config).await {
        Ok(info) => Json(serde_json::to_value(&info).unwrap_or_default()),
        Err(e) => Json(serde_json::json!({"connected": false, "error": e.to_string()})),
    }
}
