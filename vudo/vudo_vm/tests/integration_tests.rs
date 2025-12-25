//! Integration Tests for VUDO VM
//!
//! Comprehensive test suite covering:
//! - Spirit compilation and execution
//! - Sandbox isolation
//! - Capability enforcement
//! - Fuel metering
//! - Concurrent sandbox execution

use std::sync::{Arc, Barrier};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};
use wasmtime::Val;

// Use the capability types from capability.rs module for host functions
use vudo_vm::{CapabilityGrant, CapabilityScope, CapabilitySet, CapabilityType, StorageBackend};

// Use sandbox-specific types (sandbox has its own CapabilityType/Grant definitions)
use vudo_vm::fuel::FuelManager;
use vudo_vm::host::{
    host_log, host_random_bytes, host_storage_read, host_storage_write, host_time_now,
    InMemoryStorage, LogLevel,
};
use vudo_vm::sandbox::{
    CapabilityGrant as SandboxCapabilityGrant, CapabilityType as SandboxCapabilityType,
    ResourceLimits, Sandbox, SandboxState,
};

// ═══════════════════════════════════════════════════════════════════════════
// HELPER FUNCTIONS
// ═══════════════════════════════════════════════════════════════════════════

/// Get current Unix timestamp in seconds
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
}

/// Create a capability grant for host function testing
fn create_grant(
    id: u64,
    capability: CapabilityType,
    scope: CapabilityScope,
    expires_at: Option<u64>,
) -> CapabilityGrant {
    CapabilityGrant::new(
        id,
        capability,
        scope,
        [0u8; 32], // granter
        [1u8; 32], // grantee
        current_timestamp(),
        expires_at,
        [0u8; 64], // signature (not verified in tests)
    )
}

/// Create a capability grant for sandbox testing
fn create_sandbox_grant(
    id: u64,
    capability: SandboxCapabilityType,
    expires_at: Option<u64>,
) -> SandboxCapabilityGrant {
    // Sandbox module's CapabilityGrant is a plain struct without constructor
    use vudo_vm::sandbox::CapabilityScope as SandboxCapabilityScope;
    SandboxCapabilityGrant {
        id,
        capability,
        scope: SandboxCapabilityScope::Global,
        granter: [0u8; 32],
        grantee: [1u8; 32],
        granted_at: current_timestamp(),
        expires_at,
        revoked: false,
        signature: [0u8; 64],
    }
}

/// Create a minimal capability set with basic permissions
fn create_minimal_capset() -> CapabilitySet {
    let mut cap_set = CapabilitySet::new();
    cap_set.add_grant(create_grant(
        1,
        CapabilityType::SensorTime,
        CapabilityScope::Global,
        None,
    ));
    cap_set.add_grant(create_grant(
        2,
        CapabilityType::SensorRandom,
        CapabilityScope::Global,
        None,
    ));
    cap_set.add_grant(create_grant(
        3,
        CapabilityType::ActuatorLog,
        CapabilityScope::Global,
        None,
    ));
    cap_set
}

/// Create a capability set with storage permissions
fn create_storage_capset() -> CapabilitySet {
    let mut cap_set = create_minimal_capset();
    cap_set.add_grant(create_grant(
        4,
        CapabilityType::StorageRead,
        CapabilityScope::Sandboxed,
        None,
    ));
    cap_set.add_grant(create_grant(
        5,
        CapabilityType::StorageWrite,
        CapabilityScope::Sandboxed,
        None,
    ));
    cap_set.add_grant(create_grant(
        6,
        CapabilityType::StorageDelete,
        CapabilityScope::Sandboxed,
        None,
    ));
    cap_set
}

/// Create an unrestricted capability set (for system spirits)
fn create_unrestricted_capset() -> CapabilitySet {
    let mut cap_set = CapabilitySet::new();
    cap_set.add_grant(create_grant(
        1,
        CapabilityType::Unrestricted,
        CapabilityScope::Global,
        None,
    ));
    cap_set
}

// ═══════════════════════════════════════════════════════════════════════════
// TEST 1: SPIRIT COMPILE AND RUN
// ═══════════════════════════════════════════════════════════════════════════

/// Tests basic WASM module compilation and execution in a sandbox
#[test]
fn test_spirit_compile_and_run() {
    // Create a simple WASM module that returns 42
    let wasm = wat::parse_str(
        r#"
        (module
            (func (export "get_answer") (result i32)
                i32.const 42
            )
        )
    "#,
    )
    .expect("Failed to parse WAT");

    let owner = [0u8; 32];
    let limits = ResourceLimits::default();

    // Create and initialize sandbox
    let mut sandbox = Sandbox::new(&wasm, owner, limits).expect("Failed to create sandbox");
    assert_eq!(sandbox.get_state(), SandboxState::Initializing);

    sandbox.initialize().expect("Failed to initialize sandbox");
    assert_eq!(sandbox.get_state(), SandboxState::Ready);

    // Invoke the function
    let result = sandbox.invoke("get_answer", &[]).expect("Failed to invoke");
    assert!(result.success);
    assert_eq!(result.return_value.as_ref().unwrap()[0].unwrap_i32(), 42);
    assert!(result.fuel_consumed > 0);
}

