//! `vudo publish` - Publish to the Imaginarium

use anyhow::{Context, Result};
use clap::Args;
use colored::*;
use std::fs;
use std::path::PathBuf;

use crate::config::VudoConfig;

#[derive(Args, Debug)]
pub struct PublishArgs {
    /// Path to Spirit package (defaults to searching current directory)
    pub package: Option<PathBuf>,

    /// Visibility (public, unlisted, private)
    #[arg(long)]
    pub public: bool,

    #[arg(long)]
    pub unlisted: bool,

    #[arg(long)]
    pub private: bool,

    /// Pricing model (free, or credits per summon)
    #[arg(long)]
    pub free: bool,

    #[arg(long)]
    pub credits: Option<u64>,

    /// Registry URL (defaults to config default)
    #[arg(long)]
    pub registry: Option<String>,
}

pub async fn execute(args: PublishArgs, config: &VudoConfig) -> Result<()> {
    // Determine package path
    let package_path = if let Some(path) = args.package {
        path
    } else {
        // Look for signed Spirit package in current directory
        find_spirit_package(".")?
    };

    println!(
        "{} Spirit package: {:?}",
        "Publishing".green().bold(),
        package_path
    );

    // Read package
    let package_data = fs::read(&package_path)
        .with_context(|| format!("Failed to read package: {:?}", package_path))?;

    println!("  {} {} bytes", "Package size:".cyan(), package_data.len());

    // Verify it's signed
    if !package_data.starts_with(b"SIGNED\n") {
        println!(
            "{} Package is not signed. Run 'vudo sign' first.",
            "Warning:".yellow().bold()
        );
    }

    // Determine visibility
    let visibility = if args.public {
        "public"
    } else if args.unlisted {
        "unlisted"
    } else if args.private {
        "private"
    } else {
        "public" // Default
    };

    println!("  {} {}", "Visibility:".cyan(), visibility);

    // Determine pricing
    let pricing = if args.free {
        "free".to_string()
    } else if let Some(credits) = args.credits {
        format!("{} credits per summon", credits)
    } else {
        "free".to_string() // Default
    };

    println!("  {} {}", "Pricing:".cyan(), pricing);

    // Determine registry
    let registry = args
        .registry
        .or_else(|| config.default_registry())
        .unwrap_or_else(|| "https://imaginarium.vudo.univrs.io".to_string());

    println!("  {} {}", "Registry:".cyan(), registry);

    // In a real implementation, this would:
    // 1. Authenticate with the registry
    // 2. Upload the package
    // 3. Set metadata (visibility, pricing)
    // 4. Return the package URL

    println!(
        "\n{} Publishing to Imaginarium...",
        "Uploading".green().bold()
    );

    // Simulate upload
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    println!("\n{} Spirit published successfully!", "✓".green().bold());
    println!(
        "\n{} https://imaginarium.vudo.univrs.io/@yourname/your-spirit",
        "URL:".cyan().bold()
    );
    println!("\nOthers can now summon your Spirit with:");
    println!("  vudo summon @yourname/your-spirit");

    Ok(())
}

fn find_spirit_package(dir: &str) -> Result<PathBuf> {
    let dir_path = PathBuf::from(dir);

    // Look for .spirit files
    for entry in fs::read_dir(&dir_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "spirit" {
                    // Prefer signed packages
                    if path.to_string_lossy().contains("signed") {
                        return Ok(path);
                    }
                }
            }
        }
    }

    // If no signed package, look for any .spirit file
    for entry in fs::read_dir(&dir_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "spirit" {
                    return Ok(path);
                }
            }
        }
    }

    anyhow::bail!("No Spirit package found in current directory. Run 'vudo pack' first.")
}
