//! Documentation generation command

use anyhow::{Context, Result};
use clap::Args;
use colored::Colorize;
use std::path::{Path, PathBuf};

use crate::config::VudoConfig;

#[derive(Args, Debug)]
pub struct DocArgs {
    /// Open documentation in browser after generating
    #[arg(long)]
    pub open: bool,

    /// Output format
    #[arg(long, value_name = "FORMAT", default_value = "html")]
    pub format: DocFormat,

    /// Output directory (defaults to ./docs)
    #[arg(short, long, value_name = "DIR")]
    pub output: Option<PathBuf>,
}

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum DocFormat {
    /// Generate HTML documentation
    Html,
    /// Generate Markdown documentation
    Markdown,
    /// Generate JSON schema
    Json,
}

pub async fn execute(args: DocArgs, _config: &VudoConfig) -> Result<()> {
    run(args).await
}

async fn run(args: DocArgs) -> Result<()> {
    println!("{}", "Generating documentation...".cyan().bold());
    println!();

    let output_dir = args.output.unwrap_or_else(|| PathBuf::from("docs"));

    println!("{} {:?}", "Format:".bold(), args.format);
    println!("{} {}", "Output:".bold(), output_dir.display());
    println!();

    // Read manifest.toml if it exists
    let manifest_path = PathBuf::from("manifest.toml");
    let project_info = if manifest_path.exists() {
        read_manifest(&manifest_path)?
    } else {
        println!(
            "{}",
            "Warning: manifest.toml not found, using defaults".yellow()
        );
        ProjectInfo::default()
    };

    // Create output directory
    std::fs::create_dir_all(&output_dir).context("Failed to create output directory")?;

    match args.format {
        DocFormat::Html => generate_html(&output_dir, &project_info).await?,
        DocFormat::Markdown => generate_markdown(&output_dir, &project_info).await?,
        DocFormat::Json => generate_json(&output_dir, &project_info).await?,
    }

    println!();
    println!("{}", "─".repeat(60).dimmed());
    println!(
        "{} Documentation generated successfully!",
        "✓".green().bold()
    );
    println!();
    println!("{} {}", "Output directory:".bold(), output_dir.display());

    if args.open {
        let index_file = match args.format {
            DocFormat::Html => output_dir.join("index.html"),
            DocFormat::Markdown => output_dir.join("README.md"),
            DocFormat::Json => output_dir.join("schema.json"),
        };

        if index_file.exists() {
            println!();
            println!("{}", "Opening documentation in browser...".cyan());
            open_in_browser(&index_file)?;
        }
    } else {
        println!();
        println!("Run {} to open in browser", "vudo doc --open".cyan());
    }

    println!();
    println!(
        "{}",
        "Note: Full DOL documentation generator coming soon."
            .dimmed()
            .italic()
    );

    Ok(())
}

async fn generate_html(output_dir: &Path, info: &ProjectInfo) -> Result<()> {
    println!("{} HTML documentation...", "Generating".cyan());

    let index_html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{} - Documentation</title>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
            max-width: 900px;
            margin: 0 auto;
            padding: 2rem;
            line-height: 1.6;
            color: #333;
        }}
        h1 {{
            color: #2c3e50;
            border-bottom: 3px solid #3498db;
            padding-bottom: 0.5rem;
        }}
        h2 {{
            color: #34495e;
            margin-top: 2rem;
        }}
        .meta {{
            background: #f8f9fa;
            padding: 1rem;
            border-radius: 5px;
            margin: 1rem 0;
        }}
        .meta-item {{
            margin: 0.5rem 0;
        }}
        .label {{
            font-weight: bold;
            color: #555;
        }}
        .note {{
            background: #fff3cd;
            border-left: 4px solid #ffc107;
            padding: 1rem;
            margin: 1rem 0;
        }}
        code {{
            background: #f4f4f4;
            padding: 0.2rem 0.4rem;
            border-radius: 3px;
            font-family: "Courier New", monospace;
        }}
    </style>
</head>
<body>
    <h1>{}</h1>
    
    <div class="meta">
        <div class="meta-item"><span class="label">Version:</span> {}</div>
        <div class="meta-item"><span class="label">Authors:</span> {}</div>
        <div class="meta-item"><span class="label">Description:</span> {}</div>
    </div>

    <h2>About</h2>
    <p>{}</p>

    <h2>Getting Started</h2>
    <p>To use this Spirit:</p>
    <pre><code>vudo run {}</code></pre>

    <h2>API Reference</h2>
    <p>API documentation will be automatically generated from DOL source files.</p>

    <div class="note">
        <strong>Note:</strong> Full documentation generation from DOL source coming soon.
        This is a basic structure generated from manifest.toml.
    </div>

    <hr>
    <p style="color: #888; font-size: 0.9rem;">
        Generated by VUDO CLI v0.1.0
    </p>
