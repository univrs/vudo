//! Spirit Manifest Module
//!
//! Defines the manifest format for Spirit packages.
//! Manifests are TOML files containing metadata, dependencies, and capabilities.
//!
//! # Features
//!
//! - **Parsing**: Load manifests from TOML or JSON format
//! - **Validation**: Validate manifest content and dependencies
//! - **Signing**: Ed25519 signing and verification
//! - **Serialization**: Serialize to TOML or JSON
//! - **File I/O**: Read/write manifests from/to files
//!
//! # Example
//!
//! ```rust
//! use spirit_runtime::manifest::{Manifest, ManifestBuilder, Capability};
//! use spirit_runtime::version::SemVer;
//!
//! // Create a manifest using the builder
//! let manifest = ManifestBuilder::new("my-spirit", SemVer::new(1, 0, 0), "a".repeat(64))
//!     .description("A sample Spirit")
//!     .license("MIT")
//!     .capability(Capability::SensorTime)
//!     .build();
//! ```

use crate::dependency::Dependency;
use crate::pricing::PricingModel;
use crate::version::SemVer;
use ed25519_dalek::{Signature, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::path::Path;
use std::str::FromStr;

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

    /// Parse manifest from JSON string
    ///
    /// # Example
    ///
    /// ```rust
    /// use spirit_runtime::manifest::Manifest;
    ///
    /// let json = r#"{"name": "test", "version": {"major": 1, "minor": 0, "patch": 0}, "author": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}"#;
    /// let manifest = Manifest::from_json(json).unwrap();
    /// assert_eq!(manifest.name, "test");
    /// ```
    pub fn from_json(content: &str) -> Result<Self, ManifestError> {
        serde_json::from_str(content).map_err(|e| ManifestError::ParseError(e.to_string()))
    }

    /// Serialize manifest to pretty-printed JSON string
    ///
    /// # Example
    ///
    /// ```rust
    /// use spirit_runtime::manifest::Manifest;
    /// use spirit_runtime::version::SemVer;
    ///
    /// let manifest = Manifest::new("test", SemVer::new(1, 0, 0), "a".repeat(64));
    /// let json = manifest.to_json().unwrap();
    /// assert!(json.contains("\"name\": \"test\""));
    /// ```
    pub fn to_json(&self) -> Result<String, ManifestError> {
        serde_json::to_string_pretty(self).map_err(|e| ManifestError::SerializeError(e.to_string()))
    }

    /// Read manifest from a file (supports .toml and .json extensions)
    ///
    /// The file format is determined by the file extension:
    /// - `.toml` files are parsed as TOML
    /// - `.json` files are parsed as JSON
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use spirit_runtime::manifest::Manifest;
    ///
    /// let manifest = Manifest::from_file("spirit.toml").unwrap();
    /// ```
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, ManifestError> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path).map_err(|e| ManifestError::IoError {
            path: path.display().to_string(),
            message: e.to_string(),
        })?;

        // Determine format from extension
        match path.extension().and_then(|e| e.to_str()) {
            Some("json") => Self::from_json(&content),
            Some("toml") | None => Self::from_toml(&content),
            Some(ext) => Err(ManifestError::ParseError(format!(
                "Unsupported file extension: {}",
                ext
            ))),
        }
    }

    /// Write manifest to a file (format determined by extension)
    ///
    /// The file format is determined by the file extension:
    /// - `.toml` files are written as TOML
    /// - `.json` files are written as JSON
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use spirit_runtime::manifest::Manifest;
    /// use spirit_runtime::version::SemVer;
    ///
    /// let manifest = Manifest::new("test", SemVer::new(1, 0, 0), "a".repeat(64));
    /// manifest.to_file("spirit.toml").unwrap();
    /// ```
    pub fn to_file(&self, path: impl AsRef<Path>) -> Result<(), ManifestError> {
        let path = path.as_ref();

        // Determine format from extension
        let content = match path.extension().and_then(|e| e.to_str()) {
            Some("json") => self.to_json()?,
            Some("toml") | None => self.to_toml()?,
            Some(ext) => {
                return Err(ManifestError::SerializeError(format!(
                    "Unsupported file extension: {}",
                    ext
                )))
            }
        };

        std::fs::write(path, content).map_err(|e| ManifestError::IoError {
            path: path.display().to_string(),
            message: e.to_string(),
        })
    }

    /// Validate manifest content
    ///
    /// Checks:
    /// - Name is non-empty, <= 128 chars, alphanumeric with dash/underscore
    /// - Author is 64 hex characters (32-byte Ed25519 public key)
    /// - Signature (if present) is 128 hex characters (64-byte Ed25519 signature)
    /// - All dependencies have valid version syntax
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

        // Validate dependencies
        self.validate_dependencies()?;

        Ok(())
    }

    /// Validate all dependency version requirements
    ///
    /// Checks that each dependency has valid version syntax.
    ///
    /// # Example
    ///
    /// ```rust
    /// use spirit_runtime::manifest::Manifest;
    /// use spirit_runtime::version::SemVer;
    /// use spirit_runtime::dependency::Dependency;
    ///
    /// let mut manifest = Manifest::new("test", SemVer::new(1, 0, 0), "a".repeat(64));
    /// manifest.add_dependency("dep1", Dependency::new("^1.0.0"));
    /// assert!(manifest.validate_dependencies().is_ok());
    /// ```
    pub fn validate_dependencies(&self) -> Result<(), ManifestError> {
        for (name, dep) in &self.dependencies {
            // Local and git dependencies don't require version validation
            if dep.is_local() || dep.is_git() {
                continue;
            }

            // Registry dependencies need valid version requirement
            if !dep.version.is_empty() {
                dep.version_requirement()
                    .map_err(|e| ManifestError::InvalidDependency {
                        name: name.clone(),
                        reason: e.to_string(),
                    })?;
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
    ///
    /// Excludes the signature field itself. The hash is computed over:
    /// - name
    /// - version
    /// - author
    /// - description (if present)
    /// - capabilities
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

    /// Sign the manifest with an Ed25519 private key
    ///
    /// Computes the content hash and signs it, returning the hex-encoded signature.
    /// The signature is NOT automatically stored in the manifest.
    ///
    /// # Example
    ///
    /// ```rust
    /// use spirit_runtime::manifest::Manifest;
    /// use spirit_runtime::version::SemVer;
    /// use ed25519_dalek::SigningKey;
    /// use rand::rngs::OsRng;
    ///
    /// // Generate a keypair
    /// let signing_key = SigningKey::generate(&mut OsRng);
    /// let public_key = signing_key.verifying_key();
    /// let author = hex::encode(public_key.as_bytes());
    ///
    /// let mut manifest = Manifest::new("test", SemVer::new(1, 0, 0), author);
    /// let signature = manifest.sign(&signing_key).unwrap();
    /// manifest.signature = Some(signature);
    /// ```
    pub fn sign(&self, private_key: &SigningKey) -> Result<String, ManifestError> {
        use ed25519_dalek::Signer;

        let hash = self.content_hash();
        let signature = private_key.sign(&hash);
        Ok(hex::encode(signature.to_bytes()))
    }

    /// Verify the manifest signature against the author's public key
    ///
    /// The author field must contain the hex-encoded Ed25519 public key.
    /// The signature field must contain the hex-encoded signature.
    ///
    /// # Example
    ///
    /// ```rust
    /// use spirit_runtime::manifest::Manifest;
    /// use spirit_runtime::version::SemVer;
    /// use ed25519_dalek::SigningKey;
    /// use rand::rngs::OsRng;
    ///
    /// // Generate a keypair
    /// let signing_key = SigningKey::generate(&mut OsRng);
    /// let public_key = signing_key.verifying_key();
    /// let author = hex::encode(public_key.as_bytes());
    ///
    /// let mut manifest = Manifest::new("test", SemVer::new(1, 0, 0), author);
    /// let signature = manifest.sign(&signing_key).unwrap();
    /// manifest.signature = Some(signature);
    ///
    /// // Verification should succeed
    /// assert!(manifest.verify().is_ok());
    /// ```
    pub fn verify(&self) -> Result<(), ManifestError> {
        // Get the signature
        let signature_hex = self
            .signature
            .as_ref()
            .ok_or_else(|| ManifestError::SignatureError("No signature present".to_string()))?;

        // Decode the signature
        let signature_bytes = hex::decode(signature_hex)
            .map_err(|e| ManifestError::CryptoError(format!("Invalid signature hex: {}", e)))?;

        let signature_array: [u8; 64] = signature_bytes
            .try_into()
            .map_err(|_| ManifestError::CryptoError("Signature must be 64 bytes".to_string()))?;

        let signature = Signature::from_bytes(&signature_array);

        // Decode the public key from author
        let public_key_bytes = hex::decode(&self.author)
            .map_err(|e| ManifestError::CryptoError(format!("Invalid author hex: {}", e)))?;

        let public_key_array: [u8; 32] = public_key_bytes.try_into().map_err(|_| {
            ManifestError::CryptoError("Author public key must be 32 bytes".to_string())
        })?;

        let public_key = VerifyingKey::from_bytes(&public_key_array)
            .map_err(|e| ManifestError::CryptoError(format!("Invalid public key: {}", e)))?;

        // Verify the signature
        let hash = self.content_hash();
        public_key
            .verify(&hash, &signature)
            .map_err(|_| ManifestError::SignatureError("Signature verification failed".to_string()))
    }
}

/// Builder for creating Manifest instances with a fluent API
///
/// # Example
///
/// ```rust
/// use spirit_runtime::manifest::{ManifestBuilder, Capability};
/// use spirit_runtime::version::SemVer;
/// use spirit_runtime::dependency::Dependency;
///
/// let manifest = ManifestBuilder::new("my-spirit", SemVer::new(1, 0, 0), "a".repeat(64))
///     .description("A sample Spirit package")
///     .license("MIT")
///     .repository("https://github.com/example/my-spirit")
///     .capability(Capability::SensorTime)
///     .capability(Capability::ActuatorLog)
///     .dependency("other-spirit", Dependency::new("^1.0"))
///     .build();
///
/// assert_eq!(manifest.name, "my-spirit");
/// assert_eq!(manifest.license, Some("MIT".to_string()));
/// assert_eq!(manifest.capabilities.len(), 2);
/// ```
#[derive(Debug, Clone)]
pub struct ManifestBuilder {
    manifest: Manifest,
}

impl ManifestBuilder {
    /// Create a new ManifestBuilder with required fields
    ///
    /// # Arguments
    ///
    /// * `name` - Package name (unique identifier)
    /// * `version` - Semantic version
    /// * `author` - Author's Ed25519 public key (hex-encoded)
    pub fn new(name: impl Into<String>, version: SemVer, author: impl Into<String>) -> Self {
        Self {
            manifest: Manifest::new(name, version, author),
        }
    }

    /// Set the description
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.manifest.description = Some(description.into());
        self
    }

    /// Set the license (SPDX identifier)
    pub fn license(mut self, license: impl Into<String>) -> Self {
        self.manifest.license = Some(license.into());
        self
    }

    /// Set the repository URL
    pub fn repository(mut self, repository: impl Into<String>) -> Self {
        self.manifest.repository = Some(repository.into());
        self
    }

    /// Add a capability requirement
    pub fn capability(mut self, capability: Capability) -> Self {
        self.manifest.add_capability(capability);
        self
    }

    /// Add multiple capabilities at once
    pub fn capabilities(mut self, capabilities: impl IntoIterator<Item = Capability>) -> Self {
        for cap in capabilities {
            self.manifest.add_capability(cap);
        }
        self
    }

    /// Add a dependency
    pub fn dependency(mut self, name: impl Into<String>, dependency: Dependency) -> Self {
        self.manifest.add_dependency(name, dependency);
        self
    }

    /// Set the pricing model
    pub fn pricing(mut self, pricing: PricingModel) -> Self {
        self.manifest.pricing = pricing;
        self
    }

    /// Set the signature
    pub fn signature(mut self, signature: impl Into<String>) -> Self {
        self.manifest.signature = Some(signature.into());
        self
    }

    /// Build the manifest
    pub fn build(self) -> Manifest {
        self.manifest
    }

    /// Build and validate the manifest
    ///
    /// Returns an error if validation fails.
    pub fn build_validated(self) -> Result<Manifest, ManifestError> {
        let manifest = self.build();
        manifest.validate()?;
        Ok(manifest)
    }
}

/// Capability requirements for Spirits
///
/// Maps to vudo_vm::CapabilityType
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    // Network capabilities
    /// Allow listening for incoming network connections
    NetworkListen,
    /// Allow outgoing network connections
    NetworkConnect,
    /// Allow broadcast/multicast network operations
    NetworkBroadcast,

    // Storage capabilities
    /// Allow reading from persistent storage
    StorageRead,
    /// Allow writing to persistent storage
    StorageWrite,
    /// Allow deleting from persistent storage
    StorageDelete,

    // Compute capabilities
    /// Allow spawning child sandboxes
    SpawnSandbox,
    /// Allow cross-sandbox function calls
    CrossSandboxCall,

    // Sensor capabilities
    /// Allow reading the current time
    SensorTime,
    /// Allow generating random numbers
    SensorRandom,
    /// Allow reading environment variables
    SensorEnvironment,

    // Actuator capabilities
    /// Allow logging output
    ActuatorLog,
    /// Allow sending notifications
    ActuatorNotify,
    /// Allow credit/billing operations
    ActuatorCredit,
}