/// Tests WASM module with parameters and arithmetic
#[test]
fn test_spirit_with_parameters() {
    let wasm = wat::parse_str(
        r#"
        (module
            (func (export "add") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.add
            )
            (func (export "multiply") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.mul
            )
        )
    "#,
    )
    .expect("Failed to parse WAT");

    let owner = [0u8; 32];
    let limits = ResourceLimits::default();

    let mut sandbox = Sandbox::new(&wasm, owner, limits).expect("Failed to create sandbox");
    sandbox.initialize().expect("Failed to initialize");

    // Test addition
    let result = sandbox
        .invoke("add", &[Val::I32(10), Val::I32(32)])
        .expect("Failed to invoke add");
    assert!(result.success);
    assert_eq!(result.return_value.as_ref().unwrap()[0].unwrap_i32(), 42);

    // Test multiplication
    let result = sandbox
        .invoke("multiply", &[Val::I32(6), Val::I32(7)])
        .expect("Failed to invoke multiply");
    assert!(result.success);
    assert_eq!(result.return_value.as_ref().unwrap()[0].unwrap_i32(), 42);
}

/// Tests WASM module with memory operations
#[test]
fn test_spirit_with_memory() {
    let wasm = wat::parse_str(
        r#"
        (module
            (memory (export "memory") 1)

            ;; Store a value at address 0 and return it
            (func (export "store_and_load") (param i32) (result i32)
                (i32.store (i32.const 0) (local.get 0))
                (i32.load (i32.const 0))
            )

            ;; Sum values stored in memory from 0 to n*4
            (func (export "sum_memory") (param i32) (result i32)
                (local $sum i32)
                (local $i i32)
                (local.set $i (i32.const 0))
                (local.set $sum (i32.const 0))
                (block $done
                    (loop $loop
                        (br_if $done (i32.ge_u (local.get $i) (local.get 0)))
                        (local.set $sum
                            (i32.add
                                (local.get $sum)
                                (i32.load (i32.mul (local.get $i) (i32.const 4)))
                            )
                        )
                        (local.set $i (i32.add (local.get $i) (i32.const 1)))
                        (br $loop)
                    )
                )
                (local.get $sum)
            )
        )
    "#,
    )
    .expect("Failed to parse WAT");

    let owner = [0u8; 32];
    let limits = ResourceLimits::default();

    let mut sandbox = Sandbox::new(&wasm, owner, limits).expect("Failed to create sandbox");
    sandbox.initialize().expect("Failed to initialize");

    // Test store and load
    let result = sandbox
        .invoke("store_and_load", &[Val::I32(12345)])
        .expect("Failed to invoke");
    assert!(result.success);
    assert_eq!(result.return_value.as_ref().unwrap()[0].unwrap_i32(), 12345);
}

/// Tests WASM module with global variables
#[test]
fn test_spirit_with_globals() {
    let wasm = wat::parse_str(
        r#"
        (module
            (global $counter (mut i32) (i32.const 0))

            (func (export "increment") (result i32)
                (global.set $counter (i32.add (global.get $counter) (i32.const 1)))
                (global.get $counter)
            )

            (func (export "get_counter") (result i32)
                (global.get $counter)
            )

            (func (export "reset") (result i32)
                (global.set $counter (i32.const 0))
                (global.get $counter)
            )
        )
    "#,
    )
    .expect("Failed to parse WAT");

    let owner = [0u8; 32];
    let limits = ResourceLimits::default();

    let mut sandbox = Sandbox::new(&wasm, owner, limits).expect("Failed to create sandbox");
    sandbox.initialize().expect("Failed to initialize");

    // Increment multiple times
    for i in 1..=5 {
        let result = sandbox.invoke("increment", &[]).expect("Failed to invoke");
        assert!(result.success);
        assert_eq!(result.return_value.as_ref().unwrap()[0].unwrap_i32(), i);
    }

    // Get counter
    let result = sandbox
        .invoke("get_counter", &[])
        .expect("Failed to invoke");
    assert_eq!(result.return_value.as_ref().unwrap()[0].unwrap_i32(), 5);

    // Reset
    let result = sandbox.invoke("reset", &[]).expect("Failed to invoke");
    assert_eq!(result.return_value.as_ref().unwrap()[0].unwrap_i32(), 0);
}

// ═══════════════════════════════════════════════════════════════════════════
// TEST 2: SANDBOX ISOLATION
// ═══════════════════════════════════════════════════════════════════════════

/// Verifies that two sandboxes cannot access each other's memory
#[test]
fn test_sandbox_isolation() {
    let wasm = wat::parse_str(
        r#"
        (module
            (memory (export "memory") 1)
            (global $value (mut i32) (i32.const 0))

            (func (export "set_value") (param i32)
                (global.set $value (local.get 0))
                (i32.store (i32.const 0) (local.get 0))
            )

            (func (export "get_value") (result i32)
                (global.get $value)
            )

            (func (export "get_memory_value") (result i32)
                (i32.load (i32.const 0))
            )
        )
    "#,
    )
    .expect("Failed to parse WAT");

    let owner1 = [1u8; 32];
    let owner2 = [2u8; 32];
    let limits = ResourceLimits::default();

    // Create two separate sandboxes
    let mut sandbox1 =
        Sandbox::new(&wasm, owner1, limits.clone()).expect("Failed to create sandbox1");
    let mut sandbox2 = Sandbox::new(&wasm, owner2, limits).expect("Failed to create sandbox2");

    sandbox1
        .initialize()
        .expect("Failed to initialize sandbox1");
    sandbox2
        .initialize()
        .expect("Failed to initialize sandbox2");

    // Set different values in each sandbox
    sandbox1
        .invoke("set_value", &[Val::I32(100)])
        .expect("Failed to set value in sandbox1");
    sandbox2
        .invoke("set_value", &[Val::I32(200)])
        .expect("Failed to set value in sandbox2");

    // Verify global isolation
    let result1 = sandbox1
        .invoke("get_value", &[])
        .expect("Failed to get value from sandbox1");
    let result2 = sandbox2
        .invoke("get_value", &[])
        .expect("Failed to get value from sandbox2");

    assert_eq!(result1.return_value.as_ref().unwrap()[0].unwrap_i32(), 100);
    assert_eq!(result2.return_value.as_ref().unwrap()[0].unwrap_i32(), 200);

    // Verify memory isolation
    let mem_result1 = sandbox1
        .invoke("get_memory_value", &[])
        .expect("Failed to get memory from sandbox1");
    let mem_result2 = sandbox2
        .invoke("get_memory_value", &[])
        .expect("Failed to get memory from sandbox2");

    assert_eq!(
        mem_result1.return_value.as_ref().unwrap()[0].unwrap_i32(),
        100
    );
    assert_eq!(
        mem_result2.return_value.as_ref().unwrap()[0].unwrap_i32(),
        200
    );

    // Verify sandboxes have different IDs
    assert_ne!(sandbox1.id, sandbox2.id);
}

