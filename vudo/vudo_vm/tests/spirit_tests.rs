//! Spirit Tests - Integration tests for WASM Spirit modules
//!
//! These tests load WAT files from examples/spirits and verify
//! they execute correctly in the VUDO VM Sandbox.

use std::fs;
use std::path::PathBuf;

use vudo_vm::sandbox::{ResourceLimits, Sandbox, SandboxState};
use wasmtime::Val;

/// Helper function to get the path to the spirits examples directory
fn spirits_dir() -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest_dir).join("examples").join("spirits")
}

/// Helper function to load a WAT file and convert to WASM bytes
fn load_wat(name: &str) -> Vec<u8> {
    let path = spirits_dir().join(name);
    let wat_source = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", path.display(), e));
    wat::parse_str(&wat_source)
        .unwrap_or_else(|e| panic!("Failed to parse WAT {}: {}", path.display(), e))
}

/// Helper function to create a sandbox with default settings
fn create_sandbox(wasm: &[u8]) -> Sandbox {
    let owner = [0u8; 32];
    let limits = ResourceLimits::default();
    let mut sandbox = Sandbox::new(wasm, owner, limits).expect("Failed to create sandbox");
    sandbox.initialize().expect("Failed to initialize sandbox");
    sandbox
}

// ═══════════════════════════════════════════════════════════════════════════
// HELLO WORLD SPIRIT TESTS
// ═══════════════════════════════════════════════════════════════════════════

mod hello_world {
    use super::*;

    #[test]
    fn test_greet_returns_42() {
        let wasm = load_wat("hello_world.wat");
        let mut sandbox = create_sandbox(&wasm);

        let result = sandbox
            .invoke("greet", &[])
            .expect("Failed to invoke greet");

        assert!(result.success, "greet should succeed");
        let return_value = result.return_value.expect("Should have return value");
        assert_eq!(return_value.len(), 1, "Should return exactly one value");
        assert_eq!(return_value[0].unwrap_i32(), 42, "greet should return 42");
    }

    #[test]
    fn test_sandbox_state_after_greet() {
        let wasm = load_wat("hello_world.wat");
        let mut sandbox = create_sandbox(&wasm);

        assert_eq!(sandbox.get_state(), SandboxState::Ready);

        sandbox
            .invoke("greet", &[])
            .expect("Failed to invoke greet");

        // After successful execution, sandbox should return to Ready state
        assert_eq!(sandbox.get_state(), SandboxState::Ready);
    }

