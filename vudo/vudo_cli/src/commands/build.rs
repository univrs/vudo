//! `vudo build` - Compile DOL source to Spirit package

use anyhow::{Context, Result};
use clap::Args;
use colored::*;
use std::fs;
use std::path::PathBuf;

use crate::config::VudoConfig;

#[derive(Args, Debug)]
pub struct BuildArgs {
    /// Path to the Spirit project (defaults to current directory)
    #[arg(short, long)]
    pub path: Option<PathBuf>,

    /// Emit intermediate representation (ast, hir, mlir, wasm)
    #[arg(long)]
    pub emit: Option<String>,

    /// Target architecture (wasm32, wasm64, native)
    #[arg(long, default_value = "wasm32")]
    pub target: String,

    /// Build in release mode with optimizations
    #[arg(short, long)]
    pub release: bool,

    /// Enable specific features
    #[arg(long)]
    pub features: Option<Vec<String>>,

    /// Output file path
    #[arg(short, long)]
    pub output: Option<PathBuf>,
}

pub async fn execute(args: BuildArgs, _config: &VudoConfig) -> Result<()> {
    let project_path = args.path.unwrap_or_else(|| PathBuf::from("."));
    let manifest_path = project_path.join("manifest.toml");

    println!(
        "{} Spirit project at {:?}",
        "Building".green().bold(),
        project_path
    );

    // Load and parse manifest
    let manifest_content = fs::read_to_string(&manifest_path)
        .with_context(|| format!("Failed to read manifest at {:?}", manifest_path))?;

    let manifest: spirit_runtime::Manifest =
        toml::from_str(&manifest_content).context("Failed to parse manifest.toml")?;

    println!("  {} {}", "Spirit:".cyan(), manifest.name);
    println!("  {} {}", "Version:".cyan(), manifest.version);
    println!("  {} {}", "Target:".cyan(), args.target);

    if args.release {
        println!("  {} {}", "Mode:".cyan(), "release".yellow());
    } else {
        println!("  {} {}", "Mode:".cyan(), "debug".yellow());
    }

    // Find DOL source files
    let src_path = project_path.join("src");
    let dol_files = find_dol_files(&src_path)?;

    println!("\n{} DOL source files:", "Compiling".green().bold());
    for file in &dol_files {
        println!("  - {:?}", file.strip_prefix(&project_path).unwrap_or(file));
    }

    // The entry point is src/main.dol (the Spirit project convention).
    let entry = src_path.join("main.dol");
    if !entry.exists() {
        anyhow::bail!("Spirit entry point not found at {:?}", entry);
    }

    let source = fs::read_to_string(&entry)
        .with_context(|| format!("Failed to read DOL source {:?}", entry))?;

    let (ast, wasm_module) =
        compile_dol(&source).with_context(|| format!("DOL compilation failed for {:?}", entry))?;

    let output_path = args
        .output
        .unwrap_or_else(|| project_path.join(format!("{}.spirit", manifest.name)));

    fs::write(&output_path, &wasm_module)
        .with_context(|| format!("Failed to write output to {:?}", output_path))?;

    println!(
        "\n{} Built Spirit package: {:?} ({} bytes)",
        "✓".green().bold(),
        output_path,
        wasm_module.len()
    );

    // If emit flag is set, show intermediate representation
    if let Some(emit_type) = args.emit {
        println!(
            "\n{} {} representation:",
            "Emitting".yellow().bold(),
            emit_type
        );
        match emit_type.as_str() {
            "ast" => println!("{:#?}", ast),
            "wasm" => println!("  {} bytes of WASM bytecode", wasm_module.len()),
            other => anyhow::bail!("--emit {} is not supported (supported: ast, wasm)", other),
        }
    }

    Ok(())
}

/// Parse DOL source and compile it to WASM with the real DOL compiler.
///
/// Mirrors `metadol::compile_source` from univrs/dol main (dol#31). The
/// published 0.8.1 `compile_source` still emits a placeholder module, so the
/// parser and `WasmCompiler` are called directly.
fn compile_dol(source: &str) -> Result<(metadol::DolFile, Vec<u8>)> {
    let ast = metadol::parse_dol_file(source).map_err(|e| anyhow::anyhow!("parse error: {}", e))?;
    let wasm = metadol::WasmCompiler::new()
        .compile_file(&ast)
        .map_err(|e| anyhow::anyhow!("WASM codegen error: {}", e))?;
    Ok((ast, wasm))
}

fn find_dol_files(dir: &PathBuf) -> Result<Vec<PathBuf>> {
    let mut dol_files = Vec::new();

    if !dir.exists() {
        return Ok(dol_files);
    }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("dol") {
            dol_files.push(path);
        } else if path.is_dir() {
            dol_files.extend(find_dol_files(&path)?);
        }
    }

    Ok(dol_files)
}
