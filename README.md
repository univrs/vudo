# VUDO

**Secure WebAssembly runtime for Spirit packages.**

VUDO provides a capability-based sandbox for executing Spirits - secure, metered WebAssembly programs with cryptographic verification.

---

## Quick Start

```bash
# Create a new Spirit
vudo new my-spirit
cd my-spirit

# Build and run
vudo build
vudo run

# Package and sign
vudo pack
vudo sign my-spirit.spirit

# Install to local registry
vudo summon ./my-spirit.spirit
```

---

## Features

| Feature | Description |
|---------|-------------|
| **Sandbox Isolation** | 6-state lifecycle (Created → Initializing → Running → Suspended → Terminated) |
| **Capability System** | 14 capability types with cryptographic grants |
| **Fuel Metering** | Deterministic execution bounds (default: 1B units) |
| **Ed25519 Signatures** | Full signing and verification for Spirit packages |
| **Local Registry** | Filesystem-based Spirit storage (~/.vudo/registry/) |
| **15+ CLI Commands** | Complete developer workflow |

---

## CLI Commands

```bash
# Project
vudo new <name>         # Create new Spirit project
vudo build              # Compile to WASM
vudo run                # Execute in sandbox
vudo test               # Run Spirit tests

# Packaging
vudo pack               # Create .spirit package
vudo sign <file>        # Ed25519 signature
vudo check              # Validate manifest

# Registry
vudo summon <source>    # Install Spirit
vudo search <query>     # Search registry
vudo list               # List installed
vudo info <name>        # Show Spirit info
vudo uninstall <name>   # Remove Spirit

# Tools
vudo fmt                # Format manifest
vudo doc                # Generate docs
vudo dol                # DOL REPL (stub)
```

---

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                      vudo CLI                           │
│  new, build, run, test, pack, sign, publish, summon     │
└─────────────────────────┬───────────────────────────────┘
                          │
          ┌───────────────┴───────────────┐
          ▼                               ▼
┌─────────────────────┐       ┌─────────────────────┐
│   spirit_runtime    │       │    Local Registry   │
│                     │◄─────►│                     │
│  Manifest Parsing   │       │  ~/.vudo/registry/  │
│  Ed25519 Signing    │       │  index.json         │
│  Dep Resolution     │       │  spirits/           │
└─────────┬───────────┘       └─────────────────────┘
          │
          ▼
┌─────────────────────────────────────────────────────────┐
│                       vudo_vm                           │
│                                                         │
│  Sandbox Manager ──► Lifecycle (6 states)               │
│  Capability Set  ──► Permission Enforcement             │
│  Fuel Manager    ──► Execution Metering                 │
│  Host Functions  ──► 15 syscalls                        │
└─────────────────────────┬───────────────────────────────┘
                          │
                          ▼
                    ┌───────────┐
                    │  wasmtime │
                    └───────────┘
```

---

## Capability Types

```
Network:    NetworkListen, NetworkConnect, NetworkBroadcast
Storage:    StorageRead, StorageWrite, StorageDelete
Compute:    SpawnSandbox, CrossSandboxCall
Sensor:     SensorTime, SensorRandom, SensorEnvironment
Actuator:   ActuatorLog, ActuatorNotify, ActuatorCredit
Special:    Unrestricted (system Spirits only)
```

---

## Spirit Manifest

```toml
name = "my-spirit"
version = { major = 1, minor = 0, patch = 0 }
author = "<64-char-hex-ed25519-public-key>"
description = "My first Spirit"
license = "MIT"
capabilities = ["sensor_time", "actuator_log"]

[dependencies]
other-spirit = "^1.0"

[pricing]
base_cost = 100
per_fuel_cost = 1
```

---

## Crates

| Crate | Description | Tests |
|-------|-------------|-------|
| `vudo_vm` | WebAssembly sandbox with capabilities | 188+ |
| `spirit_runtime` | Package management and signatures | 98+ |
| `vudo_cli` | Command-line interface | 34+ |

**Total: 362+ tests passing**

---

## Development

```bash
# Build all crates
cargo build --workspace

# Run tests
cargo test --workspace

# Check code quality
cargo clippy --workspace -- -D warnings
cargo fmt --all -- --check
```

---

## Status

**Phase 2: Complete**

- VUDO VM with sandbox lifecycle and fuel metering
- Spirit Runtime with Ed25519 signatures
- Local registry with dependency resolution
- Full CLI with 15+ commands

**Next: Phase 3 - Hyphal Network**

- P2P Spirit distribution
- Imaginarium distributed registry
- Cross-node sandbox migration
- DOL v0.3.0 → WASM pipeline

---

## Links

- **DOL Language**: [github.com/univrs/dol](https://github.com/univrs/dol)
- **DOL on Crates.io**: [crates.io/crates/dol](https://crates.io/crates/dol)
- **Documentation**: [learn.univrs.io](https://learn.univrs.io)

---

## License

MIT
