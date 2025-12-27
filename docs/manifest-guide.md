# Manifest Guide

The Spirit manifest is the metadata file that describes a Spirit package. This guide covers the complete manifest format, validation rules, and best practices.

## File Format

Manifests can be written in either TOML or JSON:

- `manifest.toml` - Human-readable, recommended for authoring
- `manifest.json` - Machine-readable, used in registry storage

## Complete Manifest Reference

### TOML Format

```toml
# Required fields
name = "my-spirit"
author = "64-character-hex-encoded-ed25519-public-key"

[version]
major = 1
minor = 0
patch = 0

# Optional fields
description = "A brief description of what this Spirit does"
license = "MIT"
repository = "https://github.com/example/my-spirit"

# Capabilities - what permissions this Spirit needs
capabilities = [
    "sensor_time",
    "sensor_random",
    "actuator_log",
    "storage_read",
    "storage_write",
]

# Dependencies on other Spirits
[dependencies]
# Registry dependency with version constraint
crypto-utils = { version = "^1.0.0" }

# Local path dependency (for development)
my-local-lib = { path = "../my-local-lib" }

# Git dependency
remote-lib = { git = "https://github.com/example/remote-lib", branch = "main" }

# Pricing model (optional, defaults shown)
[pricing]
base_cost = 100
per_fuel_cost = 1
per_byte_cost = 0
per_call_cost = 0

# Signature (added after signing)
signature = "128-character-hex-encoded-ed25519-signature"
```

### JSON Format

```json
{
  "name": "my-spirit",
  "version": {
    "major": 1,
    "minor": 0,
    "patch": 0
  },
  "author": "64-character-hex-encoded-ed25519-public-key",
  "description": "A brief description",
  "license": "MIT",
  "repository": "https://github.com/example/my-spirit",
  "capabilities": [
    "sensor_time",
    "actuator_log"
  ],
  "dependencies": {
    "crypto-utils": {
      "version": "^1.0.0"
    }
  },
  "pricing": {
    "base_cost": 100,
    "per_fuel_cost": 1
  },
  "signature": "128-character-hex-encoded-signature"
}
```

## Field Reference

### Required Fields

| Field | Type | Description |
|-------|------|-------------|
| `name` | string | Package name (1-128 chars, alphanumeric/dash/underscore) |
| `version` | object | Semantic version with `major`, `minor`, `patch` |
| `author` | string | 64-character hex-encoded Ed25519 public key |

### Optional Fields

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `description` | string | `null` | Human-readable package description |
| `license` | string | `null` | SPDX license identifier (e.g., "MIT", "Apache-2.0") |
| `repository` | string | `null` | Source code repository URL |
| `capabilities` | array | `[]` | Required runtime capabilities |
| `dependencies` | object | `{}` | Dependencies on other Spirits |
| `pricing` | object | default | Execution pricing model |
| `signature` | string | `null` | Ed25519 signature over manifest content |

## Validation Rules

### Name Validation

```rust
// Valid names
"my-spirit"
"spirit_v2"
"ExampleSpirit123"

// Invalid names
""                  // Empty
"a".repeat(200)     // Too long (max 128)
"my spirit"         // Contains space
"my.spirit"         // Contains period
"@my/spirit"        // Contains special characters
```

### Author Validation

The author field must be exactly 64 hex characters representing a 32-byte Ed25519 public key:

```rust
// Valid author
"abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789"

// Invalid
"short"                    // Too short
"ghijkl..."                // Non-hex characters
"ABCDEF..."                // Case doesn't matter (valid)
```

### Signature Validation

The signature field, if present, must be exactly 128 hex characters representing a 64-byte Ed25519 signature:

```rust
// Valid signature (128 hex chars)
"0123456789abcdef..." // (64 bytes = 128 hex characters)
```

## Capabilities

### Available Capabilities

```toml
capabilities = [
    # Network
    "network_listen",       # Accept incoming connections
    "network_connect",      # Make outgoing connections
    "network_broadcast",    # Broadcast/multicast

    # Storage
    "storage_read",         # Read from persistent storage
    "storage_write",        # Write to persistent storage
    "storage_delete",       # Delete from storage

    # Compute
    "spawn_sandbox",        # Spawn child sandboxes
    "cross_sandbox_call",   # Call functions in other sandboxes

    # Sensors
    "sensor_time",          # Read current time
    "sensor_random",        # Generate random numbers
    "sensor_environment",   # Read environment variables

    # Actuators
    "actuator_log",         # Write to log output
    "actuator_notify",      # Send notifications
    "actuator_credit",      # Credit/billing operations
]
```

### Capability Best Practices

1. **Request minimal capabilities** - Only request what you need
2. **Document capability usage** - Explain why each capability is required
3. **Consider security implications** - Network and storage capabilities require extra trust

## Dependencies

### Version Constraints

```toml
[dependencies]
# Caret (compatible version)
dep1 = { version = "^1.0.0" }     # >=1.0.0, <2.0.0
dep2 = { version = "^0.1.0" }     # >=0.1.0, <0.2.0

# Tilde (minor-compatible)
dep3 = { version = "~1.2.3" }     # >=1.2.3, <1.3.0

# Comparison operators
dep4 = { version = ">=1.0.0" }    # Minimum version
dep5 = { version = ">1.0.0" }     # Greater than
dep6 = { version = "<=2.0.0" }    # Maximum version
dep7 = { version = "<2.0.0" }     # Less than
dep8 = { version = "=1.0.0" }     # Exact version

# Wildcard
dep9 = { version = "*" }          # Any version

# Compound
dep10 = { version = ">=1.0.0, <2.0.0" }
```

