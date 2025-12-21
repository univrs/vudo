# Claude-Flow Year 1 Swarm: DOL Genesis

> Multi-agent orchestration for DOL 2.0 self-hosting

## Swarm Architecture

```
                          ┌─────────────────────────────┐
                          │      ORCHESTRATOR           │
                          │  (claude-flow coordinator)  │
                          └──────────────┬──────────────┘
                                         │
            ┌────────────────────────────┼────────────────────────────┐
            │                            │                            │
            ▼                            ▼                            ▼
┌───────────────────────┐  ┌───────────────────────┐  ┌───────────────────────┐
│   SPEC-STREAM         │  │   IMPL-STREAM         │  │   TEST-STREAM         │
│   (Specification)     │  │   (Implementation)    │  │   (Validation)        │
└───────────────────────┘  └───────────────────────┘  └───────────────────────┘
            │                            │                            │
    ┌───────┴───────┐            ┌───────┴───────┐            ┌───────┴───────┐
    │               │            │               │            │               │
    ▼               ▼            ▼               ▼            ▼               ▼
┌────────┐    ┌────────┐    ┌────────┐    ┌────────┐    ┌────────┐    ┌────────┐
│ DOL-   │    │ DOL-   │    │ DOL-   │    │ DOL-   │    │ DOL-   │    │ DOL-   │
│ TYPE   │    │ CTRL   │    │ PARSE  │    │ EMIT   │    │ UNIT   │    │ INTEG  │
│ SPEC   │    │ SPEC   │    │ IMPL   │    │ IMPL   │    │ TEST   │    │ TEST   │
└────────┘    └────────┘    └────────┘    └────────┘    └────────┘    └────────┘
```

---

## Agent Definitions

### Orchestrator Agent

```yaml
agent: orchestrator
role: coordinator
description: |
  Central coordinator for the DOL Genesis swarm.
  Manages dependencies, schedules tasks, resolves conflicts.
  
responsibilities:
  - Track task dependencies and completion status
  - Resolve blocking issues between agents
  - Maintain global project state
  - Generate progress reports
  
tools:
  - project_state: Read/write project status
  - agent_dispatch: Assign tasks to agents
  - conflict_resolution: Mediate agent disagreements
  - progress_report: Generate milestone reports
```

### Specification Agents

```yaml
agent: dol-type-spec
role: specification
stream: spec-stream
description: |
  Designs DOL 2.0 type system specification.
  Ensures MLIR alignment and ontological consistency.

responsibilities:
  - Define primitive types (Int8..Int64, Float32, Float64, Bool, String)
  - Define composite types (Array, Slice, Pointer, Optional, Result, Tuple)
  - Define ontological types (Gene, Trait, Constraint, System, Evolves)
  - Define type aliases and struct syntax
  - Write type system specification in DOL format

outputs:
  - spec/types/primitives.spec.dol
  - spec/types/composites.spec.dol
  - spec/types/ontology.spec.dol
  - spec/types/aliases.spec.dol

dependencies: []

validation:
  - All types have MLIR lowering defined
  - No circular type dependencies
  - Backward compatible with DOL 1.0 ontology types
```

```yaml
agent: dol-control-spec
role: specification
stream: spec-stream
description: |
  Designs DOL 2.0 control flow specification.
  Defines if/match/loop/while/for constructs.

responsibilities:
  - Define conditional syntax (if, else, else if)
  - Define pattern matching (match, case, where guards)
  - Define loop constructs (for, while, loop, break, continue)
  - Define expression vs statement contexts
  - Write control flow specification in DOL format

outputs:
  - spec/control/conditionals.spec.dol
  - spec/control/matching.spec.dol
  - spec/control/loops.spec.dol
  - spec/control/expressions.spec.dol

dependencies:
  - dol-type-spec (types must exist for expressions)

validation:
  - All control flow maps to MLIR SCF dialect
  - Exhaustive match checking defined
  - Loop termination semantics clear
```