/// Tests that sandbox owners are properly tracked
#[test]
fn test_sandbox_owner_isolation() {
    let wasm = wat::parse_str(
        r#"
        (module
            (func (export "noop") (result i32)
                i32.const 1
            )
        )
    "#,
    )
    .expect("Failed to parse WAT");

    let owner_alice = [0xAA; 32];
    let owner_bob = [0xBB; 32];
    let limits = ResourceLimits::default();

    let sandbox_alice =
        Sandbox::new(&wasm, owner_alice, limits.clone()).expect("Failed to create sandbox");
    let sandbox_bob = Sandbox::new(&wasm, owner_bob, limits).expect("Failed to create sandbox");

    // Verify owners are different
    assert_ne!(sandbox_alice.owner, sandbox_bob.owner);
    assert_eq!(sandbox_alice.owner, owner_alice);
    assert_eq!(sandbox_bob.owner, owner_bob);
}

/// Tests sandbox state isolation after termination
#[test]
fn test_sandbox_termination_isolation() {
    let wasm = wat::parse_str(
        r#"
        (module
            (global $counter (mut i32) (i32.const 0))
            (func (export "increment") (result i32)
                (global.set $counter (i32.add (global.get $counter) (i32.const 1)))
                (global.get $counter)
            )
        )
    "#,
    )
    .expect("Failed to parse WAT");

    let owner = [0u8; 32];
    let limits = ResourceLimits::default();

    // Create and use first sandbox
    let mut sandbox1 =
        Sandbox::new(&wasm, owner, limits.clone()).expect("Failed to create sandbox");
    sandbox1.initialize().expect("Failed to initialize");
    sandbox1.invoke("increment", &[]).expect("Failed to invoke");
    sandbox1.invoke("increment", &[]).expect("Failed to invoke");
    sandbox1.terminate();
    assert_eq!(sandbox1.get_state(), SandboxState::Terminated);

    // Create a new sandbox - should start fresh
    let mut sandbox2 = Sandbox::new(&wasm, owner, limits).expect("Failed to create sandbox");
    sandbox2.initialize().expect("Failed to initialize");

    let result = sandbox2.invoke("increment", &[]).expect("Failed to invoke");
    // New sandbox should start at 1, not continue from sandbox1's state
    assert_eq!(result.return_value.as_ref().unwrap()[0].unwrap_i32(), 1);
}

// ═══════════════════════════════════════════════════════════════════════════
// TEST 3: CAPABILITY ENFORCEMENT
// ═══════════════════════════════════════════════════════════════════════════

/// Tests that capabilities are properly checked for host functions
#[test]
fn test_capability_enforcement_storage() {
    let storage = InMemoryStorage::new();

    // Test without capabilities
    let empty_caps = CapabilitySet::new();

    // Storage read should fail without capability
    let result = host_storage_read(&empty_caps, &storage, b"test_key");
    assert!(!result.success);
    assert!(result.error.as_ref().unwrap().contains("Capability denied"));

    // Storage write should fail without capability
    let result = host_storage_write(&empty_caps, &storage, b"test_key", b"test_value");
    assert!(!result.success);
    assert!(result.error.as_ref().unwrap().contains("Capability denied"));

    // Test with storage capabilities
    let storage_caps = create_storage_capset();

    // Storage write should succeed with capability
    let result = host_storage_write(&storage_caps, &storage, b"test_key", b"test_value");
    assert!(result.success);

    // Storage read should succeed with capability
    let result = host_storage_read(&storage_caps, &storage, b"test_key");
    assert!(result.success);
    assert_eq!(result.return_value, Some(b"test_value".to_vec()));
}

/// Tests capability enforcement for time sensor
#[test]
fn test_capability_enforcement_time() {
    // Without capability
    let empty_caps = CapabilitySet::new();
    let result = host_time_now(&empty_caps);
    assert!(!result.success);
    assert!(result.error.as_ref().unwrap().contains("Capability denied"));

    // With capability
    let minimal_caps = create_minimal_capset();
    let result = host_time_now(&minimal_caps);
    assert!(result.success);
    assert!(result.return_value.is_some());
}

/// Tests capability enforcement for random sensor
#[test]
fn test_capability_enforcement_random() {
    // Without capability
    let empty_caps = CapabilitySet::new();
    let result = host_random_bytes(&empty_caps, 32);
    assert!(!result.success);
    assert!(result.error.as_ref().unwrap().contains("Capability denied"));

    // With capability
    let minimal_caps = create_minimal_capset();
    let result = host_random_bytes(&minimal_caps, 32);
    assert!(result.success);
    let bytes = result.return_value.unwrap();
    assert_eq!(bytes.len(), 32);
}

