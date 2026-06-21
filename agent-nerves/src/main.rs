use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(version)]
#[command(
    name = "agent-nerves",
    about = "Distributed event bus for autonomic agents"
)]
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
    /// Multi-node cluster coordination
    Cluster {
        #[command(subcommand)]
        command: ClusterCommands,
    },
    /// Event filter registry (JSON + WASM)
    Filter {
        #[command(subcommand)]
        command: FilterCommands,
    },
    /// Inspect and tail NATS / JetStream traffic
    Stream {
        #[command(subcommand)]
        command: StreamCommands,
    },
    /// View supervisor process logs
    Log {
        /// Log name to view (omitting lists available logs)
        name: Option<String>,
        /// Follow log output (tail -f)
        #[arg(short, long)]
        follow: bool,
        /// List available log files
        #[arg(short, long)]
        list: bool,
    },
}

#[derive(Subcommand)]
enum ClusterCommands {
    /// Initialize cluster state (leader election + state file)
    Init,
    /// Show cluster status (leader, peers, WireGuard)
    Status,
    /// Render NATS cluster config to stdout
    RenderConfig {
        /// Client port (default: from NATS URL)
        #[arg(long)]
        port: Option<u16>,
    },
}

#[derive(Subcommand)]
enum FilterCommands {
    /// List loaded filter rules
    List,
    /// Test a subject/payload against filter rules
    Test {
        subject: String,
        payload: String,
        /// Override filters directory
        #[arg(long)]
        dir: Option<PathBuf>,
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
        /// Apply filter rules while tailing (drops denied messages)
        #[arg(long)]
        filter: bool,
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
        Commands::Ping => match agent_nerves::nats::ping(&config).await {
            Ok(info) => {
                let json = serde_json::to_string_pretty(&info)?;
                println!("{}", json);
            }
            Err(e) => {
                eprintln!("NATS ping failed: {}", e);
                std::process::exit(1);
            }
        },
        Commands::Status => {
            println!("agent-nerves status");
            println!(
                "  config: {}",
                agent_nerves::config::Config::config_path().display()
            );
            println!("  port: {}", config.server.port);
            println!("  nats_url: {}", config.nats.url);
            println!("  spine: {}", config.spine.url);
            println!(
                "  cluster: enabled={} node_id={}",
                config.cluster.enabled, config.cluster.node_id
            );
            println!(
                "  filters: enabled={} dir={}",
                config.filters.enabled,
                agent_nerves::filter::filters_dir(
                    config
                        .filters
                        .directory
                        .as_deref()
                        .map(std::path::Path::new)
                )
                .display()
            );
        }
        Commands::Cluster { command } => match command {
            ClusterCommands::Init => {
                let state = agent_nerves::cluster::init_cluster(&config.cluster)?;
                println!("{}", serde_json::to_string_pretty(&state)?);
            }
            ClusterCommands::Status => {
                let status = agent_nerves::cluster::cluster_status(&config.cluster).await?;
                println!("{}", serde_json::to_string_pretty(&status)?);
            }
            ClusterCommands::RenderConfig { port } => {
                let client_port = port.unwrap_or_else(|| parse_nats_port(&config.nats.url));
                let body = agent_nerves::cluster::render_nats_cluster_config(
                    &config.cluster,
                    &config.broker_store_dir(),
                    client_port,
                )?;
                print!("{body}");
            }
        },
        Commands::Filter { command } => {
            let filters_dir = agent_nerves::filter::filters_dir(
                config
                    .filters
                    .directory
                    .as_deref()
                    .map(std::path::Path::new),
            );
            match command {
                FilterCommands::List => {
                    let rules = agent_nerves::filter::load_rules(&filters_dir)?;
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "directory": filters_dir.display().to_string(),
                            "rules": rules,
                        }))?
                    );
                }
                FilterCommands::Test {
                    subject,
                    payload,
                    dir,
                } => {
                    let dir = dir.as_deref().unwrap_or(&filters_dir);
                    let decision =
                        agent_nerves::filter::evaluate_event(dir, &subject, payload.as_bytes())?;
                    println!("{}", serde_json::to_string_pretty(&decision)?);
                }
            }
        }
        Commands::Stream {
            command:
                StreamCommands::Tail {
                    subject,
                    raw,
                    from,
                    jetstream_only,
                    filter,
                },
        } => {
            let from = agent_nerves::stream::TailFrom::parse(&from)?;
            let apply_filters = filter || config.filters.enabled;
            let filters_dir = if apply_filters {
                Some(agent_nerves::filter::filters_dir(
                    config
                        .filters
                        .directory
                        .as_deref()
                        .map(std::path::Path::new),
                ))
            } else {
                None
            };
            agent_nerves::stream::tail_stream(
                &config.nats.url,
                &subject,
                raw,
                from,
                jetstream_only,
                filters_dir.as_deref(),
            )
            .await?;
        }
        Commands::Log { name, follow, list } => {
            if list || name.is_none() {
                let logs = agent_nerves::log::list_logs()?;
                if logs.is_empty() {
                    println!("no logs found");
                } else {
                    for log in logs {
                        println!("{log}");
                    }
                }
            } else if let Some(name) = name {
                if follow {
                    agent_nerves::log::follow_log(&name)?;
                } else {
                    agent_nerves::log::print_log(&name)?;
                }
            }
        }
    }
    Ok(())
}

fn parse_nats_port(url: &str) -> u16 {
    url.rsplit(':')
        .next()
        .and_then(|p| p.parse().ok())
        .unwrap_or(4222)
}
