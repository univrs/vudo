//! Host Network Functions
//!
//! Provides network-related host functions for WASM sandboxes.
//! All network operations are capability-gated to ensure secure sandbox execution.

use super::{CapabilityScope, CapabilitySet, CapabilityType, HostCallResult};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Maximum address length in bytes
const MAX_ADDRESS_SIZE: usize = 256;

/// Maximum message size in bytes for broadcast
const MAX_MESSAGE_SIZE: usize = 64 * 1024; // 64KB

// ═══════════════════════════════════════════════════════════════════════════
// NETWORK BACKEND TRAIT
// ═══════════════════════════════════════════════════════════════════════════

/// Connection handle returned by successful connect operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConnectionHandle(pub u64);

impl ConnectionHandle {
    /// Create a new connection handle from a u64 identifier
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    /// Get the underlying connection ID
    pub fn id(&self) -> u64 {
        self.0
    }

    /// Serialize the handle to bytes (little-endian u64)
    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.to_le_bytes().to_vec()
    }

    /// Deserialize handle from bytes
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() == 8 {
            let arr: [u8; 8] = bytes.try_into().ok()?;
            Some(Self(u64::from_le_bytes(arr)))
        } else {
            None
        }
    }
}

/// Listener handle returned by successful listen operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ListenerHandle(pub u64);

impl ListenerHandle {
    /// Create a new listener handle from a u64 identifier
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    /// Get the underlying listener ID
    pub fn id(&self) -> u64 {
        self.0
    }

    /// Serialize the handle to bytes (little-endian u64)
    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.to_le_bytes().to_vec()
    }

    /// Deserialize handle from bytes
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() == 8 {
            let arr: [u8; 8] = bytes.try_into().ok()?;
            Some(Self(u64::from_le_bytes(arr)))
        } else {
            None
        }
    }
}

/// Network backend trait
///
/// Implementations provide the actual network operations.
/// This abstraction allows for testing with mock backends and
/// swapping implementations (e.g., in-memory, TCP, WebSocket).
pub trait NetworkBackend: Send + Sync {
    /// Establish a connection to the given address
    ///
    /// Returns:
    /// - Ok(ConnectionHandle) on success
    /// - Err(msg) if connection fails
    fn connect(&self, address: &str) -> Result<ConnectionHandle, String>;

    /// Start listening on the given port
    ///
    /// Returns:
    /// - Ok(ListenerHandle) on success
    /// - Err(msg) if binding fails
    fn listen(&self, port: u16) -> Result<ListenerHandle, String>;

    /// Broadcast a message to all connected peers
    ///
    /// Returns:
    /// - Ok(count) - number of peers that received the message
    /// - Err(msg) on failure
    fn broadcast(&self, message: &[u8]) -> Result<usize, String>;

    /// Close a connection
    fn close_connection(&self, handle: ConnectionHandle) -> Result<(), String>;

    /// Stop listening on a port
    fn close_listener(&self, handle: ListenerHandle) -> Result<(), String>;

    /// Get number of active connections
    fn connection_count(&self) -> usize;

    /// Get number of active listeners
    fn listener_count(&self) -> usize;
}

// ═══════════════════════════════════════════════════════════════════════════
// MOCK NETWORK BACKEND (FOR TESTING)
// ═══════════════════════════════════════════════════════════════════════════

/// Mock network backend for testing
///
/// Simulates network operations without actual network I/O.
/// Useful for unit testing capability checks and error handling.
#[derive(Debug, Clone)]
pub struct MockNetworkBackend {
    connections: Arc<RwLock<HashMap<u64, String>>>,
    listeners: Arc<RwLock<HashMap<u64, u16>>>,
    next_connection_id: Arc<RwLock<u64>>,
    next_listener_id: Arc<RwLock<u64>>,
    broadcast_messages: Arc<RwLock<Vec<Vec<u8>>>>,
    /// If set, connect operations will fail with this error
    pub connect_error: Arc<RwLock<Option<String>>>,
    /// If set, listen operations will fail with this error
    pub listen_error: Arc<RwLock<Option<String>>>,
    /// If set, broadcast operations will fail with this error
    pub broadcast_error: Arc<RwLock<Option<String>>>,
}

