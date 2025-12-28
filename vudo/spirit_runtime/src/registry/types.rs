//! Registry types for Spirit package management
//!
//! This module defines the core types for the Spirit registry system:
//! - Registry index structure
//! - Installed spirit entries
//! - Search queries and results
//! - Error types

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::manifest::Manifest;

// ═══════════════════════════════════════════════════════════════════════════
// REGISTRY CONFIGURATION
// ═══════════════════════════════════════════════════════════════════════════

/// Registry configuration
///
/// Controls signature verification and trust policies for Spirit installation.
#[derive(Debug, Clone, Default)]
pub struct RegistryConfig {
    /// Require all spirits to be signed
    pub require_signatures: bool,
    /// Path to trusted keys directory
    pub trusted_keys_dir: Option<PathBuf>,
    /// Allow unsigned spirits from these authors
    pub unsigned_allowed_authors: Vec<String>,
}

// ═══════════════════════════════════════════════════════════════════════════
// REGISTRY INDEX
// ═══════════════════════════════════════════════════════════════════════════

/// Registry index containing all installed spirits
///
/// The index is persisted to `index.json` in the registry root directory.
/// It provides fast lookup of installed spirits without scanning the filesystem.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RegistryIndex {
    /// Schema version for forward compatibility
    pub schema_version: u32,
    /// Installed spirits
    pub spirits: Vec<InstalledSpirit>,
}

impl RegistryIndex {
    /// Create a new empty registry index
    pub fn new() -> Self {
        Self {
            schema_version: 1,
            spirits: Vec::new(),
        }
    }

    /// Find a spirit by name
    pub fn find(&self, name: &str) -> Option<&InstalledSpirit> {
        self.spirits.iter().find(|s| s.name == name)
    }

    /// Find a spirit by name (mutable)
    pub fn find_mut(&mut self, name: &str) -> Option<&mut InstalledSpirit> {
        self.spirits.iter_mut().find(|s| s.name == name)
    }

    /// Check if a spirit is installed
    pub fn contains(&self, name: &str) -> bool {
        self.spirits.iter().any(|s| s.name == name)
    }

    /// Check if a specific version is installed
    pub fn contains_version(&self, name: &str, version: &str) -> bool {
        self.spirits
            .iter()
            .find(|s| s.name == name)
            .map(|s| s.versions.contains(&version.to_string()))
            .unwrap_or(false)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// INSTALLED SPIRIT
// ═══════════════════════════════════════════════════════════════════════════

/// An installed spirit entry in the index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledSpirit {
    /// Spirit name (unique identifier)
    pub name: String,
    /// Available versions (semver strings)
    pub versions: Vec<String>,
    /// Latest/default version
    pub latest: String,
    /// Installation timestamp (Unix epoch seconds)
    pub installed_at: u64,
    /// Installation source
    pub source: InstallSource,
}

impl InstalledSpirit {
    /// Check if a specific version is installed
    pub fn has_version(&self, version: &str) -> bool {
        self.versions.contains(&version.to_string())
    }

    /// Add a version to the installed list
    pub fn add_version(&mut self, version: String) {
        if !self.versions.contains(&version) {
            self.versions.push(version.clone());
            self.latest = version;
        }
    }

