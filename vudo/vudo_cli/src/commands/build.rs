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

    // For now, we'll create a placeholder WASM output
    // In the real implementation, this would invoke the DOL compiler
    let output_path = args
        .output
        .unwrap_or_else(|| project_path.join(format!("{}.spirit", manifest.name)));

    // Create a minimal valid WASM module as placeholder
    let wasm_module = create_placeholder_wasm(&manifest);

    fs::write(&output_path, wasm_module)
        .with_context(|| format!("Failed to write output to {:?}", output_path))?;

    println!(
        "\n{} Built Spirit package: {:?}",
        "✓".green().bold(),
        output_path
    );

    // If emit flag is set, show intermediate representation
    if let Some(emit_type) = args.emit {
        println!(
            "\n{} {} representation:",
            "Emitting".yellow().bold(),
            emit_type
        );
        match emit_type.as_str() {
            "ast" => println!("  AST output would be shown here"),
            "hir" => println!("  HIR output would be shown here"),
            "mlir" => println!("  MLIR output would be shown here"),
            "wasm" => println!("  WASM disassembly would be shown here"),
            _ => println!("  Unknown emit type: {}", emit_type),
        }
    }

    Ok(())
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

fn create_placeholder_wasm(_manifest: &spirit_runtime::Manifest) -> Vec<u8> {
    // Create a minimal valid WASM module
    // Magic number: \0asm
    let mut wasm = vec![0x00, 0x61, 0x73, 0x6D];
    // Version: 1
    wasm.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]);

    // Type section (1 function type: () -> ())
    wasm.push(0x01); // Section ID: Type
    wasm.push(0x04); // Section size
    wasm.push(0x01); // 1 type
    wasm.push(0x60); // func type
    wasm.push(0x00); // 0 params
    wasm.push(0x00); // 0 results

    // Function section (1 function)
    wasm.push(0x03); // Section ID: Function
    wasm.push(0x02); // Section size
    wasm.push(0x01); // 1 function
    wasm.push(0x00); // Type index 0

    // Export section (export the function as "main")
    wasm.push(0x07); // Section ID: Export
    wasm.push(0x08); // Section size
    wasm.push(0x01); // 1 export
    wasm.push(0x04); // Name length
    wasm.extend_from_slice(b"main"); // Name
    wasm.push(0x00); // Export kind: function
    wasm.push(0x00); // Function index 0

    // Code section (empty function body)
    wasm.push(0x0A); // Section ID: Code
    wasm.push(0x04); // Section size
    wasm.push(0x01); // 1 function body
    wasm.push(0x02); // Body size
    wasm.push(0x00); // 0 locals
    wasm.push(0x0B); // end

    wasm
}
