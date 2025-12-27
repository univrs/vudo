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
    /// Create a new HostState with the given backends and capabilities.
    ///
    /// # Arguments
    /// * `storage` - Storage backend for read/write/delete operations
    /// * `credit` - Credit ledger for resource accounting
    /// * `network` - Network backend for connection/listen/broadcast operations
    /// * `capabilities` - Capability set defining allowed operations
    /// * `timeout` - Maximum duration allowed for execution
    /// * `account` - The Ed25519 public key identifying this sandbox's account
    pub fn new(
        storage: Arc<dyn StorageBackend>,
        credit: Arc<dyn CreditBackend>,
        network: Arc<dyn NetworkBackend>,
        capabilities: CapabilitySet,
        timeout: Duration,
        account: PublicKey,
    ) -> Self {
        Self {
            storage,
            credit,
            network,
            capabilities,
            fuel_consumed: 0,
            start_time: None,
            timeout,
            account,
            memory: None,
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

    /// Set the WASM memory reference.
    ///
    /// This should be called after module instantiation to enable
    /// host functions that need to read/write WASM linear memory.
    pub fn set_memory(&mut self, memory: Memory) {
        self.memory = Some(memory);
    }

    /// Get a reference to the WASM memory, if set.
    pub fn memory(&self) -> Option<&Memory> {
        self.memory.as_ref()
    }

    /// Get the account (Ed25519 public key) associated with this sandbox.
    pub fn account(&self) -> &PublicKey {
        &self.account
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
    use crate::capability::{CapabilityGrant, CapabilityScope, CapabilityType};
    use crate::host::{InMemoryCreditLedger, InMemoryStorage, MockNetworkBackend};
    use std::time::{SystemTime, UNIX_EPOCH};
    use wasmtime::{Config, Module, Store};

    fn create_test_host_state() -> HostState {
        let storage = Arc::new(InMemoryStorage::new());
        let credit = Arc::new(InMemoryCreditLedger::new());
        let network = Arc::new(MockNetworkBackend::new());
        let capabilities = CapabilitySet::new();
        let timeout = Duration::from_secs(30);
        let account = [0u8; 32]; // Test account (zero key)

        HostState::new(storage, credit, network, capabilities, timeout, account)
    }

    fn create_host_state_with_capabilities(caps: &[CapabilityType]) -> HostState {
        let storage = Arc::new(InMemoryStorage::new());
        let credit = Arc::new(InMemoryCreditLedger::new());
        let network = Arc::new(MockNetworkBackend::new());
        let timeout = Duration::from_secs(30);
        let account = [1u8; 32];

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();

        let mut capabilities = CapabilitySet::new();
        for (i, &cap_type) in caps.iter().enumerate() {
            let grant = CapabilityGrant::new(
                i as u64 + 1,
                cap_type,
                CapabilityScope::Global,
                [0u8; 32],
                [1u8; 32],
                now,
                None,
                [0u8; 64],
            );
            capabilities.add_grant(grant);
        }

        HostState::new(storage, credit, network, capabilities, timeout, account)
    }

    fn create_engine() -> Engine {
        let mut config = Config::new();
        config.consume_fuel(true);
        Engine::new(&config).expect("Failed to create engine")
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ERROR CODES TESTS
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_error_codes_constants() {
        assert_eq!(error_codes::SUCCESS, 0);
        assert_eq!(error_codes::CAPABILITY_DENIED, -1);
        assert_eq!(error_codes::INVALID_MEMORY, -2);
        assert_eq!(error_codes::INVALID_PARAMETER, -3);
        assert_eq!(error_codes::STORAGE_ERROR, -4);
        assert_eq!(error_codes::NETWORK_ERROR, -5);
        assert_eq!(error_codes::CREDIT_ERROR, -6);
        assert_eq!(error_codes::BUFFER_TOO_SMALL, -7);
        assert_eq!(error_codes::INTERNAL_ERROR, -8);
    }

    #[test]
    fn test_host_error_and_success_constants() {
        assert_eq!(HOST_ERROR, -1);
        assert_eq!(HOST_SUCCESS, 0);
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // HOST STATE TESTS
    // ═══════════════════════════════════════════════════════════════════════════

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
    fn test_host_state_timeout_detection() {
        let storage = Arc::new(InMemoryStorage::new());
        let credit = Arc::new(InMemoryCreditLedger::new());
        let network = Arc::new(MockNetworkBackend::new());
        let capabilities = CapabilitySet::new();
        // Set a very short timeout
        let timeout = Duration::from_millis(1);
        let account = [0u8; 32];

        let mut state = HostState::new(storage, credit, network, capabilities, timeout, account);

        // Start execution
        state.start_execution();

        // Wait a bit longer than timeout
        std::thread::sleep(Duration::from_millis(5));

        // Should now be timed out
        assert!(state.is_timed_out());
    }

    #[test]
    fn test_host_state_memory_operations() {
        let engine = create_engine();

        // Create a module with memory
        let wasm = wat::parse_str(
            r#"
            (module
                (memory (export "memory") 1)
            )
        "#,
        )
        .expect("Failed to parse WAT");

        let module = Module::new(&engine, &wasm).expect("Failed to compile module");
        let linker = create_linker(&engine);

        let state = create_test_host_state();

        // Memory should be None initially
        assert!(state.memory().is_none());

        let mut store = Store::new(&engine, state);
        store.set_fuel(1_000_000).expect("Failed to set fuel");

        let instance = linker
            .instantiate(&mut store, &module)
            .expect("Failed to instantiate module");

        // Get and set memory
        let memory = instance
            .get_memory(&mut store, "memory")
            .expect("Failed to get memory");

        store.data_mut().set_memory(memory);

        // Now memory should be Some
        assert!(store.data().memory().is_some());
    }

    #[test]
    fn test_host_state_account() {
        let storage = Arc::new(InMemoryStorage::new());
        let credit = Arc::new(InMemoryCreditLedger::new());
        let network = Arc::new(MockNetworkBackend::new());
        let capabilities = CapabilitySet::new();
        let timeout = Duration::from_secs(30);
        let account = [42u8; 32];

        let state = HostState::new(storage, credit, network, capabilities, timeout, account);

        assert_eq!(state.account(), &[42u8; 32]);
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // LINKER CREATION TESTS
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_create_linker() {
        let engine = create_engine();
        let linker = create_linker(&engine);
        drop(linker);
    }

    #[test]
    fn test_linker_with_simple_module() {
        let engine = create_engine();

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

        let instance = linker
            .instantiate(&mut store, &module)
            .expect("Failed to instantiate module");

        let answer = instance
            .get_typed_func::<(), i32>(&mut store, "answer")
            .expect("Failed to get function");

        let result = answer
            .call(&mut store, ())
            .expect("Failed to call function");
        assert_eq!(result, 42);
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // HOST_TIME_NOW TESTS
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_host_time_now_with_capability() {
        let engine = create_engine();

        let wasm = wat::parse_str(
            r#"
            (module
                (import "vudo" "host_time_now" (func $time_now (result i64)))
                (func (export "get_time") (result i64)
                    call $time_now
                )
            )
        "#,
        )
        .expect("Failed to parse WAT");

        let module = Module::new(&engine, &wasm).expect("Failed to compile module");
        let linker = create_linker(&engine);

        let state = create_host_state_with_capabilities(&[CapabilityType::SensorTime]);
        let mut store = Store::new(&engine, state);
        store.set_fuel(1_000_000).expect("Failed to set fuel");

        let instance = linker
            .instantiate(&mut store, &module)
            .expect("Failed to instantiate module");

        let get_time = instance
            .get_typed_func::<(), i64>(&mut store, "get_time")
            .expect("Failed to get function");

        let result = get_time
            .call(&mut store, ())
            .expect("Failed to call function");

        // Should return a positive timestamp (in nanoseconds)
        assert!(result > 0);
    }

    #[test]
    fn test_host_time_now_without_capability() {
        let engine = create_engine();

        let wasm = wat::parse_str(
            r#"
            (module
                (import "vudo" "host_time_now" (func $time_now (result i64)))
                (func (export "get_time") (result i64)
                    call $time_now
                )
            )
        "#,
        )
        .expect("Failed to parse WAT");

        let module = Module::new(&engine, &wasm).expect("Failed to compile module");
        let linker = create_linker(&engine);

        // No capabilities
        let state = create_test_host_state();
        let mut store = Store::new(&engine, state);
        store.set_fuel(1_000_000).expect("Failed to set fuel");

        let instance = linker
            .instantiate(&mut store, &module)
            .expect("Failed to instantiate module");

        let get_time = instance
            .get_typed_func::<(), i64>(&mut store, "get_time")
            .expect("Failed to get function");

        let result = get_time
            .call(&mut store, ())
            .expect("Failed to call function");

        // Should return -1 (error) without capability
        assert_eq!(result, -1);
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // HOST_RANDOM_BYTES TESTS
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_host_random_bytes_with_capability() {
        let engine = create_engine();

        let wasm = wat::parse_str(
            r#"
            (module
                (import "vudo" "host_random_bytes" (func $random (param i32 i32) (result i32)))
                (memory (export "memory") 1)
                (func (export "get_random") (result i32)
                    ;; Request 8 random bytes at memory offset 0
                    i32.const 0
                    i32.const 8
                    call $random
                )
                (func (export "read_byte") (param i32) (result i32)
                    local.get 0
                    i32.load8_u
                )
            )
        "#,
        )
        .expect("Failed to parse WAT");

        let module = Module::new(&engine, &wasm).expect("Failed to compile module");
        let linker = create_linker(&engine);

        let state = create_host_state_with_capabilities(&[CapabilityType::SensorRandom]);
        let mut store = Store::new(&engine, state);
        store.set_fuel(1_000_000).expect("Failed to set fuel");

        let instance = linker
            .instantiate(&mut store, &module)
            .expect("Failed to instantiate module");

        let get_random = instance
            .get_typed_func::<(), i32>(&mut store, "get_random")
            .expect("Failed to get function");

        let result = get_random
            .call(&mut store, ())
            .expect("Failed to call function");

        // Should return 0 (success)
        assert_eq!(result, HOST_SUCCESS);
    }

    #[test]
    fn test_host_random_bytes_zero_length() {
        let engine = create_engine();

        let wasm = wat::parse_str(
            r#"
            (module
                (import "vudo" "host_random_bytes" (func $random (param i32 i32) (result i32)))
                (memory (export "memory") 1)
                (func (export "get_random_zero") (result i32)
                    i32.const 0
                    i32.const 0
                    call $random
                )
            )
        "#,
        )
        .expect("Failed to parse WAT");

        let module = Module::new(&engine, &wasm).expect("Failed to compile module");
        let linker = create_linker(&engine);

        let state = create_host_state_with_capabilities(&[CapabilityType::SensorRandom]);
        let mut store = Store::new(&engine, state);
        store.set_fuel(1_000_000).expect("Failed to set fuel");

        let instance = linker
            .instantiate(&mut store, &module)
            .expect("Failed to instantiate module");

        let get_random = instance
            .get_typed_func::<(), i32>(&mut store, "get_random_zero")
            .expect("Failed to get function");

        let result = get_random
            .call(&mut store, ())
            .expect("Failed to call function");

        // Should return -1 (error) for zero/negative length
        assert_eq!(result, HOST_ERROR);
    }

    #[test]
    fn test_host_random_bytes_without_capability() {
        let engine = create_engine();

        let wasm = wat::parse_str(
            r#"
            (module
                (import "vudo" "host_random_bytes" (func $random (param i32 i32) (result i32)))
                (memory (export "memory") 1)
                (func (export "get_random") (result i32)
                    i32.const 0
                    i32.const 8
                    call $random
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

        let instance = linker
            .instantiate(&mut store, &module)
            .expect("Failed to instantiate module");

        let get_random = instance
            .get_typed_func::<(), i32>(&mut store, "get_random")
            .expect("Failed to get function");

        let result = get_random
            .call(&mut store, ())
            .expect("Failed to call function");

        // Should return -1 (error) without capability
        assert_eq!(result, HOST_ERROR);
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // HOST_LOG TESTS
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_host_log_with_capability() {
        let engine = create_engine();

        let wasm = wat::parse_str(
            r#"
            (module
                (import "vudo" "host_log" (func $log (param i32 i32 i32) (result i32)))
                (memory (export "memory") 1)
                (data (i32.const 0) "Hello, VUDO!")
                (func (export "log_message") (result i32)
                    ;; Log level 1 (INFO), message at offset 0, length 12
                    i32.const 1
                    i32.const 0
                    i32.const 12
                    call $log
                )
            )
        "#,
        )
        .expect("Failed to parse WAT");

        let module = Module::new(&engine, &wasm).expect("Failed to compile module");
        let linker = create_linker(&engine);

        let state = create_host_state_with_capabilities(&[CapabilityType::ActuatorLog]);
        let mut store = Store::new(&engine, state);
        store.set_fuel(1_000_000).expect("Failed to set fuel");

        let instance = linker
            .instantiate(&mut store, &module)
            .expect("Failed to instantiate module");

        let log_message = instance
            .get_typed_func::<(), i32>(&mut store, "log_message")
            .expect("Failed to get function");

        let result = log_message
            .call(&mut store, ())
            .expect("Failed to call function");

        // Should return 0 (success)
        assert_eq!(result, HOST_SUCCESS);
    }

    #[test]
    fn test_host_log_invalid_level() {
        let engine = create_engine();

        let wasm = wat::parse_str(
            r#"
            (module
                (import "vudo" "host_log" (func $log (param i32 i32 i32) (result i32)))
                (memory (export "memory") 1)
                (data (i32.const 0) "Test message")
                (func (export "log_invalid") (result i32)
                    ;; Invalid log level 255
                    i32.const 255
                    i32.const 0
                    i32.const 12
                    call $log
                )
            )
        "#,
        )
        .expect("Failed to parse WAT");

        let module = Module::new(&engine, &wasm).expect("Failed to compile module");
        let linker = create_linker(&engine);

        let state = create_host_state_with_capabilities(&[CapabilityType::ActuatorLog]);
        let mut store = Store::new(&engine, state);
        store.set_fuel(1_000_000).expect("Failed to set fuel");

        let instance = linker
            .instantiate(&mut store, &module)
            .expect("Failed to instantiate module");

        let log_invalid = instance
            .get_typed_func::<(), i32>(&mut store, "log_invalid")
            .expect("Failed to get function");

        let result = log_invalid
            .call(&mut store, ())
            .expect("Failed to call function");

        // Should return -1 (error) for invalid log level
        assert_eq!(result, HOST_ERROR);
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // STORAGE FUNCTION TESTS
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_host_storage_write_and_read() {
        let engine = create_engine();

        let wasm = wat::parse_str(
            r#"
            (module
                (import "vudo" "host_storage_write" (func $write (param i32 i32 i32 i32) (result i32)))
                (import "vudo" "host_storage_read" (func $read (param i32 i32 i32 i32) (result i32)))
                (memory (export "memory") 1)
                ;; Key "test" at offset 0
                (data (i32.const 0) "test")
                ;; Value "hello" at offset 16
                (data (i32.const 16) "hello")
                ;; Read buffer at offset 32

                (func (export "write_value") (result i32)
                    ;; key_ptr=0, key_len=4, val_ptr=16, val_len=5
                    i32.const 0
                    i32.const 4
                    i32.const 16
                    i32.const 5
                    call $write
                )
                (func (export "read_value") (result i32)
                    ;; key_ptr=0, key_len=4, val_ptr=32, val_cap=64
                    i32.const 0
                    i32.const 4
                    i32.const 32
                    i32.const 64
                    call $read
                )
            )
        "#,
        )
        .expect("Failed to parse WAT");

        let module = Module::new(&engine, &wasm).expect("Failed to compile module");
        let linker = create_linker(&engine);

        let state = create_host_state_with_capabilities(&[
            CapabilityType::StorageRead,
            CapabilityType::StorageWrite,
        ]);
        let mut store = Store::new(&engine, state);
        store.set_fuel(1_000_000).expect("Failed to set fuel");

        let instance = linker
            .instantiate(&mut store, &module)
            .expect("Failed to instantiate module");

        // Write the value
        let write_value = instance
            .get_typed_func::<(), i32>(&mut store, "write_value")
            .expect("Failed to get function");

        let write_result = write_value
            .call(&mut store, ())
            .expect("Failed to call function");
        assert_eq!(write_result, HOST_SUCCESS);

        // Read it back
        let read_value = instance
            .get_typed_func::<(), i32>(&mut store, "read_value")
            .expect("Failed to get function");

        let read_result = read_value
            .call(&mut store, ())
            .expect("Failed to call function");

        // Should return the number of bytes read (5)
        assert_eq!(read_result, 5);
    }

    #[test]
    fn test_host_storage_delete() {
        let engine = create_engine();

        let wasm = wat::parse_str(
            r#"
            (module
                (import "vudo" "host_storage_write" (func $write (param i32 i32 i32 i32) (result i32)))
                (import "vudo" "host_storage_delete" (func $delete (param i32 i32) (result i32)))
                (import "vudo" "host_storage_read" (func $read (param i32 i32 i32 i32) (result i32)))
                (memory (export "memory") 1)
                (data (i32.const 0) "key1")
                (data (i32.const 16) "value1")

                (func (export "write_then_delete") (result i32)
                    ;; Write first
                    i32.const 0
                    i32.const 4
                    i32.const 16
                    i32.const 6
                    call $write
                    drop

                    ;; Delete
                    i32.const 0
                    i32.const 4
                    call $delete
                )
                (func (export "read_deleted") (result i32)
                    i32.const 0
                    i32.const 4
                    i32.const 32
                    i32.const 64
                    call $read
                )
            )
        "#,
        )
        .expect("Failed to parse WAT");

        let module = Module::new(&engine, &wasm).expect("Failed to compile module");
        let linker = create_linker(&engine);

        let state = create_host_state_with_capabilities(&[
            CapabilityType::StorageRead,
            CapabilityType::StorageWrite,
            CapabilityType::StorageDelete,
        ]);
        let mut store = Store::new(&engine, state);
        store.set_fuel(1_000_000).expect("Failed to set fuel");

        let instance = linker
            .instantiate(&mut store, &module)
            .expect("Failed to instantiate module");

        // Write then delete
        let write_delete = instance
            .get_typed_func::<(), i32>(&mut store, "write_then_delete")
            .expect("Failed to get function");

        let result = write_delete
            .call(&mut store, ())
            .expect("Failed to call function");

        // Delete should return 1 (key was deleted)
        assert_eq!(result, 1);

        // Now try to read - should return 0 (no value)
        let read_deleted = instance
            .get_typed_func::<(), i32>(&mut store, "read_deleted")
            .expect("Failed to get function");

        let read_result = read_deleted
            .call(&mut store, ())
            .expect("Failed to call function");
        assert_eq!(read_result, 0);
    }

    #[test]
    fn test_host_storage_without_capability() {
        let engine = create_engine();

        let wasm = wat::parse_str(
            r#"
            (module
                (import "vudo" "host_storage_read" (func $read (param i32 i32 i32 i32) (result i32)))
                (memory (export "memory") 1)
                (func (export "try_read") (result i32)
                    i32.const 0
                    i32.const 4
                    i32.const 16
                    i32.const 64
                    call $read
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

        let instance = linker
            .instantiate(&mut store, &module)
            .expect("Failed to instantiate module");

        let try_read = instance
            .get_typed_func::<(), i32>(&mut store, "try_read")
            .expect("Failed to get function");

        let result = try_read
            .call(&mut store, ())
            .expect("Failed to call function");

        // Should return -1 (error) without capability
        assert_eq!(result, HOST_ERROR);
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // NETWORK FUNCTION TESTS
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_host_network_connect() {
        let engine = create_engine();

        let wasm = wat::parse_str(
            r#"
            (module
                (import "vudo" "host_network_connect" (func $connect (param i32 i32) (result i64)))
                (memory (export "memory") 1)
                (data (i32.const 0) "127.0.0.1:8080")

                (func (export "connect") (result i64)
                    i32.const 0
                    i32.const 14
                    call $connect
                )
            )
        "#,
        )
        .expect("Failed to parse WAT");

        let module = Module::new(&engine, &wasm).expect("Failed to compile module");
        let linker = create_linker(&engine);

        let state = create_host_state_with_capabilities(&[CapabilityType::NetworkConnect]);
        let mut store = Store::new(&engine, state);
        store.set_fuel(1_000_000).expect("Failed to set fuel");

        let instance = linker
            .instantiate(&mut store, &module)
            .expect("Failed to instantiate module");

        let connect = instance
            .get_typed_func::<(), i64>(&mut store, "connect")
            .expect("Failed to get function");

        let result = connect
            .call(&mut store, ())
            .expect("Failed to call function");

        // MockNetworkBackend returns a connection handle >= 0
        assert!(result >= 0);
    }

    #[test]
    fn test_host_network_listen() {
        let engine = create_engine();

        let wasm = wat::parse_str(
            r#"
            (module
                (import "vudo" "host_network_listen" (func $listen (param i32) (result i64)))

                (func (export "listen") (result i64)
                    i32.const 8080
                    call $listen
                )
            )
        "#,
        )
        .expect("Failed to parse WAT");

        let module = Module::new(&engine, &wasm).expect("Failed to compile module");
        let linker = create_linker(&engine);

        let state = create_host_state_with_capabilities(&[CapabilityType::NetworkListen]);
        let mut store = Store::new(&engine, state);
        store.set_fuel(1_000_000).expect("Failed to set fuel");

        let instance = linker
            .instantiate(&mut store, &module)
            .expect("Failed to instantiate module");

        let listen = instance
            .get_typed_func::<(), i64>(&mut store, "listen")
            .expect("Failed to get function");

        let result = listen
            .call(&mut store, ())
            .expect("Failed to call function");

        // MockNetworkBackend returns a listener handle >= 0
        assert!(result >= 0);
    }

    #[test]
    fn test_host_network_listen_invalid_port() {
        let engine = create_engine();

        let wasm = wat::parse_str(
            r#"
            (module
                (import "vudo" "host_network_listen" (func $listen (param i32) (result i64)))

                (func (export "listen_invalid") (result i64)
                    i32.const 70000  ;; Invalid port > 65535
                    call $listen
                )
            )
        "#,
        )
        .expect("Failed to parse WAT");

        let module = Module::new(&engine, &wasm).expect("Failed to compile module");
        let linker = create_linker(&engine);

        let state = create_host_state_with_capabilities(&[CapabilityType::NetworkListen]);
        let mut store = Store::new(&engine, state);
        store.set_fuel(1_000_000).expect("Failed to set fuel");

        let instance = linker
            .instantiate(&mut store, &module)
            .expect("Failed to instantiate module");

        let listen_invalid = instance
            .get_typed_func::<(), i64>(&mut store, "listen_invalid")
            .expect("Failed to get function");

        let result = listen_invalid
            .call(&mut store, ())
            .expect("Failed to call function");

        // Should return -1 for invalid port
        assert_eq!(result, -1);
    }

    #[test]
    fn test_host_network_broadcast() {
        let engine = create_engine();

        let wasm = wat::parse_str(
            r#"
            (module
                (import "vudo" "host_network_broadcast" (func $broadcast (param i32 i32) (result i64)))
                (memory (export "memory") 1)
                (data (i32.const 0) "broadcast message")

                (func (export "broadcast") (result i64)
                    i32.const 0
                    i32.const 17
                    call $broadcast
                )
            )
        "#,
        )
        .expect("Failed to parse WAT");

        let module = Module::new(&engine, &wasm).expect("Failed to compile module");
        let linker = create_linker(&engine);

        let state = create_host_state_with_capabilities(&[CapabilityType::NetworkBroadcast]);
        let mut store = Store::new(&engine, state);
        store.set_fuel(1_000_000).expect("Failed to set fuel");

        let instance = linker
            .instantiate(&mut store, &module)
            .expect("Failed to instantiate module");

        let broadcast = instance
            .get_typed_func::<(), i64>(&mut store, "broadcast")
            .expect("Failed to get function");

        let result = broadcast
            .call(&mut store, ())
            .expect("Failed to call function");

        // MockNetworkBackend returns peer count >= 0
        assert!(result >= 0);
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // CREDIT FUNCTION TESTS
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_host_credit_balance() {
        let engine = create_engine();

        let wasm = wat::parse_str(
            r#"
            (module
                (import "vudo" "host_credit_balance" (func $balance (param i32) (result i64)))
                (memory (export "memory") 1)
                ;; 32-byte account key at offset 0
                (data (i32.const 0) "\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01")

                (func (export "get_balance") (result i64)
                    i32.const 0
                    call $balance
                )
            )
        "#,
        )
        .expect("Failed to parse WAT");

        let module = Module::new(&engine, &wasm).expect("Failed to compile module");
        let linker = create_linker(&engine);

        let state = create_host_state_with_capabilities(&[CapabilityType::ActuatorCredit]);
        let mut store = Store::new(&engine, state);
        store.set_fuel(1_000_000).expect("Failed to set fuel");

        let instance = linker
            .instantiate(&mut store, &module)
            .expect("Failed to instantiate module");

        let get_balance = instance
            .get_typed_func::<(), i64>(&mut store, "get_balance")
            .expect("Failed to get function");

        let result = get_balance
            .call(&mut store, ())
            .expect("Failed to call function");

        // InMemoryCreditLedger returns 0 balance for unknown accounts
        assert!(result >= 0);
    }

    #[test]
    fn test_host_credit_transfer() {
        let engine = create_engine();

        let wasm = wat::parse_str(
            r#"
            (module
                (import "vudo" "host_credit_transfer" (func $transfer (param i32 i32 i64) (result i32)))
                (memory (export "memory") 1)
                ;; From account at offset 0
                (data (i32.const 0) "\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01")
                ;; To account at offset 32
                (data (i32.const 32) "\02\02\02\02\02\02\02\02\02\02\02\02\02\02\02\02\02\02\02\02\02\02\02\02\02\02\02\02\02\02\02\02")

                (func (export "transfer") (result i32)
                    i32.const 0   ;; from_ptr
                    i32.const 32  ;; to_ptr
                    i64.const 100 ;; amount
                    call $transfer
                )
            )
        "#,
        )
        .expect("Failed to parse WAT");

        let module = Module::new(&engine, &wasm).expect("Failed to compile module");
        let linker = create_linker(&engine);

        let state = create_host_state_with_capabilities(&[CapabilityType::ActuatorCredit]);
        let mut store = Store::new(&engine, state);
        store.set_fuel(1_000_000).expect("Failed to set fuel");

        let instance = linker
            .instantiate(&mut store, &module)
            .expect("Failed to instantiate module");

        let transfer = instance
            .get_typed_func::<(), i32>(&mut store, "transfer")
            .expect("Failed to get function");

        let result = transfer
            .call(&mut store, ())
            .expect("Failed to call function");

        // Transfer may succeed or fail based on balance, but shouldn't crash
        assert!(result == HOST_SUCCESS || result == HOST_ERROR);
    }

    #[test]
    fn test_host_credit_transfer_negative_amount() {
        let engine = create_engine();

        let wasm = wat::parse_str(
            r#"
            (module
                (import "vudo" "host_credit_transfer" (func $transfer (param i32 i32 i64) (result i32)))
                (memory (export "memory") 1)
                (data (i32.const 0) "\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01")
                (data (i32.const 32) "\02\02\02\02\02\02\02\02\02\02\02\02\02\02\02\02\02\02\02\02\02\02\02\02\02\02\02\02\02\02\02\02")

                (func (export "transfer_negative") (result i32)
                    i32.const 0
                    i32.const 32
                    i64.const -100  ;; Negative amount
                    call $transfer
                )
            )
        "#,
        )
        .expect("Failed to parse WAT");

        let module = Module::new(&engine, &wasm).expect("Failed to compile module");
        let linker = create_linker(&engine);

        let state = create_host_state_with_capabilities(&[CapabilityType::ActuatorCredit]);
        let mut store = Store::new(&engine, state);
        store.set_fuel(1_000_000).expect("Failed to set fuel");

        let instance = linker
            .instantiate(&mut store, &module)
            .expect("Failed to instantiate module");

        let transfer_negative = instance
            .get_typed_func::<(), i32>(&mut store, "transfer_negative")
            .expect("Failed to get function");

        let result = transfer_negative
            .call(&mut store, ())
            .expect("Failed to call function");

        // Should return -1 for negative amount
        assert_eq!(result, HOST_ERROR);
    }

    #[test]
    fn test_host_credit_reserve_and_release() {
        let engine = create_engine();

        let wasm = wat::parse_str(
            r#"
            (module
                (import "vudo" "host_credit_reserve" (func $reserve (param i32 i64) (result i64)))
                (import "vudo" "host_credit_release" (func $release (param i64) (result i32)))
                (memory (export "memory") 1)
                (data (i32.const 0) "\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01")

                (global $reservation_id (mut i64) (i64.const 0))

                (func (export "reserve") (result i64)
                    i32.const 0
                    i64.const 50
                    call $reserve
                )
                (func (export "release") (param i64) (result i32)
                    local.get 0
                    call $release
                )
            )
        "#,
        )
        .expect("Failed to parse WAT");

        let module = Module::new(&engine, &wasm).expect("Failed to compile module");
        let linker = create_linker(&engine);

        let state = create_host_state_with_capabilities(&[CapabilityType::ActuatorCredit]);
        let mut store = Store::new(&engine, state);
        store.set_fuel(1_000_000).expect("Failed to set fuel");

        let instance = linker
            .instantiate(&mut store, &module)
            .expect("Failed to instantiate module");

        // Reserve
        let reserve = instance
            .get_typed_func::<(), i64>(&mut store, "reserve")
            .expect("Failed to get function");

        let reservation_id = reserve
            .call(&mut store, ())
            .expect("Failed to call function");

        // May succeed or fail based on ledger state
        if reservation_id >= 0 {
            // Try to release
            let release = instance
                .get_typed_func::<i64, i32>(&mut store, "release")
                .expect("Failed to get function");

            let result = release
                .call(&mut store, reservation_id)
                .expect("Failed to call function");

            assert!(result == HOST_SUCCESS || result == HOST_ERROR);
        }
    }

    #[test]
    fn test_host_credit_reserve_zero_amount() {
        let engine = create_engine();

        let wasm = wat::parse_str(
            r#"
            (module
                (import "vudo" "host_credit_reserve" (func $reserve (param i32 i64) (result i64)))
                (memory (export "memory") 1)
                (data (i32.const 0) "\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01")

                (func (export "reserve_zero") (result i64)
                    i32.const 0
                    i64.const 0  ;; Zero amount
                    call $reserve
                )
            )
        "#,
        )
        .expect("Failed to parse WAT");

        let module = Module::new(&engine, &wasm).expect("Failed to compile module");
        let linker = create_linker(&engine);

        let state = create_host_state_with_capabilities(&[CapabilityType::ActuatorCredit]);
        let mut store = Store::new(&engine, state);
        store.set_fuel(1_000_000).expect("Failed to set fuel");

        let instance = linker
            .instantiate(&mut store, &module)
            .expect("Failed to instantiate module");

        let reserve_zero = instance
            .get_typed_func::<(), i64>(&mut store, "reserve_zero")
            .expect("Failed to get function");

        let result = reserve_zero
            .call(&mut store, ())
            .expect("Failed to call function");

        // Should return -1 for zero/negative amount
        assert_eq!(result, -1);
    }

    #[test]
    fn test_host_credit_consume() {
        let engine = create_engine();

        let wasm = wat::parse_str(
            r#"
            (module
                (import "vudo" "host_credit_consume" (func $consume (param i64) (result i32)))

                (func (export "consume") (param i64) (result i32)
                    local.get 0
                    call $consume
                )
                (func (export "consume_invalid") (result i32)
                    i64.const -1
                    call $consume
                )
            )
        "#,
        )
        .expect("Failed to parse WAT");

        let module = Module::new(&engine, &wasm).expect("Failed to compile module");
        let linker = create_linker(&engine);

        let state = create_host_state_with_capabilities(&[CapabilityType::ActuatorCredit]);
        let mut store = Store::new(&engine, state);
        store.set_fuel(1_000_000).expect("Failed to set fuel");

        let instance = linker
            .instantiate(&mut store, &module)
            .expect("Failed to instantiate module");

        // Try to consume with invalid reservation ID
        let consume_invalid = instance
            .get_typed_func::<(), i32>(&mut store, "consume_invalid")
            .expect("Failed to get function");

        let result = consume_invalid
            .call(&mut store, ())
            .expect("Failed to call function");

        // Should return -1 for negative reservation ID
        assert_eq!(result, HOST_ERROR);
    }

    #[test]
    fn test_host_credit_available() {
        let engine = create_engine();

        let wasm = wat::parse_str(
            r#"
            (module
                (import "vudo" "host_credit_available" (func $available (param i32) (result i64)))
                (memory (export "memory") 1)
                (data (i32.const 0) "\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01\01")

                (func (export "get_available") (result i64)
                    i32.const 0
                    call $available
                )
            )
        "#,
        )
        .expect("Failed to parse WAT");

        let module = Module::new(&engine, &wasm).expect("Failed to compile module");
        let linker = create_linker(&engine);

        let state = create_host_state_with_capabilities(&[CapabilityType::ActuatorCredit]);
        let mut store = Store::new(&engine, state);
        store.set_fuel(1_000_000).expect("Failed to set fuel");

        let instance = linker
            .instantiate(&mut store, &module)
            .expect("Failed to instantiate module");

        let get_available = instance
            .get_typed_func::<(), i64>(&mut store, "get_available")
            .expect("Failed to get function");

        let result = get_available
            .call(&mut store, ())
            .expect("Failed to call function");

        // Should return available balance >= 0
        assert!(result >= 0);
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // UNRESTRICTED CAPABILITY TESTS
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_unrestricted_capability_grants_all() {
        let engine = create_engine();

        let wasm = wat::parse_str(
            r#"
            (module
                (import "vudo" "host_time_now" (func $time_now (result i64)))
                (import "vudo" "host_random_bytes" (func $random (param i32 i32) (result i32)))
                (import "vudo" "host_log" (func $log (param i32 i32 i32) (result i32)))
                (memory (export "memory") 1)
                (data (i32.const 0) "Test log")

                (func (export "test_time") (result i64)
                    call $time_now
                )
                (func (export "test_random") (result i32)
                    i32.const 16
                    i32.const 8
                    call $random
                )
                (func (export "test_log") (result i32)
                    i32.const 1
                    i32.const 0
                    i32.const 8
                    call $log
                )
            )
        "#,
        )
        .expect("Failed to parse WAT");

        let module = Module::new(&engine, &wasm).expect("Failed to compile module");
        let linker = create_linker(&engine);

        // Only Unrestricted capability
        let state = create_host_state_with_capabilities(&[CapabilityType::Unrestricted]);
        let mut store = Store::new(&engine, state);
        store.set_fuel(1_000_000).expect("Failed to set fuel");

        let instance = linker
            .instantiate(&mut store, &module)
            .expect("Failed to instantiate module");

        // All functions should work with Unrestricted
        let test_time = instance
            .get_typed_func::<(), i64>(&mut store, "test_time")
            .expect("Failed to get function");
        let time_result = test_time.call(&mut store, ()).expect("Failed to call");
        assert!(time_result > 0);

        let test_random = instance
            .get_typed_func::<(), i32>(&mut store, "test_random")
            .expect("Failed to get function");
        let random_result = test_random.call(&mut store, ()).expect("Failed to call");
        assert_eq!(random_result, HOST_SUCCESS);

        let test_log = instance
            .get_typed_func::<(), i32>(&mut store, "test_log")
            .expect("Failed to get function");
        let log_result = test_log.call(&mut store, ()).expect("Failed to call");
        assert_eq!(log_result, HOST_SUCCESS);
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // MEMORY BOUNDARY TESTS
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_memory_read_out_of_bounds() {
        let engine = create_engine();

        let wasm = wat::parse_str(
            r#"
            (module
                (import "vudo" "host_storage_read" (func $read (param i32 i32 i32 i32) (result i32)))
                (memory (export "memory") 1)  ;; 1 page = 64KB

                (func (export "read_oob") (result i32)
                    ;; Try to read from beyond memory
                    i32.const 65536  ;; At the memory boundary
                    i32.const 10
                    i32.const 0
                    i32.const 64
                    call $read
                )
            )
        "#,
        )
        .expect("Failed to parse WAT");

        let module = Module::new(&engine, &wasm).expect("Failed to compile module");
        let linker = create_linker(&engine);

        let state = create_host_state_with_capabilities(&[CapabilityType::StorageRead]);
        let mut store = Store::new(&engine, state);
        store.set_fuel(1_000_000).expect("Failed to set fuel");

        let instance = linker
            .instantiate(&mut store, &module)
            .expect("Failed to instantiate module");

        let read_oob = instance
            .get_typed_func::<(), i32>(&mut store, "read_oob")
            .expect("Failed to get function");

        let result = read_oob
            .call(&mut store, ())
            .expect("Failed to call function");

        // Should return -1 for out of bounds read
        assert_eq!(result, HOST_ERROR);
    }

    #[test]
    fn test_memory_write_out_of_bounds() {
        let engine = create_engine();

        let wasm = wat::parse_str(
            r#"
            (module
                (import "vudo" "host_random_bytes" (func $random (param i32 i32) (result i32)))
                (memory (export "memory") 1)  ;; 1 page = 64KB

                (func (export "write_oob") (result i32)
                    ;; Try to write beyond memory
                    i32.const 65530
                    i32.const 100  ;; Would overflow past 64KB
                    call $random
                )
            )
        "#,
        )
        .expect("Failed to parse WAT");

        let module = Module::new(&engine, &wasm).expect("Failed to compile module");
        let linker = create_linker(&engine);

        let state = create_host_state_with_capabilities(&[CapabilityType::SensorRandom]);
        let mut store = Store::new(&engine, state);
        store.set_fuel(1_000_000).expect("Failed to set fuel");

        let instance = linker
            .instantiate(&mut store, &module)
            .expect("Failed to instantiate module");

        let write_oob = instance
            .get_typed_func::<(), i32>(&mut store, "write_oob")
            .expect("Failed to get function");

        let result = write_oob
            .call(&mut store, ())
            .expect("Failed to call function");

        // Should return -1 for out of bounds write
        assert_eq!(result, HOST_ERROR);
    }

    #[test]
    fn test_negative_pointer() {
        let engine = create_engine();

        let wasm = wat::parse_str(
            r#"
            (module
                (import "vudo" "host_log" (func $log (param i32 i32 i32) (result i32)))
                (memory (export "memory") 1)

                (func (export "log_negative_ptr") (result i32)
                    i32.const 1
                    i32.const -1  ;; Negative pointer
                    i32.const 10
                    call $log
                )
            )
        "#,
        )
        .expect("Failed to parse WAT");

        let module = Module::new(&engine, &wasm).expect("Failed to compile module");
        let linker = create_linker(&engine);

        let state = create_host_state_with_capabilities(&[CapabilityType::ActuatorLog]);
        let mut store = Store::new(&engine, state);
        store.set_fuel(1_000_000).expect("Failed to set fuel");

        let instance = linker
            .instantiate(&mut store, &module)
            .expect("Failed to instantiate module");

        let log_negative = instance
            .get_typed_func::<(), i32>(&mut store, "log_negative_ptr")
            .expect("Failed to get function");

        let result = log_negative
            .call(&mut store, ())
            .expect("Failed to call function");

        // Should return -1 for negative pointer
        assert_eq!(result, HOST_ERROR);
    }
}
