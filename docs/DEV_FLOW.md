# Create
vudo new my-spirit --template cli-tool
cd my-spirit

# Develop
vudo dol                    # REPL
vudo check                  # Validate
vudo fmt                    # Format

# Build & Test
vudo build
vudo test

# Package & Sign
vudo pack                   # Creates .spirit
vudo sign my-spirit.spirit  # Ed25519 signature

# Verify & Run
vudo sign --verify my-spirit.spirit
vudo run                    # Execute in sandbox

# Registry (with signature verification!)
# Registry.install() now verifies signatures before accepting
```

## Security Chain Complete
```
Creator                          User
   │                               │
   ▼                               ▼
┌──────────────┐              ┌──────────────┐
│ SigningKey   │              │ VerifyingKey │
│ (private)    │              │ (public)     │
└──────┬───────┘              └──────┬───────┘
       │                             │
       ▼                             ▼
┌──────────────┐              ┌──────────────┐
│ sign()       │───────────────│ verify()     │
│ manifest     │  .spirit pkg  │ on install   │
└──────────────┘              └──────────────┘
```

---

## Ready for Phase 3: Hyphal Network 🍄
```
╔═══════════════════════════════════════════════════════════════════════════╗
║                         PHASE 3: HYPHAL NETWORK                           ║
╠═══════════════════════════════════════════════════════════════════════════╣
║                                                                           ║
║  Foundation Ready:                                                        ║
║  ├── ✅ Ed25519 Identity (univrs-identity + spirit_runtime)               ║
║  ├── ✅ Chitchat Gossip (univrs-network)                                  ║
║  ├── ✅ Spirit Sandbox (vudo_vm)                                          ║
║  └── ✅ DOL Biomimicry Traits (Hyphal, Transport in stdlib)               ║
║                                                                           ║
║  Phase 3 Work:                                                            ║
║  ├── Physarum-inspired routing                                            ║
║  ├── OpenRaft consensus for credits                                       ║
║  ├── Extended Chitchat for resource gradients                             ║
║  ├── P2P Spirit distribution                                              ║
║  └── WASM 3D visualization (Bondieu network)                              ║
║                                                                           ║
║  Parallel Track:                                                          ║
║  └── DOL v0.3.0 HIR → WASM pipeline                                       ║
║                                                                           ║
╚═══════════════════════════════════════════════════════════════════════════╝