/// Tests capability enforcement for logging
#[test]
fn test_capability_enforcement_log() {
    // Without capability
    let empty_caps = CapabilitySet::new();
    let result = host_log(&empty_caps, LogLevel::Info, "test message");
    assert!(!result.success);
    assert!(result.error.as_ref().unwrap().contains("Capability denied"));

    // With capability
    let minimal_caps = create_minimal_capset();
    let result = host_log(&minimal_caps, LogLevel::Info, "test message");
    assert!(result.success);
}

/// Tests that unrestricted capability grants access to everything
#[test]
fn test_capability_unrestricted_access() {
    let unrestricted_caps = create_unrestricted_capset();
    let storage = InMemoryStorage::new();

    // All operations should succeed with unrestricted capability
    assert!(host_time_now(&unrestricted_caps).success);
    assert!(host_random_bytes(&unrestricted_caps, 16).success);
    assert!(host_log(&unrestricted_caps, LogLevel::Debug, "test").success);
    assert!(host_storage_write(&unrestricted_caps, &storage, b"key", b"value").success);
    assert!(host_storage_read(&unrestricted_caps, &storage, b"key").success);
}

/// Tests capability expiration
#[test]
fn test_capability_expiration() {
    let storage = InMemoryStorage::new();

    // Create an already-expired grant
    let expired_grant = CapabilityGrant::new(
        1,
        CapabilityType::StorageRead,
        CapabilityScope::Sandboxed,
        [0u8; 32],
        [1u8; 32],
        current_timestamp() - 3600,       // Granted 1 hour ago
        Some(current_timestamp() - 1800), // Expired 30 minutes ago
        [0u8; 64],
    );

    let mut caps = CapabilitySet::new();
    caps.add_grant(expired_grant);

    // Operation should fail with expired capability
    let result = host_storage_read(&caps, &storage, b"key");
    assert!(!result.success);
    assert!(result.error.as_ref().unwrap().contains("Capability denied"));
}

/// Tests capability revocation
#[test]
fn test_capability_revocation() {
    let mut grant = create_grant(
        1,
        CapabilityType::StorageRead,
        CapabilityScope::Sandboxed,
        None,
    );

    // Initially valid
    assert!(grant.is_valid());

    // Revoke the grant
    grant.revoke();

    // Now invalid
    assert!(!grant.is_valid());

    // Cannot use revoked capability
    let mut caps = CapabilitySet::new();
    caps.add_grant(grant);

    let storage = InMemoryStorage::new();
    let result = host_storage_read(&caps, &storage, b"key");
    assert!(!result.success);
}

/// Tests capability scope enforcement
#[test]
fn test_capability_scope_enforcement() {
    // Create a capability with Sandboxed scope
    let sandboxed_grant = create_grant(
        1,
        CapabilityType::StorageRead,
        CapabilityScope::Sandboxed,
        None,
    );

    let mut caps = CapabilitySet::new();
    caps.add_grant(sandboxed_grant);

    // Sandboxed scope should be sufficient for Sandboxed operations
    assert!(caps.has_capability(CapabilityType::StorageRead, CapabilityScope::Sandboxed));

    // Sandboxed scope is NOT sufficient for Global scope operations
    // (Global scope requires Global grant)
    let global_grant = create_grant(
        2,
        CapabilityType::StorageWrite,
        CapabilityScope::Global,
        None,
    );
    caps.add_grant(global_grant);

    // Global grant covers Sandboxed scope
    assert!(caps.has_capability(CapabilityType::StorageWrite, CapabilityScope::Global));
    assert!(caps.has_capability(CapabilityType::StorageWrite, CapabilityScope::Sandboxed));
}

/// Tests sandbox capability management
#[test]
fn test_sandbox_capability_management() {
    let wasm = wat::parse_str(
        r#"
        (module
            (func (export "noop") (result i32)
                i32.const 1
            )
        )
    "#,
    )
    .expect("Failed to parse WAT");

    let owner = [0u8; 32];
    let limits = ResourceLimits::default();

    let mut sandbox = Sandbox::new(&wasm, owner, limits).expect("Failed to create sandbox");

    // Initially no capabilities
    assert!(!sandbox.has_capability(SandboxCapabilityType::StorageRead));
    assert!(!sandbox.has_capability(SandboxCapabilityType::NetworkConnect));

    // Grant a capability
    let grant = create_sandbox_grant(1, SandboxCapabilityType::StorageRead, None);
    sandbox.grant_capability(grant);

    // Now has the capability
    assert!(sandbox.has_capability(SandboxCapabilityType::StorageRead));
    assert!(!sandbox.has_capability(SandboxCapabilityType::NetworkConnect)); // Still doesn't have this one
}

// ═══════════════════════════════════════════════════════════════════════════
// TEST 4: FUEL METERING
// ═══════════════════════════════════════════════════════════════════════════

/// Tests basic fuel consumption during execution
#[test]
fn test_fuel_metering_basic() {
    let wasm = wat::parse_str(
        r#"
        (module
            (func (export "compute") (param i32) (result i32)
                (local $sum i32)
                (local $i i32)
                (local.set $i (i32.const 0))
                (local.set $sum (i32.const 0))
                (block $done
                    (loop $loop
                        (br_if $done (i32.ge_u (local.get $i) (local.get 0)))
                        (local.set $sum (i32.add (local.get $sum) (local.get $i)))
                        (local.set $i (i32.add (local.get $i) (i32.const 1)))
                        (br $loop)
                    )
                )
                (local.get $sum)
            )
        )
    "#,
    )
    .expect("Failed to parse WAT");

    let owner = [0u8; 32];
    let limits = ResourceLimits::default();

    let mut sandbox = Sandbox::new(&wasm, owner, limits).expect("Failed to create sandbox");
    sandbox.initialize().expect("Failed to initialize");

    // Execute with a small loop
    let result1 = sandbox
        .invoke("compute", &[Val::I32(10)])
        .expect("Failed to invoke");
    assert!(result1.success);
    let fuel1 = result1.fuel_consumed;

    // Execute with a larger loop - should consume more fuel
    let result2 = sandbox
        .invoke("compute", &[Val::I32(100)])
        .expect("Failed to invoke");
    assert!(result2.success);
    let fuel2 = result2.fuel_consumed;

    // Larger computation should consume more fuel
    assert!(fuel2 > fuel1);
}

