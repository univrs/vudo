//! Host Storage Functions
//!
//! Provides persistent storage capabilities for WASM sandboxes.

use super::{CapabilitySet, CapabilityType, CapabilityScope, HostCallResult};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Maximum key size in bytes
const MAX_KEY_SIZE: usize = 1024; // 1KB

/// Maximum value size in bytes
const MAX_VALUE_SIZE: usize = 10 * 1024 * 1024; // 10MB

/// Storage backend trait
///
/// Implementations provide the actual storage mechanism (in-memory, disk, database, etc.)
pub trait StorageBackend: Send + Sync {
    /// Read a value by key
    ///
    /// Returns:
    /// - Ok(Some(value)) if key exists
    /// - Ok(None) if key doesn't exist
    /// - Err(msg) on storage error
    fn read(&self, key: &[u8]) -> Result<Option<Vec<u8>>, String>;

    /// Write a key-value pair
    ///
    /// Overwrites existing value if key already exists.
    fn write(&self, key: &[u8], value: &[u8]) -> Result<(), String>;

    /// Delete a key-value pair
    ///
    /// Returns:
    /// - Ok(true) if key existed and was deleted
    /// - Ok(false) if key didn't exist
    /// - Err(msg) on storage error
    fn delete(&self, key: &[u8]) -> Result<bool, String>;

    /// Get number of stored key-value pairs
    fn count(&self) -> Result<usize, String>;

    /// Clear all stored data
    fn clear(&self) -> Result<(), String>;
}

/// In-memory storage implementation
///
/// This is a simple HashMap-based storage for testing and development.
/// For production, use a persistent storage backend.
#[derive(Debug, Clone)]
pub struct InMemoryStorage {
    data: Arc<RwLock<HashMap<Vec<u8>, Vec<u8>>>>,
}

