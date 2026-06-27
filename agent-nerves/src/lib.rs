pub mod broker;
pub mod cluster;
pub mod config;
pub mod filter;
pub mod jetstream;
pub mod log;
pub mod mcp_server;
pub mod nats;
pub mod serve;
pub mod spine;
pub mod stream;
pub mod update;
pub mod wasm_filter;

use anyhow::Result;
use config::Config;

pub async fn serve_mcp(config: Config) -> Result<()> {
    mcp_server::NervesMcp::run(config).await?;
    Ok(())
}
