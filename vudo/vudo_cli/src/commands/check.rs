//! DOL syntax and type validation command

use anyhow::{Context, Result};
use clap::Args;
use colored::Colorize;
use std::path::PathBuf;

use crate::config::VudoConfig;

#[derive(Args, Debug)]
pub struct CheckArgs {
    /// File or directory to check (defaults to current project)
    #[arg(value_name = "PATH")]
    pub path: Option<PathBuf>,

    /// Enable strict type checking
    #[arg(long)]
    pub strict: bool,

    /// Output format
    #[arg(long, value_name = "FORMAT", default_value = "pretty")]
    pub format: OutputFormat,
}

#[derive(Debug, Clone, PartialEq, clap::ValueEnum)]
pub enum OutputFormat {
    /// Human-readable output with colors
    Pretty,
    /// JSON output for tooling
    Json,
}

pub async fn execute(args: CheckArgs, _config: &VudoConfig) -> Result<()> {
    let path = args.path.unwrap_or_else(|| PathBuf::from("."));

    if args.format == OutputFormat::Json {
        run_json_check(&path, args.strict).await
    } else {
        run_pretty_check(&path, args.strict).await
    }
}

async fn run_pretty_check(path: &PathBuf, strict: bool) -> Result<()> {
    println!("{}", "Checking DOL files...".cyan().bold());
    println!();

    if strict {
        println!("{} {}", "Mode:".bold(), "strict type checking".yellow());
    } else {
        println!("{} standard type checking", "Mode:".bold());
    }

    println!("{} {}", "Path:".bold(), path.display());
    println!();

    // Check if path exists
    if !path.exists() {
        anyhow::bail!("Path does not exist: {}", path.display());
    }

    // Collect .dol files
    let dol_files = collect_dol_files(path)?;

    if dol_files.is_empty() {
        println!("{}", "No .dol files found.".yellow());
        return Ok(());
    }

    println!("{} {} DOL file(s)", "Found:".bold(), dol_files.len());
    println!();

    // Check each file
    let mut errors = 0;
    let mut warnings = 0;

    for file in &dol_files {
        let relative_path = file.strip_prefix(std::env::current_dir()?).unwrap_or(file);
        print!("  {} {}... ", "Checking".cyan(), relative_path.display());

        // For now, just verify the file can be read
        match std::fs::read_to_string(file) {
            Ok(content) => {
                // Basic validation: check for empty files
                if content.trim().is_empty() {
                    println!("{}", "WARN".yellow());
                    println!("    {} File is empty", "→".yellow());
                    warnings += 1;
                } else {
                    println!("{}", "OK".green());
                }
            }
            Err(e) => {
                println!("{}", "ERROR".red());
                println!("    {} Failed to read file: {}", "→".red(), e);
                errors += 1;
            }
        }
    }

    println!();
    println!("{}", "─".repeat(60).dimmed());

    if errors == 0 && warnings == 0 {
        println!("{} All checks passed!", "✓".green().bold());
    } else {
        if errors > 0 {
            println!("{} {} error(s) found", "✗".red().bold(), errors);
        }
        if warnings > 0 {
            println!("{} {} warning(s) found", "⚠".yellow().bold(), warnings);
        }
    }

    println!();
    println!(
        "{}",
        "Note: Full DOL parser integration coming soon."
            .dimmed()
            .italic()
    );

    if errors > 0 {
        anyhow::bail!("Check failed with {} error(s)", errors);
    }

    Ok(())
}

async fn run_json_check(path: &PathBuf, strict: bool) -> Result<()> {
    // Check if path exists
    if !path.exists() {
        let output = serde_json::json!({
            "success": false,
            "error": format!("Path does not exist: {}", path.display()),
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
        anyhow::bail!("Path does not exist");
    }

    let dol_files = collect_dol_files(path)?;

    let mut file_results = Vec::new();
    let mut total_errors = 0;
    let mut total_warnings = 0;

    for file in &dol_files {
        let relative_path = file.strip_prefix(std::env::current_dir()?).unwrap_or(file);

        match std::fs::read_to_string(file) {
            Ok(content) => {
                if content.trim().is_empty() {
                    file_results.push(serde_json::json!({
                        "file": relative_path.to_string_lossy(),
                        "status": "warning",
                        "messages": ["File is empty"],
                    }));
                    total_warnings += 1;
                } else {
                    file_results.push(serde_json::json!({
                        "file": relative_path.to_string_lossy(),
                        "status": "ok",
                        "messages": [],
                    }));
                }
            }
            Err(e) => {
                file_results.push(serde_json::json!({
                    "file": relative_path.to_string_lossy(),
                    "status": "error",
                    "messages": [format!("Failed to read file: {}", e)],
                }));
                total_errors += 1;
            }
        }
    }

    let output = serde_json::json!({
        "success": total_errors == 0,
        "mode": if strict { "strict" } else { "standard" },
        "path": path.to_string_lossy(),
        "files_checked": dol_files.len(),
        "errors": total_errors,
        "warnings": total_warnings,
        "results": file_results,
        "note": "Full DOL parser integration coming soon",
    });

    println!("{}", serde_json::to_string_pretty(&output)?);

    if total_errors > 0 {
        anyhow::bail!("Check failed with {} error(s)", total_errors);
    }

    Ok(())
}

fn collect_dol_files(path: &PathBuf) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    if path.is_file() {
        if path.extension().and_then(|s| s.to_str()) == Some("dol") {
            files.push(path.clone());
        }
    } else if path.is_dir() {
        walk_dir(path, &mut files)?;
    }

    Ok(files)
}

fn walk_dir(dir: &PathBuf, files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in std::fs::read_dir(dir).context("Failed to read directory")? {
        let entry = entry.context("Failed to read directory entry")?;
        let path = entry.path();

        if path.is_dir() {
            walk_dir(&path, files)?;
        } else if path.extension().and_then(|s| s.to_str()) == Some("dol") {
            files.push(path);
        }
    }

    Ok(())
}
