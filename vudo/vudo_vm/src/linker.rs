//! VUDO VM Linker Module
//!
//! Provides the Wasmtime linker configuration for host function resolution.
//! This module bridges WASM modules with host functions through a unified
//! HostState that holds all necessary backends and capabilities.
//!
//! ## Host Functions
//! All host functions are registered under the "vudo" namespace:
//! - Time: host_time_now
//! - Random: host_random_bytes
//! - Logging: host_log
//! - Storage: host_storage_read, host_storage_write, host_storage_delete
//! - Network: host_network_connect, host_network_listen, host_network_broadcast
//! - Credit: host_credit_balance, host_credit_transfer, host_credit_reserve, host_credit_release
//!
//! ## Memory Layout
//! Functions that operate on memory use the following conventions:
//! - Pointers are i32 offsets into WASM linear memory
//! - Lengths are i32 byte counts
//! - Return values of -1 indicate errors
//! - Return values of 0 or positive indicate success (may contain result data)

use std::sync::Arc;
use std::time::{Duration, Instant};
use wasmtime::{Caller, Engine, Linker, Memory};

use crate::capability::CapabilitySet;
use crate::host::credit::PublicKey;
use crate::host::log::LogLevel;
use crate::host::{
    host_credit_available, host_credit_balance, host_credit_consume, host_credit_release,
    host_credit_reserve, host_credit_transfer, host_log, host_network_broadcast,
    host_network_connect, host_network_listen, host_random_bytes, host_storage_delete,
    host_storage_read, host_storage_write, host_time_now, CreditBackend, NetworkBackend,
    StorageBackend,
};

// ═══════════════════════════════════════════════════════════════════════════
// ERROR CODES
// ═══════════════════════════════════════════════════════════════════════════

/// Error codes returned by host functions to WASM.
///
/// These codes follow a convention where:
/// - `SUCCESS` (0) indicates successful operation
/// - Negative values indicate errors
/// - Positive values may indicate success with data (e.g., bytes read/written)
pub mod error_codes {
    /// Operation succeeded
    pub const SUCCESS: i32 = 0;
    /// Capability denied - the sandbox lacks required permissions
    pub const CAPABILITY_DENIED: i32 = -1;
    /// Invalid memory access - pointer/length out of bounds
    pub const INVALID_MEMORY: i32 = -2;
    /// Invalid parameter - bad input value
    pub const INVALID_PARAMETER: i32 = -3;
    /// Storage operation error
    pub const STORAGE_ERROR: i32 = -4;
    /// Network operation error
    pub const NETWORK_ERROR: i32 = -5;
    /// Credit/accounting operation error
    pub const CREDIT_ERROR: i32 = -6;
    /// Output buffer too small for result
    pub const BUFFER_TOO_SMALL: i32 = -7;
    /// Internal error in host function
    pub const INTERNAL_ERROR: i32 = -8;
}

// ═══════════════════════════════════════════════════════════════════════════
// HOST STATE
// ═══════════════════════════════════════════════════════════════════════════

/// HostState holds all the context needed for host function execution.
///
/// This structure is stored in the Wasmtime Store and provides:
/// - Storage backend for persistent data operations
/// - Credit backend for resource accounting
/// - Network backend for communication operations
/// - Capability set for permission checking
/// - Account identity for credit operations
/// - WASM memory reference for reading/writing data
/// - Timing information for timeout tracking
///
/// The HostState is accessed by host functions through the Caller context.
///
/// # Example
///
/// ```ignore
/// use vudo_vm::linker::HostState;
/// use vudo_vm::host::{InMemoryStorage, InMemoryCreditLedger, MockNetworkBackend};
/// use vudo_vm::CapabilitySet;
/// use std::sync::Arc;
/// use std::time::Duration;
///
/// let state = HostState::new(
///     Arc::new(InMemoryStorage::new()),
///     Arc::new(InMemoryCreditLedger::new()),
///     Arc::new(MockNetworkBackend::new()),
///     CapabilitySet::new(),
///     Duration::from_secs(30),
///     [0u8; 32],
/// );
/// ```
pub struct HostState {
    /// Storage backend for read/write/delete operations
    pub storage: Arc<dyn StorageBackend>,

    /// Credit ledger for resource accounting
    pub credit: Arc<dyn CreditBackend>,

    /// Network backend for connection/listen/broadcast operations
    pub network: Arc<dyn NetworkBackend>,

    /// Capability set defining allowed operations
    pub capabilities: CapabilitySet,