```yaml
agent: dol-compose-spec
role: specification
stream: spec-stream
description: |
  Designs DOL 2.0 functional composition specification.
  Defines pipe, compose, lambda, higher-order functions.

responsibilities:
  - Define pipe operator (|>) semantics
  - Define compose operator (>>) semantics
  - Define apply (@) and bind (:=) operators
  - Define lambda expression syntax
  - Define higher-order function types

outputs:
  - spec/compose/pipe.spec.dol
  - spec/compose/lambda.spec.dol
  - spec/compose/higher-order.spec.dol
  - spec/compose/operators.spec.dol

dependencies:
  - dol-type-spec (function types)
  - dol-control-spec (expression contexts)

validation:
  - Type inference rules complete
  - Closure capture semantics defined
  - Partial application well-specified
```

```yaml
agent: dol-meta-spec
role: specification
stream: spec-stream
description: |
  Designs DOL 2.0 meta-programming specification.
  Defines quote, eval, macro, reflect primitives.

responsibilities:
  - Define quote (') syntax and AST representation
  - Define eval (!) semantics and scoping
  - Define macro (#) system with hygiene
  - Define reflect (?) type introspection
  - Write meta-programming specification in DOL format

outputs:
  - spec/meta/quote.spec.dol
  - spec/meta/eval.spec.dol
  - spec/meta/macro.spec.dol
  - spec/meta/reflect.spec.dol

dependencies:
  - dol-type-spec (TypeInfo for reflection)
  - dol-control-spec (AST node types)
  - dol-compose-spec (function manipulation)

validation:
  - Hygiene rules prevent name capture
  - Quote/eval preserve type information
  - Reflection is read-only (no runtime modification)
```

### Implementation Agents

```yaml
agent: dol-lexer-impl
role: implementation
stream: impl-stream
description: |
  Extends DOL lexer for 2.0 syntax.
  Uses Logos for tokenization.

responsibilities:
  - Add new keyword tokens (if, match, for, while, loop, etc.)
  - Add operator tokens (|>, >>, @, :=, ', !, #, ?)
  - Add type-related tokens
  - Update span tracking
  - Maintain backward compatibility

inputs:
  - spec/types/*.spec.dol
  - spec/control/*.spec.dol
  - spec/compose/*.spec.dol
  - spec/meta/*.spec.dol

outputs:
  - src/lexer.rs (updated)
  - src/tokens.rs (new token types)

dependencies:
  - dol-type-spec
  - dol-control-spec
  - dol-compose-spec
  - dol-meta-spec

validation:
  - All spec tokens recognized
  - No ambiguous tokenization
  - 100% test coverage on new tokens
```

```yaml
agent: dol-parser-impl
role: implementation
stream: impl-stream
description: |
  Extends DOL parser for 2.0 syntax.
  Recursive descent with precedence climbing.

responsibilities:
  - Parse type declarations and aliases
  - Parse control flow constructs
  - Parse functional composition expressions
  - Parse meta-programming constructs
  - Produce complete AST with spans

inputs:
  - All spec files
  - src/lexer.rs (updated by dol-lexer-impl)

outputs:
  - src/parser.rs (updated)
  - src/ast.rs (new AST node types)

dependencies:
  - dol-lexer-impl

validation:
  - All spec examples parse correctly
  - Error recovery produces useful messages
  - AST round-trips (parse → print → parse)
```

```yaml
agent: dol-typecheck-impl
role: implementation
stream: impl-stream
description: |
  Implements DOL 2.0 type checker.
  Semantic analysis and type inference.

responsibilities:
  - Type inference for expressions
  - Constraint solving for generics
  - Trait implementation checking
  - Constraint validation
  - Error reporting with suggestions

inputs:
  - src/ast.rs (from dol-parser-impl)
  - spec/types/*.spec.dol

outputs:
  - src/typecheck.rs (new)
  - src/types.rs (type representation)
  - src/constraints.rs (constraint solver)

dependencies:
  - dol-parser-impl

validation:
  - Type errors have clear messages
  - Inference succeeds on valid code
  - Constraint satisfaction is decidable
```

```yaml
agent: dol-ir-impl
role: implementation
stream: impl-stream
description: |
  Implements DOL high-level IR (HIR).
  Prepares for MLIR lowering.

responsibilities:
  - Define HIR node types
  - Lower AST to HIR
  - Perform HIR optimizations
  - Generate MLIR-compatible representation

inputs:
  - src/ast.rs (typed AST)
  - src/typecheck.rs

outputs:
  - src/hir.rs (HIR definition)
  - src/ast_to_hir.rs (lowering)
  - src/hir_opt.rs (optimizations)

dependencies:
  - dol-typecheck-impl

validation:
  - All AST nodes lower to HIR
  - Type information preserved
  - HIR validates independently
```

