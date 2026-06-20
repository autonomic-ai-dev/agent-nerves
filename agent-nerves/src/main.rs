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
    /// Inspect and tail NATS / JetStream traffic
    Stream {
        #[command(subcommand)]
        command: StreamCommands,
    },
}

#[derive(Subcommand)]
enum StreamCommands {
    /// Tail JetStream messages (falls back to core NATS subscribe)
    Tail {
        /// Subject filter (default: autonomic.>)
        #[arg(default_value = "autonomic.>")]
        subject: String,
        /// Print raw message payload without formatting
        #[arg(short, long)]
        raw: bool,
        /// Deliver policy: `new` (live only) or `all` (replay from stream start)
        #[arg(long, default_value = "new")]
        from: String,
        /// Do not fall back to core NATS when JetStream is unavailable
        #[arg(long)]
        jetstream_only: bool,
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
        Commands::Stream {
            command: StreamCommands::Tail {
                subject,
                raw,
                from,
                jetstream_only,
            },
        } => {
            let from = agent_nerves::stream::TailFrom::parse(&from)?;
            agent_nerves::stream::tail_stream(
                &config.nats.url,
                &subject,
                raw,
                from,
                jetstream_only,
            )
            .await?;
        }
    }
    Ok(())
}
