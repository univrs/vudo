//! `vudo install` - Install Spirit to registry

use anyhow::{Context, Result};
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::config::VudoConfig;
use spirit_runtime::registry::{LocalRegistry, Registry};

#[derive(Args, Debug)]
pub struct InstallArgs {
    /// Path to Spirit project or package
    pub source: PathBuf,

    /// Force reinstall if already installed
    #[arg(short, long)]
    pub force: bool,
}

pub async fn execute(args: InstallArgs, _config: &VudoConfig) -> Result<()> {
    println!(
        "{} Spirit from: {}",
        "Installing".green().bold(),
        args.source.display().to_string().cyan()
    );

    // Initialize registry
    let mut registry = LocalRegistry::new();
    registry
        .init()
        .await
        .context("Failed to initialize registry")?;

    // If force is set and spirit is already installed, uninstall first
    if args.force {
        // Read manifest to get spirit name
        let manifest_path = if args.source.is_dir() {
            let json_path = args.source.join("manifest.json");
            let toml_path = args.source.join("manifest.toml");
            if json_path.exists() {
                json_path
            } else if toml_path.exists() {
                toml_path
            } else {
                anyhow::bail!("No manifest found in source directory");
            }
        } else {
            anyhow::bail!("Source must be a directory containing manifest and spirit.wasm");
        };

        let content = tokio::fs::read_to_string(&manifest_path)
            .await
            .context("Failed to read manifest")?;

        let name = if manifest_path.extension().and_then(|s| s.to_str()) == Some("json") {
            let manifest: serde_json::Value = serde_json::from_str(&content)?;
            manifest["name"].as_str().unwrap_or("unknown").to_string()
        } else {
            let manifest: toml::Value = toml::from_str(&content)?;
            manifest
                .get("spirit")
                .and_then(|s| s.get("name"))
                .and_then(|n| n.as_str())
                .or_else(|| manifest.get("name").and_then(|n| n.as_str()))
                .unwrap_or("unknown")
                .to_string()
        };

        if registry.is_installed(&name) {
            println!("  {} existing installation...", "Removing".yellow());
            registry
                .uninstall(&name)
                .await
                .context("Failed to uninstall existing version")?;
        }
    }

    // Install spirit
    let source_str = args
        .source
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("Invalid source path"))?;

    let installed = registry
        .install(source_str)
        .await
        .context("Failed to install Spirit")?;

    println!(
        "{} Installed: {}@{}",
        "✓".green().bold(),
        installed.name.cyan(),
        installed.latest.yellow()
    );

    println!();
    println!("Run with:");
    println!("  vudo run {}", installed.name);

    Ok(())
}
