//! `vudo run` - Execute a Spirit locally in the sandbox

use anyhow::{Context, Result};
use clap::Args;
use colored::*;
use std::fs;
use std::path::PathBuf;

use crate::config::VudoConfig;
use spirit_runtime::manifest::Capability;
use vudo_vm::sandbox::{ResourceLimits, Sandbox};
use vudo_vm::{CapabilityGrant, CapabilityScope, CapabilitySet, CapabilityType};
use wasmtime::Val;

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

    /// Exported function to invoke
    #[arg(long, default_value = "main")]
    pub entry: String,

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

    // Determine the WASM file to execute, and the manifest if there is one
    let (wasm_file, manifest) = if spirit_path.is_file() {
        (spirit_path.clone(), None)
    } else {
        let manifest_path = spirit_path.join("manifest.toml");
        if manifest_path.exists() {
            let manifest_content =
                fs::read_to_string(&manifest_path).context("Failed to read manifest.toml")?;
            let manifest: spirit_runtime::Manifest =
                toml::from_str(&manifest_content).context("Failed to parse manifest.toml")?;
            (
                spirit_path.join(format!("{}.spirit", manifest.name)),
                Some(manifest),
            )
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
    let mut limits = ResourceLimits {
        max_fuel: args.fuel,
        ..Default::default()
    };
    if let Some(mem) = memory_bytes {
        limits.memory_bytes = mem as u64;
    }

    println!("  {} {}", "Fuel:".cyan(), args.fuel);
    println!("  {} {} bytes", "Memory:".cyan(), limits.memory_bytes);
    println!("  {} {}", "Sandbox:".cyan(), args.sandbox);
    println!("  {} {}", "Entry:".cyan(), args.entry);

    // Capabilities: those declared in the manifest plus those granted on the CLI
    let mut requested: Vec<CapabilityType> = manifest
        .as_ref()
        .map(|m| m.capabilities.iter().map(to_capability_type).collect())
        .unwrap_or_default();
    for cap in args.capabilities.iter().flatten() {
        requested.extend(parse_cli_capability(cap)?);
    }
    requested.sort_by_key(|c| *c as u8);
    requested.dedup();
    for cap in &requested {
        println!("  {} {:?}", "Capability:".cyan(), cap);
    }

    if args.trace {
        println!("  {} Enabled", "Trace:".cyan());
    }

    let call_args = parse_call_args(&args.args)?;

    // Load WASM module
    let wasm_bytes = fs::read(&wasm_file)
        .with_context(|| format!("Failed to read WASM file: {:?}", wasm_file))?;

    println!("\n{} Spirit execution...", "Starting".green().bold());

    let result = execute_in_sandbox(&wasm_bytes, limits, &requested, &args.entry, &call_args)?;

    if args.trace {
        println!(
            "  {} {} fuel consumed",
            "Trace:".yellow(),
            result.fuel_consumed
        );
        println!("  {} {:?}", "Trace:".yellow(), result.duration);
    }

    if !result.success {
        anyhow::bail!(
            "Spirit `{}` failed: {}",
            args.entry,
            result.error.unwrap_or_else(|| "unknown error".to_string())
        );
    }

    let values = result.return_value.unwrap_or_default();
    let rendered: Vec<String> = values.iter().map(format_val).collect();
    println!(
        "  {} {}",
        "Result:".green(),
        if rendered.is_empty() {
            "()".to_string()
        } else {
            rendered.join(", ")
        }
    );

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

/// Create a real VUDO VM sandbox, grant the requested capabilities and invoke `entry`.
fn execute_in_sandbox(
    wasm_bytes: &[u8],
    limits: ResourceLimits,
    capabilities: &[CapabilityType],
    entry: &str,
    call_args: &[Val],
) -> Result<vudo_vm::sandbox::ExecutionResult> {
    if wasm_bytes.len() < 8 || &wasm_bytes[0..4] != b"\0asm" {
        anyhow::bail!("Invalid WASM module: missing magic number");
    }

    println!(
        "  {} WASM module ({} bytes)",
        "Validated".green(),
        wasm_bytes.len()
    );

    // Local runs: the operator running the CLI grants capabilities to the Spirit
    // with an ephemeral key. Network hosting (plan H1) replaces this with the
    // node's identity.
    let operator = ed25519_dalek::SigningKey::generate(&mut rand::rngs::OsRng);
    let owner = operator.verifying_key().to_bytes();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let mut capability_set = CapabilitySet::new();
    for (i, cap) in capabilities.iter().enumerate() {
        let mut grant = CapabilityGrant::new(
            i as u64 + 1,
            *cap,
            CapabilityScope::Sandboxed,
            owner,
            owner,
            now,
            None,
            [0u8; 64],
        );
        use ed25519_dalek::Signer;
        grant.signature = operator.sign(&grant.hash_for_signing()).to_bytes();
        capability_set.add_grant(grant);
    }

    let storage = std::sync::Arc::new(vudo_vm::host::InMemoryStorage::new());
    let credit = std::sync::Arc::new(vudo_vm::host::InMemoryCreditLedger::new());
    let network = std::sync::Arc::new(vudo_vm::host::MockNetworkBackend::new());

    let mut sandbox = Sandbox::new(
        wasm_bytes,
        owner,
        limits,
        storage,
        credit,
        network,
        capability_set,
    )
    .map_err(|e| anyhow::anyhow!("Failed to create sandbox: {}", e))?;

    sandbox
        .initialize()
        .map_err(|e| anyhow::anyhow!("Failed to load Spirit: {}", e))?;

    println!("  {} {}", "Calling".cyan(), entry);
    sandbox
        .invoke(entry, call_args)
        .map_err(|e| anyhow::anyhow!("Failed to invoke `{}`: {}", entry, e))
}

fn to_capability_type(cap: &Capability) -> CapabilityType {
    match cap {
        Capability::NetworkListen => CapabilityType::NetworkListen,
        Capability::NetworkConnect => CapabilityType::NetworkConnect,
        Capability::NetworkBroadcast => CapabilityType::NetworkBroadcast,
        Capability::StorageRead => CapabilityType::StorageRead,
        Capability::StorageWrite => CapabilityType::StorageWrite,
        Capability::StorageDelete => CapabilityType::StorageDelete,
        Capability::SpawnSandbox => CapabilityType::SpawnSandbox,
        Capability::CrossSandboxCall => CapabilityType::CrossSandboxCall,
        Capability::SensorTime => CapabilityType::SensorTime,
        Capability::SensorRandom => CapabilityType::SensorRandom,
        Capability::SensorEnvironment => CapabilityType::SensorEnvironment,
        Capability::ActuatorLog => CapabilityType::ActuatorLog,
        Capability::ActuatorNotify => CapabilityType::ActuatorNotify,
        Capability::ActuatorCredit => CapabilityType::ActuatorCredit,
    }
}

/// Map a `--capabilities` value to VM capabilities. Accepts the shorthands
/// `net`/`network`, `fs`/`storage`, `log`, `time`, `random`, or any manifest
/// capability name (e.g. `storage_read`).
fn parse_cli_capability(cap: &str) -> Result<Vec<CapabilityType>> {
    use CapabilityType::*;
    Ok(match cap {
        "net" | "network" => vec![NetworkConnect, NetworkListen, NetworkBroadcast],
        "fs" | "filesystem" | "storage" => vec![StorageRead, StorageWrite, StorageDelete],
        "log" => vec![ActuatorLog],
        "time" => vec![SensorTime],
        "random" => vec![SensorRandom],
        other => {
            let parsed: Capability =
                serde_json::from_value(serde_json::Value::String(other.to_string()))
                    .map_err(|_| anyhow::anyhow!("Unknown capability: {}", other))?;
            vec![to_capability_type(&parsed)]
        }
    })
}

/// Parse trailing CLI arguments into WASM values: integers become i64, other
/// numbers f64.
fn parse_call_args(args: &[String]) -> Result<Vec<Val>> {
    args.iter()
        .map(|a| {
            if let Ok(i) = a.parse::<i64>() {
                Ok(Val::I64(i))
            } else if let Ok(f) = a.parse::<f64>() {
                Ok(Val::F64(f.to_bits()))
            } else {
                anyhow::bail!("Argument `{}` is not a number", a)
            }
        })
        .collect()
}

fn format_val(v: &Val) -> String {
    match v {
        Val::I32(x) => x.to_string(),
        Val::I64(x) => x.to_string(),
        Val::F32(x) => f32::from_bits(*x).to_string(),
        Val::F64(x) => f64::from_bits(*x).to_string(),
        other => format!("{:?}", other),
    }
}
