//! Host Interface Module
//!
//! Provides the host functions available to WASM sandboxes.
//! All host functions are capability-gated and return HostCallResult.

pub mod time;
pub mod random;
pub mod log;
pub mod storage;

// Re-export capability types from parent module
pub use crate::capability::{CapabilitySet, CapabilityType, CapabilityScope, CapabilityGrant};

// Re-exports for convenience
pub use time::host_time_now;
pub use random::host_random_bytes;
pub use log::{host_log, LogLevel};
pub use storage::{StorageBackend, InMemoryStorage, host_storage_read, host_storage_write, host_storage_delete};

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
