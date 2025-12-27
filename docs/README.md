# VUDO Developer Documentation

Welcome to the VUDO platform documentation. This guide covers the core components for building, signing, and running Spirit packages on the VUDO VM.

## Table of Contents

1. [Platform Overview](./platform-overview.md) - Architecture and concepts
2. [Spirit Runtime Guide](./spirit-runtime-guide.md) - Package management system
3. [Manifest Guide](./manifest-guide.md) - Spirit package manifests
4. [Signing Guide](./signing-guide.md) - Ed25519 cryptographic signing
5. [Registry Guide](./registry-guide.md) - Installing and managing Spirits

## Quick Start

### Creating Your First Spirit

1. **Create a manifest** (`manifest.toml`):

```toml
name = "hello-spirit"
author = "your-64-char-hex-public-key-here"
description = "My first Spirit"

[version]
major = 0
minor = 1
patch = 0

[[capabilities]]
sensor_time = {}

[[capabilities]]
actuator_log = {}
```

2. **Compile your WASM module** to `spirit.wasm`

3. **Sign the manifest** using your Ed25519 private key

4. **Install to the registry**:

```rust
use spirit_runtime::registry::{LocalRegistry, Registry};

let mut registry = LocalRegistry::new();
registry.init().await?;
registry.install("./hello-spirit/").await?;
```

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                         VUDO Platform                            │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────────┐     ┌──────────────────┐                  │
│  │  Spirit Runtime  │     │     VUDO VM      │                  │
│  │                  │     │                  │                  │
│  │  ┌────────────┐  │     │  ┌────────────┐  │                  │
│  │  │  Manifest  │  │     │  │  Sandbox   │  │                  │
│  │  └────────────┘  │     │  └────────────┘  │                  │
│  │  ┌────────────┐  │     │  ┌────────────┐  │                  │
│  │  │ Signature  │  │     │  │   Linker   │  │                  │
│  │  └────────────┘  │     │  └────────────┘  │                  │
│  │  ┌────────────┐  │     │  ┌────────────┐  │                  │
│  │  │  Registry  │  │     │  │   Fuel     │  │                  │
│  │  └────────────┘  │     │  └────────────┘  │                  │
│  │  ┌────────────┐  │     │  ┌────────────┐  │                  │
│  │  │  Pricing   │  │     │  │Capabilities│  │                  │
│  │  └────────────┘  │     │  └────────────┘  │                  │
│  │                  │     │                  │                  │
│  └──────────────────┘     └──────────────────┘                  │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

## Core Concepts

### Spirit
A Spirit is a packaged WebAssembly module containing:
- **Manifest**: Metadata, dependencies, capabilities, and pricing
- **WASM bytecode**: Compiled executable code
- **Ed25519 signature**: Cryptographic proof of authenticity

### Capabilities
Spirits declare required capabilities in their manifest. The VM enforces these at runtime:

| Category | Capability | Description |
|----------|------------|-------------|
| Network | `network_listen` | Accept incoming connections |
| Network | `network_connect` | Make outgoing connections |
| Network | `network_broadcast` | Broadcast/multicast |
| Storage | `storage_read` | Read persistent storage |
| Storage | `storage_write` | Write persistent storage |
| Storage | `storage_delete` | Delete from storage |
| Compute | `spawn_sandbox` | Spawn child sandboxes |
| Compute | `cross_sandbox_call` | Call other sandboxes |
| Sensor | `sensor_time` | Read current time |
| Sensor | `sensor_random` | Generate random numbers |
| Sensor | `sensor_environment` | Read environment variables |
| Actuator | `actuator_log` | Write to logs |
| Actuator | `actuator_notify` | Send notifications |
| Actuator | `actuator_credit` | Credit/billing operations |

### Fuel Metering
VUDO uses fuel-based execution limits. Every WASM instruction consumes fuel. When fuel is exhausted, execution halts gracefully.

### Semantic Versioning
All Spirit versions follow [SemVer](https://semver.org/):
- `MAJOR.MINOR.PATCH` (e.g., `1.2.3`)
- Constraint syntax: `^1.0` (compatible), `>=1.0.0` (minimum), `*` (any)

## Directory Structure

```
~/.vudo/
└── registry/
    ├── index.json           # Registry index
    ├── spirits/             # Installed spirits
    │   ├── my-spirit/
    │   │   ├── 1.0.0/
    │   │   │   ├── manifest.json
    │   │   │   └── spirit.wasm
    │   │   └── latest -> 1.0.0/
    │   └── ...
    └── cache/               # Downloaded packages
```

## Crate Dependencies

```toml
[dependencies]
spirit_runtime = { path = "./spirit_runtime" }
vudo_vm = { path = "./vudo_vm" }
```

## Related Documentation

- [univrs-docs/vudo](https://github.com/example/univrs-docs/tree/main/vudo) - Wiki documentation
- [API Reference](./api-reference.md) - Generated API docs

## License

MIT License - See [LICENSE](../LICENSE) for details.
