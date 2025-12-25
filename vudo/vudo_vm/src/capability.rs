// VUDO VM Capability Module
// Implementation of the capability system for sandboxed execution
// Based on: docs/ontology/prospective/vudo-vm/genes/capability.dol

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Serialize/deserialize wrapper for [u8; 64]
mod signature_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(bytes: &[u8; 64], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        bytes.as_slice().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8; 64], D::Error>
    where
        D: Deserializer<'de>,
    {
        let vec = Vec::<u8>::deserialize(deserializer)?;
        if vec.len() != 64 {
            return Err(serde::de::Error::custom("Expected 64 bytes"));
        }
        let mut arr = [0u8; 64];
        arr.copy_from_slice(&vec);
        Ok(arr)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// CAPABILITY TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// Defines the categories of privileged operations.
///
/// Capabilities are organized into functional groups:
/// - Network: Communication with peers and external services
/// - Storage: Persistent data access and modification
/// - Compute: Sandbox creation and cross-sandbox calls
/// - Sensor: Read-only access to system state (time, random, env)
/// - Actuator: Write operations that affect external state
///
/// The Unrestricted capability is only granted to system Spirits
/// and bypasses all capability checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CapabilityType {
    // Network capabilities
    NetworkListen,
    NetworkConnect,
    NetworkBroadcast,

    // Storage capabilities
    StorageRead,
    StorageWrite,
    StorageDelete,

    // Compute capabilities
    SpawnSandbox,
    CrossSandboxCall,

    // Sensor capabilities (read external state)
    SensorTime,
    SensorRandom,
    SensorEnvironment,

    // Actuator capabilities (affect external state)
    ActuatorLog,
    ActuatorNotify,
    ActuatorCredit,

    // Special capabilities
    Unrestricted, // Only for system Spirits
}

// ═══════════════════════════════════════════════════════════════════════════
// CAPABILITY SCOPE
// ═══════════════════════════════════════════════════════════════════════════

/// Defines the boundaries of a capability grant.
///
/// Scope types:
/// - Global: No restriction on target
/// - Sandboxed: Only within the sandbox's own context
/// - Peer: Limited to specific peer (peer_id stored in grant metadata)
/// - Domain: Network domain pattern match (pattern stored in grant metadata)
///
/// Scopes enable the principle of least privilege by limiting
/// the reach of granted capabilities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CapabilityScope {
    Global,
    Sandboxed,
    Peer,
    Domain,
}

impl CapabilityScope {
    /// Check if this scope covers (is broader than or equal to) another scope
    pub fn covers(&self, other: &CapabilityScope) -> bool {
        use CapabilityScope::*;
        matches!(
            (self, other),
            (Global, _) | (Sandboxed, Sandboxed) | (Peer, Peer) | (Domain, Domain)
        )
    }

    /// Check if this scope is a subset of another scope
    pub fn is_subset_of(&self, other: &CapabilityScope) -> bool {
        other.covers(self)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// CAPABILITY GRANT
// ═══════════════════════════════════════════════════════════════════════════

/// A cryptographically signed permission.
///
/// Grants are:
/// - Explicit: No implicit permissions
/// - Signed: Verifiable authenticity
/// - Scoped: Limited to specific operations/targets
/// - Temporal: Can expire
/// - Revocable: Can be withdrawn
///
/// The granter must have the capability to grant it.
/// Capabilities follow the principle of least privilege.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityGrant {
    pub id: u64,
    pub capability: CapabilityType,
    pub scope: CapabilityScope,
    pub granter: [u8; 32], // Ed25519 public key
    pub grantee: [u8; 32], // Ed25519 public key
    pub granted_at: u64,   // Unix timestamp in seconds
    pub expires_at: Option<u64>,
    pub revoked: bool,
    #[serde(with = "signature_serde")]
    pub signature: [u8; 64], // Ed25519 signature
}

impl CapabilityGrant {
    /// Create a new capability grant
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: u64,
        capability: CapabilityType,
        scope: CapabilityScope,
        granter: [u8; 32],
        grantee: [u8; 32],
        granted_at: u64,
        expires_at: Option<u64>,
        signature: [u8; 64],
    ) -> Self {
        Self {
            id,
            capability,
            scope,
            granter,
            grantee,
            granted_at,
            expires_at,
            revoked: false,
            signature,
        }
    }

    /// Check if the grant is currently valid (not expired and not revoked)
    pub fn is_valid(&self) -> bool {
        self.is_valid_at(current_timestamp())
    }

    /// Check if the grant is valid at a specific timestamp
    pub fn is_valid_at(&self, now: u64) -> bool {
        if self.revoked {
            return false;
        }

        match self.expires_at {
            Some(expiry) => now < expiry,
            None => true,
        }
    }

    /// Revoke this grant
    pub fn revoke(&mut self) {
        self.revoked = true;
    }

