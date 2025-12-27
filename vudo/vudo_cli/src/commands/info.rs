//! `vudo info` - Show Spirit details

use anyhow::{Context, Result};
use clap::Args;
use colored::*;
use std::fs;
use std::path::PathBuf;

use crate::config::VudoConfig;

#[derive(Args, Debug)]
pub struct InfoArgs {
    /// Path to Spirit package or name from Imaginarium
    pub spirit: String,

    /// Show detailed information
    #[arg(short, long)]
    pub verbose: bool,
}

pub async fn execute(args: InfoArgs, _config: &VudoConfig) -> Result<()> {
    println!(
        "{} Spirit information: {}",
        "Fetching".green().bold(),
        args.spirit.cyan()
    );
    println!();

    // Check if it's a local file or remote Spirit
    let spirit_path = PathBuf::from(&args.spirit);

    if spirit_path.exists() {
        show_local_spirit_info(&spirit_path, args.verbose)?;
    } else {
        show_remote_spirit_info(&args.spirit, args.verbose).await?;
    }

    Ok(())
}

fn show_local_spirit_info(path: &PathBuf, verbose: bool) -> Result<()> {
    println!("{} Local Spirit Package", "Type:".cyan().bold());
    println!("{} {:?}", "Path:".cyan(), path);

    // Read package
    let package_data =
        fs::read(path).with_context(|| format!("Failed to read Spirit package: {:?}", path))?;

    println!("{} {} bytes", "Size:".cyan(), package_data.len());

    // Check if signed
    if package_data.starts_with(b"SIGNED\n") {
        println!("{} {}", "Signed:".cyan(), "Yes".green());

        // Extract signature info
        let content = String::from_utf8_lossy(&package_data);
        let lines: Vec<&str> = content.lines().collect();
        if lines.len() >= 2 {
            println!("{} {}", "Public Key:".cyan(), lines[1]);
            if verbose && lines.len() >= 3 {
                println!("{} {}", "Signature:".cyan(), lines[2]);
            }
        }
    } else {
        println!("{} {}", "Signed:".cyan(), "No".yellow());
    }

    // Try to extract manifest
    let content = String::from_utf8_lossy(&package_data);
    if let Some(manifest_start) = content.find("MANIFEST\n") {
        if let Some(manifest_end) = content[manifest_start..].find("\n\nWASM") {
            let manifest_text = &content[manifest_start + 9..manifest_start + manifest_end];

            if let Ok(manifest) = toml::from_str::<spirit_runtime::Manifest>(manifest_text) {
                println!();
                println!("{}", "Manifest:".cyan().bold());
                println!("{} {}", "  Name:".cyan(), manifest.name);
                println!("{} {}", "  Version:".cyan(), manifest.version);

                if verbose {
                    if let Some(desc) = &manifest.description {
                        println!("{} {}", "  Description:".cyan(), desc);
                    }
                    println!("{} {}", "  Author:".cyan(), manifest.author);
                }
            }
        }
    }

    Ok(())
}

async fn show_remote_spirit_info(name: &str, verbose: bool) -> Result<()> {
    println!("{} Remote Spirit", "Type:".cyan().bold());
    println!("{} {}", "Name:".cyan(), name);

    // In a real implementation, this would:
    // 1. Query the registry
    // 2. Fetch metadata
    // 3. Display information

    println!("{} {}", "Version:".cyan(), "1.0.0");
    println!("{} {}", "Creator:".cyan(), "@example");
    println!("{} {}", "Downloads:".cyan(), "1,234");
    println!("{} {}", "Rating:".cyan(), "★★★★☆ (4.5/5)");

    if verbose {
        println!();
        println!("{}", "Description:".cyan().bold());
        println!("  A fantastic Spirit that does amazing things.");
        println!();
        println!("{}", "Capabilities:".cyan().bold());
        println!("  - Compute");
        println!("  - Memory");
        println!();
        println!("{}", "Dependencies:".cyan().bold());
        println!("  - @std/io@1.0.0");
        println!("  - @std/math@2.1.0");
    }

    Ok(())
}
