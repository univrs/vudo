//! Host Interface Module
//!
//! Provides the host functions available to WASM sandboxes.
//! All host functions are capability-gated and return HostCallResult.

pub mod credit;
pub mod log;
pub mod network;
pub mod random;
pub mod storage;
pub mod time;

// Re-export capability types from parent module
pub use crate::capability::{CapabilityGrant, CapabilityScope, CapabilitySet, CapabilityType};

// Re-exports for convenience
pub use credit::{
    host_credit_available, host_credit_balance, host_credit_consume, host_credit_release,
    host_credit_reserve, host_credit_transfer, CreditBackend, InMemoryCreditLedger, PublicKey,
};
pub use log::{host_log, LogLevel};
pub use network::{
    host_network_broadcast, host_network_connect, host_network_listen, ConnectionHandle,
    ListenerHandle, MockNetworkBackend, NetworkBackend,
};
pub use random::host_random_bytes;
pub use storage::{
    host_storage_delete, host_storage_read, host_storage_write, InMemoryStorage, StorageBackend,
};
pub use time::host_time_now;

// ═══════════════════════════════════════════════════════════════════════════
// HOST CALL RESULT
// ═══════════════════════════════════════════════════════════════════════════

/// Result of a host function call
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostCallResult {
    pub success: bool,
    pub return_value: Option<Vec<u8>>,
    pub error: Option<String>,
}

impl HostCallResult {
    /// Create a successful result with no return value
    pub fn success() -> Self {
        Self {
            success: true,
            return_value: None,
            error: None,
        }
    }

    /// Create a successful result with a return value
    pub fn success_with_value(value: Vec<u8>) -> Self {
        Self {
            success: true,
            return_value: Some(value),
            error: None,
        }
    }

    /// Create an error result
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            return_value: None,
            error: Some(message.into()),
        }
    }

    /// Create a capability denied error
    pub fn capability_denied(capability: CapabilityType) -> Self {
        Self::error(format!("Capability denied: {:?}", capability))
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// HOST INTERFACE TRAIT
// ═══════════════════════════════════════════════════════════════════════════

/// Host Interface trait defining the syscall-like API available to WASM
pub trait HostInterface {
    /// Get current time
    fn host_time_now(&self, caps: &CapabilitySet) -> HostCallResult;

    /// Generate random bytes
    fn host_random_bytes(&self, caps: &CapabilitySet, count: u32) -> HostCallResult;

    /// Log a message
    fn host_log(&self, caps: &CapabilitySet, level: LogLevel, message: &str) -> HostCallResult;

    /// Read from storage
    fn host_storage_read(&self, caps: &CapabilitySet, key: &[u8]) -> HostCallResult;

    /// Write to storage
    fn host_storage_write(&self, caps: &CapabilitySet, key: &[u8], value: &[u8]) -> HostCallResult;

    /// Delete from storage
    fn host_storage_delete(&self, caps: &CapabilitySet, key: &[u8]) -> HostCallResult;

    /// Connect to a network address
    fn host_network_connect(&self, caps: &CapabilitySet, address: &str) -> HostCallResult;

    /// Listen on a network port
    fn host_network_listen(&self, caps: &CapabilitySet, port: u16) -> HostCallResult;

    /// Broadcast a message to connected peers
    fn host_network_broadcast(&self, caps: &CapabilitySet, message: &[u8]) -> HostCallResult;

    /// Get credit balance for an account
    fn host_credit_balance(&self, caps: &CapabilitySet, account: &[u8; 32]) -> HostCallResult;

    /// Transfer credits between accounts
    fn host_credit_transfer(
        &self,
        caps: &CapabilitySet,
        from: &[u8; 32],
        to: &[u8; 32],
        amount: u64,
    ) -> HostCallResult;

    /// Reserve credits for a pending operation
    fn host_credit_reserve(
        &self,
        caps: &CapabilitySet,
        account: &[u8; 32],
        amount: u64,
    ) -> HostCallResult;

    /// Release a credit reservation
    fn host_credit_release(&self, caps: &CapabilitySet, reservation_id: u64) -> HostCallResult;

    /// Consume a credit reservation (permanently deduct)
    fn host_credit_consume(&self, caps: &CapabilitySet, reservation_id: u64) -> HostCallResult;

    /// Get available credit balance (total - reserved)
    fn host_credit_available(&self, caps: &CapabilitySet, account: &[u8; 32]) -> HostCallResult;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::MINIMAL_CAPABILITIES;

    fn create_minimal_capset() -> CapabilitySet {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();

        let mut cap_set = CapabilitySet::new();
        for &cap_type in MINIMAL_CAPABILITIES {
            let grant = CapabilityGrant::new(
                1,
                cap_type,
                CapabilityScope::Global,
                [0u8; 32],
                [1u8; 32],
                now,
                None,
                [0u8; 64],
            );
            cap_set.add_grant(grant);
        }
        cap_set
    }

    fn create_unrestricted_capset() -> CapabilitySet {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();

        let mut cap_set = CapabilitySet::new();
        let grant = CapabilityGrant::new(
            1,
            CapabilityType::Unrestricted,
            CapabilityScope::Global,
            [0u8; 32],
            [1u8; 32],
            now,
            None,
            [0u8; 64],
        );
        cap_set.add_grant(grant);
        cap_set
    }

    #[test]
    fn test_host_call_result() {
        let success = HostCallResult::success();
        assert!(success.success);
        assert!(success.error.is_none());

        let error = HostCallResult::error("test error");
        assert!(!error.success);
        assert_eq!(error.error, Some("test error".to_string()));

        let denied = HostCallResult::capability_denied(CapabilityType::StorageRead);
        assert!(!denied.success);
        assert!(denied.error.unwrap().contains("Capability denied"));
    }
}
