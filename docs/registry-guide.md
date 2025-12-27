# Registry Guide

This guide covers the local Spirit registry for installing, managing, and retrieving Spirit packages.

## Overview

The VUDO registry provides:
- **Local storage** at `~/.vudo/registry/`
- **Version management** with semantic versioning
- **Search capabilities** by name, author, and capabilities
- **WASM retrieval** for VM execution

## Directory Structure

```
~/.vudo/registry/
├── index.json              # Registry metadata
├── spirits/                # Installed Spirit packages
│   ├── my-spirit/
│   │   ├── 1.0.0/
│   │   │   ├── manifest.json
│   │   │   ├── spirit.wasm
│   │   │   └── assets/     # Optional assets
│   │   ├── 1.1.0/
│   │   │   ├── manifest.json
│   │   │   └── spirit.wasm
│   │   └── latest -> 1.1.0/   # Symlink (Unix)
│   └── other-spirit/
│       └── ...
└── cache/                  # Downloaded packages (future)
```

## Initialization

Before using the registry, initialize it:

```rust
use spirit_runtime::registry::{LocalRegistry, Registry};

// Default location (~/.vudo/registry/)
let mut registry = LocalRegistry::new();

// Custom location
let mut registry = LocalRegistry::with_root("/custom/path");

// Initialize (creates directories and index)
registry.init().await?;
```

## Installing Spirits

### From Local Directory

A Spirit package directory must contain:
- `manifest.json` or `manifest.toml`
- `spirit.wasm`
- (optional) `assets/` directory

```rust
use spirit_runtime::registry::{LocalRegistry, Registry};

let mut registry = LocalRegistry::new();
registry.init().await?;

// Install from a local directory
let installed = registry.install("./my-spirit/").await?;

println!("Installed: {} v{}", installed.name, installed.latest);
println!("Versions: {:?}", installed.versions);
```

### Installing Multiple Versions

```rust
// Install version 1.0.0
registry.install("./my-spirit-v1/").await?;

// Install version 1.1.0
registry.install("./my-spirit-v1.1/").await?;

// Both versions are now available
let all = registry.list().await?;
for spirit in all {
    println!("{}: {:?}", spirit.name, spirit.versions);
}
```

### Bulk Installation

```rust
use spirit_runtime::registry::RegistryExt;

let sources = vec![
    "./spirit-one/",
    "./spirit-two/",
    "./spirit-three/",
];

let results = registry.install_all(&sources).await?;
for installed in results {
    println!("Installed {}", installed.name);
}
```

## Retrieving Spirits

### Get Latest Version

```rust
let result = registry.get("my-spirit").await?;

println!("Name: {}", result.name);
println!("Version: {}", result.version);
println!("Path: {:?}", result.path);
println!("Description: {:?}", result.manifest.description);
```

### Get Specific Version

```rust
let result = registry.get_version("my-spirit", "1.0.0").await?;
```

### Get All Versions

```rust
use spirit_runtime::registry::RegistryExt;

let versions = registry.get_all_versions("my-spirit").await?;
for result in versions {
    println!("  {} - {}", result.version, result.path.display());
}
```

### Get WASM Bytes

```rust
// Get latest version
let wasm: Vec<u8> = registry.get_wasm("my-spirit", None).await?;

// Get specific version
let wasm: Vec<u8> = registry.get_wasm("my-spirit", Some("1.0.0")).await?;

// Verify WASM magic bytes
assert_eq!(&wasm[0..4], b"\0asm");
```

### Get Manifest

```rust
// Get latest
let manifest = registry.get_manifest("my-spirit", None).await?;

// Get specific version
let manifest = registry.get_manifest("my-spirit", Some("1.0.0")).await?;
```

## Searching

### Basic Search

```rust
use spirit_runtime::registry::SpiritQuery;

// Search by name (partial match)
let query = SpiritQuery::new().with_name("network");
let results = registry.search(&query).await?;

for result in results {
    println!("{} v{}", result.name, result.version);
}
```

### Filter by Author

```rust
let query = SpiritQuery::new()
    .with_author("abcd1234...");  // Partial match on public key

let results = registry.search(&query).await?;
```

### Filter by Capability

```rust
let query = SpiritQuery::new()
    .with_capability("network_connect")
    .with_capability("storage_read");

let results = registry.search(&query).await?;
```

### Combined Filters

