# Spirit Runtime Guide

The `spirit_runtime` crate provides the complete package management infrastructure for VUDO Spirits. This guide covers all modules and their usage patterns.

## Module Overview

| Module | Purpose |
|--------|---------|
| `manifest` | Package metadata, capabilities, validation |
| `signature` | Ed25519 signing and verification |
| `registry` | Local filesystem package storage |
| `version` | Semantic versioning and constraints |
| `dependency` | Dependency specification and resolution |
| `pricing` | Credit-based execution costs |

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
spirit_runtime = { path = "path/to/spirit_runtime" }

# Required for signature operations
ed25519-dalek = "2.1"
rand = "0.8"
hex = "0.4"
```

## Quick Start

```rust
use spirit_runtime::{
    Manifest, ManifestBuilder, Capability, SemVer,
    KeyPair, LocalRegistry, Registry,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Generate a keypair for signing
    let keypair = KeyPair::generate();
    let public_key_hex = keypair.verifying_key().to_hex();

    // 2. Create a manifest
    let mut manifest = ManifestBuilder::new(
        "my-spirit",
        SemVer::new(1, 0, 0),
        public_key_hex,
    )
    .description("My first Spirit package")
    .license("MIT")
    .capability(Capability::SensorTime)
    .capability(Capability::ActuatorLog)
    .build();

    // 3. Sign the manifest
    let signature = manifest.sign(&keypair.signing_key().0)?;
    manifest.signature = Some(signature);

    // 4. Save the manifest
    manifest.to_file("./my-spirit/manifest.toml")?;

    // 5. Install to registry
    let mut registry = LocalRegistry::new();
    registry.init().await?;
    registry.install("./my-spirit/").await?;

    Ok(())
}
```

## Manifest Module

### Creating Manifests

#### Using the Builder Pattern (Recommended)

```rust
use spirit_runtime::{ManifestBuilder, Capability, SemVer, Dependency};

let manifest = ManifestBuilder::new(
    "network-spirit",
    SemVer::new(2, 1, 0),
    "a".repeat(64),  // 64-char hex public key
)
.description("A network-enabled Spirit")
.license("Apache-2.0")
.repository("https://github.com/example/network-spirit")
.capability(Capability::NetworkConnect)
.capability(Capability::NetworkListen)
.capability(Capability::StorageRead)
.dependency("crypto-spirit", Dependency::new("^1.0"))
.dependency("logging-spirit", Dependency::from_path("../logging"))
.build();
```

#### Using Constructor Directly

```rust
use spirit_runtime::{Manifest, Capability, SemVer};

let mut manifest = Manifest::new(
    "simple-spirit",
    SemVer::new(0, 1, 0),
    "a".repeat(64),
);

manifest.description = Some("A simple Spirit".to_string());
manifest.license = Some("MIT".to_string());
manifest.add_capability(Capability::SensorTime);
```

### Parsing and Serialization

```rust
use spirit_runtime::Manifest;

// From TOML string
let toml = r#"
name = "example"
author = "aaaa...64chars...aaaa"
[version]
major = 1
minor = 0
patch = 0
capabilities = ["sensor_time", "actuator_log"]
"#;
let manifest = Manifest::from_toml(toml)?;

// From JSON string
let json = r#"{"name": "example", "version": {"major": 1, ...}}"#;
let manifest = Manifest::from_json(json)?;

// From file (format detected by extension)
let manifest = Manifest::from_file("manifest.toml")?;
let manifest = Manifest::from_file("manifest.json")?;

// To TOML string
let toml_str = manifest.to_toml()?;

// To JSON string
let json_str = manifest.to_json()?;

// To file
manifest.to_file("output.toml")?;
manifest.to_file("output.json")?;
```

### Validation

```rust
use spirit_runtime::Manifest;

let manifest = Manifest::from_file("manifest.toml")?;

// Validate all fields
match manifest.validate() {
    Ok(()) => println!("Manifest is valid"),
    Err(e) => println!("Validation failed: {}", e),
}

// Validated builder
let manifest = ManifestBuilder::new("test", SemVer::new(1, 0, 0), "a".repeat(64))
    .build_validated()?;  // Returns Err if invalid
```

### Capabilities

All available capabilities:

```rust
use spirit_runtime::Capability;

// Network
Capability::NetworkListen      // Accept incoming connections
Capability::NetworkConnect     // Make outgoing connections
Capability::NetworkBroadcast   // Broadcast/multicast

// Storage
Capability::StorageRead        // Read persistent storage
Capability::StorageWrite       // Write persistent storage
Capability::StorageDelete      // Delete from storage

// Compute
Capability::SpawnSandbox       // Spawn child sandboxes
Capability::CrossSandboxCall   // Call other sandboxes

// Sensors
Capability::SensorTime         // Read current time
Capability::SensorRandom       // Generate random numbers
Capability::SensorEnvironment  // Read environment variables

