# VUDO & Univrs CLI Specification

> **Version:** 1.0.0  
> **Status:** Design Specification  
> **Date:** December 25, 2025

-----

## Overview

Two CLIs serving two audiences, unified by the Mycelium:

|CLI     |Audience |Purpose           |Install                                    |
|--------|---------|------------------|-------------------------------------------|
|`vudo`  |Creators |Make Spirits      |`curl -fsSL vudo.univrs.io/install.sh | sh`|
|`univrs`|Operators|Run Infrastructure|`cargo install univrs`                     |

-----

## `vudo` — The Creator’s CLI

### Core Philosophy

```
Simple by default. Powerful when needed.
```

Every command has a “just works” mode and power-user flags.

-----

### Command Structure

```
vudo
├── new          # Create new Spirit project
├── build        # Compile .dol → .spirit
├── run          # Execute locally
├── test         # Run tests
├── pack         # Package for distribution
├── sign         # Sign with Ed25519
├── publish      # Publish to Imaginarium
├── summon       # Download Spirit from Imaginarium
├── search       # Search Imaginarium
├── info         # Spirit details
├── check        # Validate .dol syntax
├── fmt          # Format .dol files
├── doc          # Generate documentation
├── dol          # Enter DOL REPL ← Interactive mode
├── upgrade      # Update vudo CLI
└── help         # Help
```

-----

### Command Details

#### `vudo new`

Create a new Spirit project.

```bash
# Basic
vudo new hello-spirit

# With template
vudo new hello-spirit --template web-service
vudo new hello-spirit --template cli-tool
vudo new hello-spirit --template library

# Creates:
# hello-spirit/
# ├── manifest.toml      # Spirit metadata
# ├── src/
# │   └── main.dol       # Entry point
# └── tests/
#     └── main_test.dol  # Tests
```

#### `vudo build`

Compile DOL source to Spirit package.

```bash
# Simple - just works
vudo build

# Power user
vudo build --emit=ast          # Show AST
vudo build --emit=hir          # Show HIR  
vudo build --emit=mlir         # Show MLIR
vudo build --emit=wasm         # Show WASM (default output)
vudo build --target=wasm32     # 32-bit WASM
vudo build --target=wasm64     # 64-bit WASM
vudo build --target=native     # Native binary
vudo build --release           # Optimized build
vudo build --features=network  # Enable features
vudo build -o custom.spirit    # Custom output name
```

#### `vudo run`

Execute a Spirit locally in the sandbox.

```bash
# Run current project
vudo run

# Run specific Spirit
vudo run ./my-spirit.spirit

# Run from Imaginarium (auto-summon)
vudo run @creator/hello-spirit

# With arguments
vudo run -- --port 8080 --verbose

# Power user
vudo run --fuel=1000000        # Set fuel limit
vudo run --memory=64mb         # Set memory limit
vudo run --capabilities=net,fs # Grant capabilities
vudo run --sandbox=strict      # Maximum isolation
vudo run --trace               # Execution trace
```

#### `vudo test`

Run Spirit tests.

```bash
# Run all tests
vudo test

# Run specific test
vudo test test_greeting

# With coverage
vudo test --coverage

# Watch mode
vudo test --watch
```

#### `vudo pack`

Package Spirit for distribution.

```bash
# Basic
vudo pack

# Creates: hello-spirit-1.0.0.spirit

# Power user
vudo pack --include=assets/    # Include extra files
vudo pack --exclude=*.log      # Exclude patterns
vudo pack --compress=zstd      # Compression algorithm
```

#### `vudo sign`

Sign package with Ed25519 identity.

```bash
# Sign with default identity
vudo sign hello-spirit-1.0.0.spirit

# Sign with specific key
vudo sign hello-spirit-1.0.0.spirit --key ~/.vudo/keys/release.key

# Verify signature
vudo sign --verify hello-spirit-1.0.0.spirit
```

#### `vudo publish`

Publish to the Imaginarium.