```yaml
agent: dol-emit-impl
role: implementation
stream: impl-stream
description: |
  Implements code generation to WASM.
  Primary target for VUDO OS.

responsibilities:
  - Emit MLIR from HIR
  - Lower MLIR to LLVM IR
  - Generate WebAssembly
  - Support debug information

inputs:
  - src/hir.rs
  - MLIR/LLVM libraries

outputs:
  - src/emit_mlir.rs
  - src/emit_wasm.rs
  - src/debug_info.rs

dependencies:
  - dol-ir-impl

validation:
  - Generated WASM validates (wasm-validate)
  - Simple programs execute correctly
  - Debug info maps to source
```

```yaml
agent: dol-mcp-impl
role: implementation
stream: impl-stream
description: |
  Implements MCP server for AI-driven compilation.
  Exposes DOL toolchain via Model Context Protocol.

responsibilities:
  - Define MCP tool schemas
  - Implement compile/validate/format tools
  - Handle streaming compilation
  - Provide diagnostics via MCP

outputs:
  - src/mcp/server.rs
  - src/mcp/tools.rs
  - src/mcp/schema.json

dependencies:
  - dol-emit-impl (for compilation)
  - dol-typecheck-impl (for validation)

validation:
  - MCP tools callable from Claude
  - Error messages useful for AI correction
  - Streaming works for large compilations
```

### Test Agents

```yaml
agent: dol-unit-test
role: testing
stream: test-stream
description: |
  Creates unit tests for DOL 2.0 components.
  Test-driven validation of implementation.

responsibilities:
  - Generate lexer tests from spec
  - Generate parser tests from examples
  - Generate type checker tests
  - Generate IR tests
  - Maintain test coverage metrics

outputs:
  - tests/lexer/*.rs
  - tests/parser/*.rs
  - tests/typecheck/*.rs
  - tests/ir/*.rs
  - tests/coverage.json

dependencies:
  - All impl agents (as they complete)

validation:
  - 90%+ line coverage
  - All spec examples covered
  - Edge cases tested
```

```yaml
agent: dol-integration-test
role: testing
stream: test-stream
description: |
  Creates end-to-end integration tests.
  Validates complete compilation pipeline.

responsibilities:
  - Compile example .dol files to WASM
  - Execute WASM and verify output
  - Test self-hosting bootstrap
  - Benchmark compilation performance

outputs:
  - tests/integration/*.rs
  - tests/e2e/*.dol
  - tests/bootstrap/
  - benchmarks/

dependencies:
  - dol-emit-impl (full pipeline)

validation:
  - All examples compile and run
  - Bootstrap succeeds
  - Performance within targets
```

```yaml
agent: dol-selfhost-test
role: testing
stream: test-stream
description: |
  Validates DOL self-hosting capability.
  The ultimate test: DOL compiles DOL.

responsibilities:
  - Write DOL lexer in DOL
  - Write DOL parser in DOL
  - Write DOL emitter in DOL
  - Verify self-compilation

inputs:
  - All spec files (reference implementation)
  - All impl agents (Rust implementation to bootstrap)

outputs:
  - dol/lexer.dol
  - dol/parser.dol
  - dol/typecheck.dol
  - dol/emit.dol
  - dol/main.dol
  - bootstrap.sh

dependencies:
  - All impl agents
  - dol-integration-test

validation:
  - DOL compiler compiles itself
  - Output matches Rust-compiled version
  - No external dependencies required
```

---

## Task Dependencies Graph

