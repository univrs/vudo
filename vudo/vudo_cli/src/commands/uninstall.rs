//! `vudo uninstall` - Remove Spirit from registry

use anyhow::{Context, Result};
use clap::Args;
use colored::*;

use crate::config::VudoConfig;
use spirit_runtime::registry::{LocalRegistry, Registry};

#[derive(Args, Debug)]
pub struct UninstallArgs {
    /// Name of the Spirit to uninstall
    pub name: String,

    /// Specific version to uninstall (default: all versions)
    #[arg(long = "ver", id = "spirit_version")]
    pub spirit_version: Option<String>,

    /// Skip confirmation prompt
    #[arg(short = 'y', long)]
    pub yes: bool,
}

pub async fn execute(args: UninstallArgs, _config: &VudoConfig) -> Result<()> {
    // Initialize registry
    let mut registry = LocalRegistry::new();
    registry
        .init()
        .await
        .context("Failed to initialize registry")?;

    // Check if spirit is installed
    if !registry.is_installed(&args.name) {
        anyhow::bail!("Spirit '{}' is not installed", args.name);
    }

    if let Some(ref version) = args.spirit_version {
        // Uninstall specific version
        if !registry.is_version_installed(&args.name, version) {
            anyhow::bail!("Version {}@{} is not installed", args.name, version);
        }

        if !args.yes {
            println!(
                "{} {}@{}?",
                "Uninstall".yellow().bold(),
                args.name.cyan(),
                version.yellow()
            );
            print!("Continue? [y/N] ");
            use std::io::{self, Write};
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            if !input.trim().eq_ignore_ascii_case("y") {
                println!("Aborted.");
                return Ok(());
            }
        }

        println!(
            "{} {}@{}...",
            "Uninstalling".green().bold(),
            args.name.cyan(),
            version.yellow()
        );

        registry
            .uninstall_version(&args.name, version)
            .await
            .context("Failed to uninstall version")?;

        println!(
            "{} Removed {}@{}",
            "✓".green().bold(),
            args.name.cyan(),
            version.yellow()
        );
    } else {
        // Uninstall all versions
        if !args.yes {
            println!(
                "{} {} (all versions)?",
                "Uninstall".yellow().bold(),
                args.name.cyan()
            );
            print!("Continue? [y/N] ");
            use std::io::{self, Write};
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            if !input.trim().eq_ignore_ascii_case("y") {
                println!("Aborted.");
                return Ok(());
            }
        }

        println!("{} {}...", "Uninstalling".green().bold(), args.name.cyan());

        registry
            .uninstall(&args.name)
            .await
            .context("Failed to uninstall Spirit")?;

        println!(
            "{} Removed {} (all versions)",
            "✓".green().bold(),
            args.name.cyan()
        );
    }

    Ok(())
}