```rust
let query = SpiritQuery::new()
    .with_name("api")
    .with_author("trusted-author-key")
    .with_capability("network_connect");

let results = registry.search(&query).await?;
```

### Using QueryBuilder

```rust
use spirit_runtime::registry::QueryBuilder;

let results = QueryBuilder::new()
    .name_contains("spirit")
    .with_author("abcd")
    .build()
    .search(&registry)
    .await?;
```

## Listing Spirits

```rust
// List all installed spirits
let spirits = registry.list().await?;

for spirit in spirits {
    println!("{}", spirit.name);
    println!("  Latest: {}", spirit.latest);
    println!("  Versions: {:?}", spirit.versions);
    println!("  Installed: {}", spirit.installed_at);
    println!("  Source: {:?}", spirit.source);
}
```

## Checking Installation Status

```rust
// Check if any version is installed
if registry.is_installed("my-spirit") {
    println!("Spirit is installed");
}

// Check specific version
if registry.is_version_installed("my-spirit", "1.0.0") {
    println!("Version 1.0.0 is installed");
}
```

## Uninstalling

### Remove All Versions

```rust
registry.uninstall("old-spirit").await?;
println!("Removed all versions of old-spirit");
```

### Remove Specific Version

```rust
registry.uninstall_version("my-spirit", "0.9.0").await?;
println!("Removed version 0.9.0");

// If this was the last version, the entire spirit is removed
```

## Registry Types

### InstalledSpirit

```rust
use spirit_runtime::registry::InstalledSpirit;

pub struct InstalledSpirit {
    /// Package name
    pub name: String,

    /// All installed versions
    pub versions: Vec<String>,

    /// Latest version string
    pub latest: String,

    /// Unix timestamp of installation
    pub installed_at: u64,

    /// Where the Spirit was installed from
    pub source: InstallSource,
}
```

### InstallSource

```rust
use spirit_runtime::registry::InstallSource;

pub enum InstallSource {
    /// Installed from a local directory
    Local { path: PathBuf },

    /// Installed from a remote URL (future)
    Remote { url: String },

    /// Installed from a registry (future)
    Registry { name: String },
}
```

### SpiritSearchResult

```rust
use spirit_runtime::registry::SpiritSearchResult;

pub struct SpiritSearchResult {
    /// Package name
    pub name: String,

    /// Version string
    pub version: String,

    /// Full manifest
    pub manifest: Manifest,

    /// Path to version directory
    pub path: PathBuf,
}
```

### SpiritQuery

```rust
use spirit_runtime::registry::SpiritQuery;

pub struct SpiritQuery {
    /// Filter by name (partial match)
    pub name: Option<String>,

    /// Filter by author (partial match)
    pub author: Option<String>,

    /// Filter by required capabilities
    pub capabilities: Vec<String>,
}

impl SpiritQuery {
    pub fn new() -> Self;
    pub fn with_name(self, name: &str) -> Self;
    pub fn with_author(self, author: &str) -> Self;
    pub fn with_capability(self, capability: &str) -> Self;
}
```

## Error Handling

```rust
use spirit_runtime::registry::RegistryError;

match registry.install("./my-spirit/").await {
    Ok(installed) => println!("Success: {}", installed.name),

    Err(RegistryError::NotFound(name)) => {
        println!("Spirit '{}' not found", name);
    }

    Err(RegistryError::VersionNotFound { name, version }) => {
        println!("{}@{} not found", name, version);
    }

    Err(RegistryError::AlreadyInstalled { name, version }) => {
        println!("{}@{} already installed", name, version);
    }

    Err(RegistryError::InvalidSource(msg)) => {
        println!("Invalid source: {}", msg);
    }

    Err(RegistryError::InvalidManifest(msg)) => {
        println!("Invalid manifest: {}", msg);
    }

    Err(RegistryError::MissingWasm(msg)) => {
        println!("WASM file not found: {}", msg);
    }

    Err(e) => {
        println!("Error: {}", e);
    }
}
```

## Complete Example

