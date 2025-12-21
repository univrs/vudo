# VUDO OS: 3-Year Roadmap

> From Self-Referential Language to Global Imaginarium

---

## Timeline Overview

```
Year 1: GENESIS              Year 2: MANIFESTATION        Year 3: EMERGENCE
───────────────────────────  ───────────────────────────  ───────────────────────────
DOL becomes Turing complete  VUDO OS runs Spirits         The Imaginarium goes live
Self-hosting achieved        Tauri IDE launches           Creator economy activates
WASM pipeline works          Network layer connects       Organic growth begins

     Q1   Q2   Q3   Q4            Q1   Q2   Q3   Q4            Q1   Q2   Q3   Q4
     ▓▓▓▓ ▓▓▓▓ ▓▓▓▓ ▓▓▓▓          ░░░░ ░░░░ ░░░░ ░░░░          ░░░░ ░░░░ ░░░░ ░░░░
     ▲                            ▲                            ▲
     │                            │                            │
     DOL 2.0 Complete             First Séance                 Public Launch
```

---

## Year 1: Genesis — Self-Referential DOL

*"The language that writes itself"*

### Exit Criterion

```bash
$ dol compile dol/compiler.dol -o dol-compiler.wasm
$ ./dol-compiler.wasm dol/compiler.dol -o dol-compiler-2.wasm
$ diff dol-compiler.wasm dol-compiler-2.wasm  # identical
```

DOL successfully compiles itself to WASM, and the output can recompile the source.

---

### Q1: Turing Extensions (Jan-Mar)

**Goal:** DOL gains control flow, explicit types, and becomes computationally complete.

| Week | Milestone | Deliverable |
|------|-----------|-------------|
| 1-2 | Type system design | `types.spec.dol` — Full type hierarchy spec |
| 3-4 | Control flow syntax | `control.spec.dol` — if/match/loop/while/for |
| 5-6 | Lexer extension | Updated Logos lexer with new tokens |
| 7-8 | Parser extension | Recursive descent for new constructs |
| 9-10 | AST updates | Complete AST types with spans |
| 11-12 | Integration tests | 100+ test cases for new syntax |

**Artifacts:**
- `dol-parse` crate v2.0 with Turing extensions
- DOL 2.0 syntax specification (in DOL format)
- Test suite: `tests/control/*.dol.test`

**Dependencies:** Current DOL parser (complete)

---

### Q2: Functional Composition (Apr-Jun)

**Goal:** DOL supports functional programming patterns and meta-programming.

| Week | Milestone | Deliverable |
|------|-----------|-------------|
| 1-2 | Pipe operator `\|>` | Left-to-right composition |
| 3-4 | Compose operator `>>` | Function pipeline building |
| 5-6 | Lambda expressions | Closure syntax and semantics |
| 7-8 | Higher-order functions | Type system support |
| 9-10 | Quote/Eval | AST manipulation primitives |
| 11-12 | Macro system | Compile-time code generation |

**Artifacts:**
- `dol-parse` crate v2.1 with composition operators
- Meta-programming guide
- Test suite: `tests/functional/*.dol.test`

**Dependencies:** Q1 type system

---

### Q3: LLVM MCP Server (Jul-Sep)

**Goal:** AI agents can request compilation via MCP protocol.

| Week | Milestone | Deliverable |
|------|-----------|-------------|
| 1-2 | DOL-IR design | High-level intermediate representation |
| 3-4 | HIR → MLIR lowering | Type-preserving transformation |
| 5-6 | MLIR → WASM backend | WebAssembly code generation |
| 7-8 | MCP server impl | `dol-mcp` crate with compile/validate tools |
| 9-10 | Secondary targets | Zig and Rust emission (optional) |
| 11-12 | Integration testing | End-to-end compilation tests |

**Artifacts:**
- `dol-ir` crate — HIR/MLIR representation
- `dol-emit` crate — Multi-target code generation
- `dol-mcp` crate — MCP server for AI integration
- Pipeline: `.dol → HIR → MLIR → .wasm`

**Dependencies:** Q2 complete language, MLIR libraries

---

### Q4: Self-Hosting (Oct-Dec)

**Goal:** DOL compiler written in DOL, bootstrapped from Rust.

| Week | Milestone | Deliverable |
|------|-----------|-------------|
| 1-2 | Lexer in DOL | `dol/lexer.dol` — Token definitions |
| 3-4 | Parser in DOL | `dol/parser.dol` — Recursive descent |
| 5-6 | Type checker in DOL | `dol/types.dol` — Semantic analysis |
| 7-8 | IR generator in DOL | `dol/ir.dol` — HIR emission |
| 9-10 | WASM emitter in DOL | `dol/emit.dol` — Code generation |
| 11-12 | Bootstrap validation | Self-compilation succeeds |

**Artifacts:**
- `dol/` directory — Complete DOL compiler in DOL
- Bootstrap script — Rust compiler → DOL compiler → self
- Verification tests — Output equivalence checks

**Dependencies:** All Q1-Q3 complete

---

## Year 2: Manifestation — VUDO OS

*"The machine that runs Spirits"*

### Exit Criterion

Two VUDO nodes on different machines successfully:
1. Exchange a Spirit package over the network
2. Execute the Spirit locally
3. Communicate results via Séance

---

### Q1: VUDO VM Core (Jan-Mar)

**Goal:** WebAssembly runtime with DOL-specific extensions.