    /// Get the hash of the grant for signing
    /// This includes all fields except the signature itself
    pub fn hash_for_signing(&self) -> [u8; 32] {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        hasher.update(self.id.to_le_bytes());
        hasher.update([self.capability as u8]);
        hasher.update([self.scope as u8]);
        hasher.update(self.granter);
        hasher.update(self.grantee);
        hasher.update(self.granted_at.to_le_bytes());

        if let Some(expires_at) = self.expires_at {
            hasher.update([1u8]); // Some marker
            hasher.update(expires_at.to_le_bytes());
        } else {
            hasher.update([0u8]); // None marker
        }

        hasher.update([self.revoked as u8]);

        hasher.finalize().into()
    }

    /// Verify the signature on this grant (requires ed25519-dalek dependency)
    pub fn verify_signature(&self) -> bool {
        use ed25519_dalek::{Signature, Verifier, VerifyingKey};

        let public_key = match VerifyingKey::from_bytes(&self.granter) {
            Ok(pk) => pk,
            Err(_) => return false,
        };

        let signature = Signature::from_bytes(&self.signature);

        let message = self.hash_for_signing();
        public_key.verify(&message, &signature).is_ok()
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// CAPABILITY SET
// ═══════════════════════════════════════════════════════════════════════════

/// The effective permissions for a Sandbox.
///
/// Computed from:
/// - Spirit manifest required capabilities
/// - User-granted capabilities
/// - System default capabilities
///
/// Checked before every privileged operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilitySet {
    pub grants: HashMap<CapabilityType, Vec<CapabilityGrant>>,
}

impl CapabilitySet {
    /// Create a new empty capability set
    pub fn new() -> Self {
        Self {
            grants: HashMap::new(),
        }
    }

    /// Create a capability set from a list of grants
    pub fn from_grants(grants: Vec<CapabilityGrant>) -> Self {
        let mut capability_set = Self::new();
        for grant in grants {
            capability_set.add_grant(grant);
        }
        capability_set
    }

    /// Add a grant to this capability set
    pub fn add_grant(&mut self, grant: CapabilityGrant) {
        self.grants
            .entry(grant.capability)
            .or_default()
            .push(grant);
    }

    /// Remove a grant by ID
    pub fn remove_grant(&mut self, grant_id: u64) -> bool {
        for grants in self.grants.values_mut() {
            if let Some(pos) = grants.iter().position(|g| g.id == grant_id) {
                grants.remove(pos);
                return true;
            }
        }
        false
    }

    /// Check if this set has a specific capability with the given scope
    pub fn has_capability(&self, cap: CapabilityType, scope: CapabilityScope) -> bool {
        // Unrestricted capability bypasses all checks
        if let Some(grants) = self.grants.get(&CapabilityType::Unrestricted) {
            if grants.iter().any(|g| g.is_valid()) {
                return true;
            }
        }

        // Check for specific capability
        match self.grants.get(&cap) {
            Some(grants) => grants
                .iter()
                .any(|grant| grant.scope.covers(&scope) && grant.is_valid()),
            None => false,
        }
    }

    /// Get the effective scope for a capability (union of all valid grant scopes)
    pub fn effective_scope(&self, cap: CapabilityType) -> Option<CapabilityScope> {
        // Unrestricted capability gives global scope for everything
        if let Some(grants) = self.grants.get(&CapabilityType::Unrestricted) {
            if grants.iter().any(|g| g.is_valid()) {
                return Some(CapabilityScope::Global);
            }
        }

        match self.grants.get(&cap) {
            Some(grants) => {
                let valid_grants: Vec<_> = grants.iter().filter(|g| g.is_valid()).collect();

                if valid_grants.is_empty() {
                    return None;
                }

                // If any grant has Global scope, return Global
                if valid_grants
                    .iter()
                    .any(|g| g.scope == CapabilityScope::Global)
                {
                    return Some(CapabilityScope::Global);
                }

                // Otherwise return the first valid scope (in a real implementation,
                // this would compute a proper union of scopes)
                valid_grants.first().map(|g| g.scope)
            }
            None => None,
        }
    }

    /// Remove expired grants
    pub fn clean_expired(&mut self) {
        for grants in self.grants.values_mut() {
            grants.retain(|g| g.is_valid());
        }
    }

    /// Get all valid grants
    pub fn valid_grants(&self) -> Vec<&CapabilityGrant> {
        self.grants
            .values()
            .flat_map(|grants| grants.iter())
            .filter(|g| g.is_valid())
            .collect()
    }

