//! Spirit Manifest Module
//!
//! Defines the manifest format for Spirit packages.
//! Manifests are TOML files containing metadata, dependencies, and capabilities.

use crate::dependency::Dependency;
use crate::pricing::PricingModel;
use crate::version::SemVer;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Spirit manifest - metadata for a Spirit package
///
/// A manifest contains:
/// - Package identity (name, version, author)
/// - Dependencies on other Spirits
/// - Required capabilities for execution
/// - Pricing model for credit consumption
/// - Ed25519 signature for authenticity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    /// Package name (unique identifier)
    pub name: String,

    /// Semantic version
    pub version: SemVer,

    /// Author's Ed25519 public key (32 bytes, hex-encoded)
    pub author: String,

    /// Optional human-readable description
    pub description: Option<String>,

    /// Optional license identifier (SPDX)
    pub license: Option<String>,

    /// Optional repository URL
    pub repository: Option<String>,

    /// Required capabilities for execution
    #[serde(default)]
    pub capabilities: Vec<Capability>,

    /// Dependencies on other Spirits
    #[serde(default)]
    pub dependencies: HashMap<String, Dependency>,

    /// Pricing model for credit consumption
    #[serde(default)]
    pub pricing: PricingModel,

    /// Ed25519 signature over manifest content (hex-encoded)
    pub signature: Option<String>,
}

impl Manifest {
    /// Create a new manifest with minimal required fields
    pub fn new(name: impl Into<String>, version: SemVer, author: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version,
            author: author.into(),
            description: None,
            license: None,
            repository: None,
            capabilities: Vec::new(),
            dependencies: HashMap::new(),
            pricing: PricingModel::default(),
            signature: None,
        }
    }

    /// Parse manifest from TOML string
    pub fn from_toml(content: &str) -> Result<Self, ManifestError> {
        toml::from_str(content).map_err(|e| ManifestError::ParseError(e.to_string()))
    }

    /// Serialize manifest to TOML string
    pub fn to_toml(&self) -> Result<String, ManifestError> {
        toml::to_string_pretty(self).map_err(|e| ManifestError::SerializeError(e.to_string()))
    }

    /// Validate manifest content
    pub fn validate(&self) -> Result<(), ManifestError> {
        // Name validation
        if self.name.is_empty() {
            return Err(ManifestError::InvalidName(
                "Name cannot be empty".to_string(),
            ));
        }
        if self.name.len() > 128 {
            return Err(ManifestError::InvalidName(
                "Name too long (max 128 chars)".to_string(),
            ));
        }
        if !self
            .name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err(ManifestError::InvalidName(
                "Name must contain only alphanumeric, dash, or underscore".to_string(),
            ));
        }

        // Author validation (should be 64 hex chars = 32 bytes)
        if self.author.len() != 64 {
            return Err(ManifestError::InvalidAuthor(
                "Author must be 64 hex characters (32 bytes Ed25519 public key)".to_string(),
            ));
        }
        if !self.author.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(ManifestError::InvalidAuthor(
                "Author must be hex-encoded".to_string(),
            ));
        }

        // Signature validation (if present)
        if let Some(ref sig) = self.signature {
            if sig.len() != 128 {
                return Err(ManifestError::InvalidSignature(
                    "Signature must be 128 hex characters (64 bytes Ed25519 signature)".to_string(),
                ));
            }
            if !sig.chars().all(|c| c.is_ascii_hexdigit()) {
                return Err(ManifestError::InvalidSignature(
                    "Signature must be hex-encoded".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Add a capability requirement
    pub fn add_capability(&mut self, capability: Capability) {
        if !self.capabilities.contains(&capability) {
            self.capabilities.push(capability);
        }
    }

    /// Add a dependency
    pub fn add_dependency(&mut self, name: impl Into<String>, dependency: Dependency) {
        self.dependencies.insert(name.into(), dependency);
    }

    /// Check if manifest requires a specific capability
    pub fn requires_capability(&self, cap: &Capability) -> bool {
        self.capabilities.contains(cap)
    }

    /// Get the hash of manifest content for signing
    /// Excludes the signature field itself
    pub fn content_hash(&self) -> Vec<u8> {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        hasher.update(self.name.as_bytes());
        hasher.update(self.version.to_string().as_bytes());
        hasher.update(self.author.as_bytes());

        if let Some(ref desc) = self.description {
            hasher.update(desc.as_bytes());
        }

        for cap in &self.capabilities {
            hasher.update(format!("{:?}", cap).as_bytes());
        }

        hasher.finalize().to_vec()
    }
}

/// Capability requirements for Spirits
///
/// Maps to vudo_vm::CapabilityType
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
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

    // Sensor capabilities
    SensorTime,
    SensorRandom,
    SensorEnvironment,

    // Actuator capabilities
    ActuatorLog,
    ActuatorNotify,
    ActuatorCredit,
}

/// Manifest parsing/validation errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManifestError {
    ParseError(String),
    SerializeError(String),
    InvalidName(String),
    InvalidAuthor(String),
    InvalidSignature(String),
    MissingField(String),
}

impl std::fmt::Display for ManifestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ManifestError::ParseError(e) => write!(f, "Parse error: {}", e),
            ManifestError::SerializeError(e) => write!(f, "Serialize error: {}", e),
            ManifestError::InvalidName(e) => write!(f, "Invalid name: {}", e),
            ManifestError::InvalidAuthor(e) => write!(f, "Invalid author: {}", e),
            ManifestError::InvalidSignature(e) => write!(f, "Invalid signature: {}", e),
            ManifestError::MissingField(e) => write!(f, "Missing field: {}", e),
        }
    }
}

