use rmcp::model::{CallToolResult, Content, ErrorData as McpError, ServerInfo};
use rmcp::serve_server;
use rmcp::tool;
use rmcp::ServerHandler;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::config::Config;

#[derive(Clone)]
pub struct NervesMcp {
    config: Config,
}

impl NervesMcp {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub async fn run(config: Config) -> anyhow::Result<()> {
        let server = Self::new(config);
        let service = serve_server(server, rmcp::transport::io::stdio()).await?;
        service.waiting().await?;
        Ok(())
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
struct FilterTestParams {
    subject: String,
    payload: String,
}

#[tool(tool_box)]
impl NervesMcp {
    #[tool(description = "Ping the NATS broker and return connection info")]
    async fn nerves_ping(&self) -> Result<CallToolResult, McpError> {
        match crate::nats::ping(&self.config).await {
            Ok(info) => {
                let text = serde_json::to_string_pretty(&info).unwrap_or_else(|_| "{}".to_string());
                Ok(CallToolResult::success(vec![Content::text(text)]))
            }
            Err(e) => Err(McpError::internal_error(format!("{e}"), None)),
        }
    }

    #[tool(description = "Return agent-nerves daemon health status")]
    async fn nerves_health(&self) -> Result<CallToolResult, McpError> {
        let now = chrono::Utc::now().to_rfc3339();
        let jetstream_ready = match crate::nats::connect(&self.config.nats.url).await {
            Ok(client) => crate::jetstream::ensure_autonomic_stream(&client).await.is_ok(),
            Err(_) => false,
        };
        let status = serde_json::json!({
            "status": "ok",
            "version": env!("CARGO_PKG_VERSION"),
            "broker_dir": self.config.broker_store_dir().display().to_string(),
            "jetstream_ready": jetstream_ready,
            "checked_at": now,
        });
        let text = serde_json::to_string_pretty(&status).unwrap_or_else(|_| "{}".to_string());
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }

    #[tool(description = "Show JetStream stream configuration and readiness")]
    async fn nerves_jetstream_status(&self) -> Result<CallToolResult, McpError> {
        let status = serde_json::json!({
            "stream": agent_body_core::STREAM_NAME,
            "subjects": agent_body_core::STREAM_SUBJECT_WILDCARD,
            "broker_dir": self.config.broker_store_dir().display().to_string(),
            "ack_wait_secs": agent_body_core::default_ack_wait().as_secs(),
            "duplicate_window_secs": agent_body_core::default_duplicate_window().as_secs(),
        });
        let text = serde_json::to_string_pretty(&status).unwrap_or_else(|_| "{}".to_string());
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }

    #[tool(description = "Show multi-node cluster status")]
    async fn nerves_cluster_status(&self) -> Result<CallToolResult, McpError> {
        match crate::cluster::cluster_status(&self.config.cluster).await {
            Ok(status) => {
                let text =
                    serde_json::to_string_pretty(&status).unwrap_or_else(|_| "{}".to_string());
                Ok(CallToolResult::success(vec![Content::text(text)]))
            }
            Err(e) => Err(McpError::internal_error(format!("{e}"), None)),
        }
    }

    #[tool(description = "List loaded event filter rules")]
    async fn nerves_filter_list(&self) -> Result<CallToolResult, McpError> {
        let filters_dir = crate::filter::filters_dir(
            self.config
                .filters
                .directory
                .as_deref()
                .map(std::path::Path::new),
        );
        match crate::filter::load_rules(&filters_dir) {
            Ok(rules) => {
                let result = serde_json::json!({
                    "directory": filters_dir.display().to_string(),
                    "rules": rules,
                });
                let text =
                    serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string());
                Ok(CallToolResult::success(vec![Content::text(text)]))
            }
            Err(e) => Err(McpError::internal_error(format!("{e}"), None)),
        }
    }

    #[tool(description = "Test a subject/payload against loaded filter rules")]
    async fn nerves_filter_test(
        &self,
        #[tool(aggr)] params: FilterTestParams,
    ) -> Result<CallToolResult, McpError> {
        let filters_dir = crate::filter::filters_dir(
            self.config
                .filters
                .directory
                .as_deref()
                .map(std::path::Path::new),
        );
        match crate::filter::evaluate_event(&filters_dir, &params.subject, params.payload.as_bytes())
        {
            Ok(decision) => {
                let text =
                    serde_json::to_string_pretty(&decision).unwrap_or_else(|_| "{}".to_string());
                Ok(CallToolResult::success(vec![Content::text(text)]))
            }
            Err(e) => Err(McpError::internal_error(format!("{e}"), None)),
        }
    }
}

#[tool(tool_box)]
impl ServerHandler for NervesMcp {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "NATS/JetStream event bridge for autonomic agents. Tools: nerves_ping (ping NATS), nerves_health (daemon health), nerves_jetstream_status (stream config), nerves_cluster_status (cluster state), nerves_filter_list (loaded rules), nerves_filter_test (evaluate event)."
                    .into(),
            ),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nerves_mcp_implements_server_handler() {
        fn assert_handler<T: rmcp::ServerHandler>() {}
        assert_handler::<NervesMcp>();
    }
}