| Milestone | Deliverable |
|-----------|-------------|
| WASM runtime selection | Integration with wasmer or wasmtime |
| .dol bytecode format | Specification for compiled modules |
| Memory sandboxing | Resource limits and isolation |
| System call interface | Host function bindings |
| Debug support | Source maps and breakpoints |

**Artifacts:**
- `vudo-vm` crate — WASM execution engine
- `vudo-syscalls` — Host function definitions
- Runtime specification document

---

### Q2: VUDO OS Primitives (Apr-Jun)

**Goal:** Core OS concepts: processes, IPC, persistence, identity.

| Concept | Implementation |
|---------|----------------|
| **Ghost** (process) | Ephemeral WASM instance with lifecycle |
| **Spell** (IPC) | Message passing between Ghosts |
| **Spirit** (storage) | Persistent .dol package format |
| **Identity** | Ed25519 node keys (Univrs pattern) |
| **Permissions** | Capability-based access control |

**Artifacts:**
- `vudo-os` crate — Core OS primitives
- Spirit package format specification
- Permission model documentation

---

### Q3: Tauri IDE Alpha (Jul-Sep)

**Goal:** Desktop IDE for DOL development.

| Feature | Description |
|---------|-------------|
| Editor | Syntax highlighting, auto-complete |
| Live preview | Hot reload compiled output |
| Spirit packaging | Bundle .dol files for distribution |
| Local Séance | Test multiplayer locally |
| Debugger | Step through DOL execution |

**Artifacts:**
- Tauri application binary (macOS, Windows, Linux)
- IDE plugin architecture
- User documentation

---

### Q4: Network Layer (Oct-Dec)

**Goal:** P2P Spirit exchange between nodes.

| Component | Technology |
|-----------|------------|
| Discovery | Chitchat gossip protocol |
| Transport | Encrypted WebSocket/QUIC |
| Syncing | Spirit package transfer |
| Reputation | Basic trust scoring |

**Artifacts:**
- `vudo-net` crate — P2P networking
- Protocol specification
- Network simulation tests

---

## Year 3: Emergence — The Imaginarium

*"The playground that grows itself"*

### Exit Criterion

- 1,000+ registered creators
- 100+ published Spirits in marketplace
- Organic growth (new users discovering via network)
- Positive credit flow (creators earning)

---

### Q1: Mycelial Credits (Jan-Mar)

**Goal:** Economic layer for Spirit exchange.

| Component | Description |
|-----------|-------------|
| Credit ledger | Distributed accounting |
| Pricing tiers | Free, Explorer, Creator, Houngan |
| Creator payouts | Credit → creator wallets |
| Attribution chains | Derivative credit sharing |

**Artifacts:**
- `mycelial-credits` crate
- Economic whitepaper
- Creator onboarding guide

---

### Q2: Spirit Marketplace (Apr-Jun)

**Goal:** Discovery and distribution platform.

| Feature | Description |
|---------|-------------|
| Browse | Category-based Spirit discovery |
| Search | Full-text search across Spirits |
| Ratings | User reviews and quality scores |
| Remixing | Fork Spirits, maintain attribution |
| Collections | Curated Spirit bundles |

**Artifacts:**
- Marketplace web interface
- API for third-party clients
- Moderation tools

---

### Q3: Browser IDE (Jul-Sep)

**Goal:** Low-memory VUDO client for web access.

| Feature | Description |
|---------|-------------|
| Web editor | DOL editing in browser |
| WASM-native | Runs entirely client-side |
| Collaborative Séances | Real-time multi-user editing |
| Mobile support | Responsive design |

**Artifacts:**
- Web application (React + WASM)
- Progressive Web App manifest
- Mobile-optimized UI

---

### Q4: Imaginarium Launch (Oct-Dec)

**Goal:** Public launch with growth infrastructure.

| Activity | Description |
|----------|-------------|
| Marketing | Developer outreach, content creation |
| Onboarding | Tutorial Spirits, learning paths |
| Showcases | Featured creator Spirits |
| Metrics | Growth dashboards, health monitoring |
| Community | Forums, Discord, governance |

**Artifacts:**
- Launch campaign materials
- Showcase Spirit collection
- Community guidelines
- Growth metrics dashboard

---

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| DOL self-hosting complexity | Start simple, iterate; keep Rust fallback |
| WASM performance | Benchmark early; optimize hot paths |
| Network security | Security audit before public launch |
| Creator adoption | Seed with internal Spirits; incentivize early creators |
| Scope creep | Strict quarterly reviews; cut non-essential features |

---

## Dependencies

```
External:
├── LLVM/MLIR libraries
├── Wasmer or Wasmtime
├── Tauri framework
├── Chitchat gossip protocol
└── Ed25519 cryptography (ring/dalek)

Internal:
├── Univrs orchestrator (platform integration)
├── MCP server infrastructure
└── DOL 1.0 parser (bootstrap base)
```

---

## Success Metrics

| Metric | Y1 Target | Y2 Target | Y3 Target |
|--------|-----------|-----------|-----------|
| DOL test coverage | 90% | 95% | 95% |
| Self-host success | ✓ | — | — |
| Active VUDO nodes | — | 100 | 10,000 |
| Spirits published | — | 10 | 1,000 |
| Monthly active users | — | 50 | 5,000 |
| Creator earnings | — | 0 | 10K credits/mo |
| Network uptime | — | 99% | 99.9% |

---

*"From ontology to ecosystem, specification to civilization."*