    /// Fuel consumed during execution (tracked separately for metrics)
    pub fuel_consumed: u64,

    /// Start time of the current execution (for timeout tracking)
    pub start_time: Option<Instant>,

    /// Maximum duration allowed for execution
    pub timeout: Duration,

    /// The account (Ed25519 public key) associated with this sandbox.
    /// Used for credit operations to identify the caller.
    pub account: PublicKey,

    /// WASM linear memory, set after module instantiation.
    /// This is required for host functions that read/write memory.
    memory: Option<Memory>,
}

impl HostState {
    /// Create a new HostState with the given backends and capabilities
    pub fn new(
        storage: Arc<dyn StorageBackend>,
        credit: Arc<dyn CreditBackend>,
        network: Arc<dyn NetworkBackend>,
        capabilities: CapabilitySet,
        timeout: Duration,
    ) -> Self {
        Self {
            storage,
            credit,
            network,
            capabilities,
            fuel_consumed: 0,
            start_time: None,
            timeout,
        }
    }

    /// Check if the execution has timed out
    pub fn is_timed_out(&self) -> bool {
        if let Some(start) = self.start_time {
            start.elapsed() >= self.timeout
        } else {
            false
        }
    }

    /// Start execution timer
    pub fn start_execution(&mut self) {
        self.start_time = Some(Instant::now());
    }

    /// Get elapsed time since execution started
    pub fn elapsed(&self) -> Option<Duration> {
        self.start_time.map(|start| start.elapsed())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// LINKER CREATION
// ═══════════════════════════════════════════════════════════════════════════

/// Error code returned when a host function fails
pub const HOST_ERROR: i32 = -1;

/// Success code for host functions that don't return data
pub const HOST_SUCCESS: i32 = 0;

/// Helper to get memory from a caller
fn get_memory(caller: &mut Caller<'_, HostState>) -> Option<wasmtime::Memory> {
    caller.get_export("memory")?.into_memory()
}

/// Helper to read bytes from WASM memory
fn read_memory(
    caller: &Caller<'_, HostState>,
    memory: &wasmtime::Memory,
    ptr: i32,
    len: i32,
) -> Option<Vec<u8>> {
    if ptr < 0 || len < 0 {
        return None;
    }
    let ptr = ptr as usize;
    let len = len as usize;
    let data = memory.data(caller);
    if ptr.saturating_add(len) > data.len() {
        return None;
    }
    Some(data[ptr..ptr + len].to_vec())
}

/// Helper to write bytes to WASM memory
fn write_memory(
    caller: &mut Caller<'_, HostState>,
    memory: &wasmtime::Memory,
    ptr: i32,
    data: &[u8],
) -> bool {
    if ptr < 0 {
        return false;
    }
    let ptr = ptr as usize;
    let mem_data = memory.data_mut(caller);
    if ptr.saturating_add(data.len()) > mem_data.len() {
        return false;
    }
    mem_data[ptr..ptr + data.len()].copy_from_slice(data);
    true
}

/// Create a new Linker configured with VUDO host functions.
///
/// The returned linker is ready to instantiate WASM modules that import
/// functions from the "vudo" namespace. All host functions are registered
/// and will use the HostState stored in the wasmtime Store for backends
/// and capability checking.
///
/// # Arguments
/// * `engine` - The Wasmtime engine to create the linker for
///
/// # Returns
/// A configured Linker ready for module instantiation
///
/// # Example
/// ```ignore
/// let engine = Engine::new(&config)?;
/// let linker = create_linker(&engine);
/// let instance = linker.instantiate(&mut store, &module)?;
/// ```
pub fn create_linker(engine: &Engine) -> Linker<HostState> {
    let mut linker = Linker::new(engine);

    // ═══════════════════════════════════════════════════════════════════════
    // TIME FUNCTIONS
    // ═══════════════════════════════════════════════════════════════════════

    // host_time_now: fn() -> i64
    // Returns current Unix timestamp in nanoseconds, or -1 on error
    linker
        .func_wrap(
            "vudo",
            "host_time_now",
            |caller: Caller<'_, HostState>| -> i64 {
                let state = caller.data();
                let result = host_time_now(&state.capabilities);
                if result.success {
                    if let Some(bytes) = result.return_value {
                        if bytes.len() == 8 {
                            return i64::from_le_bytes(bytes.try_into().unwrap());
                        }
                    }
                }
                -1
            },
        )
        .expect("Failed to register host_time_now");

    // ═══════════════════════════════════════════════════════════════════════
    // RANDOM FUNCTIONS
    // ═══════════════════════════════════════════════════════════════════════

    // host_random_bytes: fn(ptr: i32, len: i32) -> i32
    // Fills memory at ptr with len random bytes, returns 0 on success, -1 on error
    linker
        .func_wrap(
            "vudo",
            "host_random_bytes",
            |mut caller: Caller<'_, HostState>, ptr: i32, len: i32| -> i32 {
                if len <= 0 {
                    return HOST_ERROR;
                }
                let memory = match get_memory(&mut caller) {
                    Some(m) => m,
                    None => return HOST_ERROR,
                };
                let result = host_random_bytes(&caller.data().capabilities, len as u32);
                if result.success {
                    if let Some(bytes) = result.return_value {
                        if write_memory(&mut caller, &memory, ptr, &bytes) {
                            return HOST_SUCCESS;
                        }
                    }
                }
                HOST_ERROR
            },
        )
        .expect("Failed to register host_random_bytes");