/// Tests that fuel is tracked cumulatively in sandbox metrics
#[test]
fn test_fuel_metering_cumulative() {
    let wasm = wat::parse_str(
        r#"
        (module
            (func (export "work") (result i32)
                i32.const 1
                i32.const 2
                i32.add
            )
        )
    "#,
    )
    .expect("Failed to parse WAT");

    let owner = [0u8; 32];
    let limits = ResourceLimits::default();

    let mut sandbox = Sandbox::new(&wasm, owner, limits).expect("Failed to create sandbox");
    sandbox.initialize().expect("Failed to initialize");

    // Initial fuel consumed should be 0
    assert_eq!(sandbox.fuel_consumed, 0);

    // Execute multiple times
    let _result1 = sandbox.invoke("work", &[]).expect("Failed to invoke");
    let fuel_after_1 = sandbox.fuel_consumed;

    let _result2 = sandbox.invoke("work", &[]).expect("Failed to invoke");
    let fuel_after_2 = sandbox.fuel_consumed;

    let _result3 = sandbox.invoke("work", &[]).expect("Failed to invoke");
    let fuel_after_3 = sandbox.fuel_consumed;

    // Fuel should be accumulating
    assert!(fuel_after_1 > 0);
    assert!(fuel_after_2 > fuel_after_1);
    assert!(fuel_after_3 > fuel_after_2);

    // Verify metrics tracking
    let metrics = sandbox.metrics();
    assert_eq!(metrics.execution_count, 3);
    assert_eq!(metrics.total_fuel_consumed, fuel_after_3);
}

/// Tests fuel exhaustion causes sandbox to pause
#[test]
fn test_fuel_exhaustion() {
    let wasm = wat::parse_str(
        r#"
        (module
            ;; Infinite loop that will exhaust fuel
            (func (export "infinite_loop")
                (loop $forever
                    (br $forever)
                )
            )

            ;; Simpler function for after refuel
            (func (export "simple") (result i32)
                i32.const 42
            )
        )
    "#,
    )
    .expect("Failed to parse WAT");

    let owner = [0u8; 32];
    // Very limited fuel to trigger exhaustion quickly
    let limits = ResourceLimits {
        max_fuel: 1000, // Very limited fuel
        ..Default::default()
    };

    let mut sandbox = Sandbox::new(&wasm, owner, limits).expect("Failed to create sandbox");
    sandbox.initialize().expect("Failed to initialize");

    // This should exhaust fuel
    let result = sandbox
        .invoke("infinite_loop", &[])
        .expect("Failed to invoke");

    // Execution failed due to fuel exhaustion
    assert!(!result.success);
    assert!(
        result.error.as_ref().unwrap().contains("fuel")
            || result.error.as_ref().unwrap().contains("trap")
    );

    // Sandbox should be in Paused or Failed state
    let state = sandbox.get_state();
    assert!(state == SandboxState::Paused || state == SandboxState::Failed);
}

/// Tests refueling a paused sandbox
#[test]
fn test_fuel_refuel() {
    let wasm = wat::parse_str(
        r#"
        (module
            (func (export "simple") (result i32)
                i32.const 42
            )
        )
    "#,
    )
    .expect("Failed to parse WAT");

    let owner = [0u8; 32];
    let limits = ResourceLimits::default();

    let mut sandbox = Sandbox::new(&wasm, owner, limits).expect("Failed to create sandbox");
    sandbox.initialize().expect("Failed to initialize");

    // Execute to consume some fuel
    sandbox.invoke("simple", &[]).expect("Failed to invoke");

    // Refuel the sandbox
    sandbox.refuel(1_000_000).expect("Failed to refuel");

    // Should be able to execute again
    let result = sandbox.invoke("simple", &[]).expect("Failed to invoke");
    assert!(result.success);
    assert_eq!(result.return_value.as_ref().unwrap()[0].unwrap_i32(), 42);
}

/// Tests FuelManager directly
#[test]
fn test_fuel_manager() {
    let mut manager = FuelManager::new(10000);

    assert_eq!(manager.remaining(), 10000);
    assert_eq!(manager.total_consumed(), 0);
    assert!(!manager.is_exhausted());

    // Consume some fuel
    manager.consume(3000).expect("Failed to consume");
    assert_eq!(manager.remaining(), 7000);
    assert_eq!(manager.total_consumed(), 3000);

    // Refuel
    manager.refuel(1000).expect("Failed to refuel");
    assert_eq!(manager.remaining(), 8000);
    assert_eq!(manager.total_consumed(), 3000); // Consumed doesn't change on refuel

    // Consume remaining
    manager.consume(8000).expect("Failed to consume");
    assert_eq!(manager.remaining(), 0);
    assert!(manager.is_exhausted());

    // Attempting to consume more should fail
    let result = manager.consume(1);
    assert!(result.is_err());
}

