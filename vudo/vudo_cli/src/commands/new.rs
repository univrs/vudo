//! `vudo new` - Create a new Spirit project

use anyhow::{Context, Result};
use clap::Args;
use colored::*;
use std::fs;
use std::path::PathBuf;

use crate::config::VudoConfig;

#[derive(Args, Debug)]
pub struct NewArgs {
    /// Name of the Spirit project
    pub name: String,

    /// Template to use (web-service, cli-tool, library)
    #[arg(short, long)]
    pub template: Option<String>,

    /// Path where to create the project (defaults to current directory)
    #[arg(short, long)]
    pub path: Option<PathBuf>,
}

pub async fn execute(args: NewArgs, _config: &VudoConfig) -> Result<()> {
    let template = args.template.as_deref().unwrap_or("basic");
    let base_path = args.path.unwrap_or_else(|| PathBuf::from("."));
    let project_path = base_path.join(&args.name);

    println!(
        "{} Spirit project '{}' with template '{}'",
        "Creating".green().bold(),
        args.name.cyan(),
        template.yellow()
    );

    // Create project directory
    fs::create_dir_all(&project_path)
        .with_context(|| format!("Failed to create directory {:?}", project_path))?;

    // Create subdirectories
    fs::create_dir_all(project_path.join("src"))?;
    fs::create_dir_all(project_path.join("tests"))?;

    // Create manifest.toml
    let manifest_content = create_manifest(&args.name, template);
    fs::write(project_path.join("manifest.toml"), manifest_content)
        .context("Failed to write manifest.toml")?;

    // Create main.dol
    let main_dol_content = create_main_dol(template);
    fs::write(project_path.join("src/main.dol"), main_dol_content)
        .context("Failed to write src/main.dol")?;

    // Create test file
    let test_content = create_test_file(template);
    fs::write(project_path.join("tests/main_test.dol"), test_content)
        .context("Failed to write tests/main_test.dol")?;

    // Create README
    let readme_content = create_readme(&args.name);
    fs::write(project_path.join("README.md"), readme_content)
        .context("Failed to write README.md")?;

    println!(
        "{} Created Spirit project at {:?}",
        "✓".green().bold(),
        project_path
    );
    println!();
    println!("Next steps:");
    println!("  cd {}", args.name);
    println!("  vudo build");
    println!("  vudo run");

    Ok(())
}

fn create_manifest(name: &str, template: &str) -> String {
    format!(
        r#"[spirit]
name = "{}"
version = "0.1.0"
description = "A VUDO Spirit created from {} template"
author = "Your Name <you@example.com>"

[capabilities]
# Capabilities required by this Spirit
# compute = true
# memory = true
# network = false
# filesystem = false

[dependencies]
# Add Spirit dependencies here
# example = "1.0.0"

[build]
target = "wasm32"
optimization = "release"
"#,
        name, template
    )
}

fn create_main_dol(template: &str) -> String {
    match template {
        "web-service" => {
            r#"// VUDO Spirit - Web Service Template
// This Spirit demonstrates a simple web service

use std::net::http

gene AppState {
    has port: UInt64
    has host: String
}

fun main() -> Result<Unit, String> {
    let state = AppState {
        port: 8080,
        host: "0.0.0.0"
    }

    println("Starting web service on {}:{}", state.host, state.port)

    // TODO: Implement your web service logic here

    Ok(())
}
"#
        }
        "cli-tool" => {
            r#"// VUDO Spirit - CLI Tool Template
// This Spirit demonstrates a command-line tool

gene CliArgs {
    has verbose: Bool
    has input: String
}

fun parse_args() -> CliArgs {
    // TODO: Implement argument parsing
    CliArgs {
        verbose: false,
        input: "example.txt"
    }
}

fun main() -> Result<Unit, String> {
    let args = parse_args()

    if args.verbose {
        println("Running in verbose mode")
    }

    println("Processing input: {}", args.input)

    // TODO: Implement your CLI logic here

    Ok(())
}
"#
        }
        "library" => {
            r#"// VUDO Spirit - Library Template
// This Spirit demonstrates a reusable library

// Public API of this library
fun add(a: Int64, b: Int64) -> Int64 {
    a + b
}

fun multiply(a: Int64, b: Int64) -> Int64 {
    a * b
}

gene Point {
    has x: Float64
    has y: Float64
}

fun distance(p1: Point, p2: Point) -> Float64 {
    let dx = p2.x - p1.x
    let dy = p2.y - p1.y
    (dx * dx + dy * dy).sqrt()
}

// Example usage in main
fun main() -> Result<Unit, String> {
    let result = add(5, 3)
    println("5 + 3 = {}", result)

    let p1 = Point { x: 0.0, y: 0.0 }
    let p2 = Point { x: 3.0, y: 4.0 }
    let dist = distance(p1, p2)
    println("Distance: {}", dist)

    Ok(())
}
"#
        }
        _ => {
            // Basic template
            r#"// VUDO Spirit - Basic Template
// The system that knows what it is, becomes what it knows.

fun greet(name: String) -> String {
    "Hello, " + name + "!"
}

fun main() -> Result<Unit, String> {
    let greeting = greet("World")
    println(greeting)

    Ok(())
}
"#
        }
    }
    .to_string()
}

fn create_test_file(_template: &str) -> String {
    r#"// Tests for the Spirit
// Run with: vudo test

#[test]
fun test_basic() {
    assert(true, "Basic test should pass")
}

#[test]
fun test_greeting() {
    let result = greet("VUDO")
    assert(result == "Hello, VUDO!", "Greeting should match")
}
"#
    .to_string()
}

fn create_readme(name: &str) -> String {
    format!(
        r#"# {}

A VUDO Spirit project.

## Building

```bash
vudo build
```

## Running

```bash
vudo run
```

## Testing

```bash
vudo test
```

## Publishing

```bash
vudo pack
vudo sign
vudo publish
```

## Learn More

- [VUDO Documentation](https://docs.vudo.univrs.io)
- [DOL Language Guide](https://docs.vudo.univrs.io/dol)
- [Spirit Development](https://docs.vudo.univrs.io/spirits)
"#,
        name
    )
}
