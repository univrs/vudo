//! `vudo list` - List installed Spirits

use anyhow::{Context, Result};
use clap::Args;
use colored::*;

use crate::config::VudoConfig;
use spirit_runtime::registry::{LocalRegistry, Registry};

#[derive(Args, Debug)]
pub struct ListArgs {
    /// Show detailed information including all versions
    #[arg(short, long)]
    pub verbose: bool,

    /// Output in JSON format
    #[arg(long)]
    pub json: bool,
}

pub async fn execute(args: ListArgs, _config: &VudoConfig) -> Result<()> {
    // Initialize registry
    let mut registry = LocalRegistry::new();
    registry
        .init()
        .await
        .context("Failed to initialize registry")?;

    // Get installed spirits
    let spirits = registry.list().await.context("Failed to list spirits")?;

    if spirits.is_empty() {
        if args.json {
            println!("[]");
        } else {
            println!("{}", "No Spirits installed.".yellow());
            println!();
            println!("Install a Spirit with:");
            println!("  vudo install <path>");
            println!();
            println!("Or summon from the Imaginarium:");
            println!("  vudo summon <name>");
        }
        return Ok(());
    }

    if args.json {
        // JSON output
        let json_output: Vec<serde_json::Value> = spirits
            .iter()
            .map(|s| {
                serde_json::json!({
                    "name": s.name,
                    "latest": s.latest,
                    "versions": s.versions,
                    "installed_at": s.installed_at
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&json_output)?);
    } else {
        // Human-readable output
        println!(
            "{} {} Spirit(s):\n",
            "Installed".green().bold(),
            spirits.len()
        );

        for spirit in &spirits {
            if args.verbose {
                println!("  {} {}", spirit.name.cyan().bold(), spirit.latest.yellow());
                println!(
                    "    {} {}",
                    "Versions:".dimmed(),
                    spirit.versions.join(", ")
                );
                println!(
                    "    {} {}",
                    "Installed:".dimmed(),
                    format_timestamp(spirit.installed_at)
                );
                println!();
            } else {
                println!("  {}@{}", spirit.name.cyan(), spirit.latest.yellow());
            }
        }

        if !args.verbose {
            println!();
            println!("Use {} for more details", "--verbose".cyan());
        }
    }

    Ok(())
}

fn format_timestamp(ts: u64) -> String {
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    let time = UNIX_EPOCH + Duration::from_secs(ts);
    let now = SystemTime::now();

    if let Ok(duration) = now.duration_since(time) {
        let secs = duration.as_secs();
        if secs < 60 {
            "just now".to_string()
        } else if secs < 3600 {
            format!("{} minute(s) ago", secs / 60)
        } else if secs < 86400 {
            format!("{} hour(s) ago", secs / 3600)
        } else {
            format!("{} day(s) ago", secs / 86400)
        }
    } else {
        "unknown".to_string()
    }
}