/// Tests that different operations consume different amounts of fuel
#[test]
fn test_fuel_operation_costs() {
    // Simple operation
    let simple_wasm = wat::parse_str(
        r#"
        (module
            (func (export "simple") (result i32)
                i32.const 1
            )
        )
    "#,
    )
    .expect("Failed to parse WAT");

    // Complex operation with multiple memory accesses
    let complex_wasm = wat::parse_str(
        r#"
        (module
            (memory (export "memory") 1)
            (func (export "complex") (result i32)
                (local $sum i32)
                ;; Store multiple values
                (i32.store (i32.const 0) (i32.const 1))
                (i32.store (i32.const 4) (i32.const 2))
                (i32.store (i32.const 8) (i32.const 3))
                ;; Load and sum them
                (local.set $sum (i32.load (i32.const 0)))
                (local.set $sum (i32.add (local.get $sum) (i32.load (i32.const 4))))
                (local.set $sum (i32.add (local.get $sum) (i32.load (i32.const 8))))
                (local.get $sum)
            )
        )
    "#,
    )
    .expect("Failed to parse WAT");

    let owner = [0u8; 32];
    let limits = ResourceLimits::default();

    let mut simple_sandbox =
        Sandbox::new(&simple_wasm, owner, limits.clone()).expect("Failed to create sandbox");
    simple_sandbox.initialize().expect("Failed to initialize");

    let mut complex_sandbox =
        Sandbox::new(&complex_wasm, owner, limits).expect("Failed to create sandbox");
    complex_sandbox.initialize().expect("Failed to initialize");

    let simple_result = simple_sandbox
        .invoke("simple", &[])
        .expect("Failed to invoke");
    let complex_result = complex_sandbox
        .invoke("complex", &[])
        .expect("Failed to invoke");

    // Complex operation should consume more fuel
    assert!(complex_result.fuel_consumed > simple_result.fuel_consumed);
}

// ═══════════════════════════════════════════════════════════════════════════
// TEST 5: CONCURRENT SANDBOXES
// ═══════════════════════════════════════════════════════════════════════════

/// Tests running multiple sandboxes concurrently
#[test]
fn test_concurrent_sandboxes_basic() {
    let wasm = wat::parse_str(
        r#"
        (module
            (global $counter (mut i32) (i32.const 0))
            (func (export "work") (param i32) (result i32)
                (local $i i32)
                (local $sum i32)
                (local.set $i (i32.const 0))
                (local.set $sum (i32.const 0))
                (block $done
                    (loop $loop
                        (br_if $done (i32.ge_u (local.get $i) (local.get 0)))
                        (local.set $sum (i32.add (local.get $sum) (local.get $i)))
                        (local.set $i (i32.add (local.get $i) (i32.const 1)))
                        (global.set $counter (i32.add (global.get $counter) (i32.const 1)))
                        (br $loop)
                    )
                )
                (local.get $sum)
            )
        )
    "#,
    )
    .expect("Failed to parse WAT");

    let num_threads = 4;
    let barrier = Arc::new(Barrier::new(num_threads));
    let wasm = Arc::new(wasm);

    let handles: Vec<_> = (0..num_threads)
        .map(|thread_id| {
            let wasm = Arc::clone(&wasm);
            let barrier = Arc::clone(&barrier);

            thread::spawn(move || {
                // Wait for all threads to be ready
                barrier.wait();

                let owner = [thread_id as u8; 32];
                let limits = ResourceLimits::default();

                let mut sandbox =
                    Sandbox::new(&wasm, owner, limits).expect("Failed to create sandbox");
                sandbox.initialize().expect("Failed to initialize");

                // Each thread does different amount of work
                let iterations = (thread_id + 1) * 100;
                let result = sandbox
                    .invoke("work", &[Val::I32(iterations as i32)])
                    .expect("Failed to invoke");

                assert!(result.success);

                // Return the result for verification
                (
                    thread_id,
                    result.return_value.as_ref().unwrap()[0].unwrap_i32(),
                    result.fuel_consumed,
                )
            })
        })
        .collect();

    // Collect results
    let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();

    // Verify each thread got correct results (sum of 0..n = n*(n-1)/2)
    for (thread_id, sum, fuel) in results {
        let iterations = (thread_id + 1) * 100;
        let expected_sum = (iterations * (iterations - 1)) / 2;
        assert_eq!(sum, expected_sum as i32);
        assert!(fuel > 0);
    }
}

/// Tests concurrent access to shared storage with capability checks
#[test]
fn test_concurrent_sandboxes_with_storage() {
    let storage = Arc::new(InMemoryStorage::new());
    let num_threads = 4;
    let barrier = Arc::new(Barrier::new(num_threads));

    let handles: Vec<_> = (0..num_threads)
        .map(|thread_id| {
            let storage = Arc::clone(&storage);
            let barrier = Arc::clone(&barrier);

            thread::spawn(move || {
                // Each thread uses its own capability set
                let caps = create_storage_capset();

                // Wait for all threads
                barrier.wait();

                // Each thread writes to its own key
                let key = format!("thread_{}", thread_id);
                let value = format!("value_{}", thread_id);

                let write_result =
                    host_storage_write(&caps, storage.as_ref(), key.as_bytes(), value.as_bytes());
                assert!(write_result.success);

                // Read back
                let read_result = host_storage_read(&caps, storage.as_ref(), key.as_bytes());
                assert!(read_result.success);
                assert_eq!(read_result.return_value, Some(value.into_bytes()));

                thread_id
            })
        })
        .collect();

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    // Verify all keys are present in storage
    assert_eq!(storage.count().unwrap(), num_threads);
}