    // ═══════════════════════════════════════════════════════════════════════
    // LOGGING FUNCTIONS
    // ═══════════════════════════════════════════════════════════════════════

    // host_log: fn(level: i32, ptr: i32, len: i32) -> i32
    // Logs a message at the specified level, returns 0 on success, -1 on error
    linker
        .func_wrap(
            "vudo",
            "host_log",
            |mut caller: Caller<'_, HostState>, level: i32, ptr: i32, len: i32| -> i32 {
                let memory = match get_memory(&mut caller) {
                    Some(m) => m,
                    None => return HOST_ERROR,
                };
                let log_level = match LogLevel::from_u8(level as u8) {
                    Some(l) => l,
                    None => return HOST_ERROR,
                };
                let message_bytes = match read_memory(&caller, &memory, ptr, len) {
                    Some(b) => b,
                    None => return HOST_ERROR,
                };
                let message = match String::from_utf8(message_bytes) {
                    Ok(s) => s,
                    Err(_) => return HOST_ERROR,
                };
                let result = host_log(&caller.data().capabilities, log_level, &message);
                if result.success {
                    HOST_SUCCESS
                } else {
                    HOST_ERROR
                }
            },
        )
        .expect("Failed to register host_log");

    // ═══════════════════════════════════════════════════════════════════════
    // STORAGE FUNCTIONS
    // ═══════════════════════════════════════════════════════════════════════

    // host_storage_read: fn(key_ptr: i32, key_len: i32, val_ptr: i32, val_cap: i32) -> i32
    // Reads value for key into val_ptr buffer, returns bytes written or -1 on error
    linker
        .func_wrap(
            "vudo",
            "host_storage_read",
            |mut caller: Caller<'_, HostState>,
             key_ptr: i32,
             key_len: i32,
             val_ptr: i32,
             val_cap: i32|
             -> i32 {
                let memory = match get_memory(&mut caller) {
                    Some(m) => m,
                    None => return HOST_ERROR,
                };
                let key = match read_memory(&caller, &memory, key_ptr, key_len) {
                    Some(k) => k,
                    None => return HOST_ERROR,
                };
                let state = caller.data();
                let result = host_storage_read(&state.capabilities, state.storage.as_ref(), &key);
                if result.success {
                    if let Some(value) = result.return_value {
                        if value.len() > val_cap as usize {
                            return HOST_ERROR; // Buffer too small
                        }
                        if write_memory(&mut caller, &memory, val_ptr, &value) {
                            return value.len() as i32;
                        }
                    }
                    return 0; // Key not found (no value)
                }
                HOST_ERROR
            },
        )
        .expect("Failed to register host_storage_read");

    // host_storage_write: fn(key_ptr: i32, key_len: i32, val_ptr: i32, val_len: i32) -> i32
    // Writes value at val_ptr to storage under key, returns 0 on success, -1 on error
    linker
        .func_wrap(
            "vudo",
            "host_storage_write",
            |mut caller: Caller<'_, HostState>,
             key_ptr: i32,
             key_len: i32,
             val_ptr: i32,
             val_len: i32|
             -> i32 {
                let memory = match get_memory(&mut caller) {
                    Some(m) => m,
                    None => return HOST_ERROR,
                };
                let key = match read_memory(&caller, &memory, key_ptr, key_len) {
                    Some(k) => k,
                    None => return HOST_ERROR,
                };
                let value = match read_memory(&caller, &memory, val_ptr, val_len) {
                    Some(v) => v,
                    None => return HOST_ERROR,
                };
                let state = caller.data();
                let result =
                    host_storage_write(&state.capabilities, state.storage.as_ref(), &key, &value);
                if result.success {
                    HOST_SUCCESS
                } else {
                    HOST_ERROR
                }
            },
        )
        .expect("Failed to register host_storage_write");

    // host_storage_delete: fn(key_ptr: i32, key_len: i32) -> i32
    // Deletes key from storage, returns 1 if deleted, 0 if not found, -1 on error
    linker
        .func_wrap(
            "vudo",
            "host_storage_delete",
            |mut caller: Caller<'_, HostState>, key_ptr: i32, key_len: i32| -> i32 {
                let memory = match get_memory(&mut caller) {
                    Some(m) => m,
                    None => return HOST_ERROR,
                };
                let key = match read_memory(&caller, &memory, key_ptr, key_len) {
                    Some(k) => k,
                    None => return HOST_ERROR,
                };
                let state = caller.data();
                let result = host_storage_delete(&state.capabilities, state.storage.as_ref(), &key);
                if result.success {
                    if let Some(bytes) = result.return_value {
                        if !bytes.is_empty() {
                            return bytes[0] as i32; // 1 if deleted, 0 if not found
                        }
                    }
                    HOST_SUCCESS
                } else {
                    HOST_ERROR
                }
            },
        )
        .expect("Failed to register host_storage_delete");

    // ═══════════════════════════════════════════════════════════════════════
    // NETWORK FUNCTIONS
    // ═══════════════════════════════════════════════════════════════════════

    // host_network_connect: fn(addr_ptr: i32, addr_len: i32) -> i64
    // Connects to address, returns connection handle on success, -1 on error
    linker
        .func_wrap(
            "vudo",
            "host_network_connect",
            |mut caller: Caller<'_, HostState>, addr_ptr: i32, addr_len: i32| -> i64 {
                let memory = match get_memory(&mut caller) {
                    Some(m) => m,
                    None => return -1,
                };
                let addr_bytes = match read_memory(&caller, &memory, addr_ptr, addr_len) {
                    Some(b) => b,
                    None => return -1,
                };
                let address = match String::from_utf8(addr_bytes) {
                    Ok(s) => s,
                    Err(_) => return -1,
                };
                let state = caller.data();
                let result =
                    host_network_connect(&state.capabilities, state.network.as_ref(), &address);
                if result.success {
                    if let Some(bytes) = result.return_value {
                        if bytes.len() == 8 {
                            return i64::from_le_bytes(bytes.try_into().unwrap());
                        }
                    }
                }
                -1
            },
        )
        .expect("Failed to register host_network_connect");

    // host_network_listen: fn(port: i32) -> i64
    // Starts listening on port, returns listener handle on success, -1 on error
    linker
        .func_wrap(
            "vudo",
            "host_network_listen",
            |caller: Caller<'_, HostState>, port: i32| -> i64 {
                if !(0..=65535).contains(&port) {
                    return -1;
                }
                let state = caller.data();
                let result =
                    host_network_listen(&state.capabilities, state.network.as_ref(), port as u16);
                if result.success {
                    if let Some(bytes) = result.return_value {
                        if bytes.len() == 8 {
                            return i64::from_le_bytes(bytes.try_into().unwrap());
                        }
                    }
                }
                -1
            },
        )
        .expect("Failed to register host_network_listen");

    // host_network_broadcast: fn(msg_ptr: i32, msg_len: i32) -> i64
    // Broadcasts message to peers, returns peer count on success, -1 on error
    linker
        .func_wrap(
            "vudo",
            "host_network_broadcast",
            |mut caller: Caller<'_, HostState>, msg_ptr: i32, msg_len: i32| -> i64 {
                let memory = match get_memory(&mut caller) {
                    Some(m) => m,
                    None => return -1,
                };
                let message = match read_memory(&caller, &memory, msg_ptr, msg_len) {
                    Some(m) => m,
                    None => return -1,
                };
                let state = caller.data();
                let result =
                    host_network_broadcast(&state.capabilities, state.network.as_ref(), &message);
                if result.success {
                    if let Some(bytes) = result.return_value {
                        if bytes.len() == 8 {
                            return i64::from_le_bytes(bytes.try_into().unwrap());
                        }
                    }
                }
                -1
            },
        )
        .expect("Failed to register host_network_broadcast");

    // ═══════════════════════════════════════════════════════════════════════
    // CREDIT FUNCTIONS
    // ═══════════════════════════════════════════════════════════════════════

    // host_credit_balance: fn(account_ptr: i32) -> i64
    // Returns credit balance for account (32 bytes at ptr), or -1 on error
    linker
        .func_wrap(
            "vudo",
            "host_credit_balance",
            |mut caller: Caller<'_, HostState>, account_ptr: i32| -> i64 {
                let memory = match get_memory(&mut caller) {
                    Some(m) => m,
                    None => return -1,
                };
                let account_bytes = match read_memory(&caller, &memory, account_ptr, 32) {
                    Some(b) => b,
                    None => return -1,
                };
                let account: [u8; 32] = match account_bytes.try_into() {
                    Ok(a) => a,
                    Err(_) => return -1,
                };
                let state = caller.data();
                let result =
                    host_credit_balance(&state.capabilities, state.credit.as_ref(), &account);
                if result.success {
                    if let Some(bytes) = result.return_value {
                        if bytes.len() == 8 {
                            return i64::from_le_bytes(bytes.try_into().unwrap());
                        }
                    }
                }
                -1
            },
        )
        .expect("Failed to register host_credit_balance");

    // host_credit_transfer: fn(from_ptr: i32, to_ptr: i32, amount: i64) -> i32
    // Transfers credits between accounts (32 bytes each), returns 0 on success, -1 on error
    linker
        .func_wrap(
            "vudo",
            "host_credit_transfer",
            |mut caller: Caller<'_, HostState>, from_ptr: i32, to_ptr: i32, amount: i64| -> i32 {
                if amount < 0 {
                    return HOST_ERROR;
                }
                let memory = match get_memory(&mut caller) {
                    Some(m) => m,
                    None => return HOST_ERROR,
                };
                let from_bytes = match read_memory(&caller, &memory, from_ptr, 32) {
                    Some(b) => b,
                    None => return HOST_ERROR,
                };
                let to_bytes = match read_memory(&caller, &memory, to_ptr, 32) {
                    Some(b) => b,
                    None => return HOST_ERROR,
                };
                let from: [u8; 32] = match from_bytes.try_into() {
                    Ok(a) => a,
                    Err(_) => return HOST_ERROR,
                };
                let to: [u8; 32] = match to_bytes.try_into() {
                    Ok(a) => a,
                    Err(_) => return HOST_ERROR,
                };
                let state = caller.data();
                let result = host_credit_transfer(
                    &state.capabilities,
                    state.credit.as_ref(),
                    &from,
                    &to,
                    amount as u64,
                );
                if result.success {
                    HOST_SUCCESS
                } else {
                    HOST_ERROR
                }
            },
        )
        .expect("Failed to register host_credit_transfer");

    // host_credit_reserve: fn(account_ptr: i32, amount: i64) -> i64
    // Reserves credits for account, returns reservation ID on success, -1 on error
    linker
        .func_wrap(
            "vudo",
            "host_credit_reserve",
            |mut caller: Caller<'_, HostState>, account_ptr: i32, amount: i64| -> i64 {
                if amount <= 0 {
                    return -1;
                }
                let memory = match get_memory(&mut caller) {
                    Some(m) => m,
                    None => return -1,
                };
                let account_bytes = match read_memory(&caller, &memory, account_ptr, 32) {
                    Some(b) => b,
                    None => return -1,
                };
                let account: [u8; 32] = match account_bytes.try_into() {
                    Ok(a) => a,
                    Err(_) => return -1,
                };
                let state = caller.data();
                let result = host_credit_reserve(
                    &state.capabilities,
                    state.credit.as_ref(),
                    &account,
                    amount as u64,
                );
                if result.success {
                    if let Some(bytes) = result.return_value {
                        if bytes.len() == 8 {
                            return i64::from_le_bytes(bytes.try_into().unwrap());
                        }
                    }
                }
                -1
            },
        )
        .expect("Failed to register host_credit_reserve");

    // host_credit_release: fn(reservation_id: i64) -> i32
    // Releases a reservation, returns 0 on success, -1 on error
    linker
        .func_wrap(
            "vudo",
            "host_credit_release",
            |caller: Caller<'_, HostState>, reservation_id: i64| -> i32 {
                if reservation_id < 0 {
                    return HOST_ERROR;
                }
                let state = caller.data();
                let result = host_credit_release(
                    &state.capabilities,
                    state.credit.as_ref(),
                    reservation_id as u64,
                );
                if result.success {
                    HOST_SUCCESS
                } else {
                    HOST_ERROR
                }
            },
        )
        .expect("Failed to register host_credit_release");

    // host_credit_consume: fn(reservation_id: i64) -> i32
    // Consumes a reservation (permanently deducts credits), returns 0 on success, -1 on error
    linker
        .func_wrap(
            "vudo",
            "host_credit_consume",
            |caller: Caller<'_, HostState>, reservation_id: i64| -> i32 {
                if reservation_id < 0 {
                    return HOST_ERROR;
                }
                let state = caller.data();
                let result = host_credit_consume(
                    &state.capabilities,
                    state.credit.as_ref(),
                    reservation_id as u64,
                );
                if result.success {
                    HOST_SUCCESS
                } else {
                    HOST_ERROR
                }
            },
        )
        .expect("Failed to register host_credit_consume");

    // host_credit_available: fn(account_ptr: i32) -> i64
    // Returns available credit balance (total - reserved), or -1 on error
    linker
        .func_wrap(
            "vudo",
            "host_credit_available",
            |mut caller: Caller<'_, HostState>, account_ptr: i32| -> i64 {
                let memory = match get_memory(&mut caller) {
                    Some(m) => m,
                    None => return -1,
                };
                let account_bytes = match read_memory(&caller, &memory, account_ptr, 32) {
                    Some(b) => b,
                    None => return -1,
                };
                let account: [u8; 32] = match account_bytes.try_into() {
                    Ok(a) => a,
                    Err(_) => return -1,
                };
                let state = caller.data();
                let result =
                    host_credit_available(&state.capabilities, state.credit.as_ref(), &account);
                if result.success {
                    if let Some(bytes) = result.return_value {
                        if bytes.len() == 8 {
                            return i64::from_le_bytes(bytes.try_into().unwrap());
                        }
                    }
                }
                -1
            },
        )
        .expect("Failed to register host_credit_available");

    linker
}

