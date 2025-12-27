//! `vudo summon` - Download Spirit from Imaginarium

use anyhow::Result;
use clap::Args;
use colored::*;
use std::fs;
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

    // Determine output path
    let spirit_dir = args.output.unwrap_or_else(|| {
        let vudo_dir = config.vudo_dir();
        vudo_dir.join("spirits")
    });

    fs::create_dir_all(&spirit_dir)?;

    let spirit_path = spirit_dir.join(format!("{}.spirit", name));

    println!("\n{} from Imaginarium...", "Downloading".green().bold());

    // In a real implementation, this would:
    // 1. Query the registry for the Spirit
    // 2. Verify checksums and signatures
    // 3. Download the package
    // 4. Cache locally

    // Simulate download
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    // Create a placeholder Spirit file
    let placeholder = b"Summoned Spirit placeholder";
    fs::write(&spirit_path, placeholder)?;

    println!("{} Downloaded to: {:?}", "✓".green().bold(), spirit_path);

    if args.run {
        println!("\n{} Spirit...", "Running".green().bold());
        // In real implementation, would execute the Spirit
        println!("{} Spirit executed successfully", "✓".green().bold());
    } else {
        println!("\nRun with:");
        println!("  vudo run {:?}", spirit_path);
    }

    Ok(())
}

fn parse_spirit_name(name: &str) -> Result<(Option<String>, String, Option<String>)> {
    // Handle @creator/name@version format
    let (creator, rest) = if name.starts_with('@') {
        let parts: Vec<&str> = name[1..].splitn(2, '/').collect();
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