impl Capability {
    /// Get all available capabilities
    pub fn all() -> Vec<Capability> {
        vec![
            Capability::NetworkListen,
            Capability::NetworkConnect,
            Capability::NetworkBroadcast,
            Capability::StorageRead,
            Capability::StorageWrite,
            Capability::StorageDelete,
            Capability::SpawnSandbox,
            Capability::CrossSandboxCall,
            Capability::SensorTime,
            Capability::SensorRandom,
            Capability::SensorEnvironment,
            Capability::ActuatorLog,
            Capability::ActuatorNotify,
            Capability::ActuatorCredit,
        ]
    }
}

impl fmt::Display for Capability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Capability::NetworkListen => "network_listen",
            Capability::NetworkConnect => "network_connect",
            Capability::NetworkBroadcast => "network_broadcast",
            Capability::StorageRead => "storage_read",
            Capability::StorageWrite => "storage_write",
            Capability::StorageDelete => "storage_delete",
            Capability::SpawnSandbox => "spawn_sandbox",
            Capability::CrossSandboxCall => "cross_sandbox_call",
            Capability::SensorTime => "sensor_time",
            Capability::SensorRandom => "sensor_random",
            Capability::SensorEnvironment => "sensor_environment",
            Capability::ActuatorLog => "actuator_log",
            Capability::ActuatorNotify => "actuator_notify",
            Capability::ActuatorCredit => "actuator_credit",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for Capability {
    type Err = ManifestError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "network_listen" => Ok(Capability::NetworkListen),
            "network_connect" => Ok(Capability::NetworkConnect),
            "network_broadcast" => Ok(Capability::NetworkBroadcast),
            "storage_read" => Ok(Capability::StorageRead),
            "storage_write" => Ok(Capability::StorageWrite),
            "storage_delete" => Ok(Capability::StorageDelete),
            "spawn_sandbox" => Ok(Capability::SpawnSandbox),
            "cross_sandbox_call" => Ok(Capability::CrossSandboxCall),
            "sensor_time" => Ok(Capability::SensorTime),
            "sensor_random" => Ok(Capability::SensorRandom),
            "sensor_environment" => Ok(Capability::SensorEnvironment),
            "actuator_log" => Ok(Capability::ActuatorLog),
            "actuator_notify" => Ok(Capability::ActuatorNotify),
            "actuator_credit" => Ok(Capability::ActuatorCredit),
            _ => Err(ManifestError::ParseError(format!(
                "Unknown capability: {}",
                s
            ))),
        }
    }
}

