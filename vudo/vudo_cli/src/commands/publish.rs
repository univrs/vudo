//! `vudo publish` - Publish a signed Spirit to a DOLOS node
//!
//! The package (`vudo sign` output) is verified locally, then posted to the
//! node's `POST /api/packages`. The node stores it under its content hash,
//! announces it to its peers and records its name in the DHT, so any node in
//! the network can summon it by `author/name@version` (Spirit Hosting H2d-3).

use anyhow::{Context, Result};
use clap::Args;
use colored::*;
use std::fs;
use std::path::PathBuf;

use crate::config::VudoConfig;
use vudo_cli::node;

#[derive(Args, Debug)]
pub struct PublishArgs {
    /// Path to the signed Spirit package (defaults to the signed package in
    /// the current directory)
    pub package: Option<PathBuf>,

    /// Node API base URL, e.g. http://127.0.0.1:8080
    #[arg(long, default_value = "http://127.0.0.1:8080")]
    pub node: String,
}

pub async fn execute(args: PublishArgs, _config: &VudoConfig) -> Result<()> {
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
    println!("  {} {}", "Node:".cyan(), args.node);

    let published = node::publish(&args.node, &package_data).await?;

    println!("  {} {}", "Content hash:".cyan(), published.hash);
    println!("  {} {}", "Name:".cyan(), published.spirit_name());
    println!(
        "\n{} Published. Summon it from any node with:\n  vudo summon {} --node <url>",
        "✓".green().bold(),
        published.spirit_name()
    );

    Ok(())
}

fn find_spirit_package(dir: &str) -> Result<PathBuf> {
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_file() && path.to_string_lossy().ends_with(".signed.spirit") {
            return Ok(path);
        }
    }

    anyhow::bail!("No signed Spirit package found in current directory. Run 'vudo sign' first.")
}