impl MockNetworkBackend {
    /// Create a new mock network backend
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            listeners: Arc::new(RwLock::new(HashMap::new())),
            next_connection_id: Arc::new(RwLock::new(1)),
            next_listener_id: Arc::new(RwLock::new(1)),
            broadcast_messages: Arc::new(RwLock::new(Vec::new())),
            connect_error: Arc::new(RwLock::new(None)),
            listen_error: Arc::new(RwLock::new(None)),
            broadcast_error: Arc::new(RwLock::new(None)),
        }
    }

    /// Get list of addresses that have been connected to
    pub fn connected_addresses(&self) -> Vec<String> {
        self.connections.read().unwrap().values().cloned().collect()
    }

    /// Get list of ports being listened on
    pub fn listening_ports(&self) -> Vec<u16> {
        self.listeners.read().unwrap().values().copied().collect()
    }

    /// Get list of broadcast messages sent
    pub fn broadcast_messages(&self) -> Vec<Vec<u8>> {
        self.broadcast_messages.read().unwrap().clone()
    }

    /// Set an error for connect operations
    pub fn set_connect_error(&self, error: Option<String>) {
        *self.connect_error.write().unwrap() = error;
    }

    /// Set an error for listen operations
    pub fn set_listen_error(&self, error: Option<String>) {
        *self.listen_error.write().unwrap() = error;
    }

    /// Set an error for broadcast operations
    pub fn set_broadcast_error(&self, error: Option<String>) {
        *self.broadcast_error.write().unwrap() = error;
    }

    /// Clear all state
    pub fn clear(&self) {
        self.connections.write().unwrap().clear();
        self.listeners.write().unwrap().clear();
        self.broadcast_messages.write().unwrap().clear();
        *self.next_connection_id.write().unwrap() = 1;
        *self.next_listener_id.write().unwrap() = 1;
    }
}

impl Default for MockNetworkBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl NetworkBackend for MockNetworkBackend {
    fn connect(&self, address: &str) -> Result<ConnectionHandle, String> {
        // Check for simulated error
        if let Some(ref error) = *self.connect_error.read().map_err(|e| e.to_string())? {
            return Err(error.clone());
        }

        let mut connections = self
            .connections
            .write()
            .map_err(|e| format!("Lock error: {}", e))?;
        let mut next_id = self
            .next_connection_id
            .write()
            .map_err(|e| format!("Lock error: {}", e))?;

        let id = *next_id;
        *next_id += 1;
        connections.insert(id, address.to_string());

        Ok(ConnectionHandle::new(id))
    }

    fn listen(&self, port: u16) -> Result<ListenerHandle, String> {
        // Check for simulated error
        if let Some(ref error) = *self.listen_error.read().map_err(|e| e.to_string())? {
            return Err(error.clone());
        }

        let mut listeners = self
            .listeners
            .write()
            .map_err(|e| format!("Lock error: {}", e))?;

        // Check if port is already in use
        if listeners.values().any(|&p| p == port) {
            return Err(format!("Port {} is already in use", port));
        }

        let mut next_id = self
            .next_listener_id
            .write()
            .map_err(|e| format!("Lock error: {}", e))?;

        let id = *next_id;
        *next_id += 1;
        listeners.insert(id, port);

        Ok(ListenerHandle::new(id))
    }

    fn broadcast(&self, message: &[u8]) -> Result<usize, String> {
        // Check for simulated error
        if let Some(ref error) = *self.broadcast_error.read().map_err(|e| e.to_string())? {
            return Err(error.clone());
        }

        let mut messages = self
            .broadcast_messages
            .write()
            .map_err(|e| format!("Lock error: {}", e))?;

        messages.push(message.to_vec());

        // Return the number of "peers" (connections) that received the message
        let connection_count = self.connection_count();
        Ok(connection_count)
    }

    fn close_connection(&self, handle: ConnectionHandle) -> Result<(), String> {
        let mut connections = self
            .connections
            .write()
            .map_err(|e| format!("Lock error: {}", e))?;

        if connections.remove(&handle.id()).is_some() {
            Ok(())
        } else {
            Err(format!("Connection {} not found", handle.id()))
        }
    }