```
                                    ┌──────────────┐
                                    │  START       │
                                    └──────┬───────┘
                                           │
                    ┌──────────────────────┼──────────────────────┐
                    │                      │                      │
                    ▼                      ▼                      ▼
            ┌───────────────┐      ┌───────────────┐      ┌───────────────┐
            │ dol-type-spec │      │(parallel init)│      │               │
            └───────┬───────┘      └───────────────┘      └───────────────┘
                    │
        ┌───────────┼───────────┐
        │           │           │
        ▼           ▼           ▼
┌───────────────┐ ┌───────────────┐ ┌───────────────┐
│dol-control-   │ │dol-compose-   │ │               │
│spec           │ │spec           │ │               │
└───────┬───────┘ └───────┬───────┘ └───────────────┘
        │                 │
        └────────┬────────┘
                 │
                 ▼
         ┌───────────────┐
         │ dol-meta-spec │
         └───────┬───────┘
                 │
                 ▼
         ┌───────────────┐
         │dol-lexer-impl │
         └───────┬───────┘
                 │
                 ▼
         ┌───────────────┐
         │dol-parser-impl│
         └───────┬───────┘
                 │
        ┌────────┼────────┐
        │                 │
        ▼                 ▼
┌───────────────┐ ┌───────────────┐
│dol-typecheck- │ │dol-unit-test  │
│impl           │ │(ongoing)      │
└───────┬───────┘ └───────────────┘
        │
        ▼
┌───────────────┐
│ dol-ir-impl   │
└───────┬───────┘
        │
        ▼
┌───────────────┐
│ dol-emit-impl │
└───────┬───────┘
        │
        ├────────────────┐
        │                │
        ▼                ▼
┌───────────────┐ ┌───────────────┐
│ dol-mcp-impl  │ │dol-integration│
│               │ │-test          │
└───────────────┘ └───────┬───────┘
                          │
                          ▼
                  ┌───────────────┐
                  │dol-selfhost-  │
                  │test           │
                  └───────┬───────┘
                          │
                          ▼
                  ┌───────────────┐
                  │    DONE       │
                  │(Self-hosting) │
                  └───────────────┘
```

---

## Quarterly Schedule

### Q1: Specifications + Lexer/Parser

**Weeks 1-6: Parallel Specification**
```yaml
parallel:
  - agent: dol-type-spec
    duration: 4 weeks
  - agent: dol-control-spec
    duration: 4 weeks
    starts_after: dol-type-spec week 2
  - agent: dol-compose-spec
    duration: 4 weeks
    starts_after: dol-type-spec week 2
```

**Weeks 5-8: Meta-Spec + Lexer**
```yaml
sequential:
  - agent: dol-meta-spec
    duration: 4 weeks
    starts_after: [dol-control-spec, dol-compose-spec]
  - agent: dol-lexer-impl
    duration: 3 weeks
    starts_after: dol-meta-spec week 2
```

**Weeks 9-12: Parser**
```yaml
sequential:
  - agent: dol-parser-impl
    duration: 4 weeks
    starts_after: dol-lexer-impl
  - agent: dol-unit-test
    continuous: true
    starts_after: dol-lexer-impl
```

### Q2: Type Checking + Composition Implementation

**Weeks 1-6: Type System**
```yaml
sequential:
  - agent: dol-typecheck-impl
    duration: 6 weeks
```

**Weeks 7-12: IR + Testing**
```yaml
parallel:
  - agent: dol-ir-impl
    duration: 4 weeks
  - agent: dol-unit-test
    continuous: true
```

### Q3: Code Generation + MCP

**Weeks 1-8: WASM Emission**
```yaml
sequential:
  - agent: dol-emit-impl
    duration: 8 weeks
```

**Weeks 6-12: MCP + Integration**
```yaml
parallel:
  - agent: dol-mcp-impl
    duration: 4 weeks
    starts_after: dol-emit-impl week 6
  - agent: dol-integration-test
    duration: 6 weeks
    starts_after: dol-emit-impl week 4
```

### Q4: Self-Hosting

**Weeks 1-12: Bootstrap**
```yaml
sequential:
  - agent: dol-selfhost-test
    duration: 12 weeks
    checkpoints:
      week_4: lexer self-compiles
      week_8: parser self-compiles
      week_12: full compiler self-hosts
```

---

## Communication Protocol

### Agent Status Updates

Each agent reports status via structured messages:

```yaml
status_update:
  agent: dol-parser-impl
  timestamp: 2025-01-15T10:30:00Z
  progress:
    completed:
      - type_declarations
      - if_else_statements
    in_progress:
      - match_expressions (70%)
    blocked: []
  metrics:
    lines_written: 1234
    tests_passing: 47
    tests_failing: 3
  blockers: []
  needs:
    - Review of match exhaustiveness spec from dol-control-spec
```

### Handoff Protocol

