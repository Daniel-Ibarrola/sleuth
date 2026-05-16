use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "sleuth",
    version,
    about = "Diagnose CI/CD failures faster",
    propagate_version = true
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Analyze a CI/CD artifact and produce a diagnostic report.
    Analyze {
        #[command(subcommand)]
        target: AnalyzeTarget,
    },
    /// Fetch and display raw CI/CD data without running the LLM.
    Fetch {
        #[command(subcommand)]
        target: FetchTarget,
    },
    /// List past analyses recorded locally.
    History {
        /// Filter to a single GitHub repository (owner/name).
        #[arg(long)]
        repo: Option<String>,
        /// Maximum number of rows to display.
        #[arg(long, default_value_t = 20)]
        limit: u32,
    },
    /// Re-render a past analysis from the local store.
    Show {
        /// The analysis ID returned by `sleuth history`.
        analysis_id: i64,
        /// Emit as JSON instead of formatted output.
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand, Debug)]
enum AnalyzeTarget {
    /// Analyze a GitHub Actions workflow run.
    Run {
        /// The GitHub Actions run ID.
        run_id: u64,
        /// The GitHub repository the run belongs to (owner/name).
        #[arg(long)]
        repo: Option<String>,
        /// Emit the report as JSON instead of formatted output.
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand, Debug)]
enum FetchTarget {
    /// Fetch a GitHub Actions workflow run.
    Run {
        /// The GitHub Actions run ID.
        run_id: u64,
        /// The GitHub repository the run belongs to (owner/name).
        #[arg(long)]
        repo: Option<String>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    let cli = Cli::parse();
    tracing::debug!(?cli, "parsed CLI");

    match cli.command {
        Command::Analyze {
            target: AnalyzeTarget::Run { run_id, repo, json },
        } => {
            tracing::info!(run_id, ?repo, json, "analyze run");
            println!("not implemented yet");
        }
        Command::Fetch {
            target: FetchTarget::Run { run_id, repo },
        } => {
            tracing::info!(run_id, ?repo, "fetch run");
            println!("not implemented yet");
        }
        Command::History { repo, limit } => {
            tracing::info!(?repo, limit, "history");
            println!("not implemented yet");
        }
        Command::Show { analysis_id, json } => {
            tracing::info!(analysis_id, json, "show");
            println!("not implemented yet");
        }
    }

    Ok(())
}

fn init_tracing() {
    use tracing_subscriber::{EnvFilter, fmt};

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn"));
    fmt().with_env_filter(filter).with_target(false).init();
}