/// Tests that concurrent sandboxes maintain isolation under stress
#[test]
fn test_concurrent_sandboxes_isolation_stress() {
    let wasm = wat::parse_str(
        r#"
        (module
            (memory (export "memory") 1)
            (global $state (mut i32) (i32.const 0))

            ;; Write thread-specific pattern to memory and global
            (func (export "init_state") (param i32)
                (global.set $state (local.get 0))
                (i32.store (i32.const 0) (local.get 0))
            )

            ;; Verify state hasn't been corrupted
            (func (export "verify_state") (param i32) (result i32)
                (if (result i32)
                    (i32.and
                        (i32.eq (global.get $state) (local.get 0))
                        (i32.eq (i32.load (i32.const 0)) (local.get 0))
                    )
                    (then (i32.const 1))
                    (else (i32.const 0))
                )
            )
        )
    "#,
    )
    .expect("Failed to parse WAT");

    let num_threads = 8;
    let iterations_per_thread = 100;
    let barrier = Arc::new(Barrier::new(num_threads));
    let wasm = Arc::new(wasm);

    let handles: Vec<_> = (0..num_threads)
        .map(|thread_id| {
            let wasm = Arc::clone(&wasm);
            let barrier = Arc::clone(&barrier);

            thread::spawn(move || {
                let owner = [thread_id as u8; 32];
                let limits = ResourceLimits::default();

                let mut sandbox =
                    Sandbox::new(&wasm, owner, limits).expect("Failed to create sandbox");
                sandbox.initialize().expect("Failed to initialize");

                // Thread-specific magic number
                let magic = (thread_id * 1000 + 42) as i32;

                // Wait for all threads
                barrier.wait();

                // Initialize with thread-specific value
                sandbox
                    .invoke("init_state", &[Val::I32(magic)])
                    .expect("Failed to init");

                // Repeatedly verify state hasn't been corrupted
                for _ in 0..iterations_per_thread {
                    let result = sandbox
                        .invoke("verify_state", &[Val::I32(magic)])
                        .expect("Failed to verify");

                    assert!(result.success);
                    let verified = result.return_value.as_ref().unwrap()[0].unwrap_i32();
                    assert_eq!(
                        verified, 1,
                        "State corruption detected in thread {}",
                        thread_id
                    );
                }

                thread_id
            })
        })
        .collect();

    // Collect all results - all threads should complete successfully
    let completed: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    assert_eq!(completed.len(), num_threads);
}

