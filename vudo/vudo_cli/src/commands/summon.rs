//! `vudo summon` - Download Spirit from Imaginarium

use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::config::VudoConfig;

#[derive(Args, Debug)]
pub struct SummonArgs {
    /// Spirit name or path (@creator/name, name@version)
    pub spirit: String,

    /// Run immediately after summoning
    #[arg(long)]
    pub run: bool,

    /// Output directory (defaults to ~/.vudo/spirits/)
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Registry URL (defaults to config default)
    #[arg(long)]
    pub registry: Option<String>,
}

pub async fn execute(args: SummonArgs, config: &VudoConfig) -> Result<()> {
    println!(
        "{} Spirit: {}",
        "Summoning".green().bold(),
        args.spirit.cyan()
    );

    // Parse spirit name
    let (creator, name, version) = parse_spirit_name(&args.spirit)?;

    if let Some(c) = &creator {
        println!("  {} {}", "Creator:".cyan(), c);
    }
    println!("  {} {}", "Name:".cyan(), name);
    if let Some(v) = &version {
        println!("  {} {}", "Version:".cyan(), v);
    } else {
        println!("  {} latest", "Version:".cyan());
    }

    // Determine registry
    let registry = args
        .registry
        .or_else(|| config.default_registry())
        .unwrap_or_else(|| "https://imaginarium.vudo.univrs.io".to_string());

    println!("  {} {}", "Registry:".cyan(), registry);

    // There is no network registry yet. Fetching Spirits by content hash over
    // libp2p is Spirit Hosting Plan H2. Never write a placeholder package or
    // report success without doing the work.
    let _ = (args.output, args.run);
    anyhow::bail!("not implemented: network registry arrives in Spirit Hosting Plan H2")
}

fn parse_spirit_name(name: &str) -> Result<(Option<String>, String, Option<String>)> {
    // Handle @creator/name@version format
    let (creator, rest) = if let Some(stripped) = name.strip_prefix('@') {
        let parts: Vec<&str> = stripped.splitn(2, '/').collect();
        if parts.len() == 2 {
            (Some(parts[0].to_string()), parts[1])
        } else {
            (None, name)
        }
    } else {
        (None, name)
    };

    // Handle name@version
    let (spirit_name, version) = if let Some(idx) = rest.rfind('@') {
        (rest[..idx].to_string(), Some(rest[idx + 1..].to_string()))
    } else {
        (rest.to_string(), None)
    };

    Ok((creator, spirit_name, version))
}
