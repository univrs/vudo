# VUDO Platform Overview

The VUDO (Virtual Universal Distributed Operations) platform provides a secure, capability-based WebAssembly execution environment for running untrusted code safely.

## Architecture

### Two-Crate Design

The platform consists of two main crates:

1. **`vudo_vm`** - The WebAssembly sandbox runtime
2. **`spirit_runtime`** - Package management and cryptographic signing

```
┌─────────────────────────────────────────────────────────────────────┐
│                           vudo_vm                                    │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌────────────┐  │
│  │  Sandbox    │  │   Linker    │  │    Fuel     │  │ Capability │  │
│  │  (wasmtime) │  │ (host fns)  │  │  (metering) │  │  (grants)  │  │
│  └─────────────┘  └─────────────┘  └─────────────┘  └────────────┘  │
└─────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────┐
│                        spirit_runtime                                │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌────────────┐  │
│  │  Manifest   │  │  Signature  │  │   Registry  │  │   Pricing  │  │
│  │  (metadata) │  │  (ed25519)  │  │   (local)   │  │  (credits) │  │
│  └─────────────┘  └─────────────┘  └─────────────┘  └────────────┘  │
└─────────────────────────────────────────────────────────────────────┘
```

## VUDO VM Components

### Sandbox (`sandbox.rs`)
The core execution environment built on [wasmtime](https://wasmtime.dev/). It provides:
- Memory isolation between instances
- Configurable resource limits
- Fuel-based execution metering
- Capability enforcement

### Linker (`linker.rs`)
Registers host functions that WASM modules can call. All functions are namespaced under `vudo.*`:

```rust
// Host function namespace
linker.func_wrap("vudo", "log", |msg: i32, len: i32| { ... })?;
linker.func_wrap("vudo", "get_time", || -> i64 { ... })?;
linker.func_wrap("vudo", "random", || -> i64 { ... })?;
```

### Capabilities (`capability.rs`)
Fine-grained permission system. Capabilities are:
- Declared in the Spirit manifest
- Verified at VM initialization
- Enforced at every host function call

```rust
use vudo_vm::{CapabilitySet, CapabilityType};

let mut caps = CapabilitySet::new();
caps.grant(CapabilityType::SensorTime, CapabilityScope::Global);
caps.grant(CapabilityType::ActuatorLog, CapabilityScope::Global);
```

### Fuel Metering (`fuel.rs`)
Prevents infinite loops and resource exhaustion:

```rust
use vudo_vm::ResourceLimits;

let limits = ResourceLimits {
    max_fuel: 1_000_000,      // Instruction count limit
    max_memory_bytes: 64 * 1024 * 1024,  // 64 MB
    max_execution_time: Duration::from_secs(30),
    ..Default::default()
};
```

### Host Interface (`host.rs`)
Bridges WASM to the outside world:
- `StorageBackend` - Persistent key-value storage
- `LogLevel` - Log message severity levels
- `HostCallResult` - Return type for host operations

## Spirit Runtime Components

### Manifest (`manifest.rs`)
Package metadata in TOML or JSON format:

```toml
name = "my-spirit"
author = "64-hex-char-public-key"
description = "Example Spirit package"

[version]
major = 1
minor = 0
patch = 0

capabilities = ["sensor_time", "actuator_log"]

[dependencies.other-spirit]
version = "^1.0"

[pricing]
base_cost = 100
per_fuel_cost = 1
```

### Signature (`signature.rs`)
Ed25519 digital signatures for package authentication:

```rust
use spirit_runtime::signature::{KeyPair, SigningKey};

// Generate a new keypair
let keypair = KeyPair::generate();

// Sign data
let signature = keypair.sign(b"message");

// Verify signature
keypair.verifying_key().verify(b"message", &signature)?;
```

### Registry (`registry/`)
Local package storage at `~/.vudo/registry/`:

```rust
use spirit_runtime::registry::{LocalRegistry, Registry};

let mut registry = LocalRegistry::new();
registry.init().await?;

// Install a spirit
registry.install("./path/to/spirit/").await?;

// Retrieve WASM bytes
let wasm = registry.get_wasm("my-spirit", None).await?;
```

### Versioning (`version.rs`)
Semantic versioning with constraint matching:

```rust
use spirit_runtime::SemVer;

let version = SemVer::new(1, 2, 3);
let requirement = "^1.0".parse()?;
assert!(requirement.matches(&version));
```

### Dependency Resolution (`dependency.rs`)
Supports multiple dependency sources:

```rust
use spirit_runtime::Dependency;

// Registry dependency
let dep = Dependency::new("^1.0.0");

// Local path dependency
let dep = Dependency::from_path("../local-spirit");

// Git dependency
let dep = Dependency::from_git("https://github.com/example/spirit", Some("main"));
```

### Pricing (`pricing.rs`)
Credit-based execution costs:

```rust
use spirit_runtime::PricingModel;

let pricing = PricingModel {
    base_cost: 100,      // Fixed cost per invocation
    per_fuel_cost: 1,    // Cost per fuel unit consumed
    ..Default::default()
};
```

## Security Model

### Defense in Depth

1. **Cryptographic Verification**
   - All Spirits must be signed with Ed25519
   - Author's public key embedded in manifest
   - Signature verified before execution

2. **Capability Enforcement**
   - Spirits declare required capabilities
   - VM refuses to run if undeclared capability is used
   - Each host function checks permissions

3. **Resource Limits**
   - Memory capped per sandbox
   - CPU limited via fuel metering
   - Execution time bounded

4. **Isolation**
   - Each Spirit runs in isolated memory space
   - No shared state between sandboxes
   - Cross-sandbox calls require explicit capability

### Threat Mitigation

| Threat | Mitigation |
|--------|------------|
| Malicious code | Signature verification, capability restrictions |
| Infinite loops | Fuel metering |
| Memory exhaustion | Memory limits |
| Time-based attacks | Execution time limits |
| Unauthorized access | Capability enforcement |
| Tampering | Content hash in signature |

## Data Flow

```
┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│    Spirit    │    │    Verify    │    │     Run      │
│   Package    │───>│   Manifest   │───>│   in VM      │
│  (wasm+sig)  │    │  & Signature │    │  (sandbox)   │
└──────────────┘    └──────────────┘    └──────────────┘
       │                   │                   │
       ▼                   ▼                   ▼
┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│   Registry   │    │  Check Caps  │    │  Host Calls  │
│   Storage    │    │   Match VM   │    │  Via Linker  │
└──────────────┘    └──────────────┘    └──────────────┘
```

## API Quick Reference

### spirit_runtime exports

```rust
pub use dependency::{Dependency, DependencyResolver};
pub use manifest::{Capability, Manifest, ManifestBuilder, ManifestError};
pub use pricing::{CreditCost, PricingModel};
pub use registry::{LocalRegistry, QueryBuilder, Registry, RegistryError};
pub use signature::{KeyPair, Signature, SignatureError, SigningKey, VerifyingKey};
pub use version::SemVer;
```

### vudo_vm exports

```rust
pub use error::SandboxError;
pub use limits::ResourceLimits;
pub use capability::{CapabilityGrant, CapabilityScope, CapabilitySet, CapabilityType};
pub use host::{HostCallResult, HostInterface, InMemoryStorage, LogLevel, StorageBackend};
pub use linker::{create_linker, HostState, HOST_ERROR, HOST_SUCCESS};
```

## Next Steps

- [Spirit Runtime Guide](./spirit-runtime-guide.md) - Detailed package management
- [Manifest Guide](./manifest-guide.md) - Manifest format reference
- [Signing Guide](./signing-guide.md) - Cryptographic signing workflow
- [Registry Guide](./registry-guide.md) - Local registry operations