    #[test]
    fn test_greet_multiple_invocations() {
        let wasm = load_wat("hello_world.wat");
        let mut sandbox = create_sandbox(&wasm);

        // Call greet multiple times to ensure it's idempotent
        for i in 0..5 {
            let result = sandbox
                .invoke("greet", &[])
                .unwrap_or_else(|e| panic!("Invocation {} failed: {}", i, e));
            assert!(result.success);
            assert_eq!(result.return_value.unwrap()[0].unwrap_i32(), 42);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// COUNTER SPIRIT TESTS
// ═══════════════════════════════════════════════════════════════════════════

mod counter {
    use super::*;

    #[test]
    fn test_init_resets_counter() {
        let wasm = load_wat("counter.wat");
        let mut sandbox = create_sandbox(&wasm);

        // Increment a few times
        sandbox.invoke("increment", &[]).unwrap();
        sandbox.invoke("increment", &[]).unwrap();

        // Init should reset to 0
        sandbox.invoke("init", &[]).unwrap();

        let result = sandbox.invoke("get", &[]).unwrap();
        assert_eq!(result.return_value.unwrap()[0].unwrap_i32(), 0);
    }

    #[test]
    fn test_increment_adds_one() {
        let wasm = load_wat("counter.wat");
        let mut sandbox = create_sandbox(&wasm);

        // Initialize first
        sandbox.invoke("init", &[]).unwrap();

        // First increment: 0 -> 1
        let result = sandbox.invoke("increment", &[]).unwrap();
        assert_eq!(result.return_value.unwrap()[0].unwrap_i32(), 1);

        // Second increment: 1 -> 2
        let result = sandbox.invoke("increment", &[]).unwrap();
        assert_eq!(result.return_value.unwrap()[0].unwrap_i32(), 2);

        // Third increment: 2 -> 3
        let result = sandbox.invoke("increment", &[]).unwrap();
        assert_eq!(result.return_value.unwrap()[0].unwrap_i32(), 3);
    }

    #[test]
    fn test_decrement_subtracts_one() {
        let wasm = load_wat("counter.wat");
        let mut sandbox = create_sandbox(&wasm);

        // Initialize to 0
        sandbox.invoke("init", &[]).unwrap();

        // Increment to 5
        for _ in 0..5 {
            sandbox.invoke("increment", &[]).unwrap();
        }

        // Decrement: 5 -> 4
        let result = sandbox.invoke("decrement", &[]).unwrap();
        assert_eq!(result.return_value.unwrap()[0].unwrap_i32(), 4);

        // Decrement: 4 -> 3
        let result = sandbox.invoke("decrement", &[]).unwrap();
        assert_eq!(result.return_value.unwrap()[0].unwrap_i32(), 3);
    }

    #[test]
    fn test_get_returns_current_value() {
        let wasm = load_wat("counter.wat");
        let mut sandbox = create_sandbox(&wasm);

        sandbox.invoke("init", &[]).unwrap();

        // Get should return 0 after init
        let result = sandbox.invoke("get", &[]).unwrap();
        assert_eq!(result.return_value.unwrap()[0].unwrap_i32(), 0);

        // Increment and check get
        sandbox.invoke("increment", &[]).unwrap();
        let result = sandbox.invoke("get", &[]).unwrap();
        assert_eq!(result.return_value.unwrap()[0].unwrap_i32(), 1);

        sandbox.invoke("increment", &[]).unwrap();
        let result = sandbox.invoke("get", &[]).unwrap();
        assert_eq!(result.return_value.unwrap()[0].unwrap_i32(), 2);
    }

    #[test]
    fn test_decrement_can_go_negative() {
        let wasm = load_wat("counter.wat");
        let mut sandbox = create_sandbox(&wasm);

        sandbox.invoke("init", &[]).unwrap();

        // Decrement from 0 should go to -1
        let result = sandbox.invoke("decrement", &[]).unwrap();
        assert_eq!(result.return_value.unwrap()[0].unwrap_i32(), -1);

        // Decrement again: -1 -> -2
        let result = sandbox.invoke("decrement", &[]).unwrap();
        assert_eq!(result.return_value.unwrap()[0].unwrap_i32(), -2);
    }

    #[test]
    fn test_counter_state_persists() {
        let wasm = load_wat("counter.wat");
        let mut sandbox = create_sandbox(&wasm);

        // Build up counter
        sandbox.invoke("init", &[]).unwrap();
        for _ in 0..10 {
            sandbox.invoke("increment", &[]).unwrap();
        }

        // Verify state persisted across calls
        let result = sandbox.invoke("get", &[]).unwrap();
        assert_eq!(result.return_value.unwrap()[0].unwrap_i32(), 10);

        // Decrement a few times
        for _ in 0..3 {
            sandbox.invoke("decrement", &[]).unwrap();
        }

        let result = sandbox.invoke("get", &[]).unwrap();
        assert_eq!(result.return_value.unwrap()[0].unwrap_i32(), 7);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// ECHO SPIRIT TESTS
// ═══════════════════════════════════════════════════════════════════════════

mod echo {
    use super::*;

    #[test]
    fn test_echo_returns_input() {
        let wasm = load_wat("echo.wat");
        let mut sandbox = create_sandbox(&wasm);

        // Test various values
        let test_cases = [0, 1, -1, 42, 100, -100, i32::MAX, i32::MIN];

        for &value in &test_cases {
            let result = sandbox
                .invoke("echo", &[Val::I32(value)])
                .expect("echo should succeed");
            assert!(result.success);
            assert_eq!(
                result.return_value.unwrap()[0].unwrap_i32(),
                value,
                "echo({}) should return {}",
                value,
                value
            );
        }
    }

    #[test]
    fn test_double_multiplies_by_two() {
        let wasm = load_wat("echo.wat");
        let mut sandbox = create_sandbox(&wasm);

        let test_cases = [(0, 0), (1, 2), (5, 10), (21, 42), (-5, -10), (100, 200)];

        for (input, expected) in test_cases {
            let result = sandbox
                .invoke("double", &[Val::I32(input)])
                .expect("double should succeed");
            assert!(result.success);
            assert_eq!(
                result.return_value.unwrap()[0].unwrap_i32(),
                expected,
                "double({}) should return {}",
                input,
                expected
            );
        }
    }

    #[test]
    fn test_triple_multiplies_by_three() {
        let wasm = load_wat("echo.wat");
        let mut sandbox = create_sandbox(&wasm);

        let test_cases = [(0, 0), (1, 3), (5, 15), (14, 42), (-5, -15), (100, 300)];

        for (input, expected) in test_cases {
            let result = sandbox
                .invoke("triple", &[Val::I32(input)])
                .expect("triple should succeed");
            assert!(result.success);
            assert_eq!(
                result.return_value.unwrap()[0].unwrap_i32(),
                expected,
                "triple({}) should return {}",
                input,
                expected
            );
        }
    }

    #[test]
    fn test_add_sums_two_numbers() {
        let wasm = load_wat("echo.wat");
        let mut sandbox = create_sandbox(&wasm);

        let test_cases = [
            (0, 0, 0),
            (1, 1, 2),
            (10, 32, 42),
            (-5, 5, 0),
            (-10, -20, -30),
            (i32::MAX - 1, 1, i32::MAX),
        ];

        for (a, b, expected) in test_cases {
            let result = sandbox
                .invoke("add", &[Val::I32(a), Val::I32(b)])
                .expect("add should succeed");
            assert!(result.success);
            assert_eq!(
                result.return_value.unwrap()[0].unwrap_i32(),
                expected,
                "add({}, {}) should return {}",
                a,
                b,
                expected
            );
        }
    }

    #[test]
    fn test_echo_preserves_bit_patterns() {
        let wasm = load_wat("echo.wat");
        let mut sandbox = create_sandbox(&wasm);

        // Test some specific bit patterns
        let patterns: [i32; 4] = [
            0x12345678_u32 as i32,
            0xDEADBEEF_u32 as i32,
            0x00000000_u32 as i32,
            0xFFFFFFFF_u32 as i32,
        ];

        for &pattern in &patterns {
            let result = sandbox
                .invoke("echo", &[Val::I32(pattern)])
                .expect("echo should succeed");
            assert_eq!(result.return_value.unwrap()[0].unwrap_i32(), pattern);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// SANDBOX BEHAVIOR TESTS
// ═══════════════════════════════════════════════════════════════════════════

mod sandbox_behavior {
    use super::*;

    #[test]
    fn test_function_not_found_error() {
        let wasm = load_wat("hello_world.wat");
        let mut sandbox = create_sandbox(&wasm);

        let result = sandbox.invoke("nonexistent_function", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_fuel_consumption() {
        let wasm = load_wat("counter.wat");
        let mut sandbox = create_sandbox(&wasm);

        sandbox.invoke("init", &[]).unwrap();

        // Track fuel consumption over multiple operations
        let mut total_fuel = 0u64;

        for _ in 0..10 {
            let result = sandbox.invoke("increment", &[]).unwrap();
            total_fuel += result.fuel_consumed;
        }

        // Should have consumed some fuel
        assert!(total_fuel > 0, "Operations should consume fuel");
    }

    #[test]
    fn test_metrics_tracking() {
        let wasm = load_wat("echo.wat");
        let mut sandbox = create_sandbox(&wasm);

        // Perform several operations
        for i in 0..5 {
            sandbox.invoke("echo", &[Val::I32(i)]).unwrap();
        }

        let metrics = sandbox.metrics();
        assert_eq!(metrics.execution_count, 5);
        assert!(metrics.total_fuel_consumed > 0);
        assert_eq!(metrics.trap_count, 0);
    }

    #[test]
    fn test_all_spirits_load_successfully() {
        let spirits = ["hello_world.wat", "counter.wat", "echo.wat"];

        for spirit in &spirits {
            let wasm = load_wat(spirit);
            let sandbox = create_sandbox(&wasm);
            assert_eq!(
                sandbox.get_state(),
                SandboxState::Ready,
                "{} should initialize to Ready state",
                spirit
            );
        }
    }
}