impl std::error::Error for ManifestError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_author() -> String {
        "a".repeat(64) // 64 hex chars
    }

    #[test]
    fn test_manifest_new() {
        let manifest = Manifest::new("test-spirit", SemVer::new(1, 0, 0), valid_author());
        assert_eq!(manifest.name, "test-spirit");
        assert_eq!(manifest.version, SemVer::new(1, 0, 0));
    }

    #[test]
    fn test_manifest_validate_valid() {
        let manifest = Manifest::new("test-spirit", SemVer::new(1, 0, 0), valid_author());
        assert!(manifest.validate().is_ok());
    }

    #[test]
    fn test_manifest_validate_empty_name() {
        let manifest = Manifest::new("", SemVer::new(1, 0, 0), valid_author());
        assert!(matches!(
            manifest.validate(),
            Err(ManifestError::InvalidName(_))
        ));
    }

    #[test]
    fn test_manifest_validate_invalid_author() {
        let manifest = Manifest::new("test", SemVer::new(1, 0, 0), "short");
        assert!(matches!(
            manifest.validate(),
            Err(ManifestError::InvalidAuthor(_))
        ));
    }

    #[test]
    fn test_manifest_toml_roundtrip() {
        let mut manifest = Manifest::new("hello-world", SemVer::new(0, 1, 0), valid_author());
        manifest.description = Some("A test Spirit".to_string());
        manifest.add_capability(Capability::SensorTime);
        manifest.add_capability(Capability::ActuatorLog);

        let toml = manifest.to_toml().unwrap();
        let parsed = Manifest::from_toml(&toml).unwrap();

        assert_eq!(parsed.name, manifest.name);
        assert_eq!(parsed.version, manifest.version);
        assert_eq!(parsed.capabilities.len(), 2);
    }

    #[test]
    fn test_manifest_capabilities() {
        let mut manifest = Manifest::new("test", SemVer::new(1, 0, 0), valid_author());
        manifest.add_capability(Capability::NetworkConnect);

        assert!(manifest.requires_capability(&Capability::NetworkConnect));
        assert!(!manifest.requires_capability(&Capability::StorageWrite));
    }

    #[test]
    fn test_manifest_from_toml() {
        let toml = r#"
name = "example-spirit"
version = { major = 1, minor = 0, patch = 0 }
author = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
description = "An example Spirit"
capabilities = ["sensor_time", "actuator_log"]

[dependencies]

[pricing]
base_cost = 100
per_fuel_cost = 1
"#;

        let manifest = Manifest::from_toml(toml).unwrap();
        assert_eq!(manifest.name, "example-spirit");
        assert_eq!(manifest.capabilities.len(), 2);
    }
}