/// Manifest parsing/validation errors
#[derive(Debug, Clone, thiserror::Error)]
pub enum ManifestError {
    /// Error parsing manifest content
    #[error("Parse error: {0}")]
    ParseError(String),

    /// Error serializing manifest
    #[error("Serialize error: {0}")]
    SerializeError(String),

    /// Invalid package name
    #[error("Invalid name: {0}")]
    InvalidName(String),

    /// Invalid author public key
    #[error("Invalid author: {0}")]
    InvalidAuthor(String),

    /// Invalid signature format
    #[error("Invalid signature: {0}")]
    InvalidSignature(String),

    /// Missing required field
    #[error("Missing field: {0}")]
    MissingField(String),

    /// Signature verification error
    #[error("Signature error: {0}")]
    SignatureError(String),

    /// Cryptographic operation error
    #[error("Crypto error: {0}")]
    CryptoError(String),

    /// File I/O error
    #[error("I/O error for {path}: {message}")]
    IoError {
        /// Path that caused the error
        path: String,
        /// Error message
        message: String,
    },

    /// Invalid dependency specification
    #[error("Invalid dependency '{name}': {reason}")]
    InvalidDependency {
        /// Dependency name
        name: String,
        /// Reason for invalidity
        reason: String,
    },
}

// Implement PartialEq manually since thiserror doesn't derive it
impl PartialEq for ManifestError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ManifestError::ParseError(a), ManifestError::ParseError(b)) => a == b,
            (ManifestError::SerializeError(a), ManifestError::SerializeError(b)) => a == b,
            (ManifestError::InvalidName(a), ManifestError::InvalidName(b)) => a == b,
            (ManifestError::InvalidAuthor(a), ManifestError::InvalidAuthor(b)) => a == b,
            (ManifestError::InvalidSignature(a), ManifestError::InvalidSignature(b)) => a == b,
            (ManifestError::MissingField(a), ManifestError::MissingField(b)) => a == b,
            (ManifestError::SignatureError(a), ManifestError::SignatureError(b)) => a == b,
            (ManifestError::CryptoError(a), ManifestError::CryptoError(b)) => a == b,
            (
                ManifestError::IoError {
                    path: p1,
                    message: m1,
                },
                ManifestError::IoError {
                    path: p2,
                    message: m2,
                },
            ) => p1 == p2 && m1 == m2,
            (
                ManifestError::InvalidDependency {
                    name: n1,
                    reason: r1,
                },
                ManifestError::InvalidDependency {
                    name: n2,
                    reason: r2,
                },
            ) => n1 == n2 && r1 == r2,
            _ => false,
        }
    }
}

