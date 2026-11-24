//! Spirit Hosting Plan H1a: the sandbox enforces `ResourceLimits::memory_bytes`.
//!
//! Decision: growing linear memory past the limit makes `memory.grow` return -1
//! (standard WebAssembly failure semantics) instead of trapping, so a Spirit can
//! observe and handle the refusal. A module whose *initial* memory already
//! exceeds the limit is rejected before it runs.

use std::time::Duration;
use vudo_vm::sandbox::{ResourceLimits, Sandbox, SandboxError};
use wasmtime::Val;

const PAGE: u64 = 65_536;

fn limits(memory_bytes: u64) -> ResourceLimits {
    ResourceLimits {
        memory_bytes,
        cpu_quota: 0.5,
        max_fuel: 10_000_000,
        max_duration: Duration::from_secs(5),
        ..ResourceLimits::default()
    }
}

/// Module with `initial` pages; `grow(n)` returns memory.grow's result.
fn module(initial: u32) -> Vec<u8> {
    wat::parse_str(format!(
        r#"(module
  (memory (export "memory") {initial})
  (func (export "grow") (param i32) (result i32) (memory.grow (local.get 0)))
  (func (export "size") (result i32) (memory.size)))"#
    ))
    .expect("valid WAT")
}

fn i32_result(sb: &mut Sandbox, func: &str, args: &[Val]) -> i32 {
    let r = sb.invoke(func, args).expect("invoke");
    assert!(r.success, "{} failed: {:?}", func, r.error);
    r.return_value.expect("return value")[0].unwrap_i32()
}

fn sandbox(initial: u32, limit_pages: u64) -> Result<Sandbox, SandboxError> {
    let mut sb =
        Sandbox::new_with_defaults(&module(initial), [7u8; 32], limits(limit_pages * PAGE))?;
    sb.initialize()?;
    Ok(sb)
}

#[test]
fn memory_grow_within_limit_succeeds() {
    let mut sb = sandbox(1, 4).expect("sandbox");
    assert_eq!(
        i32_result(&mut sb, "grow", &[Val::I32(2)]),
        1,
        "old size returned"
    );
    assert_eq!(i32_result(&mut sb, "size", &[]), 3);
}

#[test]
fn memory_grow_beyond_limit_never_succeeds() {
    let mut sb = sandbox(1, 4).expect("sandbox");
    assert_eq!(
        i32_result(&mut sb, "grow", &[Val::I32(4)]),
        -1,
        "growing 1 -> 5 pages past a 4-page limit must fail"
    );
    assert_eq!(
        i32_result(&mut sb, "size", &[]),
        1,
        "memory unchanged after refused grow"
    );
    // Exactly to the limit is still allowed.
    assert_eq!(i32_result(&mut sb, "grow", &[Val::I32(3)]), 1);
    assert_eq!(i32_result(&mut sb, "grow", &[Val::I32(1)]), -1);
}

#[test]
fn initial_memory_above_limit_is_rejected() {
    let err = match sandbox(8, 4) {
        Err(e) => e,
        Ok(mut sb) => sb
            .invoke("size", &[])
            .expect_err("an 8-page module must never run under a 4-page limit"),
    };
    let msg = err.to_string();
    assert!(
        msg.contains("memory") && msg.contains("limit"),
        "error must say the memory limit was exceeded, got: {msg}"
    );
}

/// Load-time check (Spirit Hosting H1d): an oversized module is refused by
/// `initialize()`, before anything is instantiated or invoked, so a host can
/// tell a load failure apart from a run failure. (wasmtime reports the
/// initial size in Wasm pages; comparing it to bytes made this check dead.)
#[test]
fn initial_memory_above_limit_is_rejected_at_initialize() {
    let mut sb = Sandbox::new_with_defaults(&module(8), [7u8; 32], limits(4 * PAGE)).unwrap();
    let err = sb
        .initialize()
        .expect_err("an 8-page module must not initialize under a 4-page limit");
    assert!(matches!(err, SandboxError::InvalidModule(_)), "{err:?}");
    assert!(
        err.to_string().contains("exceeds sandbox memory limit"),
        "{err}"
    );
    assert_eq!(sb.get_state(), vudo_vm::sandbox::SandboxState::Failed);
}

#[test]
fn initial_memory_at_limit_initializes() {
    let mut sb = sandbox(4, 4).expect("a 4-page module fits a 4-page limit");
    assert_eq!(i32_result(&mut sb, "size", &[]), 4);
}