### Dependency Sources

```toml
[dependencies]
# Registry (default)
from-registry = { version = "^1.0" }

# Local path (development)
local-dev = { path = "../my-local-spirit" }

# Git repository
from-git = { git = "https://github.com/example/spirit" }
from-git-branch = { git = "https://github.com/example/spirit", branch = "develop" }
from-git-tag = { git = "https://github.com/example/spirit", tag = "v1.0.0" }
from-git-rev = { git = "https://github.com/example/spirit", rev = "abc123" }
```

## Pricing Model

```toml
[pricing]
# Fixed cost charged per invocation
base_cost = 100

# Cost multiplied by fuel consumed
per_fuel_cost = 1

# Cost multiplied by bytes of memory used
per_byte_cost = 0

# Cost multiplied by number of host function calls
per_call_cost = 0
```

### Pricing Calculation

```
total_cost = base_cost + (fuel_consumed * per_fuel_cost)
           + (memory_bytes * per_byte_cost)
           + (host_calls * per_call_cost)
```

## Working with Manifests in Code

### Creating a Manifest

```rust
use spirit_runtime::{ManifestBuilder, Capability, SemVer, Dependency};

let manifest = ManifestBuilder::new(
    "my-spirit",
    SemVer::new(1, 0, 0),
    keypair.verifying_key().to_hex(),
)
.description("Example Spirit package")
.license("MIT")
.repository("https://github.com/example/my-spirit")
.capability(Capability::SensorTime)
.capability(Capability::ActuatorLog)
.dependency("utils", Dependency::new("^1.0"))
.build_validated()?;
```

### Reading a Manifest

```rust
use spirit_runtime::Manifest;

// From file (extension determines format)
let manifest = Manifest::from_file("manifest.toml")?;
let manifest = Manifest::from_file("manifest.json")?;

// From string
let manifest = Manifest::from_toml(toml_string)?;
let manifest = Manifest::from_json(json_string)?;
```

### Validating a Manifest

```rust
use spirit_runtime::Manifest;

let manifest = Manifest::from_file("manifest.toml")?;

// Full validation
manifest.validate()?;

// Dependency-only validation
manifest.validate_dependencies()?;
```

### Signing a Manifest

```rust
use spirit_runtime::Manifest;
use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;

// Generate keypair
let signing_key = SigningKey::generate(&mut OsRng);
let public_key = signing_key.verifying_key();
let author = hex::encode(public_key.as_bytes());

// Create manifest with author's public key
let mut manifest = Manifest::new("signed-spirit", SemVer::new(1, 0, 0), author);

// Sign and store signature
let signature = manifest.sign(&signing_key)?;
manifest.signature = Some(signature);

// Save signed manifest
manifest.to_file("manifest.toml")?;
```

### Verifying a Manifest

```rust
use spirit_runtime::Manifest;

let manifest = Manifest::from_file("manifest.toml")?;

match manifest.verify() {
    Ok(()) => println!("Manifest signature is valid"),
    Err(e) => println!("Verification failed: {}", e),
}
```

## Content Hashing

The manifest content hash is computed from:
- `name`
- `version` (as string)
- `author`
- `description` (if present)
- `capabilities` (debug format)

The hash excludes the `signature` field to allow signing.

```rust
use spirit_runtime::Manifest;

let manifest = Manifest::from_file("manifest.toml")?;
let hash: Vec<u8> = manifest.content_hash();
println!("Content hash: {}", hex::encode(&hash));
```

## Example Manifests

### Minimal Manifest

```toml
name = "minimal-spirit"
author = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"

[version]
major = 0
minor = 1
patch = 0
```

### Full-Featured Manifest

```toml
name = "full-featured-spirit"
author = "ed25519pubkey64hexcharshere..."
description = "A comprehensive Spirit with all features"
license = "Apache-2.0"
repository = "https://github.com/example/full-featured"

[version]
major = 2
minor = 5
patch = 1

capabilities = [
    "network_connect",
    "storage_read",
    "storage_write",
    "sensor_time",
    "sensor_random",
    "actuator_log",
]

[dependencies]
crypto = { version = "^1.0" }
logging = { version = ">=2.0, <3.0" }
dev-utils = { path = "../dev-utils" }

[pricing]
base_cost = 500
per_fuel_cost = 2
per_byte_cost = 1
per_call_cost = 10

signature = "ed25519sig128hexcharshere..."
```

## Troubleshooting

### Common Errors

| Error | Cause | Fix |
|-------|-------|-----|
| `InvalidName: Name cannot be empty` | Empty name field | Provide a valid name |
| `InvalidName: Name too long` | Name > 128 chars | Shorten the name |
| `InvalidAuthor: must be 64 hex characters` | Wrong key length | Use correct public key format |
| `SignatureError: No signature present` | Calling verify() without signature | Sign the manifest first |
| `SignatureError: Signature verification failed` | Tampered content or wrong key | Re-sign with correct key |
| `InvalidDependency: bad version` | Invalid version constraint | Fix version syntax |

## Next Steps

- [Signing Guide](./signing-guide.md) - Detailed cryptographic signing workflow
- [Registry Guide](./registry-guide.md) - Installing and managing Spirits