// Actuators
Capability::ActuatorLog        // Write to logs
Capability::ActuatorNotify     // Send notifications
Capability::ActuatorCredit     // Credit/billing operations

// Check if manifest requires a capability
if manifest.requires_capability(&Capability::NetworkConnect) {
    println!("This Spirit needs network access");
}

// Get all capabilities
let all_caps = Capability::all();  // Vec of 14 capabilities

// Parse from string
let cap: Capability = "network_connect".parse()?;

// Display as string
println!("{}", Capability::SensorTime);  // "sensor_time"
```

## Signature Module

### Key Generation

```rust
use spirit_runtime::signature::{KeyPair, SigningKey, VerifyingKey};

// Generate a new keypair
let keypair = KeyPair::generate();

// Or generate just a signing key
let signing_key = SigningKey::generate();
let verifying_key = signing_key.verifying_key();
```

### Signing and Verification

```rust
use spirit_runtime::signature::KeyPair;

let keypair = KeyPair::generate();

// Sign a message
let message = b"Hello, VUDO!";
let signature = keypair.sign(message);

// Verify the signature
match keypair.verifying_key().verify(message, &signature) {
    Ok(()) => println!("Signature valid"),
    Err(e) => println!("Signature invalid: {}", e),
}
```

### Serialization

```rust
use spirit_runtime::signature::{SigningKey, VerifyingKey, Signature};

// To/from bytes
let signing_key = SigningKey::generate();
let bytes: [u8; 32] = signing_key.to_bytes();
let restored = SigningKey::from_bytes(&bytes)?;

// To/from hex
let hex_string = signing_key.to_hex();
let restored = SigningKey::from_hex(&hex_string)?;

// VerifyingKey and Signature also support serde
let vk = signing_key.verifying_key();
let json = serde_json::to_string(&vk)?;
let restored: VerifyingKey = serde_json::from_str(&json)?;
```

### Prehashed Signing

For large messages, use prehashed signing (SHA-256 hash first):

```rust
let keypair = KeyPair::generate();
let large_message = vec![0u8; 1024 * 1024];  // 1 MB

let signature = keypair.sign_prehashed(&large_message);
let result = keypair.verifying_key().verify_prehashed(&large_message, &signature);
```

### Signing Manifests

```rust
use spirit_runtime::{Manifest, SemVer};
use spirit_runtime::signature::SigningKey;

// Create signing key
let signing_key = SigningKey::generate();
let public_key = signing_key.verifying_key();
let author = public_key.to_hex();

// Create manifest with author's public key
let mut manifest = Manifest::new("signed-spirit", SemVer::new(1, 0, 0), author);

// Sign the manifest
let signature = manifest.sign(&signing_key.0)?;
manifest.signature = Some(signature);

// Verify the manifest
manifest.verify()?;
```

## Registry Module

### Initialization

```rust
use spirit_runtime::registry::{LocalRegistry, Registry};

// Default location: ~/.vudo/registry/
let mut registry = LocalRegistry::new();

// Custom location
let mut registry = LocalRegistry::with_root("/custom/path/registry");

// Initialize (creates directories and index)
registry.init().await?;
```

### Installing Spirits

```rust
// Install from local directory
let installed = registry.install("./my-spirit/").await?;
println!("Installed {} v{}", installed.name, installed.latest);

// Install multiple
use spirit_runtime::registry::RegistryExt;
let results = registry.install_all(&["./spirit1/", "./spirit2/"]).await?;
```

### Retrieving Spirits

```rust
// Get latest version
let result = registry.get("my-spirit").await?;
println!("Name: {}", result.name);
println!("Version: {}", result.version);
println!("Path: {:?}", result.path);

// Get specific version
let result = registry.get_version("my-spirit", "1.0.0").await?;

// Get all versions
use spirit_runtime::registry::RegistryExt;
let versions = registry.get_all_versions("my-spirit").await?;

// Get WASM bytes
let wasm = registry.get_wasm("my-spirit", None).await?;  // latest
let wasm = registry.get_wasm("my-spirit", Some("1.0.0")).await?;

// Get manifest
let manifest = registry.get_manifest("my-spirit", None).await?;
```

### Searching

```rust
use spirit_runtime::registry::{SpiritQuery, QueryBuilder};

// Using SpiritQuery directly
let query = SpiritQuery::new()
    .with_name("network")
    .with_capability("network_connect");

let results = registry.search(&query).await?;

// Using QueryBuilder (fluent API)
let results = QueryBuilder::new()
    .name_contains("spirit")
    .with_author("abcd1234...")
    .build()
    .search(&registry)
    .await?;
```

### Listing and Checking

```rust
// List all installed spirits
let spirits = registry.list().await?;
for spirit in spirits {
    println!("{} ({})", spirit.name, spirit.latest);
    println!("  Versions: {:?}", spirit.versions);
}