    fn close_listener(&self, handle: ListenerHandle) -> Result<(), String> {
        let mut listeners = self
            .listeners
            .write()
            .map_err(|e| format!("Lock error: {}", e))?;

        if listeners.remove(&handle.id()).is_some() {
            Ok(())
        } else {
            Err(format!("Listener {} not found", handle.id()))
        }
    }

    fn connection_count(&self) -> usize {
        self.connections.read().map(|c| c.len()).unwrap_or(0)
    }

    fn listener_count(&self) -> usize {
        self.listeners.read().map(|l| l.len()).unwrap_or(0)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// HOST NETWORK FUNCTIONS
// ═══════════════════════════════════════════════════════════════════════════

/// Connect to a network address
///
/// Requires NetworkConnect capability.
///
/// # Arguments
/// * `caps` - Capability set to check permissions
/// * `network` - Network backend to use for the connection
/// * `address` - Address to connect to (e.g., "192.168.1.1:8080" or "example.com:443")
///
/// # Returns
/// HostCallResult with connection handle bytes (u64 in little-endian) on success, or error
///
/// # Example
/// ```ignore
/// let result = host_network_connect(&caps, &network, "localhost:8080");
/// if result.success {
///     let handle = ConnectionHandle::from_bytes(&result.return_value.unwrap());
/// }
/// ```
pub fn host_network_connect(
    caps: &CapabilitySet,
    network: &dyn NetworkBackend,
    address: &str,
) -> HostCallResult {
    // Check capability
    if !caps.has_capability(CapabilityType::NetworkConnect, CapabilityScope::Global) {
        return HostCallResult::capability_denied(CapabilityType::NetworkConnect);
    }

    // Validate address
    if address.is_empty() {
        return HostCallResult::error("Address cannot be empty");
    }

    if address.len() > MAX_ADDRESS_SIZE {
        return HostCallResult::error(format!(
            "Address size exceeds maximum of {} bytes",
            MAX_ADDRESS_SIZE
        ));
    }

    // Attempt connection
    match network.connect(address) {
        Ok(handle) => HostCallResult::success_with_value(handle.to_bytes()),
        Err(e) => HostCallResult::error(format!("Network connect error: {}", e)),
    }
}

/// Start listening on a network port
///
/// Requires NetworkListen capability.
///
/// # Arguments
/// * `caps` - Capability set to check permissions
/// * `network` - Network backend to use for listening
/// * `port` - Port number to listen on (0-65535)
///
/// # Returns
/// HostCallResult with listener handle bytes (u64 in little-endian) on success, or error
///
/// # Example
/// ```ignore
/// let result = host_network_listen(&caps, &network, 8080);
/// if result.success {
///     let handle = ListenerHandle::from_bytes(&result.return_value.unwrap());
/// }
/// ```
pub fn host_network_listen(
    caps: &CapabilitySet,
    network: &dyn NetworkBackend,
    port: u16,
) -> HostCallResult {
    // Check capability
    if !caps.has_capability(CapabilityType::NetworkListen, CapabilityScope::Global) {
        return HostCallResult::capability_denied(CapabilityType::NetworkListen);
    }

    // Note: Port validation is implicit via u16 type (max 65535)
    // Port 0 is typically for dynamic allocation but we allow it

    // Attempt to start listening
    match network.listen(port) {
        Ok(handle) => HostCallResult::success_with_value(handle.to_bytes()),
        Err(e) => HostCallResult::error(format!("Network listen error: {}", e)),
    }
}

/// Broadcast a message to all connected peers
///
/// Requires NetworkBroadcast capability.
///
/// # Arguments
/// * `caps` - Capability set to check permissions
/// * `network` - Network backend to use for broadcasting
/// * `message` - Message bytes to broadcast
///
/// # Returns
/// HostCallResult with peer count bytes (u64 in little-endian) indicating how many
/// peers received the message, or error
///
/// # Example
/// ```ignore
/// let result = host_network_broadcast(&caps, &network, b"Hello, peers!");
/// if result.success {
///     let count = u64::from_le_bytes(result.return_value.unwrap().try_into().unwrap());
///     println!("Broadcast to {} peers", count);
/// }
/// ```
pub fn host_network_broadcast(
    caps: &CapabilitySet,
    network: &dyn NetworkBackend,
    message: &[u8],
) -> HostCallResult {
    // Check capability
    if !caps.has_capability(CapabilityType::NetworkBroadcast, CapabilityScope::Global) {
        return HostCallResult::capability_denied(CapabilityType::NetworkBroadcast);
    }

    // Validate message
    if message.is_empty() {
        return HostCallResult::error("Message cannot be empty");
    }

    if message.len() > MAX_MESSAGE_SIZE {
        return HostCallResult::error(format!(
            "Message size exceeds maximum of {} bytes",
            MAX_MESSAGE_SIZE
        ));
    }

    // Attempt broadcast
    match network.broadcast(message) {
        Ok(peer_count) => {
            let count_bytes = (peer_count as u64).to_le_bytes().to_vec();
            HostCallResult::success_with_value(count_bytes)
        }
        Err(e) => HostCallResult::error(format!("Network broadcast error: {}", e)),
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::CapabilityGrant;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
    }

    fn create_network_caps() -> CapabilitySet {
        let now = current_timestamp();

        let mut cap_set = CapabilitySet::new();
        for cap_type in [
            CapabilityType::NetworkConnect,
            CapabilityType::NetworkListen,
            CapabilityType::NetworkBroadcast,
        ] {
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

    fn create_connect_only_caps() -> CapabilitySet {
        let now = current_timestamp();

        let mut cap_set = CapabilitySet::new();
        let grant = CapabilityGrant::new(
            1,
            CapabilityType::NetworkConnect,
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

    fn create_unrestricted_capset() -> CapabilitySet {
        let now = current_timestamp();

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

    // ═══════════════════════════════════════════════════════════════════════
    // CONNECTION HANDLE TESTS
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_connection_handle_serialization() {
        let handle = ConnectionHandle::new(12345);
        let bytes = handle.to_bytes();
        assert_eq!(bytes.len(), 8);

        let restored = ConnectionHandle::from_bytes(&bytes);
        assert_eq!(restored, Some(handle));
    }

    #[test]
    fn test_connection_handle_invalid_bytes() {
        let invalid = ConnectionHandle::from_bytes(&[1, 2, 3]); // Too short
        assert_eq!(invalid, None);
    }

    // ═══════════════════════════════════════════════════════════════════════
    // LISTENER HANDLE TESTS
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_listener_handle_serialization() {
        let handle = ListenerHandle::new(67890);
        let bytes = handle.to_bytes();
        assert_eq!(bytes.len(), 8);

        let restored = ListenerHandle::from_bytes(&bytes);
        assert_eq!(restored, Some(handle));
    }

    #[test]
    fn test_listener_handle_invalid_bytes() {
        let invalid = ListenerHandle::from_bytes(&[1, 2, 3, 4, 5]); // Too short
        assert_eq!(invalid, None);
    }

    // ═══════════════════════════════════════════════════════════════════════
    // MOCK NETWORK BACKEND TESTS
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_mock_network_connect() {
        let network = MockNetworkBackend::new();

        let handle = network.connect("localhost:8080").unwrap();
        assert_eq!(handle.id(), 1);
        assert_eq!(network.connection_count(), 1);

        let addresses = network.connected_addresses();
        assert!(addresses.contains(&"localhost:8080".to_string()));
    }

    #[test]
    fn test_mock_network_listen() {
        let network = MockNetworkBackend::new();

        let handle = network.listen(8080).unwrap();
        assert_eq!(handle.id(), 1);
        assert_eq!(network.listener_count(), 1);

        let ports = network.listening_ports();
        assert!(ports.contains(&8080));
    }

    #[test]
    fn test_mock_network_listen_port_in_use() {
        let network = MockNetworkBackend::new();

        network.listen(8080).unwrap();
        let result = network.listen(8080);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already in use"));
    }

    #[test]
    fn test_mock_network_broadcast() {
        let network = MockNetworkBackend::new();

        // Create some connections first
        network.connect("peer1:8080").unwrap();
        network.connect("peer2:8080").unwrap();

        let count = network.broadcast(b"Hello, peers!").unwrap();
        assert_eq!(count, 2);

        let messages = network.broadcast_messages();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0], b"Hello, peers!");
    }

    #[test]
    fn test_mock_network_close_connection() {
        let network = MockNetworkBackend::new();

        let handle = network.connect("localhost:8080").unwrap();
        assert_eq!(network.connection_count(), 1);

        network.close_connection(handle).unwrap();
        assert_eq!(network.connection_count(), 0);
    }

    #[test]
    fn test_mock_network_close_listener() {
        let network = MockNetworkBackend::new();

        let handle = network.listen(8080).unwrap();
        assert_eq!(network.listener_count(), 1);

        network.close_listener(handle).unwrap();
        assert_eq!(network.listener_count(), 0);
    }

    #[test]
    fn test_mock_network_simulated_errors() {
        let network = MockNetworkBackend::new();

        network.set_connect_error(Some("Simulated connection failure".to_string()));
        let result = network.connect("localhost:8080");
        assert!(result.is_err());

        network.set_connect_error(None);
        network.set_listen_error(Some("Simulated listen failure".to_string()));
        let result = network.listen(8080);
        assert!(result.is_err());

        network.set_listen_error(None);
        network.set_broadcast_error(Some("Simulated broadcast failure".to_string()));
        let result = network.broadcast(b"test");
        assert!(result.is_err());
    }

    #[test]
    fn test_mock_network_clear() {
        let network = MockNetworkBackend::new();

        network.connect("localhost:8080").unwrap();
        network.listen(9090).unwrap();
        network.broadcast(b"test").unwrap();

        network.clear();

        assert_eq!(network.connection_count(), 0);
        assert_eq!(network.listener_count(), 0);
        assert!(network.broadcast_messages().is_empty());
    }

    // ═══════════════════════════════════════════════════════════════════════
    // HOST_NETWORK_CONNECT TESTS
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_host_network_connect_with_capability() {
        let caps = create_network_caps();
        let network = MockNetworkBackend::new();

        let result = host_network_connect(&caps, &network, "localhost:8080");
        assert!(result.success);
        assert!(result.error.is_none());
        assert!(result.return_value.is_some());

        // Should be 8 bytes (u64 handle)
        let bytes = result.return_value.unwrap();
        assert_eq!(bytes.len(), 8);

        // Verify we can deserialize the handle
        let handle = ConnectionHandle::from_bytes(&bytes);
        assert!(handle.is_some());
    }

    #[test]
    fn test_host_network_connect_without_capability() {
        let caps = CapabilitySet::new(); // Empty capability set
        let network = MockNetworkBackend::new();

        let result = host_network_connect(&caps, &network, "localhost:8080");

        assert!(!result.success);
        assert!(result.return_value.is_none());
        assert!(result.error.is_some());
        assert!(result.error.unwrap().contains("Capability denied"));
    }

    #[test]
    fn test_host_network_connect_empty_address() {
        let caps = create_network_caps();
        let network = MockNetworkBackend::new();

        let result = host_network_connect(&caps, &network, "");

        assert!(!result.success);
        assert!(result.error.unwrap().contains("cannot be empty"));
    }

    #[test]
    fn test_host_network_connect_address_too_large() {
        let caps = create_network_caps();
        let network = MockNetworkBackend::new();

        let large_address = "x".repeat(MAX_ADDRESS_SIZE + 1);
        let result = host_network_connect(&caps, &network, &large_address);

        assert!(!result.success);
        assert!(result.error.unwrap().contains("exceeds maximum"));
    }

    #[test]
    fn test_host_network_connect_network_error() {
        let caps = create_network_caps();
        let network = MockNetworkBackend::new();
        network.set_connect_error(Some("Connection refused".to_string()));

        let result = host_network_connect(&caps, &network, "localhost:8080");

        assert!(!result.success);
        assert!(result.error.unwrap().contains("Connection refused"));
    }

    #[test]
    fn test_host_network_connect_with_unrestricted() {
        let caps = create_unrestricted_capset();
        let network = MockNetworkBackend::new();

        let result = host_network_connect(&caps, &network, "localhost:8080");

        assert!(result.success);
        assert!(result.return_value.is_some());
    }

    // ═══════════════════════════════════════════════════════════════════════
    // HOST_NETWORK_LISTEN TESTS
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_host_network_listen_with_capability() {
        let caps = create_network_caps();
        let network = MockNetworkBackend::new();

        let result = host_network_listen(&caps, &network, 8080);

        assert!(result.success);
        assert!(result.error.is_none());
        assert!(result.return_value.is_some());

        // Should be 8 bytes (u64 handle)
        let bytes = result.return_value.unwrap();
        assert_eq!(bytes.len(), 8);

        // Verify we can deserialize the handle
        let handle = ListenerHandle::from_bytes(&bytes);
        assert!(handle.is_some());
    }

    #[test]
    fn test_host_network_listen_without_capability() {
        let caps = CapabilitySet::new(); // Empty capability set
        let network = MockNetworkBackend::new();

        let result = host_network_listen(&caps, &network, 8080);

        assert!(!result.success);
        assert!(result.return_value.is_none());
        assert!(result.error.is_some());
        assert!(result.error.unwrap().contains("Capability denied"));
    }

    #[test]
    fn test_host_network_listen_with_only_connect_capability() {
        let caps = create_connect_only_caps();
        let network = MockNetworkBackend::new();

        let result = host_network_listen(&caps, &network, 8080);

        assert!(!result.success);
        assert!(result.error.unwrap().contains("Capability denied"));
    }

    #[test]
    fn test_host_network_listen_port_in_use() {
        let caps = create_network_caps();
        let network = MockNetworkBackend::new();

        // First listen should succeed
        let result1 = host_network_listen(&caps, &network, 8080);
        assert!(result1.success);

        // Second listen on same port should fail
        let result2 = host_network_listen(&caps, &network, 8080);
        assert!(!result2.success);
        assert!(result2.error.unwrap().contains("already in use"));
    }

    #[test]
    fn test_host_network_listen_network_error() {
        let caps = create_network_caps();
        let network = MockNetworkBackend::new();
        network.set_listen_error(Some("Bind failed".to_string()));

        let result = host_network_listen(&caps, &network, 8080);

        assert!(!result.success);
        assert!(result.error.unwrap().contains("Bind failed"));
    }

    #[test]
    fn test_host_network_listen_with_unrestricted() {
        let caps = create_unrestricted_capset();
        let network = MockNetworkBackend::new();

        let result = host_network_listen(&caps, &network, 8080);

        assert!(result.success);
        assert!(result.return_value.is_some());
    }

    #[test]
    fn test_host_network_listen_port_zero() {
        let caps = create_network_caps();
        let network = MockNetworkBackend::new();

        // Port 0 is valid (typically means dynamic port allocation)
        let result = host_network_listen(&caps, &network, 0);
        assert!(result.success);
    }

    #[test]
    fn test_host_network_listen_max_port() {
        let caps = create_network_caps();
        let network = MockNetworkBackend::new();

        let result = host_network_listen(&caps, &network, 65535);
        assert!(result.success);
    }

    // ═══════════════════════════════════════════════════════════════════════
    // HOST_NETWORK_BROADCAST TESTS
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_host_network_broadcast_with_capability() {
        let caps = create_network_caps();
        let network = MockNetworkBackend::new();

        // Add some connections first
        network.connect("peer1:8080").unwrap();
        network.connect("peer2:8080").unwrap();

        let result = host_network_broadcast(&caps, &network, b"Hello, peers!");

        assert!(result.success);
        assert!(result.error.is_none());
        assert!(result.return_value.is_some());

        // Should be 8 bytes (u64 peer count)
        let bytes = result.return_value.unwrap();
        assert_eq!(bytes.len(), 8);

        let count = u64::from_le_bytes(bytes.try_into().unwrap());
        assert_eq!(count, 2);
    }

    #[test]
    fn test_host_network_broadcast_without_capability() {
        let caps = CapabilitySet::new(); // Empty capability set
        let network = MockNetworkBackend::new();

        let result = host_network_broadcast(&caps, &network, b"Hello");

        assert!(!result.success);
        assert!(result.return_value.is_none());
        assert!(result.error.is_some());
        assert!(result.error.unwrap().contains("Capability denied"));
    }

    #[test]
    fn test_host_network_broadcast_empty_message() {
        let caps = create_network_caps();
        let network = MockNetworkBackend::new();

        let result = host_network_broadcast(&caps, &network, b"");

        assert!(!result.success);
        assert!(result.error.unwrap().contains("cannot be empty"));
    }

    #[test]
    fn test_host_network_broadcast_message_too_large() {
        let caps = create_network_caps();
        let network = MockNetworkBackend::new();

        let large_message = vec![0u8; MAX_MESSAGE_SIZE + 1];
        let result = host_network_broadcast(&caps, &network, &large_message);

        assert!(!result.success);
        assert!(result.error.unwrap().contains("exceeds maximum"));
    }

    #[test]
    fn test_host_network_broadcast_no_peers() {
        let caps = create_network_caps();
        let network = MockNetworkBackend::new();

        // No connections, broadcast should still succeed but with 0 peers
        let result = host_network_broadcast(&caps, &network, b"Hello");

        assert!(result.success);
        let bytes = result.return_value.unwrap();
        let count = u64::from_le_bytes(bytes.try_into().unwrap());
        assert_eq!(count, 0);
    }

    #[test]
    fn test_host_network_broadcast_network_error() {
        let caps = create_network_caps();
        let network = MockNetworkBackend::new();
        network.set_broadcast_error(Some("Broadcast failed".to_string()));

        let result = host_network_broadcast(&caps, &network, b"Hello");

        assert!(!result.success);
        assert!(result.error.unwrap().contains("Broadcast failed"));
    }

    #[test]
    fn test_host_network_broadcast_with_unrestricted() {
        let caps = create_unrestricted_capset();
        let network = MockNetworkBackend::new();

        let result = host_network_broadcast(&caps, &network, b"Hello");

        assert!(result.success);
        assert!(result.return_value.is_some());
    }

    // ═══════════════════════════════════════════════════════════════════════
    // INTEGRATION TESTS
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_full_network_workflow() {
        let caps = create_network_caps();
        let network = MockNetworkBackend::new();

        // Listen on a port
        let listen_result = host_network_listen(&caps, &network, 8080);
        assert!(listen_result.success);

        // Connect to some peers
        let connect_result1 = host_network_connect(&caps, &network, "peer1:9000");
        assert!(connect_result1.success);

        let connect_result2 = host_network_connect(&caps, &network, "peer2:9000");
        assert!(connect_result2.success);

        // Broadcast a message
        let broadcast_result = host_network_broadcast(&caps, &network, b"Sync message");
        assert!(broadcast_result.success);

        let bytes = broadcast_result.return_value.unwrap();
        let count = u64::from_le_bytes(bytes.try_into().unwrap());
        assert_eq!(count, 2);

        // Verify state
        assert_eq!(network.connection_count(), 2);
        assert_eq!(network.listener_count(), 1);
        assert_eq!(network.broadcast_messages().len(), 1);
    }

    #[test]
    fn test_capability_isolation() {
        let network = MockNetworkBackend::new();

        // Create caps with only NetworkConnect
        let now = current_timestamp();
        let mut connect_only = CapabilitySet::new();
        connect_only.add_grant(CapabilityGrant::new(
            1,
            CapabilityType::NetworkConnect,
            CapabilityScope::Global,
            [0u8; 32],
            [1u8; 32],
            now,
            None,
            [0u8; 64],
        ));

        // Connect should work
        let connect_result = host_network_connect(&connect_only, &network, "localhost:8080");
        assert!(connect_result.success);

        // Listen should fail
        let listen_result = host_network_listen(&connect_only, &network, 9090);
        assert!(!listen_result.success);

        // Broadcast should fail
        let broadcast_result = host_network_broadcast(&connect_only, &network, b"test");
        assert!(!broadcast_result.success);
    }
}
