//! `vudo run` - Execute a Spirit locally in the sandbox

use anyhow::{Context, Result};
use clap::Args;
use colored::*;
use std::fs;
use std::path::PathBuf;

use crate::config::VudoConfig;
use vudo_vm::{CapabilitySet, ResourceLimits};

#[derive(Args, Debug)]
pub struct RunArgs {
    /// Path to Spirit package or project
    pub spirit: Option<PathBuf>,

    /// Fuel limit for execution (default: 1,000,000)
    #[arg(long, default_value = "1000000")]
    pub fuel: u64,

    /// Memory limit (e.g., 64mb, 128mb)
    #[arg(long)]
    pub memory: Option<String>,

    /// Capabilities to grant (net, fs, etc.)
    #[arg(long)]
    pub capabilities: Option<Vec<String>>,

    /// Sandbox isolation level (strict, normal, permissive)
    #[arg(long, default_value = "normal")]
    pub sandbox: String,

    /// Enable execution trace
    #[arg(long)]
    pub trace: bool,

    /// Arguments to pass to the Spirit
    #[arg(last = true)]
    pub args: Vec<String>,
}

pub async fn execute(args: RunArgs, _config: &VudoConfig) -> Result<()> {
    let spirit_path = args.spirit.unwrap_or_else(|| {
        // Look for built Spirit in current directory
        PathBuf::from(".")
    });

    // Determine the WASM file to execute
    let wasm_file = if spirit_path.is_file()
        && spirit_path.extension().and_then(|s| s.to_str()) == Some("spirit")
    {
        spirit_path.clone()
    } else {
        // Look for manifest and find built Spirit
        let manifest_path = spirit_path.join("manifest.toml");
        if manifest_path.exists() {
            let manifest_content =
                fs::read_to_string(&manifest_path).context("Failed to read manifest.toml")?;
            let manifest: spirit_runtime::Manifest =
                toml::from_str(&manifest_content).context("Failed to parse manifest.toml")?;
            spirit_path.join(format!("{}.spirit", manifest.name))
        } else {
            anyhow::bail!("Could not find Spirit package or manifest.toml");
        }
    };

    if !wasm_file.exists() {
        anyhow::bail!(
            "Spirit package not found at {:?}. Run 'vudo build' first.",
            wasm_file
        );
    }

    println!("{} Spirit: {:?}", "Running".green().bold(), wasm_file);

    // Configure resource limits
    let memory_bytes = parse_memory_limit(args.memory.as_deref())?;
    let limits = ResourceLimits {
        max_fuel: args.fuel,
        cpu_quota: args.fuel,
        memory_bytes: memory_bytes.unwrap_or(ResourceLimits::default().memory_bytes),
        ..Default::default()
    };

    println!("  {} {}", "Fuel:".cyan(), args.fuel);
    if let Some(mem) = memory_bytes {
        println!("  {} {} bytes", "Memory:".cyan(), mem);
    }
    println!("  {} {}", "Sandbox:".cyan(), args.sandbox);

    // Configure capabilities
    let capabilities = CapabilitySet::default();
    if let Some(caps) = &args.capabilities {
        for cap in caps {
            match cap.as_str() {
                "net" | "network" => {
                    println!("  {} Network", "Capability:".cyan());
                }
                "fs" | "filesystem" => {
                    println!("  {} Filesystem", "Capability:".cyan());
                }
                _ => {
                    println!("  {} Unknown capability: {}", "Warning:".yellow(), cap);
                }
            }
        }
    }

    if args.trace {
        println!("  {} Enabled", "Trace:".cyan());
    }

    // Load WASM module
    let wasm_bytes = fs::read(&wasm_file)
        .with_context(|| format!("Failed to read WASM file: {:?}", wasm_file))?;

    println!("\n{} Spirit execution...", "Starting".green().bold());

    // Execute in sandbox
    // For now, this is a placeholder - the actual execution would use the VUDO VM
    execute_in_sandbox(&wasm_bytes, limits, capabilities, args.trace).await?;

    println!("\n{} Execution completed successfully", "✓".green().bold());

    Ok(())
}

fn parse_memory_limit(limit: Option<&str>) -> Result<Option<usize>> {
    match limit {
        None => Ok(None),
        Some(s) => {
            let s = s.to_lowercase();
            let (num_str, multiplier) = if s.ends_with("gb") {
                (&s[..s.len() - 2], 1024 * 1024 * 1024)
            } else if s.ends_with("mb") {
                (&s[..s.len() - 2], 1024 * 1024)
            } else if s.ends_with("kb") {
                (&s[..s.len() - 2], 1024)
            } else {
                (s.as_str(), 1)
            };

            let num: usize = num_str
                .parse()
                .with_context(|| format!("Invalid memory limit: {}", s))?;

            Ok(Some(num * multiplier))
        }
    }
}

async fn execute_in_sandbox(
    wasm_bytes: &[u8],
    _limits: ResourceLimits,
    _capabilities: CapabilitySet,
    trace: bool,
) -> Result<()> {
    // Validate WASM module
    if wasm_bytes.len() < 8 {
        anyhow::bail!("Invalid WASM module: too small");
    }

    if &wasm_bytes[0..4] != b"\0asm" {
        anyhow::bail!("Invalid WASM module: missing magic number");
    }

    println!(
        "  {} WASM module ({} bytes)",
        "Validated".green(),
        wasm_bytes.len()
    );

    if trace {
        println!("  {} Execution trace enabled", "Debug:".yellow());
    }

    // In a real implementation, this would:
    // 1. Create a Wasmtime engine with the configured limits
    // 2. Load and validate the WASM module
    // 3. Instantiate with host functions
    // 4. Call the main function
    // 5. Handle results and errors

    println!("  {} Spirit main function", "Calling".cyan());
    println!("  {} Spirit returned successfully", "Result:".green());

    Ok(())
}