// Check if installed
if registry.is_installed("my-spirit") {
    println!("Spirit is installed");
}

// Check specific version
if registry.is_version_installed("my-spirit", "1.0.0") {
    println!("Version 1.0.0 is installed");
}
```

### Uninstalling

```rust
// Remove all versions
registry.uninstall("old-spirit").await?;

// Remove specific version
registry.uninstall_version("my-spirit", "0.9.0").await?;
```

## Version Module

### Creating Versions

```rust
use spirit_runtime::SemVer;

// From components
let version = SemVer::new(1, 2, 3);

// From string
let version: SemVer = "1.2.3".parse()?;

// Display
println!("{}", version);  // "1.2.3"
```

### Version Constraints

```rust
use spirit_runtime::version::VersionRequirement;

let req: VersionRequirement = "^1.0".parse()?;
let version = SemVer::new(1, 5, 0);

if req.matches(&version) {
    println!("Version satisfies constraint");
}

// Supported constraint syntax:
// "^1.0"      - Compatible (>=1.0.0, <2.0.0)
// "~1.2"      - Minor-compatible (>=1.2.0, <1.3.0)
// ">=1.0.0"   - Minimum version
// ">1.0.0"    - Greater than
// "<=2.0.0"   - Maximum version
// "<2.0.0"    - Less than
// "=1.0.0"    - Exact version
// "*"         - Any version
```

## Dependency Module

### Dependency Types

```rust
use spirit_runtime::Dependency;

// Registry dependency (version constraint)
let dep = Dependency::new("^1.0.0");
let dep = Dependency::new(">=2.0.0, <3.0.0");

// Local path dependency
let dep = Dependency::from_path("../local-spirit");

// Git dependency
let dep = Dependency::from_git(
    "https://github.com/example/spirit",
    None,  // default branch
);
let dep = Dependency::from_git(
    "https://github.com/example/spirit",
    Some("v1.0.0"),  // specific tag/branch
);

// Check dependency type
if dep.is_local() {
    println!("Local dependency at: {:?}", dep.path);
}
if dep.is_git() {
    println!("Git dependency from: {:?}", dep.git);
}
```

### Adding to Manifest

```rust
use spirit_runtime::{Manifest, Dependency, SemVer};

let mut manifest = Manifest::new("parent", SemVer::new(1, 0, 0), "a".repeat(64));

manifest.add_dependency("child-spirit", Dependency::new("^1.0"));
manifest.add_dependency("local-util", Dependency::from_path("../util"));
```

## Pricing Module

### Configuring Pricing

```rust
use spirit_runtime::pricing::PricingModel;

let pricing = PricingModel {
    base_cost: 100,       // Fixed cost per invocation
    per_fuel_cost: 1,     // Cost per fuel unit
    per_byte_cost: 0,     // Cost per byte of memory
    per_call_cost: 0,     // Cost per host function call
};

// Calculate total cost
let fuel_used = 50_000u64;
let total = pricing.calculate_cost(fuel_used);
println!("Total cost: {} credits", total);
```

### In Manifest

```toml
[pricing]
base_cost = 100
per_fuel_cost = 1
per_byte_cost = 0
per_call_cost = 0
```

## Error Handling

All modules use typed errors:

```rust
use spirit_runtime::manifest::ManifestError;
use spirit_runtime::signature::SignatureError;
use spirit_runtime::registry::RegistryError;

// ManifestError variants
match result {
    Err(ManifestError::ParseError(msg)) => ...,
    Err(ManifestError::InvalidName(msg)) => ...,
    Err(ManifestError::InvalidAuthor(msg)) => ...,
    Err(ManifestError::InvalidSignature(msg)) => ...,
    Err(ManifestError::SignatureError(msg)) => ...,
    Err(ManifestError::IoError { path, message }) => ...,
    Err(ManifestError::InvalidDependency { name, reason }) => ...,
    _ => ...
}

// SignatureError variants
match result {
    Err(SignatureError::VerificationFailed) => ...,
    Err(SignatureError::InvalidKey(msg)) => ...,
    Err(SignatureError::InvalidSignature(msg)) => ...,
    Err(SignatureError::HexError(msg)) => ...,
    _ => ...
}

// RegistryError variants
match result {
    Err(RegistryError::NotFound(name)) => ...,
    Err(RegistryError::VersionNotFound { name, version }) => ...,
    Err(RegistryError::AlreadyInstalled { name, version }) => ...,
    Err(RegistryError::InvalidSource(msg)) => ...,
    Err(RegistryError::InvalidManifest(msg)) => ...,
    Err(RegistryError::MissingWasm(msg)) => ...,
    _ => ...
}
```

## Next Steps

- [Manifest Guide](./manifest-guide.md) - Complete manifest format reference
- [Signing Guide](./signing-guide.md) - Detailed signing workflow
- [Registry Guide](./registry-guide.md) - Registry deep dive
