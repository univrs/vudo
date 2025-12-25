//! Host Time Functions
//!
//! Provides time-related host functions for WASM sandboxes.

use super::{CapabilityScope, CapabilitySet, CapabilityType, HostCallResult};
use std::time::{SystemTime, UNIX_EPOCH};

/// Get current Unix timestamp in nanoseconds
///
/// Requires SensorTime capability.
///
/// # Arguments
/// * `caps` - Capability set to check permissions
///
/// # Returns
/// HostCallResult with timestamp as bytes (u64 in little-endian) or error
pub fn host_time_now(caps: &CapabilitySet) -> HostCallResult {
    // Check capability
    if !caps.has_capability(CapabilityType::SensorTime, CapabilityScope::Global) {
        return HostCallResult::capability_denied(CapabilityType::SensorTime);
    }

    // Get current time
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => {
            let nanos = duration.as_nanos() as u64;
            let bytes = nanos.to_le_bytes().to_vec();
            HostCallResult::success_with_value(bytes)
        }
        Err(e) => HostCallResult::error(format!("System time error: {}", e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::{CapabilityGrant, MINIMAL_CAPABILITIES};

    fn create_test_capset() -> CapabilitySet {
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
    fn test_host_time_now_with_capability() {
        let caps = create_test_capset();
        let result = host_time_now(&caps);

        assert!(result.success);
        assert!(result.error.is_none());
        assert!(result.return_value.is_some());

        // Should be 8 bytes (u64)
        let bytes = result.return_value.unwrap();
        assert_eq!(bytes.len(), 8);

        // Convert back to u64 and verify it's a reasonable timestamp
        let timestamp = u64::from_le_bytes(bytes.try_into().unwrap());
        assert!(timestamp > 0);
    }

    #[test]
    fn test_host_time_now_without_capability() {
        let caps = CapabilitySet::new(); // Empty capability set
        let result = host_time_now(&caps);

        assert!(!result.success);
        assert!(result.return_value.is_none());
        assert!(result.error.is_some());
        assert!(result.error.unwrap().contains("Capability denied"));
    }

    #[test]
    fn test_host_time_now_with_unrestricted() {
        let caps = create_unrestricted_capset();
        let result = host_time_now(&caps);

        assert!(result.success);
        assert!(result.return_value.is_some());
    }
}