```bash
# Publish current project
vudo publish

# Publish specific package
vudo publish hello-spirit-1.0.0.spirit

# Publish with visibility
vudo publish --public          # Anyone can summon
vudo publish --unlisted        # Link-only access
vudo publish --private         # Invite-only

# Publish with pricing
vudo publish --free            # Free tier
vudo publish --credits=10      # 10 credits per summon
```

#### `vudo summon`

Download Spirit from Imaginarium.

```bash
# Summon by name
vudo summon hello-spirit

# Summon specific version
vudo summon hello-spirit@1.2.0

# Summon from creator
vudo summon @alice/hello-spirit

# Summon and run immediately
vudo summon --run @alice/hello-spirit
```

#### `vudo search`

Search the Imaginarium.

```bash
# Search by keyword
vudo search image processing

# Search by tag
vudo search --tag=visualization

# Search by creator
vudo search --creator=alice

# Interactive browser
vudo search --interactive
```

#### `vudo check`

Validate DOL syntax and types.

```bash
# Check current project
vudo check

# Check specific file
vudo check src/main.dol

# Check with strict mode
vudo check --strict

# Output format
vudo check --format=json       # For tooling
vudo check --format=pretty     # For humans (default)
```

#### `vudo fmt`

Format DOL source files.

```bash
# Format current project
vudo fmt

# Check formatting (CI mode)
vudo fmt --check

# Format specific file
vudo fmt src/main.dol
```

#### `vudo doc`

Generate documentation.

```bash
# Generate docs
vudo doc

# Open in browser
vudo doc --open

# Output format
vudo doc --format=html         # HTML (default)
vudo doc --format=markdown     # Markdown
vudo doc --format=json         # JSON schema
```

-----

### `vudo dol` — The DOL REPL

Interactive environment for DOL development.

```bash
$ vudo dol

╔══════════════════════════════════════════════════════════════════╗
║  VUDO DOL REPL v0.2.0                                            ║
║  The system that knows what it is, becomes what it knows.        ║
║                                                                  ║
║  Type :help for commands, :quit to exit                          ║
╚══════════════════════════════════════════════════════════════════╝

DOL> 
```

#### REPL Commands

```
DOL> :help

Commands:
  :help, :h              Show this help
  :quit, :q              Exit REPL
  :clear, :c             Clear screen
  :reset                 Reset environment
  
  :load <file>           Load .dol file
  :save <file>           Save session to file
  :history               Show command history
  
  :type <expr>           Show type of expression
  :ast <expr>            Show AST
  :mlir <expr>           Show MLIR output
  :wasm <expr>           Show WASM output
  
  :check                 Type-check current definitions
  :test                  Run inline tests
  :bench <expr>          Benchmark expression
  
  :env                   Show defined symbols
  :doc <name>            Show documentation for symbol
  
  :set <option> <value>  Set REPL option
  :get <option>          Get REPL option

Options:
  show_types     true/false    Show types in output
  show_timing    true/false    Show execution time
  fuel_limit     <number>      Max fuel for execution
  strict_mode    true/false    Enable strict type checking
```

#### REPL Examples

```
DOL> 2 + 2
4 : Int64

DOL> fun square(x: Int64) -> Int64 { x * x }
square : Fun<Int64, Int64>

DOL> square(5)
25 : Int64

DOL> :type square
square : Fun<Int64, Int64>

DOL> gene Point { has x: Float64, has y: Float64 }
Point : Gene

DOL> let p = Point { x: 3.0, y: 4.0 }
p : Point

DOL> fun distance(p: Point) -> Float64 {
...>     (p.x ** 2 + p.y ** 2) ** 0.5
...> }
distance : Fun<Point, Float64>

DOL> distance(p)
5.0 : Float64

DOL> :ast 2 + 2
BinaryOp {
  left: Literal(Int64(2)),
  op: Plus,
  right: Literal(Int64(2))
}

DOL> :mlir fun add(a: Int64, b: Int64) -> Int64 { a + b }
module {
  func.func @add(%arg0: i64, %arg1: i64) -> i64 {
    %0 = arith.addi %arg0, %arg1 : i64
    return %0 : i64
  }
}

DOL> :load examples/biology/hyphal.dol
Loaded: Hyphal, HyphalTip, HyphalSegment

DOL> :env
square    : Fun<Int64, Int64>
distance  : Fun<Point, Float64>
p         : Point
Point     : Gene
Hyphal    : Trait
HyphalTip : Gene
...

DOL> :set show_timing true
show_timing = true

DOL> square(1000000)
1000000000000 : Int64    [0.003ms]

DOL> :quit
Goodbye! May your Spirits thrive in Bondieu. 🍄
```