    /// Check if the set is empty (has no valid grants)
    pub fn is_empty(&self) -> bool {
        self.valid_grants().is_empty()
    }
}

impl Default for CapabilitySet {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// DEFAULT CAPABILITY SETS
// ═══════════════════════════════════════════════════════════════════════════

/// Minimal capabilities granted to all sandboxes
pub const MINIMAL_CAPABILITIES: &[CapabilityType] = &[
    CapabilityType::SensorTime,
    CapabilityType::SensorRandom,
    CapabilityType::ActuatorLog,
];

/// Standard capabilities for network-enabled spirits
pub const NETWORK_SPIRIT_CAPABILITIES: &[CapabilityType] = &[
    CapabilityType::SensorTime,
    CapabilityType::SensorRandom,
    CapabilityType::ActuatorLog,
    CapabilityType::NetworkConnect,
    CapabilityType::StorageRead,
    CapabilityType::StorageWrite,
];

/// System spirit capabilities (unrestricted access)
pub const SYSTEM_SPIRIT_CAPABILITIES: &[CapabilityType] = &[CapabilityType::Unrestricted];

// ═══════════════════════════════════════════════════════════════════════════
// UTILITY FUNCTIONS
// ═══════════════════════════════════════════════════════════════════════════

/// Get current Unix timestamp in seconds
fn current_timestamp() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
}

// ═══════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_scope_covers() {
        assert!(CapabilityScope::Global.covers(&CapabilityScope::Global));
        assert!(CapabilityScope::Global.covers(&CapabilityScope::Sandboxed));
        assert!(CapabilityScope::Global.covers(&CapabilityScope::Peer));
        assert!(CapabilityScope::Global.covers(&CapabilityScope::Domain));

        assert!(!CapabilityScope::Sandboxed.covers(&CapabilityScope::Global));
        assert!(CapabilityScope::Sandboxed.covers(&CapabilityScope::Sandboxed));
        assert!(!CapabilityScope::Sandboxed.covers(&CapabilityScope::Peer));
    }

    #[test]
    fn test_capability_grant_validity() {
        let now = current_timestamp();
        let grant = CapabilityGrant::new(
            1,
            CapabilityType::NetworkConnect,
            CapabilityScope::Global,
            [0u8; 32],
            [1u8; 32],
            now,
            Some(now + 3600), // Expires in 1 hour
            [0u8; 64],
        );

        assert!(grant.is_valid_at(now));
        assert!(grant.is_valid_at(now + 1800)); // 30 minutes later
        assert!(!grant.is_valid_at(now + 3600)); // At expiry
        assert!(!grant.is_valid_at(now + 7200)); // After expiry
    }

    #[test]
    fn test_capability_grant_revoke() {
        let now = current_timestamp();
        let mut grant = CapabilityGrant::new(
            1,
            CapabilityType::NetworkConnect,
            CapabilityScope::Global,
            [0u8; 32],
            [1u8; 32],
            now,
            None,
            [0u8; 64],
        );

        assert!(grant.is_valid());
        grant.revoke();
        assert!(!grant.is_valid());
    }

    #[test]
    fn test_capability_set_has_capability() {
        let now = current_timestamp();
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

        let mut cap_set = CapabilitySet::new();
        cap_set.add_grant(grant);

        assert!(cap_set.has_capability(CapabilityType::NetworkConnect, CapabilityScope::Global));
        assert!(cap_set.has_capability(CapabilityType::NetworkConnect, CapabilityScope::Sandboxed));
        assert!(!cap_set.has_capability(CapabilityType::NetworkListen, CapabilityScope::Global));
    }

    #[test]
    fn test_unrestricted_capability() {
        let now = current_timestamp();
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

        let mut cap_set = CapabilitySet::new();
        cap_set.add_grant(grant);

        // Unrestricted should grant access to any capability
        assert!(cap_set.has_capability(CapabilityType::NetworkConnect, CapabilityScope::Global));
        assert!(cap_set.has_capability(CapabilityType::StorageWrite, CapabilityScope::Sandboxed));
        assert!(cap_set.has_capability(CapabilityType::SpawnSandbox, CapabilityScope::Global));
    }

    #[test]
    fn test_effective_scope() {
        let now = current_timestamp();
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

        let mut cap_set = CapabilitySet::new();
        cap_set.add_grant(grant);

        assert_eq!(
            cap_set.effective_scope(CapabilityType::NetworkConnect),
            Some(CapabilityScope::Global)
        );
        assert_eq!(cap_set.effective_scope(CapabilityType::NetworkListen), None);
    }

    #[test]
    fn test_minimal_capabilities() {
        assert_eq!(MINIMAL_CAPABILITIES.len(), 3);
        assert!(MINIMAL_CAPABILITIES.contains(&CapabilityType::SensorTime));
        assert!(MINIMAL_CAPABILITIES.contains(&CapabilityType::SensorRandom));
        assert!(MINIMAL_CAPABILITIES.contains(&CapabilityType::ActuatorLog));
    }

    #[test]
    fn test_network_spirit_capabilities() {
        assert_eq!(NETWORK_SPIRIT_CAPABILITIES.len(), 6);
        assert!(NETWORK_SPIRIT_CAPABILITIES.contains(&CapabilityType::NetworkConnect));
        assert!(NETWORK_SPIRIT_CAPABILITIES.contains(&CapabilityType::StorageRead));
        assert!(NETWORK_SPIRIT_CAPABILITIES.contains(&CapabilityType::StorageWrite));
    }
}
