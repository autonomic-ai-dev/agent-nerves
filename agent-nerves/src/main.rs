use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "agent-nerves", about = "Distributed event bus for autonomic agents")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start NATS broker health API (daemon)
    Serve,
    /// Ping the NATS server
    Ping,
    /// Show configuration and status
    Status,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();
    let config = agent_nerves::config::Config::load()?;

    match cli.command {
        Commands::Serve => {
            println!("agent-nerves serve (not yet implemented)");
            println!("  config: {}", agent_nerves::config::Config::config_path().display());
        }
        Commands::Ping => {
            match agent_nerves::nats::ping(&config).await {
                Ok(info) => {
                    let json = serde_json::to_string_pretty(&info)?;
                    println!("{}", json);
                }
                Err(e) => {
                    eprintln!("NATS ping failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Status => {
            println!("agent-nerves status");
            println!("  config: {}", agent_nerves::config::Config::config_path().display());
            println!("  nats_url: {}", config.nats.url);
        }
    }
    Ok(())
}
