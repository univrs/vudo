//! Host Random Functions
//!
//! Provides cryptographically secure random number generation for WASM sandboxes.

use super::{CapabilityScope, CapabilitySet, CapabilityType, HostCallResult};

/// Maximum number of random bytes that can be requested in a single call
const MAX_RANDOM_BYTES: u32 = 1024 * 1024; // 1MB

/// Generate cryptographically secure random bytes
///
/// Requires SensorRandom capability.
///
/// # Arguments
/// * `caps` - Capability set to check permissions
/// * `count` - Number of random bytes to generate (max 1MB)
///
/// # Returns
/// HostCallResult with random bytes or error
pub fn host_random_bytes(caps: &CapabilitySet, count: u32) -> HostCallResult {
    // Check capability
    if !caps.has_capability(CapabilityType::SensorRandom, CapabilityScope::Global) {
        return HostCallResult::capability_denied(CapabilityType::SensorRandom);
    }

    // Validate count
    if count == 0 {
        return HostCallResult::error("Count must be greater than 0");
    }

    if count > MAX_RANDOM_BYTES {
        return HostCallResult::error(format!(
            "Count exceeds maximum of {} bytes",
            MAX_RANDOM_BYTES
        ));
    }

    // Generate random bytes
    let mut bytes = vec![0u8; count as usize];

    // Use getrandom for cryptographically secure randomness
    if let Err(e) = getrandom::getrandom(&mut bytes) {
        return HostCallResult::error(format!("Failed to generate random bytes: {}", e));
    }

    HostCallResult::success_with_value(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::{CapabilityGrant, MINIMAL_CAPABILITIES};
    use std::time::{SystemTime, UNIX_EPOCH};

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
    fn test_host_random_bytes_with_capability() {
        let caps = create_test_capset();
        let result = host_random_bytes(&caps, 32);

        assert!(result.success);
        assert!(result.error.is_none());
        assert!(result.return_value.is_some());

        let bytes = result.return_value.unwrap();
        assert_eq!(bytes.len(), 32);

        // Verify bytes are not all zeros (extremely unlikely with random data)
        assert!(bytes.iter().any(|&b| b != 0));
    }

    #[test]
    fn test_host_random_bytes_without_capability() {
        let caps = CapabilitySet::new();
        let result = host_random_bytes(&caps, 32);

        assert!(!result.success);
        assert!(result.return_value.is_none());
        assert!(result.error.is_some());
        assert!(result.error.unwrap().contains("Capability denied"));
    }

    #[test]
    fn test_host_random_bytes_zero_count() {
        let caps = create_test_capset();
        let result = host_random_bytes(&caps, 0);

        assert!(!result.success);
        assert!(result.error.is_some());
        assert!(result.error.unwrap().contains("greater than 0"));
    }

    #[test]
    fn test_host_random_bytes_exceeds_max() {
        let caps = create_test_capset();
        let result = host_random_bytes(&caps, MAX_RANDOM_BYTES + 1);

        assert!(!result.success);
        assert!(result.error.is_some());
        assert!(result.error.unwrap().contains("exceeds maximum"));
    }

    #[test]
    fn test_host_random_bytes_with_unrestricted() {
        let caps = create_unrestricted_capset();
        let result = host_random_bytes(&caps, 64);

        assert!(result.success);
        assert_eq!(result.return_value.unwrap().len(), 64);
    }

    #[test]
    fn test_randomness_different_calls() {
        let caps = create_test_capset();

        let result1 = host_random_bytes(&caps, 32);
        let result2 = host_random_bytes(&caps, 32);

        assert!(result1.success && result2.success);

        let bytes1 = result1.return_value.unwrap();
        let bytes2 = result2.return_value.unwrap();

        // Two random calls should produce different results
        assert_ne!(bytes1, bytes2);
    }
}
