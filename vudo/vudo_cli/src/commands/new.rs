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

pub async fn execute(args: NewArgs, config: &VudoConfig) -> Result<()> {
    let template = args.template.as_deref().unwrap_or("basic");
    let base_path = args.path.unwrap_or_else(|| PathBuf::from("."));
    let project_path = base_path.join(&args.name);

    println!(
        "{} Spirit project '{}' with template '{}'",
        "Creating".green().bold(),
        args.name.cyan(),
        template.yellow()
    );

    // Author = public key of the developer's default signing key (the one
    // `vudo sign` uses), so nodes that require author == signer accept it.
    // Resolved before anything is written: a bad key leaves no half project.
    let key_file = super::sign::default_key_path(config);
    let signing_key = super::sign::load_or_create_signing_key(&key_file)
        .with_context(|| format!("Cannot load the default signing key {:?}", key_file))?;
    let author = hex::encode(signing_key.verifying_key().as_bytes());

    // Create project directory
    fs::create_dir_all(&project_path)
        .with_context(|| format!("Failed to create directory {:?}", project_path))?;

    // Create subdirectories
    fs::create_dir_all(project_path.join("src"))?;
    fs::create_dir_all(project_path.join("tests"))?;

    // Create manifest.toml
    let manifest_content = create_manifest(&args.name, template, &author);
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
    println!("  vudo run {}.spirit --capabilities log", args.name);

    Ok(())
}

fn create_manifest(name: &str, template: &str, author: &str) -> String {
    // Format: spirit_runtime::Manifest (what `vudo build` and nodes parse).
    format!(
        r#"name = "{name}"
version = {{ major = 0, minor = 1, patch = 0 }}
# Ed25519 public key of your default signing key (~/.config/vudo/keys/default.key).
# `vudo sign` signs with the same key; nodes require author == signer.
author = "{author}"
description = "A VUDO Spirit created from the {template} template"

[pricing]
base_cost = 100
per_fuel_cost = 1
"#
    )
}

fn create_main_dol(template: &str) -> String {
    // Every template must compile with the pinned DOL WASM backend and run in
    // the VUDO VM (oracle: test_new_then_build_integration). Host functions
    // come from the VM's `vudo` module; `sex` marks side effects.
    match template {
        "web-service" => {
            r#"// VUDO Spirit - Web Service Template
// Spirits reach the network through VM host functions (on DOLOS nodes,
// host_network_broadcast). This template logs its start and returns its port.
//   vudo run <name>.spirit --capabilities log

sex fun host_log(level: i32, msg: string) -> i32

fun port() -> i64 {
    return 8080
}

sex fun main() -> i64 {
    host_log(2, "Web service Spirit starting")
    return port()
}
"#
        }
        "cli-tool" => {
            r#"// VUDO Spirit - CLI Tool Template
// parse_args picks a mode; main runs it.
//   vudo run <name>.spirit --capabilities log

sex fun host_log(level: i32, msg: string) -> i32

fun parse_args() -> i64 {
    // TODO: read real arguments. 1 = verbose, 0 = quiet.
    return 1
}

sex fun main() -> i64 {
    let verbose = parse_args()
    if verbose == 1 {
        return host_log(2, "Running in verbose mode")
    }
    return host_log(2, "Running quietly")
}
"#
        }
        "library" => {
            r#"// VUDO Spirit - Library Template
// Pure functions other Spirits can reuse.

fun add(a: i64, b: i64) -> i64 {
    return a + b
}

fun multiply(a: i64, b: i64) -> i64 {
    return a * b
}

fun square(x: f64) -> f64 {
    return x * x
}

fun main() -> i64 {
    return add(5, multiply(3, 1))
}
"#
        }
        _ => {
            // Basic template
            r#"// VUDO Spirit - Basic Template
// The system that knows what it is, becomes what it knows.
//
// host_log is a VUDO VM host function. Grant it when you run:
//   vudo run <name>.spirit --capabilities log

sex fun host_log(level: i32, msg: string) -> i32

sex fun main() -> i64 {
    return host_log(2, "Hello from your Spirit!")
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