```rust
use spirit_runtime::{
    Manifest, ManifestBuilder, Capability, SemVer,
    signature::KeyPair,
    registry::{LocalRegistry, Registry, SpiritQuery},
};
use std::fs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a test spirit directory
    let spirit_dir = "./test-spirit";
    fs::create_dir_all(spirit_dir)?;

    // Generate keypair and create manifest
    let keypair = KeyPair::generate();
    let mut manifest = ManifestBuilder::new(
        "test-spirit",
        SemVer::new(1, 0, 0),
        keypair.verifying_key().to_hex(),
    )
    .description("Test Spirit for registry example")
    .capability(Capability::SensorTime)
    .build();

    // Sign and save manifest
    let signature = manifest.sign(&keypair.signing_key().0)?;
    manifest.signature = Some(signature);
    manifest.to_file(format!("{}/manifest.toml", spirit_dir))?;

    // Create minimal WASM file
    let wasm = vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00];
    fs::write(format!("{}/spirit.wasm", spirit_dir), &wasm)?;

    // Initialize registry
    let mut registry = LocalRegistry::new();
    registry.init().await?;

    // Install the spirit
    let installed = registry.install(spirit_dir).await?;
    println!("Installed: {} v{}", installed.name, installed.latest);

    // Search for it
    let query = SpiritQuery::new().with_name("test");
    let results = registry.search(&query).await?;
    println!("Found {} spirits matching 'test'", results.len());

    // Get the WASM bytes
    let wasm_bytes = registry.get_wasm("test-spirit", None).await?;
    println!("WASM size: {} bytes", wasm_bytes.len());

    // List all spirits
    let all = registry.list().await?;
    println!("\nInstalled spirits:");
    for spirit in all {
        println!("  {} (latest: {})", spirit.name, spirit.latest);
    }

    // Cleanup test files
    fs::remove_dir_all(spirit_dir)?;

    Ok(())
}
```

## Best Practices

### 1. Always Initialize

```rust
let mut registry = LocalRegistry::new();
registry.init().await?;  // Must be called before other operations
```

### 2. Handle Duplicates

```rust
match registry.install("./my-spirit/").await {
    Err(RegistryError::AlreadyInstalled { name, version }) => {
        println!("{}@{} already installed, skipping", name, version);
    }
    result => result?,
}
```

### 3. Verify Before Use

```rust
let result = registry.get("untrusted-spirit").await?;

// Verify signature
result.manifest.verify()?;

// Check author
if !trusted_authors.contains(&result.manifest.author) {
    return Err("Untrusted author".into());
}
```

### 4. Use Specific Versions in Production

```rust
// Development: use latest
let wasm = registry.get_wasm("my-spirit", None).await?;

// Production: pin version
let wasm = registry.get_wasm("my-spirit", Some("1.2.3")).await?;
```

### 5. Clean Up Old Versions

```rust
// Remove old versions to save space
let all_versions = registry.get_all_versions("my-spirit").await?;
for result in all_versions.iter().rev().skip(3) {
    // Keep only the 3 most recent versions
    registry.uninstall_version("my-spirit", &result.version).await?;
}
```

## Registry Index Format

The `index.json` file structure:

```json
{
  "version": "1.0.0",
  "spirits": [
    {
      "name": "my-spirit",
      "versions": ["1.0.0", "1.1.0", "2.0.0"],
      "latest": "2.0.0",
      "installed_at": 1703721600,
      "source": {
        "Local": {
          "path": "/original/install/path"
        }
      }
    }
  ]
}
```

## Troubleshooting

### "Path does not exist"

```rust
// Check if source directory exists
use std::path::Path;
let source = "./my-spirit/";
if !Path::new(source).exists() {
    println!("Directory does not exist: {}", source);
}
```

### "No manifest.json or manifest.toml found"

```rust
// Check for manifest file
let json = Path::new("./my-spirit/manifest.json");
let toml = Path::new("./my-spirit/manifest.toml");
if !json.exists() && !toml.exists() {
    println!("Missing manifest file");
}
```

### "No spirit.wasm found"

```rust
let wasm = Path::new("./my-spirit/spirit.wasm");
if !wasm.exists() {
    println!("Missing WASM file");
}
```

### Registry Corruption

If the registry becomes corrupted, reset it:

```rust
use std::fs;

// Backup then delete
let registry_path = dirs::home_dir().unwrap().join(".vudo/registry");
let backup_path = dirs::home_dir().unwrap().join(".vudo/registry.bak");
fs::rename(&registry_path, &backup_path)?;

// Reinitialize
let mut registry = LocalRegistry::new();
registry.init().await?;

// Reinstall spirits from backup
```

## Next Steps

- [Platform Overview](./platform-overview.md) - Full architecture
- [Manifest Guide](./manifest-guide.md) - Manifest format details
- [Signing Guide](./signing-guide.md) - Cryptographic signing