</body>
</html>
"#,
        info.name,
        info.name,
        info.version,
        info.authors.join(", "),
        info.description,
        info.description,
        info.name,
    );

    let index_path = output_dir.join("index.html");
    std::fs::write(&index_path, index_html).context("Failed to write index.html")?;

    println!("  {} index.html", "Created".green());

    Ok(())
}

async fn generate_markdown(output_dir: &Path, info: &ProjectInfo) -> Result<()> {
    println!("{} Markdown documentation...", "Generating".cyan());

    let readme_md = format!(
        r#"# {}

**Version:** {}  
**Authors:** {}

## Description

{}

## Getting Started

To use this Spirit:

```bash
vudo run {}
```

## API Reference

API documentation will be automatically generated from DOL source files.

---

**Note:** Full documentation generation from DOL source coming soon.
This is a basic structure generated from manifest.toml.

---

*Generated by VUDO CLI v0.1.0*
"#,
        info.name,
        info.version,
        info.authors.join(", "),
        info.description,
        info.name,
    );

    let readme_path = output_dir.join("README.md");
    std::fs::write(&readme_path, readme_md).context("Failed to write README.md")?;

    println!("  {} README.md", "Created".green());

    Ok(())
}

async fn generate_json(output_dir: &Path, info: &ProjectInfo) -> Result<()> {
    println!("{} JSON schema...", "Generating".cyan());

    let schema = serde_json::json!({
        "name": info.name,
        "version": info.version,
        "authors": info.authors,
        "description": info.description,
        "api": {
            "note": "Full API schema will be generated from DOL source files"
        },
        "generated_by": "VUDO CLI v0.1.0"
    });

    let schema_path = output_dir.join("schema.json");
    std::fs::write(&schema_path, serde_json::to_string_pretty(&schema)?)
        .context("Failed to write schema.json")?;

    println!("  {} schema.json", "Created".green());

    Ok(())
}

#[derive(Debug, Clone)]
struct ProjectInfo {
    name: String,
    version: String,
    authors: Vec<String>,
    description: String,
}

impl Default for ProjectInfo {
    fn default() -> Self {
        Self {
            name: "unnamed-spirit".to_string(),
            version: "0.1.0".to_string(),
            authors: vec!["Unknown".to_string()],
            description: "A VUDO Spirit".to_string(),
        }
    }
}

fn read_manifest(path: &Path) -> Result<ProjectInfo> {
    let content = std::fs::read_to_string(path).context("Failed to read manifest.toml")?;

    let manifest: toml::Value =
        toml::from_str(&content).context("Failed to parse manifest.toml")?;

    let name = manifest
        .get("spirit")
        .and_then(|s| s.get("name"))
        .and_then(|n| n.as_str())
        .unwrap_or("unnamed-spirit")
        .to_string();

    let version = manifest
        .get("spirit")
        .and_then(|s| s.get("version"))
        .and_then(|v| v.as_str())
        .unwrap_or("0.1.0")
        .to_string();

    let authors = manifest
        .get("spirit")
        .and_then(|s| s.get("authors"))
        .and_then(|a| a.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect()
        })
        .unwrap_or_else(|| vec!["Unknown".to_string()]);

    let description = manifest
        .get("spirit")
        .and_then(|s| s.get("description"))
        .and_then(|d| d.as_str())
        .unwrap_or("A VUDO Spirit")
        .to_string();

    Ok(ProjectInfo {
        name,
        version,
        authors,
        description,
    })
}

fn open_in_browser(path: &Path) -> Result<()> {
    let path_str = path.to_string_lossy();

    #[cfg(target_os = "macos")]
    std::process::Command::new("open")
        .arg(path_str.as_ref())
        .spawn()
        .context("Failed to open browser")?;

    #[cfg(target_os = "linux")]
    std::process::Command::new("xdg-open")
        .arg(path_str.as_ref())
        .spawn()
        .context("Failed to open browser")?;

    #[cfg(target_os = "windows")]
    std::process::Command::new("cmd")
        .args(&["/C", "start", path_str.as_ref()])
        .spawn()
        .context("Failed to open browser")?;

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        println!(
            "{}",
            "Automatic browser opening not supported on this platform.".yellow()
        );
        println!("Please open manually: {}", path.display());
    }

    Ok(())
}
