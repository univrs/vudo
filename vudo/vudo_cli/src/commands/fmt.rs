//! DOL file formatting command

use anyhow::{Context, Result};
use clap::Args;
use colored::Colorize;
use std::path::PathBuf;

use crate::config::VudoConfig;

#[derive(Args, Debug)]
pub struct FmtArgs {
    /// File or directory to format (defaults to current project)
    #[arg(value_name = "PATH")]
    pub path: Option<PathBuf>,

    /// Check formatting without modifying files (CI mode)
    #[arg(long)]
    pub check: bool,
}

pub async fn execute(args: FmtArgs, _config: &VudoConfig) -> Result<()> {
    run(args).await
}

async fn run(args: FmtArgs) -> Result<()> {
    let path = args.path.unwrap_or_else(|| PathBuf::from("."));

    if args.check {
        run_check_mode(&path).await
    } else {
        run_format_mode(&path).await
    }
}

async fn run_format_mode(path: &PathBuf) -> Result<()> {
    println!("{}", "Formatting DOL files...".cyan().bold());
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

    let mut formatted_count = 0;
    let mut unchanged_count = 0;
    let mut errors = 0;

    for file in &dol_files {
        let relative_path = file.strip_prefix(std::env::current_dir()?).unwrap_or(file);
        print!("  {} {}... ", "Formatting".cyan(), relative_path.display());

        match std::fs::read_to_string(file) {
            Ok(content) => {
                // For now, we just trim trailing whitespace and ensure newline at EOF
                let formatted = format_dol_content(&content);

                if formatted != content {
                    match std::fs::write(file, formatted) {
                        Ok(_) => {
                            println!("{}", "FORMATTED".green());
                            formatted_count += 1;
                        }
                        Err(e) => {
                            println!("{}", "ERROR".red());
                            println!("    {} Failed to write file: {}", "→".red(), e);
                            errors += 1;
                        }
                    }
                } else {
                    println!("{}", "OK".dimmed());
                    unchanged_count += 1;
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

    println!(
        "{} {} file(s) formatted",
        "✓".green().bold(),
        formatted_count
    );
    println!(
        "{} {} file(s) already formatted",
        "•".dimmed(),
        unchanged_count
    );

    if errors > 0 {
        println!("{} {} error(s)", "✗".red().bold(), errors);
    }

    println!();
    println!(
        "{}",
        "Note: Full DOL formatter integration coming soon."
            .dimmed()
            .italic()
    );

    if errors > 0 {
        anyhow::bail!("Formatting failed with {} error(s)", errors);
    }

    Ok(())
}

async fn run_check_mode(path: &PathBuf) -> Result<()> {
    println!("{}", "Checking DOL file formatting...".cyan().bold());
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

    let mut needs_formatting = Vec::new();
    let mut errors = 0;

    for file in &dol_files {
        let relative_path = file.strip_prefix(std::env::current_dir()?).unwrap_or(file);
        print!("  {} {}... ", "Checking".cyan(), relative_path.display());

        match std::fs::read_to_string(file) {
            Ok(content) => {
                let formatted = format_dol_content(&content);

                if formatted != content {
                    println!("{}", "NEEDS FORMATTING".yellow());
                    needs_formatting.push(relative_path.to_path_buf());
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

    if needs_formatting.is_empty() && errors == 0 {
        println!("{} All files are properly formatted!", "✓".green().bold());
    } else {
        if !needs_formatting.is_empty() {
            println!(
                "{} {} file(s) need formatting:",
                "⚠".yellow().bold(),
                needs_formatting.len()
            );
            for file in &needs_formatting {
                println!("    {}", file.display());
            }
            println!();
            println!("Run {} to format these files.", "vudo fmt".cyan());
        }

        if errors > 0 {
            println!("{} {} error(s)", "✗".red().bold(), errors);
        }
    }

    println!();
    println!(
        "{}",
        "Note: Full DOL formatter integration coming soon."
            .dimmed()
            .italic()
    );

    if !needs_formatting.is_empty() || errors > 0 {
        anyhow::bail!("Formatting check failed");
    }

    Ok(())
}

fn format_dol_content(content: &str) -> String {
    // Basic formatting for now:
    // 1. Trim trailing whitespace from each line
    // 2. Ensure single newline at end of file
    // 3. Convert CRLF to LF

    let mut lines: Vec<String> = content
        .replace("\r\n", "\n")
        .lines()
        .map(|line| line.trim_end().to_string())
        .collect();

    // Remove trailing empty lines
    while lines.last().is_some_and(|line| line.is_empty()) {
        lines.pop();
    }

    // Add single newline at EOF if content is not empty
    if !lines.is_empty() {
        lines.push(String::new());
    }

    lines.join("\n")
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