    /// Remove a version from the installed list
    pub fn remove_version(&mut self, version: &str) {
        self.versions.retain(|v| v != version);
        if self.latest == version && !self.versions.is_empty() {
            self.latest = self.versions.last().cloned().unwrap_or_default();
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// INSTALL SOURCE
// ═══════════════════════════════════════════════════════════════════════════

/// Where a spirit was installed from
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum InstallSource {
    /// Local filesystem path
    Local { path: PathBuf },
    /// Remote URL
    Remote { url: String },
    /// Built from source
    Built { source_path: PathBuf },
}

impl Default for InstallSource {
    fn default() -> Self {
        InstallSource::Local {
            path: PathBuf::new(),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// SEARCH TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// Query for searching spirits
#[derive(Debug, Clone, Default)]
pub struct SpiritQuery {
    /// Name pattern (supports partial matching)
    pub name: Option<String>,
    /// Required capabilities
    pub capabilities: Vec<String>,
    /// Author filter
    pub author: Option<String>,
    /// Version constraint
    pub version: Option<String>,
}

impl SpiritQuery {
    /// Create a new empty query
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by name pattern
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Filter by author
    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.author = Some(author.into());
        self
    }

    /// Require a capability
    pub fn with_capability(mut self, cap: impl Into<String>) -> Self {
        self.capabilities.push(cap.into());
        self
    }

    /// Filter by version
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Check if query is empty (matches everything)
    pub fn is_empty(&self) -> bool {
        self.name.is_none()
            && self.capabilities.is_empty()
            && self.author.is_none()
            && self.version.is_none()
    }
}

/// Result of a spirit search
#[derive(Debug, Clone)]
pub struct SpiritSearchResult {
    /// Spirit name
    pub name: String,
    /// Version found
    pub version: String,
    /// Full manifest
    pub manifest: Manifest,
    /// Path to spirit directory
    pub path: PathBuf,
}

// ═══════════════════════════════════════════════════════════════════════════
// ERRORS
// ═══════════════════════════════════════════════════════════════════════════

/// Registry error types
#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    #[error("Spirit not found: {0}")]
    NotFound(String),

    #[error("Version not found: {name}@{version}")]
    VersionNotFound { name: String, version: String },

    #[error("Spirit already installed: {name}@{version}")]
    AlreadyInstalled { name: String, version: String },

    #[error("Invalid manifest: {0}")]
    InvalidManifest(String),

    #[error("Invalid source path: {0}")]
    InvalidSource(String),

    #[error("Missing WASM file: {0}")]
    MissingWasm(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("TOML error: {0}")]
    Toml(String),

    #[error("Invalid signature for spirit '{spirit}': {reason}")]
    InvalidSignature { spirit: String, reason: String },

    #[error("Spirit '{spirit}' is not signed")]
    UnsignedSpirit { spirit: String },

    #[error("Author key not found: {author}")]
    AuthorKeyNotFound { author: String },
}

// ═══════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_index_new() {
        let index = RegistryIndex::new();
        assert_eq!(index.schema_version, 1);
        assert!(index.spirits.is_empty());
    }

    #[test]
    fn test_registry_index_find() {
        let mut index = RegistryIndex::new();
        index.spirits.push(InstalledSpirit {
            name: "test-spirit".to_string(),
            versions: vec!["0.1.0".to_string()],
            latest: "0.1.0".to_string(),
            installed_at: 0,
            source: InstallSource::default(),
        });

        assert!(index.find("test-spirit").is_some());
        assert!(index.find("nonexistent").is_none());
    }

    #[test]
    fn test_installed_spirit_versions() {
        let mut spirit = InstalledSpirit {
            name: "test".to_string(),
            versions: vec!["0.1.0".to_string()],
            latest: "0.1.0".to_string(),
            installed_at: 0,
            source: InstallSource::default(),
        };

        assert!(spirit.has_version("0.1.0"));
        assert!(!spirit.has_version("0.2.0"));

        spirit.add_version("0.2.0".to_string());
        assert!(spirit.has_version("0.2.0"));
        assert_eq!(spirit.latest, "0.2.0");

        spirit.remove_version("0.2.0");
        assert!(!spirit.has_version("0.2.0"));
        assert_eq!(spirit.latest, "0.1.0");
    }

    #[test]
    fn test_spirit_query_builder() {
        let query = SpiritQuery::new()
            .with_name("hello")
            .with_author("test-author")
            .with_capability("SensorTime");

        assert_eq!(query.name, Some("hello".to_string()));
        assert_eq!(query.author, Some("test-author".to_string()));
        assert_eq!(query.capabilities, vec!["SensorTime"]);
    }

    #[test]
    fn test_spirit_query_is_empty() {
        let empty = SpiritQuery::new();
        assert!(empty.is_empty());

        let with_name = SpiritQuery::new().with_name("test");
        assert!(!with_name.is_empty());
    }

    #[test]
    fn test_install_source_serialization() {
        let local = InstallSource::Local {
            path: PathBuf::from("/tmp/test"),
        };
        let json = serde_json::to_string(&local).unwrap();
        assert!(json.contains("\"type\":\"Local\""));

        let remote = InstallSource::Remote {
            url: "https://example.com".to_string(),
        };
        let json = serde_json::to_string(&remote).unwrap();
        assert!(json.contains("\"type\":\"Remote\""));
    }
}
