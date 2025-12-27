//! VUDO CLI - The Creator's Command Line Interface
//!
//! This is the main entry point for the `vudo` command-line tool.
//! It provides commands for creating, building, running, testing, and publishing Spirits.

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

mod commands;
mod config;

use commands::*;
use config::VudoConfig;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const LOGO: &str = r#"
╔══════════════════════════════════════════════════════════════════╗
║  VUDO CLI                                                        ║
║  The system that knows what it is, becomes what it knows.        ║
╚══════════════════════════════════════════════════════════════════╝
"#;

#[derive(Parser)]
#[command(name = "vudo")]
#[command(author = "VUDO Team")]
#[command(version = VERSION)]
#[command(about = "The Creator's CLI for VUDO", long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Silence all output except errors
    #[arg(short, long, global = true)]
    quiet: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new Spirit project
    New(NewArgs),

    /// Compile DOL source to Spirit package
    Build(BuildArgs),

    /// Execute a Spirit locally in the sandbox
    Run(RunArgs),

    /// Run Spirit tests
    Test(TestArgs),

    /// Package Spirit for distribution
    Pack(PackArgs),

    /// Sign package with Ed25519 identity
    Sign(SignArgs),

    /// Publish to the Imaginarium
    Publish(PublishArgs),

    /// Download Spirit from Imaginarium
    Summon(SummonArgs),

    /// Install Spirit to local registry
    Install(InstallArgs),

    /// Uninstall Spirit from local registry
    Uninstall(UninstallArgs),

    /// List installed Spirits
    List(ListArgs),

    /// Search the Imaginarium
    Search(SearchArgs),

    /// Show Spirit details
    Info(InfoArgs),

    /// Validate DOL syntax and types
    Check(CheckArgs),

    /// Format DOL source files
    Fmt(FmtArgs),

    /// Generate documentation
    Doc(DocArgs),

    /// Enter DOL REPL (interactive mode)
    Dol(DolArgs),

    /// Update vudo CLI
    Upgrade(UpgradeArgs),
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging based on verbosity
    let log_level = if cli.quiet {
        Level::ERROR
    } else if cli.verbose {
        Level::DEBUG
    } else {
        Level::INFO
    };

    let subscriber = FmtSubscriber::builder()
        .with_max_level(log_level)
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    // Load configuration
    let config = VudoConfig::load().unwrap_or_default();

    // Execute command
    let result = match cli.command {
        Commands::New(args) => commands::new::execute(args, &config).await,
        Commands::Build(args) => commands::build::execute(args, &config).await,
        Commands::Run(args) => commands::run::execute(args, &config).await,
        Commands::Test(args) => commands::test::execute(args, &config).await,
        Commands::Pack(args) => commands::pack::execute(args, &config).await,
        Commands::Sign(args) => commands::sign::execute(args, &config).await,
        Commands::Publish(args) => commands::publish::execute(args, &config).await,
        Commands::Summon(args) => commands::summon::execute(args, &config).await,
        Commands::Install(args) => commands::install::execute(args, &config).await,
        Commands::Uninstall(args) => commands::uninstall::execute(args, &config).await,
        Commands::List(args) => commands::list::execute(args, &config).await,
        Commands::Search(args) => commands::search::execute(args, &config).await,
        Commands::Info(args) => commands::info::execute(args, &config).await,
        Commands::Check(args) => commands::check::execute(args, &config).await,
        Commands::Fmt(args) => commands::fmt::execute(args, &config).await,
        Commands::Doc(args) => commands::doc::execute(args, &config).await,
        Commands::Dol(args) => commands::dol::execute(args, &config).await,
        Commands::Upgrade(args) => commands::upgrade::execute(args, &config).await,
    };

    // Handle result and print appropriate message
    match result {
        Ok(_) => {
            if !cli.quiet {
                tracing::info!("{}", "Done!".green().bold());
            }
            Ok(())
        }
        Err(e) => {
            eprintln!("{} {}", "Error:".red().bold(), e);
            std::process::exit(1);
        }
    }
}

/// Print the VUDO logo
pub fn print_logo() {
    println!("{}", LOGO.cyan());
}
