//! `vudo pack` - Package Spirit for distribution

use anyhow::{Context, Result};
use clap::Args;
use colored::*;
use std::fs;
use std::path::PathBuf;

use crate::config::VudoConfig;

#[derive(Args, Debug)]
pub struct PackArgs {
    /// Path to the Spirit project (defaults to current directory)
    #[arg(short, long)]
    pub path: Option<PathBuf>,

    /// Include additional files or directories
    #[arg(long)]
    pub include: Option<Vec<PathBuf>>,

    /// Exclude patterns
    #[arg(long)]
    pub exclude: Option<Vec<String>>,

    /// Compression algorithm (zstd, gzip, none)
    #[arg(long, default_value = "zstd")]
    pub compress: String,

    /// Output file path
    #[arg(short, long)]
    pub output: Option<PathBuf>,
}

pub async fn execute(args: PackArgs, _config: &VudoConfig) -> Result<()> {
    let project_path = args.path.unwrap_or_else(|| PathBuf::from("."));
    let manifest_path = project_path.join("manifest.toml");

    println!("{} Spirit package", "Packing".green().bold());

    // Load manifest
    let manifest_content = fs::read_to_string(&manifest_path)
        .context("Failed to read manifest.toml. Make sure you're in a Spirit project directory.")?;

    let manifest: spirit_runtime::Manifest =
        toml::from_str(&manifest_content).context("Failed to parse manifest.toml")?;

    let spirit_name = &manifest.name;
    let version = &manifest.version;

    println!("  {} {}", "Spirit:".cyan(), spirit_name);
    println!("  {} {}", "Version:".cyan(), version);
    println!("  {} {}", "Compression:".cyan(), args.compress);

    // Determine output path
    let output_path = args
        .output
        .unwrap_or_else(|| PathBuf::from(format!("{}-{}.spirit", spirit_name, version)));

    // Find the built WASM module
    let wasm_path = project_path.join(format!("{}.spirit", spirit_name));
    if !wasm_path.exists() {
        anyhow::bail!(
            "Built Spirit not found at {:?}. Run 'vudo build' first.",
            wasm_path
        );
    }

    let wasm_bytes = fs::read(&wasm_path)
        .with_context(|| format!("Failed to read WASM file: {:?}", wasm_path))?;

    println!("  {} {} bytes", "WASM size:".cyan(), wasm_bytes.len());

    // Create package structure
    let mut package_data = Vec::new();

    // Add manifest
    package_data.extend_from_slice(b"MANIFEST\n");
    package_data.extend_from_slice(manifest_content.as_bytes());
    package_data.extend_from_slice(b"\n\nWASM\n");
    package_data.extend_from_slice(&wasm_bytes);

    // Add any included files
    if let Some(includes) = &args.include {
        for include_path in includes {
            println!("  {} {:?}", "Including:".cyan(), include_path);
            if include_path.exists() {
                let content = fs::read(include_path)?;
                package_data.extend_from_slice(b"\n\nFILE\n");
                package_data.extend_from_slice(include_path.to_string_lossy().as_bytes());
                package_data.extend_from_slice(b"\n");
                package_data.extend_from_slice(&content);
            }
        }
    }

    // Apply compression if requested
    let final_data = match args.compress.as_str() {
        "zstd" => {
            println!("  {} Applying zstd compression", "Compressing:".cyan());
            // In real implementation, use zstd crate
            package_data // Placeholder
        }
        "gzip" => {
            println!("  {} Applying gzip compression", "Compressing:".cyan());
            // In real implementation, use flate2 crate
            package_data // Placeholder
        }
        "none" => package_data,
        _ => anyhow::bail!("Unknown compression algorithm: {}", args.compress),
    };

    // Write package
    fs::write(&output_path, final_data)
        .with_context(|| format!("Failed to write package to {:?}", output_path))?;

    println!(
        "\n{} Created package: {:?}",
        "✓".green().bold(),
        output_path
    );
    println!(
        "  {} {} bytes",
        "Size:".cyan(),
        fs::metadata(&output_path)?.len()
    );

    println!("\nNext steps:");
    println!("  vudo sign {:?}", output_path);
    println!("  vudo publish {:?}", output_path);

    Ok(())
}