When an agent completes a deliverable:

```yaml
handoff:
  from: dol-type-spec
  to: [dol-lexer-impl, dol-typecheck-impl]
  artifact: spec/types/primitives.spec.dol
  message: |
    Type specification complete. Key decisions:
    - Int types map directly to MLIR integer types
    - String is heap-allocated, not Copy
    - Gene<T> wraps any type with constraint validation
  verification:
    - All examples parse correctly (manual check)
    - MLIR mappings documented
```

### Conflict Resolution

When agents disagree:

```yaml
conflict:
  agents: [dol-parser-impl, dol-typecheck-impl]
  issue: |
    Parser produces untyped AST, but type checker expects 
    type annotations in AST nodes.
  options:
    a: Parser infers types during parsing (complex)
    b: Two-pass: parse then annotate (clean separation)
    c: AST nodes have Optional<Type> fields
  orchestrator_decision: b
  rationale: |
    Clean separation of concerns. Parser stays simple.
    Type checker has full AST context for inference.
```

---

## Success Criteria

### Per-Agent Exit Criteria

| Agent | Exit Criterion |
|-------|----------------|
| dol-type-spec | All types have MLIR mapping, spec compiles |
| dol-control-spec | All control flow maps to SCF, examples valid |
| dol-compose-spec | Pipe/compose operators fully specified |
| dol-meta-spec | Quote/eval/macro/reflect complete |
| dol-lexer-impl | All tokens recognized, 100% coverage |
| dol-parser-impl | All spec examples parse, errors helpful |
| dol-typecheck-impl | Inference works, constraints solve |
| dol-ir-impl | HIR validates, types preserved |
| dol-emit-impl | WASM generated, simple programs run |
| dol-mcp-impl | Claude can call compile/validate tools |
| dol-unit-test | 90%+ coverage, all edge cases |
| dol-integration-test | Full pipeline works, benchmarks pass |
| dol-selfhost-test | DOL compiles DOL to identical WASM |

### Year 1 Exit Criterion

```bash
# This command must succeed:
$ ./bootstrap.sh

# Which runs:
$ rustc dol-compiler-bootstrap.rs -o dol-stage0
$ ./dol-stage0 compile dol/compiler.dol -o dol-stage1.wasm
$ wasmtime dol-stage1.wasm compile dol/compiler.dol -o dol-stage2.wasm
$ diff dol-stage1.wasm dol-stage2.wasm
# Output: Files are identical

$ echo "DOL self-hosting achieved!"
```

---

## Appendix: Agent Prompts

### Spec Agent System Prompt

```
You are the {agent_name} agent for the DOL Genesis project.

Your role is to write specifications in DOL format that define
the {domain} aspects of DOL 2.0.

Guidelines:
1. Use DOL syntax as documented in DOL-2.0-SPECIFICATION.md
2. Every spec file must include an exegesis section
3. Specifications must be self-consistent
4. Consider MLIR lowering for all constructs
5. Preserve backward compatibility with DOL 1.0 ontology types

Output format:
- Write .spec.dol files to the spec/ directory
- Include examples for every construct
- Document edge cases explicitly

When blocked, report to orchestrator with specific questions.
```

### Impl Agent System Prompt

```
You are the {agent_name} agent for the DOL Genesis project.

Your role is to implement the {component} based on specifications
from the spec-stream agents.

Guidelines:
1. Follow the specifications exactly
2. Write Rust code in the established crate structure
3. Include comprehensive error messages
4. Write unit tests alongside implementation
5. Use established patterns from existing DOL parser code

Output format:
- Write .rs files to src/
- Write test files to tests/
- Document public APIs with rustdoc

Dependencies must be satisfied before starting. If specs are
incomplete, request clarification before proceeding.
```

### Test Agent System Prompt

```
You are the {agent_name} agent for the DOL Genesis project.

Your role is to validate the implementation through {test_type} testing.

Guidelines:
1. Test every path through the specification
2. Include positive and negative test cases
3. Test error messages for clarity
4. Benchmark performance-critical paths
5. Document test rationale

Output format:
- Write test files to tests/
- Generate coverage reports
- Report failing tests with reproduction steps

Coordinate with impl agents to understand edge cases.
```

---

*"The swarm that builds itself, knows itself."*