/// Tests metrics tracking in concurrent execution
#[test]
fn test_concurrent_sandboxes_metrics() {
    let wasm = wat::parse_str(
        r#"
        (module
            (func (export "work") (result i32)
                i32.const 42
            )
        )
    "#,
    )
    .expect("Failed to parse WAT");

    let num_threads = 4;
    let executions_per_thread = 10;
    let barrier = Arc::new(Barrier::new(num_threads));
    let wasm = Arc::new(wasm);

    let handles: Vec<_> = (0..num_threads)
        .map(|thread_id| {
            let wasm = Arc::clone(&wasm);
            let barrier = Arc::clone(&barrier);

            thread::spawn(move || {
                let owner = [thread_id as u8; 32];
                let limits = ResourceLimits::default();

                let mut sandbox =
                    Sandbox::new(&wasm, owner, limits).expect("Failed to create sandbox");
                sandbox.initialize().expect("Failed to initialize");

                barrier.wait();

                for _ in 0..executions_per_thread {
                    sandbox.invoke("work", &[]).expect("Failed to invoke");
                }

                let metrics = sandbox.metrics();
                (thread_id, metrics)
            })
        })
        .collect();

    for handle in handles {
        let (thread_id, metrics) = handle.join().unwrap();

        // Each sandbox should have correct execution count
        assert_eq!(
            metrics.execution_count, executions_per_thread as u64,
            "Thread {} had wrong execution count",
            thread_id
        );

        // Each sandbox should have consumed some fuel
        assert!(
            metrics.total_fuel_consumed > 0,
            "Thread {} consumed no fuel",
            thread_id
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// ADDITIONAL EDGE CASE TESTS
// ═══════════════════════════════════════════════════════════════════════════

/// Tests handling of invalid WASM module
#[test]
fn test_invalid_wasm_module() {
    let invalid_wasm = b"not a valid wasm module";
    let owner = [0u8; 32];
    let limits = ResourceLimits::default();

    // Sandbox::new doesn't validate bytecode content, only size
    // Invalid WASM will be caught during initialize() when compiling
    let mut sandbox = Sandbox::new(invalid_wasm, owner, limits).expect("new should succeed");
    let result = sandbox.initialize();
    assert!(
        result.is_err(),
        "initialize() should fail with invalid WASM"
    );
}

/// Tests handling of empty WASM module
#[test]
fn test_empty_wasm_module() {
    let empty_wasm = b"";
    let owner = [0u8; 32];
    let limits = ResourceLimits::default();

    let result = Sandbox::new(empty_wasm, owner, limits);
    assert!(result.is_err());
}

/// Tests calling non-existent function
#[test]
fn test_nonexistent_function() {
    let wasm = wat::parse_str(
        r#"
        (module
            (func (export "exists") (result i32)
                i32.const 1
            )
        )
    "#,
    )
    .expect("Failed to parse WAT");

    let owner = [0u8; 32];
    let limits = ResourceLimits::default();

    let mut sandbox = Sandbox::new(&wasm, owner, limits).expect("Failed to create sandbox");
    sandbox.initialize().expect("Failed to initialize");

    let result = sandbox.invoke("does_not_exist", &[]);
    assert!(result.is_err());
}

/// Tests WASM trap handling
#[test]
fn test_wasm_trap_handling() {
    let wasm = wat::parse_str(
        r#"
        (module
            (func (export "divide") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.div_s
            )
        )
    "#,
    )
    .expect("Failed to parse WAT");

    let owner = [0u8; 32];
    let limits = ResourceLimits::default();

    let mut sandbox = Sandbox::new(&wasm, owner, limits).expect("Failed to create sandbox");
    sandbox.initialize().expect("Failed to initialize");

    // Division by zero should trap
    let result = sandbox
        .invoke("divide", &[Val::I32(10), Val::I32(0)])
        .expect("invoke should return result");

    assert!(!result.success);
    assert!(result.error.is_some());
    assert_eq!(sandbox.get_state(), SandboxState::Failed);
}

/// Tests sandbox state transitions
#[test]
fn test_sandbox_state_machine() {
    let wasm = wat::parse_str(
        r#"
        (module
            (func (export "noop") (result i32)
                i32.const 1
            )
        )
    "#,
    )
    .expect("Failed to parse WAT");

    let owner = [0u8; 32];
    let limits = ResourceLimits::default();

    let mut sandbox = Sandbox::new(&wasm, owner, limits).expect("Failed to create sandbox");

    // Initial state
    assert_eq!(sandbox.get_state(), SandboxState::Initializing);

    // Cannot invoke before initialization
    let result = sandbox.invoke("noop", &[]);
    assert!(result.is_err());

    // Initialize
    sandbox.initialize().expect("Failed to initialize");
    assert_eq!(sandbox.get_state(), SandboxState::Ready);

    // Execute
    sandbox.invoke("noop", &[]).expect("Failed to invoke");
    assert_eq!(sandbox.get_state(), SandboxState::Ready);

    // Terminate
    sandbox.terminate();
    assert_eq!(sandbox.get_state(), SandboxState::Terminated);

    // Cannot invoke after termination
    let result = sandbox.invoke("noop", &[]);
    assert!(result.is_err());
}

/// Tests resource limits validation
#[test]
fn test_resource_limits_variants() {
    let wasm = wat::parse_str(
        r#"
        (module
            (func (export "noop") (result i32)
                i32.const 1
            )
        )
    "#,
    )
    .expect("Failed to parse WAT");

    let owner = [0u8; 32];

    // Test with default limits
    let default_limits = ResourceLimits::default();
    assert!(Sandbox::new(&wasm, owner, default_limits).is_ok());

    // Test with custom limits
    let custom_limits = ResourceLimits {
        memory_bytes: 32 * 1024 * 1024, // 32MB
        max_fuel: 500_000,
        ..Default::default()
    };
    assert!(Sandbox::new(&wasm, owner, custom_limits).is_ok());

    // Test with minimal fuel limits
    let minimal_fuel_limits = ResourceLimits {
        max_fuel: 10_000,
        ..Default::default()
    };
    assert!(Sandbox::new(&wasm, owner, minimal_fuel_limits).is_ok());
}

/// Tests 64-bit integer operations
#[test]
fn test_i64_operations() {
    let wasm = wat::parse_str(
        r#"
        (module
            (func (export "add64") (param i64 i64) (result i64)
                local.get 0
                local.get 1
                i64.add
            )
            (func (export "large_const") (result i64)
                i64.const 9223372036854775807
            )
        )
    "#,
    )
    .expect("Failed to parse WAT");

    let owner = [0u8; 32];
    let limits = ResourceLimits::default();

    let mut sandbox = Sandbox::new(&wasm, owner, limits).expect("Failed to create sandbox");
    sandbox.initialize().expect("Failed to initialize");

    // Test 64-bit addition
    let result = sandbox
        .invoke("add64", &[Val::I64(100000000000), Val::I64(200000000000)])
        .expect("Failed to invoke");
    assert!(result.success);
    assert_eq!(
        result.return_value.as_ref().unwrap()[0].unwrap_i64(),
        300000000000
    );

    // Test large constant
    let result = sandbox
        .invoke("large_const", &[])
        .expect("Failed to invoke");
    assert!(result.success);
    assert_eq!(
        result.return_value.as_ref().unwrap()[0].unwrap_i64(),
        i64::MAX
    );
}

/// Tests floating point operations
#[test]
fn test_float_operations() {
    let wasm = wat::parse_str(
        r#"
        (module
            (func (export "add_f32") (param f32 f32) (result f32)
                local.get 0
                local.get 1
                f32.add
            )
            (func (export "mul_f64") (param f64 f64) (result f64)
                local.get 0
                local.get 1
                f64.mul
            )
        )
    "#,
    )
    .expect("Failed to parse WAT");

    let owner = [0u8; 32];
    let limits = ResourceLimits::default();

    let mut sandbox = Sandbox::new(&wasm, owner, limits).expect("Failed to create sandbox");
    sandbox.initialize().expect("Failed to initialize");

    // Test f32 addition - Val::F32 expects u32 bits representation
    let result = sandbox
        .invoke(
            "add_f32",
            &[Val::F32(1.5_f32.to_bits()), Val::F32(2.5_f32.to_bits())],
        )
        .expect("Failed to invoke");
    assert!(result.success);
    // unwrap_f32() returns f32 directly
    let sum = result.return_value.as_ref().unwrap()[0].unwrap_f32();
    assert!((sum - 4.0).abs() < 0.0001);

    // Test f64 multiplication - Val::F64 expects u64 bits representation
    let result = sandbox
        .invoke(
            "mul_f64",
            &[Val::F64(std::f64::consts::PI.to_bits()), Val::F64(2.0_f64.to_bits())],
        )
        .expect("Failed to invoke");
    assert!(result.success);
    // unwrap_f64() returns f64 directly
    let product = result.return_value.as_ref().unwrap()[0].unwrap_f64();
    assert!((product - std::f64::consts::TAU).abs() < 0.0001);
}