impl InMemoryStorage {
    /// Create a new in-memory storage
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl StorageBackend for InMemoryStorage {
    fn read(&self, key: &[u8]) -> Result<Option<Vec<u8>>, String> {
        let data = self.data.read().map_err(|e| format!("Lock error: {}", e))?;
        Ok(data.get(key).cloned())
    }

    fn write(&self, key: &[u8], value: &[u8]) -> Result<(), String> {
        let mut data = self.data.write().map_err(|e| format!("Lock error: {}", e))?;
        data.insert(key.to_vec(), value.to_vec());
        Ok(())
    }

    fn delete(&self, key: &[u8]) -> Result<bool, String> {
        let mut data = self.data.write().map_err(|e| format!("Lock error: {}", e))?;
        Ok(data.remove(key).is_some())
    }

    fn count(&self) -> Result<usize, String> {
        let data = self.data.read().map_err(|e| format!("Lock error: {}", e))?;
        Ok(data.len())
    }

    fn clear(&self) -> Result<(), String> {
        let mut data = self.data.write().map_err(|e| format!("Lock error: {}", e))?;
        data.clear();
        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// HOST STORAGE FUNCTIONS
// ═══════════════════════════════════════════════════════════════════════════

/// Read from storage
///
/// Requires StorageRead capability.
///
/// # Arguments
/// * `caps` - Capability set to check permissions
/// * `storage` - Storage backend to read from
/// * `key` - Key to read
///
/// # Returns
/// HostCallResult with value bytes if found, empty if not found, or error
pub fn host_storage_read(
    caps: &CapabilitySet,
    storage: &dyn StorageBackend,
    key: &[u8],
) -> HostCallResult {
    // Check capability
    if !caps.has_capability(CapabilityType::StorageRead, CapabilityScope::Sandboxed) {
        return HostCallResult::capability_denied(CapabilityType::StorageRead);
    }

    // Validate key size
    if key.is_empty() {
        return HostCallResult::error("Key cannot be empty");
    }

    if key.len() > MAX_KEY_SIZE {
        return HostCallResult::error(format!(
            "Key size exceeds maximum of {} bytes",
            MAX_KEY_SIZE
        ));
    }

    // Read from storage
    match storage.read(key) {
        Ok(Some(value)) => HostCallResult::success_with_value(value),
        Ok(None) => HostCallResult::success(), // Key not found
        Err(e) => HostCallResult::error(format!("Storage read error: {}", e)),
    }
}

/// Write to storage
///
/// Requires StorageWrite capability.
///
/// # Arguments
/// * `caps` - Capability set to check permissions
/// * `storage` - Storage backend to write to
/// * `key` - Key to write
/// * `value` - Value to write
///
/// # Returns
/// HostCallResult indicating success or error
pub fn host_storage_write(
    caps: &CapabilitySet,
    storage: &dyn StorageBackend,
    key: &[u8],
    value: &[u8],
) -> HostCallResult {
    // Check capability
    if !caps.has_capability(CapabilityType::StorageWrite, CapabilityScope::Sandboxed) {
        return HostCallResult::capability_denied(CapabilityType::StorageWrite);
    }

    // Validate key size
    if key.is_empty() {
        return HostCallResult::error("Key cannot be empty");
    }

    if key.len() > MAX_KEY_SIZE {
        return HostCallResult::error(format!(
            "Key size exceeds maximum of {} bytes",
            MAX_KEY_SIZE
        ));
    }

    // Validate value size
    if value.len() > MAX_VALUE_SIZE {
        return HostCallResult::error(format!(
            "Value size exceeds maximum of {} bytes",
            MAX_VALUE_SIZE
        ));
    }

    // Write to storage
    match storage.write(key, value) {
        Ok(()) => HostCallResult::success(),
        Err(e) => HostCallResult::error(format!("Storage write error: {}", e)),
    }
}

/// Delete from storage
///
/// Requires StorageDelete capability.
///
/// # Arguments
/// * `caps` - Capability set to check permissions
/// * `storage` - Storage backend to delete from
/// * `key` - Key to delete
///
/// # Returns
/// HostCallResult with success (return_value contains 1 byte: 1 if deleted, 0 if not found)
pub fn host_storage_delete(
    caps: &CapabilitySet,
    storage: &dyn StorageBackend,
    key: &[u8],
) -> HostCallResult {
    // Check capability
    if !caps.has_capability(CapabilityType::StorageDelete, CapabilityScope::Sandboxed) {
        return HostCallResult::capability_denied(CapabilityType::StorageDelete);
    }

    // Validate key size
    if key.is_empty() {
        return HostCallResult::error("Key cannot be empty");
    }

    if key.len() > MAX_KEY_SIZE {
        return HostCallResult::error(format!(
            "Key size exceeds maximum of {} bytes",
            MAX_KEY_SIZE
        ));
    }

    // Delete from storage
    match storage.delete(key) {
        Ok(deleted) => {
            let result_byte = if deleted { 1u8 } else { 0u8 };
            HostCallResult::success_with_value(vec![result_byte])
        }
        Err(e) => HostCallResult::error(format!("Storage delete error: {}", e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::CapabilityGrant;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn create_storage_caps() -> CapabilitySet {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();

        let mut cap_set = CapabilitySet::new();
        for cap_type in [
            CapabilityType::StorageRead,
            CapabilityType::StorageWrite,
            CapabilityType::StorageDelete,
        ] {
            let grant = CapabilityGrant::new(
                1,
                cap_type,
                CapabilityScope::Sandboxed,
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
    fn test_in_memory_storage_basic() {
        let storage = InMemoryStorage::new();

        // Initially empty
        assert_eq!(storage.count().unwrap(), 0);

        // Write
        storage.write(b"key1", b"value1").unwrap();
        assert_eq!(storage.count().unwrap(), 1);

        // Read
        let value = storage.read(b"key1").unwrap();
        assert_eq!(value, Some(b"value1".to_vec()));

        // Read non-existent
        let value = storage.read(b"key2").unwrap();
        assert_eq!(value, None);

        // Delete existing
        let deleted = storage.delete(b"key1").unwrap();
        assert!(deleted);
        assert_eq!(storage.count().unwrap(), 0);

        // Delete non-existent
        let deleted = storage.delete(b"key1").unwrap();
        assert!(!deleted);
    }

    #[test]
    fn test_in_memory_storage_overwrite() {
        let storage = InMemoryStorage::new();

        storage.write(b"key", b"value1").unwrap();
        storage.write(b"key", b"value2").unwrap();

        let value = storage.read(b"key").unwrap();
        assert_eq!(value, Some(b"value2".to_vec()));
        assert_eq!(storage.count().unwrap(), 1);
    }

    #[test]
    fn test_in_memory_storage_clear() {
        let storage = InMemoryStorage::new();

        storage.write(b"key1", b"value1").unwrap();
        storage.write(b"key2", b"value2").unwrap();
        assert_eq!(storage.count().unwrap(), 2);

        storage.clear().unwrap();
        assert_eq!(storage.count().unwrap(), 0);
    }

    #[test]
    fn test_host_storage_read_with_capability() {
        let caps = create_storage_caps();
        let storage = InMemoryStorage::new();

        // Write directly to storage
        storage.write(b"test_key", b"test_value").unwrap();

        // Read via host function
        let result = host_storage_read(&caps, &storage, b"test_key");
        assert!(result.success);
        assert_eq!(result.return_value, Some(b"test_value".to_vec()));
    }

    #[test]
    fn test_host_storage_read_not_found() {
        let caps = create_storage_caps();
        let storage = InMemoryStorage::new();

        let result = host_storage_read(&caps, &storage, b"nonexistent");
        assert!(result.success);
        assert_eq!(result.return_value, None);
    }

    #[test]
    fn test_host_storage_read_without_capability() {
        let caps = CapabilitySet::new();
        let storage = InMemoryStorage::new();

        let result = host_storage_read(&caps, &storage, b"key");
        assert!(!result.success);
        assert!(result.error.unwrap().contains("Capability denied"));
    }

    #[test]
    fn test_host_storage_write_with_capability() {
        let caps = create_storage_caps();
        let storage = InMemoryStorage::new();

        let result = host_storage_write(&caps, &storage, b"key", b"value");
        assert!(result.success);

        // Verify it was written
        let value = storage.read(b"key").unwrap();
        assert_eq!(value, Some(b"value".to_vec()));
    }

    #[test]
    fn test_host_storage_write_without_capability() {
        let caps = CapabilitySet::new();
        let storage = InMemoryStorage::new();

        let result = host_storage_write(&caps, &storage, b"key", b"value");
        assert!(!result.success);
        assert!(result.error.unwrap().contains("Capability denied"));
    }

    #[test]
    fn test_host_storage_delete_with_capability() {
        let caps = create_storage_caps();
        let storage = InMemoryStorage::new();

        // Write first
        storage.write(b"key", b"value").unwrap();

        // Delete via host function
        let result = host_storage_delete(&caps, &storage, b"key");
        assert!(result.success);
        assert_eq!(result.return_value, Some(vec![1])); // 1 = deleted

        // Verify it was deleted
        let value = storage.read(b"key").unwrap();
        assert_eq!(value, None);
    }

    #[test]
    fn test_host_storage_delete_not_found() {
        let caps = create_storage_caps();
        let storage = InMemoryStorage::new();

        let result = host_storage_delete(&caps, &storage, b"nonexistent");
        assert!(result.success);
        assert_eq!(result.return_value, Some(vec![0])); // 0 = not found
    }

    #[test]
    fn test_host_storage_delete_without_capability() {
        let caps = CapabilitySet::new();
        let storage = InMemoryStorage::new();

        let result = host_storage_delete(&caps, &storage, b"key");
        assert!(!result.success);
        assert!(result.error.unwrap().contains("Capability denied"));
    }

    #[test]
    fn test_host_storage_empty_key() {
        let caps = create_storage_caps();
        let storage = InMemoryStorage::new();

        let result = host_storage_read(&caps, &storage, b"");
        assert!(!result.success);
        assert!(result.error.unwrap().contains("cannot be empty"));
    }

    #[test]
    fn test_host_storage_key_too_large() {
        let caps = create_storage_caps();
        let storage = InMemoryStorage::new();

        let large_key = vec![0u8; MAX_KEY_SIZE + 1];
        let result = host_storage_read(&caps, &storage, &large_key);
        assert!(!result.success);
        assert!(result.error.unwrap().contains("exceeds maximum"));
    }

    #[test]
    fn test_host_storage_value_too_large() {
        let caps = create_storage_caps();
        let storage = InMemoryStorage::new();

        let large_value = vec![0u8; MAX_VALUE_SIZE + 1];
        let result = host_storage_write(&caps, &storage, b"key", &large_value);
        assert!(!result.success);
        assert!(result.error.unwrap().contains("exceeds maximum"));
    }

    #[test]
    fn test_host_storage_with_unrestricted() {
        let caps = create_unrestricted_capset();
        let storage = InMemoryStorage::new();

        // All operations should work with unrestricted capability
        let write_result = host_storage_write(&caps, &storage, b"key", b"value");
        assert!(write_result.success);

        let read_result = host_storage_read(&caps, &storage, b"key");
        assert!(read_result.success);

        let delete_result = host_storage_delete(&caps, &storage, b"key");
        assert!(delete_result.success);
    }
}
