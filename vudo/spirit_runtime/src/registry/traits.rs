//! Registry trait definition
//!
//! Defines the `Registry` trait that all registry implementations must satisfy.
//! This enables different backends (local filesystem, remote, etc.) while
//! maintaining a consistent API.

use crate::manifest::Manifest;

use super::types::{InstalledSpirit, RegistryError, SpiritQuery, SpiritSearchResult};

// ═══════════════════════════════════════════════════════════════════════════
// REGISTRY TRAIT
// ═══════════════════════════════════════════════════════════════════════════

/// Registry trait for Spirit package management
///
/// Implementations provide the actual storage and retrieval mechanism
/// for Spirit packages. The trait is async to support both local and
/// remote operations.
///
/// # Example
///
/// ```ignore
/// use spirit_runtime::registry::{Registry, LocalRegistry};
///
/// async fn example() -> Result<(), RegistryError> {
///     let mut registry = LocalRegistry::new();
///     registry.init().await?;
///
///     // Install a spirit
///     registry.install("./my-spirit/").await?;
///
///     // Get installed spirit
///     let spirit = registry.get("my-spirit").await?;
///     Ok(())
/// }
/// ```
pub trait Registry: Send + Sync {
    /// Initialize the registry
    ///
    /// Creates necessary directories and loads the registry index.
    /// Must be called before any other operations.
    fn init(&mut self) -> impl std::future::Future<Output = Result<(), RegistryError>> + Send;

    /// Install a spirit from a source path or URL
    ///
    /// # Arguments
    /// * `source` - Path to a directory containing manifest.json and spirit.wasm,
    ///   or a URL to download from (future support)
    ///
    /// # Returns
    /// The installed spirit entry on success
    fn install(
        &mut self,
        source: &str,
    ) -> impl std::future::Future<Output = Result<InstalledSpirit, RegistryError>> + Send;

    /// Uninstall a spirit by name (all versions)
    ///
    /// Removes the spirit directory and updates the index.
    fn uninstall(
        &mut self,
        name: &str,
    ) -> impl std::future::Future<Output = Result<(), RegistryError>> + Send;

    /// Uninstall a specific version of a spirit
    ///
    /// If this is the last version, the spirit is completely removed.
    fn uninstall_version(
        &mut self,
        name: &str,
        version: &str,
    ) -> impl std::future::Future<Output = Result<(), RegistryError>> + Send;

    /// Get a spirit by name (latest version)
    ///
    /// # Returns
    /// The search result containing manifest and path
    fn get(
        &self,
        name: &str,
    ) -> impl std::future::Future<Output = Result<SpiritSearchResult, RegistryError>> + Send;

    /// Get a specific version of a spirit
    fn get_version(
        &self,
        name: &str,
        version: &str,
    ) -> impl std::future::Future<Output = Result<SpiritSearchResult, RegistryError>> + Send;

    /// Search for spirits matching a query
    ///
    /// Searches by name, author, and capabilities based on the query.
    fn search(
        &self,
        query: &SpiritQuery,
    ) -> impl std::future::Future<Output = Result<Vec<SpiritSearchResult>, RegistryError>> + Send;

    /// List all installed spirits
    ///
    /// Returns the index entries, not full search results.
    fn list(
        &self,
    ) -> impl std::future::Future<Output = Result<Vec<InstalledSpirit>, RegistryError>> + Send;

    /// Get the WASM bytes for a spirit
    ///
    /// # Arguments
    /// * `name` - Spirit name
    /// * `version` - Optional version (defaults to latest)
    fn get_wasm(
        &self,
        name: &str,
        version: Option<&str>,
    ) -> impl std::future::Future<Output = Result<Vec<u8>, RegistryError>> + Send;

    /// Get the manifest for a spirit
    ///
    /// # Arguments
    /// * `name` - Spirit name
    /// * `version` - Optional version (defaults to latest)
    fn get_manifest(
        &self,
        name: &str,
        version: Option<&str>,
    ) -> impl std::future::Future<Output = Result<Manifest, RegistryError>> + Send;

    /// Check if a spirit is installed
    fn is_installed(&self, name: &str) -> bool;

    /// Check if a specific version is installed
    fn is_version_installed(&self, name: &str, version: &str) -> bool;

    /// Get the registry root directory
    fn root(&self) -> &std::path::Path;
}

// ═══════════════════════════════════════════════════════════════════════════
// HELPER TRAITS
// ═══════════════════════════════════════════════════════════════════════════

/// Extension trait for registry operations
pub trait RegistryExt: Registry {
    /// Install multiple spirits in sequence
    fn install_all(
        &mut self,
        sources: &[&str],
    ) -> impl std::future::Future<Output = Result<Vec<InstalledSpirit>, RegistryError>> + Send
    where
        Self: Sized,
    {
        async move {
            let mut results = Vec::new();
            for source in sources {
                results.push(self.install(source).await?);
            }
            Ok(results)
        }
    }

    /// Get all versions of a spirit
    fn get_all_versions(
        &self,
        name: &str,
    ) -> impl std::future::Future<Output = Result<Vec<SpiritSearchResult>, RegistryError>> + Send
    where
        Self: Sized,
    {
        async move {
            let spirits = self.list().await?;
            let spirit = spirits
                .iter()
                .find(|s| s.name == name)
                .ok_or_else(|| RegistryError::NotFound(name.to_string()))?;

            let mut results = Vec::new();
            for version in &spirit.versions {
                results.push(self.get_version(name, version).await?);
            }
            Ok(results)
        }
    }
}

// Blanket implementation for all Registry types
impl<T: Registry> RegistryExt for T {}