#### Multi-line Input

```
DOL> gene Container {
...>     has id: UInt64
...>     has name: String
...>     has image: String
...>     
...>     constraint valid_name {
...>         name.len() > 0
...>     }
...> }
Container : Gene

DOL> :doc Container
Gene Container
  Fields:
    id    : UInt64
    name  : String
    image : String
  Constraints:
    valid_name: name.len() > 0
```

#### Testing in REPL

```
DOL> fun fib(n: Int64) -> Int64 {
...>     if n <= 1 { n }
...>     else { fib(n - 1) + fib(n - 2) }
...> }
fib : Fun<Int64, Int64>

DOL> #assert(fib(0) == 0)
✓ assertion passed

DOL> #assert(fib(10) == 55)
✓ assertion passed

DOL> :bench fib(20)
fib(20) = 6765
  mean:   1.234ms
  stddev: 0.045ms
  runs:   100
```

-----

## `univrs` — The Operator’s CLI

### Command Structure

```
univrs
├── node
│   ├── init        # Initialize node
│   ├── start       # Start daemon
│   ├── stop        # Stop daemon
│   ├── status      # Show status
│   ├── join        # Join network
│   ├── leave       # Leave network
│   └── config      # Configure node
├── network
│   ├── list        # List peers
│   ├── stats       # Network statistics
│   ├── ping        # Ping peer
│   └── prune       # Remove dead connections
├── deploy
│   ├── <spirit>    # Deploy Spirit
│   ├── list        # List deployments
│   ├── status      # Deployment status
│   ├── logs        # View logs
│   └── rollback    # Rollback deployment
├── scale
│   └── <spirit> <n># Scale replicas
├── identity
│   ├── new         # Generate keypair
│   ├── show        # Show public key
│   ├── export      # Export keys
│   └── import      # Import keys
├── credits
│   ├── balance     # Show balance
│   ├── history     # Transaction history
│   └── transfer    # Transfer credits
└── help
```

-----

### Command Details

#### `univrs node init`

Initialize a new node.

```bash
# Basic initialization
univrs node init

# With specific identity
univrs node init --identity ~/.univrs/node.key

# With role
univrs node init --role=validator
univrs node init --role=relay
univrs node init --role=storage

# Creates:
# ~/.univrs/
# ├── config.toml      # Node configuration
# ├── node.key         # Ed25519 private key
# ├── node.pub         # Ed25519 public key
# └── data/            # Node data directory
```

#### `univrs node start`

Start the node daemon.

```bash
# Start with defaults
univrs node start

# Start in foreground
univrs node start --foreground

# Start with specific config
univrs node start --config /etc/univrs/config.toml

# Start with logging
univrs node start --log-level=debug
```

#### `univrs node join`

Join a Mycelium network.

```bash
# Join mainnet
univrs node join mainnet

# Join testnet
univrs node join testnet

# Join custom network
univrs node join --bootstrap=192.168.1.100:9000

# Join with invite
univrs node join --invite=<invite-code>
```

#### `univrs deploy`

Deploy a Spirit to the network.

```bash
# Deploy Spirit
univrs deploy hello-spirit

# Deploy with replicas
univrs deploy hello-spirit --replicas=3

# Deploy with placement constraints
univrs deploy hello-spirit --region=us-west

# Deploy with resources
univrs deploy hello-spirit --memory=256mb --cpu=0.5

# Deploy from Imaginarium
univrs deploy @alice/hello-spirit
```

#### `univrs identity`

Manage cryptographic identity.

```bash
# Generate new identity
univrs identity new

# Show public key
univrs identity show
# Output: ed25519:ABC123...XYZ

# Export for backup
univrs identity export --output=backup.key

# Import identity
univrs identity import backup.key
```

