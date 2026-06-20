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
    /// Tail NATS messages from a subject
    #[command(name = "stream")]
    Stream {
        /// Subject to subscribe to (default: ">")
        #[arg(default_value = ">")]
        subject: String,
        /// Print raw message payload without formatting
        #[arg(short, long)]
        raw: bool,
    },
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
            agent_nerves::serve::start(config).await?;
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
            println!("  port: {}", config.server.port);
            println!("  nats_url: {}", config.nats.url);
            println!("  spine: {}", config.spine.url);
        }
        Commands::Stream { subject, raw } => {
            agent_nerves::stream::tail_stream(&config.nats.url, &subject, raw).await?;
        }
    }
    Ok(())
}