impl Eq for ManifestError {}

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

    // ==================== New Tests ====================

    #[test]
    fn test_manifest_json_roundtrip() {
        let mut manifest = Manifest::new("json-test", SemVer::new(1, 2, 3), valid_author());
        manifest.description = Some("JSON roundtrip test".to_string());
        manifest.license = Some("MIT".to_string());
        manifest.add_capability(Capability::SensorTime);

        let json = manifest.to_json().unwrap();
        let parsed = Manifest::from_json(&json).unwrap();

        assert_eq!(parsed.name, manifest.name);
        assert_eq!(parsed.version, manifest.version);
        assert_eq!(parsed.description, manifest.description);
        assert_eq!(parsed.license, manifest.license);
        assert_eq!(parsed.capabilities, manifest.capabilities);
    }

    #[test]
    fn test_manifest_from_json() {
        let json = r#"{
            "name": "json-spirit",
            "version": {"major": 2, "minor": 1, "patch": 0},
            "author": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "description": "A JSON manifest",
            "capabilities": ["network_connect", "storage_read"]
        }"#;

        let manifest = Manifest::from_json(json).unwrap();
        assert_eq!(manifest.name, "json-spirit");
        assert_eq!(manifest.version, SemVer::new(2, 1, 0));
        assert_eq!(manifest.capabilities.len(), 2);
    }

    #[test]
    fn test_manifest_to_json_pretty() {
        let manifest = Manifest::new("pretty", SemVer::new(1, 0, 0), valid_author());
        let json = manifest.to_json().unwrap();

        // Pretty print should have newlines and indentation
        assert!(json.contains('\n'));
        assert!(json.contains("  ")); // indentation
        assert!(json.contains("\"name\": \"pretty\""));
    }

    #[test]
    fn test_manifest_file_roundtrip_toml() {
        let manifest = Manifest::new("file-test", SemVer::new(1, 0, 0), valid_author());

        let temp_dir = std::env::temp_dir();
        let toml_path = temp_dir.join("test_manifest.toml");

        manifest.to_file(&toml_path).unwrap();
        let loaded = Manifest::from_file(&toml_path).unwrap();

        assert_eq!(loaded.name, manifest.name);
        assert_eq!(loaded.version, manifest.version);

        // Cleanup
        std::fs::remove_file(&toml_path).ok();
    }

    #[test]
    fn test_manifest_file_roundtrip_json() {
        let manifest = Manifest::new("file-json", SemVer::new(2, 0, 0), valid_author());

        let temp_dir = std::env::temp_dir();
        let json_path = temp_dir.join("test_manifest.json");

        manifest.to_file(&json_path).unwrap();
        let loaded = Manifest::from_file(&json_path).unwrap();

        assert_eq!(loaded.name, manifest.name);
        assert_eq!(loaded.version, manifest.version);

        // Cleanup
        std::fs::remove_file(&json_path).ok();
    }

    #[test]
    fn test_manifest_file_not_found() {
        let result = Manifest::from_file("/nonexistent/path/manifest.toml");
        assert!(matches!(result, Err(ManifestError::IoError { .. })));
    }

    #[test]
    fn test_manifest_sign_and_verify() {
        use ed25519_dalek::SigningKey;
        use rand::rngs::OsRng;

        // Generate keypair
        let signing_key = SigningKey::generate(&mut OsRng);
        let public_key = signing_key.verifying_key();
        let author = hex::encode(public_key.as_bytes());

        // Create and sign manifest
        let mut manifest = Manifest::new("signed-spirit", SemVer::new(1, 0, 0), author);
        manifest.description = Some("Signed manifest".to_string());
        manifest.add_capability(Capability::SensorTime);

        let signature = manifest.sign(&signing_key).unwrap();
        manifest.signature = Some(signature);

        // Verify should succeed
        assert!(manifest.verify().is_ok());
    }

    #[test]
    fn test_manifest_verify_fails_no_signature() {
        let manifest = Manifest::new("unsigned", SemVer::new(1, 0, 0), valid_author());
        let result = manifest.verify();
        assert!(matches!(result, Err(ManifestError::SignatureError(_))));
    }

    #[test]
    fn test_manifest_verify_fails_wrong_key() {
        use ed25519_dalek::SigningKey;
        use rand::rngs::OsRng;

        // Generate two different keypairs
        let signing_key1 = SigningKey::generate(&mut OsRng);
        let signing_key2 = SigningKey::generate(&mut OsRng);
        let public_key2 = signing_key2.verifying_key();
        let author2 = hex::encode(public_key2.as_bytes());

        // Sign with key1 but claim author is key2
        let mut manifest = Manifest::new("wrong-key", SemVer::new(1, 0, 0), author2);
        let signature = manifest.sign(&signing_key1).unwrap();
        manifest.signature = Some(signature);

        // Verify should fail
        let result = manifest.verify();
        assert!(matches!(result, Err(ManifestError::SignatureError(_))));
    }

    #[test]
    fn test_manifest_verify_fails_tampered() {
        use ed25519_dalek::SigningKey;
        use rand::rngs::OsRng;

        // Generate keypair
        let signing_key = SigningKey::generate(&mut OsRng);
        let public_key = signing_key.verifying_key();
        let author = hex::encode(public_key.as_bytes());

        // Create and sign manifest
        let mut manifest = Manifest::new("tampered", SemVer::new(1, 0, 0), author);
        let signature = manifest.sign(&signing_key).unwrap();
        manifest.signature = Some(signature);

        // Tamper with the manifest
        manifest.name = "tampered-name".to_string();

        // Verify should fail
        let result = manifest.verify();
        assert!(matches!(result, Err(ManifestError::SignatureError(_))));
    }

    #[test]
    fn test_manifest_builder_basic() {
        let manifest =
            ManifestBuilder::new("builder-test", SemVer::new(1, 0, 0), valid_author()).build();

        assert_eq!(manifest.name, "builder-test");
        assert_eq!(manifest.version, SemVer::new(1, 0, 0));
    }

    #[test]
    fn test_manifest_builder_full() {
        let manifest = ManifestBuilder::new("full-builder", SemVer::new(2, 1, 0), valid_author())
            .description("A fully built manifest")
            .license("MIT")
            .repository("https://github.com/example/repo")
            .capability(Capability::SensorTime)
            .capability(Capability::ActuatorLog)
            .dependency("other", Dependency::new("^1.0"))
            .build();

        assert_eq!(manifest.name, "full-builder");
        assert_eq!(manifest.version, SemVer::new(2, 1, 0));
        assert_eq!(
            manifest.description,
            Some("A fully built manifest".to_string())
        );
        assert_eq!(manifest.license, Some("MIT".to_string()));
        assert_eq!(
            manifest.repository,
            Some("https://github.com/example/repo".to_string())
        );
        assert_eq!(manifest.capabilities.len(), 2);
        assert!(manifest.requires_capability(&Capability::SensorTime));
        assert!(manifest.requires_capability(&Capability::ActuatorLog));
        assert_eq!(manifest.dependencies.len(), 1);
        assert!(manifest.dependencies.contains_key("other"));
    }

    #[test]
    fn test_manifest_builder_capabilities_iter() {
        let caps = vec![
            Capability::NetworkConnect,
            Capability::StorageRead,
            Capability::SensorTime,
        ];

        let manifest = ManifestBuilder::new("caps-iter", SemVer::new(1, 0, 0), valid_author())
            .capabilities(caps)
            .build();

        assert_eq!(manifest.capabilities.len(), 3);
    }

    #[test]
    fn test_manifest_builder_validated() {
        let result =
            ManifestBuilder::new("valid", SemVer::new(1, 0, 0), valid_author()).build_validated();

        assert!(result.is_ok());
    }

    #[test]
    fn test_manifest_builder_validated_fails() {
        let result =
            ManifestBuilder::new("", SemVer::new(1, 0, 0), valid_author()).build_validated();

        assert!(matches!(result, Err(ManifestError::InvalidName(_))));
    }

    #[test]
    fn test_capability_from_str() {
        assert_eq!(
            "network_connect".parse::<Capability>().unwrap(),
            Capability::NetworkConnect
        );
        assert_eq!(
            "storage_write".parse::<Capability>().unwrap(),
            Capability::StorageWrite
        );
        assert_eq!(
            "sensor_time".parse::<Capability>().unwrap(),
            Capability::SensorTime
        );
        assert_eq!(
            "actuator_log".parse::<Capability>().unwrap(),
            Capability::ActuatorLog
        );
    }

    #[test]
    fn test_capability_from_str_case_insensitive() {
        assert_eq!(
            "NETWORK_CONNECT".parse::<Capability>().unwrap(),
            Capability::NetworkConnect
        );
        assert_eq!(
            "Network_Listen".parse::<Capability>().unwrap(),
            Capability::NetworkListen
        );
    }

    #[test]
    fn test_capability_from_str_invalid() {
        let result = "invalid_capability".parse::<Capability>();
        assert!(matches!(result, Err(ManifestError::ParseError(_))));
    }

    #[test]
    fn test_capability_display() {
        assert_eq!(Capability::NetworkConnect.to_string(), "network_connect");
        assert_eq!(Capability::NetworkListen.to_string(), "network_listen");
        assert_eq!(
            Capability::NetworkBroadcast.to_string(),
            "network_broadcast"
        );
        assert_eq!(Capability::StorageRead.to_string(), "storage_read");
        assert_eq!(Capability::StorageWrite.to_string(), "storage_write");
        assert_eq!(Capability::StorageDelete.to_string(), "storage_delete");
        assert_eq!(Capability::SpawnSandbox.to_string(), "spawn_sandbox");
        assert_eq!(
            Capability::CrossSandboxCall.to_string(),
            "cross_sandbox_call"
        );
        assert_eq!(Capability::SensorTime.to_string(), "sensor_time");
        assert_eq!(Capability::SensorRandom.to_string(), "sensor_random");
        assert_eq!(
            Capability::SensorEnvironment.to_string(),
            "sensor_environment"
        );
        assert_eq!(Capability::ActuatorLog.to_string(), "actuator_log");
        assert_eq!(Capability::ActuatorNotify.to_string(), "actuator_notify");
        assert_eq!(Capability::ActuatorCredit.to_string(), "actuator_credit");
    }

    #[test]
    fn test_capability_roundtrip_string() {
        for cap in Capability::all() {
            let s = cap.to_string();
            let parsed: Capability = s.parse().unwrap();
            assert_eq!(cap, parsed);
        }
    }

    #[test]
    fn test_capability_all() {
        let all = Capability::all();
        assert_eq!(all.len(), 14);
    }

    #[test]
    fn test_validate_dependencies_valid() {
        let mut manifest = Manifest::new("dep-test", SemVer::new(1, 0, 0), valid_author());
        manifest.add_dependency("dep1", Dependency::new("^1.0.0"));
        manifest.add_dependency("dep2", Dependency::new(">=2.0.0"));
        manifest.add_dependency("dep3", Dependency::new("*"));

        assert!(manifest.validate_dependencies().is_ok());
    }

    #[test]
    fn test_validate_dependencies_local() {
        let mut manifest = Manifest::new("local-dep", SemVer::new(1, 0, 0), valid_author());
        manifest.add_dependency("local", Dependency::from_path("../local-spirit"));

        // Local dependencies should pass validation (no version to check)
        assert!(manifest.validate_dependencies().is_ok());
    }

    #[test]
    fn test_validate_dependencies_git() {
        let mut manifest = Manifest::new("git-dep", SemVer::new(1, 0, 0), valid_author());
        manifest.add_dependency(
            "git-spirit",
            Dependency::from_git("https://github.com/example/spirit", None),
        );

        // Git dependencies should pass validation (no version to check)
        assert!(manifest.validate_dependencies().is_ok());
    }

    #[test]
    fn test_validate_dependencies_invalid_version() {
        let mut manifest = Manifest::new("bad-dep", SemVer::new(1, 0, 0), valid_author());
        manifest.add_dependency("bad", Dependency::new("invalid-version"));

        let result = manifest.validate_dependencies();
        assert!(matches!(
            result,
            Err(ManifestError::InvalidDependency { .. })
        ));
    }

    #[test]
    fn test_validate_includes_dependencies() {
        let mut manifest = Manifest::new("validate-all", SemVer::new(1, 0, 0), valid_author());
        manifest.add_dependency("bad", Dependency::new("not-a-version"));

        // Full validate should catch invalid dependencies
        let result = manifest.validate();
        assert!(matches!(
            result,
            Err(ManifestError::InvalidDependency { .. })
        ));
    }

    #[test]
    fn test_manifest_error_display() {
        let err = ManifestError::ParseError("test error".to_string());
        assert_eq!(err.to_string(), "Parse error: test error");

        let err = ManifestError::IoError {
            path: "/some/path".to_string(),
            message: "not found".to_string(),
        };
        assert_eq!(err.to_string(), "I/O error for /some/path: not found");

        let err = ManifestError::InvalidDependency {
            name: "dep".to_string(),
            reason: "bad version".to_string(),
        };
        assert_eq!(err.to_string(), "Invalid dependency 'dep': bad version");

        let err = ManifestError::SignatureError("no sig".to_string());
        assert_eq!(err.to_string(), "Signature error: no sig");

        let err = ManifestError::CryptoError("bad key".to_string());
        assert_eq!(err.to_string(), "Crypto error: bad key");
    }

    #[test]
    fn test_manifest_error_equality() {
        let err1 = ManifestError::ParseError("test".to_string());
        let err2 = ManifestError::ParseError("test".to_string());
        let err3 = ManifestError::ParseError("other".to_string());

        assert_eq!(err1, err2);
        assert_ne!(err1, err3);
    }

    #[test]
    fn test_content_hash_consistency() {
        let manifest1 = Manifest::new("hash-test", SemVer::new(1, 0, 0), valid_author());
        let manifest2 = Manifest::new("hash-test", SemVer::new(1, 0, 0), valid_author());

        // Same content should produce same hash
        assert_eq!(manifest1.content_hash(), manifest2.content_hash());
    }

    #[test]
    fn test_content_hash_changes_with_content() {
        let manifest1 = Manifest::new("hash1", SemVer::new(1, 0, 0), valid_author());
        let manifest2 = Manifest::new("hash2", SemVer::new(1, 0, 0), valid_author());

        // Different names should produce different hash
        assert_ne!(manifest1.content_hash(), manifest2.content_hash());
    }

    #[test]
    fn test_signature_hex_length() {
        use ed25519_dalek::SigningKey;
        use rand::rngs::OsRng;

        let signing_key = SigningKey::generate(&mut OsRng);
        let public_key = signing_key.verifying_key();
        let author = hex::encode(public_key.as_bytes());

        let manifest = Manifest::new("sig-len", SemVer::new(1, 0, 0), author);
        let signature = manifest.sign(&signing_key).unwrap();

        // Ed25519 signature is 64 bytes = 128 hex chars
        assert_eq!(signature.len(), 128);
    }
}