#### `univrs credits`

Manage Mycelial Credits.

```bash
# Check balance
univrs credits balance
# Output: 1,234 credits

# View history
univrs credits history
univrs credits history --limit=10

# Transfer credits
univrs credits transfer --to=ed25519:ABC123... --amount=100
```

-----

## Configuration Files

### `~/.vudo/config.toml` (Creator)

```toml
[identity]
default_key = "~/.vudo/keys/default.key"

[build]
default_target = "wasm32"
optimization_level = 2

[publish]
default_registry = "https://imaginarium.vudo.univrs.io"
default_visibility = "public"

[repl]
show_types = true
show_timing = false
history_file = "~/.vudo/history"
```

### `~/.univrs/config.toml` (Operator)

```toml
[node]
id = "ed25519:ABC123..."
role = "validator"
data_dir = "~/.univrs/data"

[network]
listen_addr = "0.0.0.0:9000"
bootstrap_peers = [
    "mainnet.univrs.io:9000",
]
max_peers = 50

[resources]
max_memory = "4gb"
max_cpu = 2.0
max_storage = "100gb"

[logging]
level = "info"
format = "json"
output = "~/.univrs/logs/node.log"
```

-----

## Installation

### `vudo` (Creators)

```bash
# macOS / Linux
curl -fsSL https://vudo.univrs.io/install.sh | sh

# Windows
irm https://vudo.univrs.io/install.ps1 | iex

# Cargo (for Rust developers)
cargo install vudo

# Verify
vudo --version
```

### `univrs` (Operators)

```bash
# Cargo (primary method)
cargo install univrs

# From source
git clone https://github.com/univrs/univrs
cd univrs && cargo install --path univrs-cli

# Docker
docker pull univrs/univrs
docker run -d univrs/univrs node start

# Verify
univrs --version
```

-----

## User Journeys Revisited

### Creator Journey (Complete)

```bash
# 1. Install
curl -fsSL https://vudo.univrs.io/install.sh | sh

# 2. Create
vudo new my-spirit
cd my-spirit

# 3. Explore (REPL)
vudo dol
DOL> :load src/main.dol
DOL> # experiment...
DOL> :quit

# 4. Build
vudo build

# 5. Test
vudo test

# 6. Run locally
vudo run

# 7. Package
vudo pack
vudo sign

# 8. Publish
vudo publish

# 9. Share
echo "Try it: vudo summon @me/my-spirit"
```

### Operator Journey (Complete)

```bash
# 1. Install
cargo install univrs

# 2. Initialize
univrs identity new
univrs node init

# 3. Join network
univrs node start
univrs node join mainnet

# 4. Deploy Spirits
univrs deploy @alice/hello-spirit --replicas=3

# 5. Monitor
univrs deploy status hello-spirit
univrs deploy logs hello-spirit

# 6. Scale
univrs scale hello-spirit 5

# 7. Manage credits
univrs credits balance
```

-----

## Summary

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│  CREATOR                                OPERATOR                            │
│                                                                             │
│  $ vudo                                 $ univrs                            │
│  ├── new       Create Spirit            ├── node      Manage node          │
│  ├── build     Compile                  ├── network   Manage peers         │
│  ├── run       Execute locally          ├── deploy    Deploy Spirits       │
│  ├── test      Run tests                ├── scale     Scale replicas       │
│  ├── pack      Package                  ├── identity  Manage keys          │
│  ├── sign      Sign package             └── credits   Manage credits       │
│  ├── publish   Publish                                                      │
│  ├── summon    Download Spirit                                              │
│  ├── search    Search Imaginarium                                           │
│  ├── check     Validate .dol                                                │
│  ├── fmt       Format .dol                                                  │
│  ├── doc       Generate docs                                                │
│  └── dol       Enter REPL ←────── Interactive DOL environment              │
│                                                                             │
│        DOL> fun square(x: Int64) -> Int64 { x * x }                        │
│        DOL> square(5)                                                       │
│        25 : Int64                                                           │
│        DOL> :wasm square                                                    │
│        DOL> :quit                                                           │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

-----

*“Imagine. Summon. Create.”*