// ═══════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::host::{InMemoryCreditLedger, InMemoryStorage, MockNetworkBackend};
    use wasmtime::Config;

    fn create_test_host_state() -> HostState {
        let storage = Arc::new(InMemoryStorage::new());
        let credit = Arc::new(InMemoryCreditLedger::new());
        let network = Arc::new(MockNetworkBackend::new());
        let capabilities = CapabilitySet::new();
        let timeout = Duration::from_secs(30);

        HostState::new(storage, credit, network, capabilities, timeout)
    }

    #[test]
    fn test_host_state_creation() {
        let state = create_test_host_state();

        assert_eq!(state.fuel_consumed, 0);
        assert!(state.start_time.is_none());
        assert!(!state.is_timed_out());
    }

    #[test]
    fn test_host_state_execution_timing() {
        let mut state = create_test_host_state();

        // Before starting, no elapsed time
        assert!(state.elapsed().is_none());

        // Start execution
        state.start_execution();

        // Now we should have elapsed time
        assert!(state.elapsed().is_some());
        assert!(state.elapsed().unwrap() < Duration::from_secs(1));

        // Should not be timed out yet (30 second timeout)
        assert!(!state.is_timed_out());
    }

    #[test]
    fn test_create_linker() {
        let mut config = Config::new();
        config.consume_fuel(true);
        let engine = Engine::new(&config).expect("Failed to create engine");

        let linker = create_linker(&engine);

        // Linker should be created successfully
        // We can't easily test more without a module to instantiate
        drop(linker);
    }

    #[test]
    fn test_linker_with_simple_module() {
        use wasmtime::{Module, Store};

        let mut config = Config::new();
        config.consume_fuel(true);
        let engine = Engine::new(&config).expect("Failed to create engine");

        // Create a simple module with no imports
        let wasm = wat::parse_str(
            r#"
            (module
                (func (export "answer") (result i32)
                    i32.const 42
                )
            )
        "#,
        )
        .expect("Failed to parse WAT");

        let module = Module::new(&engine, &wasm).expect("Failed to compile module");
        let linker = create_linker(&engine);

        let state = create_test_host_state();
        let mut store = Store::new(&engine, state);
        store.set_fuel(1_000_000).expect("Failed to set fuel");

        // Should be able to instantiate the module
        let instance = linker
            .instantiate(&mut store, &module)
            .expect("Failed to instantiate module");

        // Get and call the exported function
        let answer = instance
            .get_typed_func::<(), i32>(&mut store, "answer")
            .expect("Failed to get function");

        let result = answer
            .call(&mut store, ())
            .expect("Failed to call function");
        assert_eq!(result, 42);
    }
}
