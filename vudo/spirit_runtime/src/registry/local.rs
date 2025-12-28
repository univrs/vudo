//! Local filesystem registry implementation
//!
//! Provides a filesystem-based Spirit registry stored in `~/.vudo/registry/`.
//!
//! # Directory Structure
//!
//! ```text
//! ~/.vudo/registry/
//! ├── index.json           # Registry index
//! ├── spirits/             # Installed spirits
//! │   ├── my-spirit/
//! │   │   ├── 0.1.0/
//! │   │   │   ├── manifest.json
//! │   │   │   └── spirit.wasm
//! │   │   └── latest -> 0.1.0/
//! │   └── ...
//! └── cache/               # Downloaded packages
//! ```

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::fs;

use crate::manifest::Manifest;
use crate::signature::VerifyingKey;

use super::traits::Registry;
use super::types::{
    InstallSource, InstalledSpirit, RegistryConfig, RegistryError, RegistryIndex, SpiritQuery,
    SpiritSearchResult,
};

// ═══════════════════════════════════════════════════════════════════════════
// LOCAL REGISTRY
// ═══════════════════════════════════════════════════════════════════════════

/// Local filesystem registry
///
/// Stores spirits in the filesystem under `~/.vudo/registry/` (or a custom path).
/// Provides async operations for installing, searching, and retrieving spirits.
pub struct LocalRegistry {
    /// Root directory (e.g., ~/.vudo/registry/)
    root: PathBuf,
    /// Loaded index
    index: RegistryIndex,
    /// Whether init() has been called
    initialized: bool,
    /// Registry configuration for signature verification
    config: RegistryConfig,
}

impl LocalRegistry {
    /// Create a new local registry at the default location (~/.vudo/registry/)
    pub fn new() -> Self {
        let root = dirs::home_dir()
            .expect("Could not determine home directory")
            .join(".vudo")
            .join("registry");
        Self {
            root,
            index: RegistryIndex::default(),
            initialized: false,
            config: RegistryConfig::default(),
        }
    }

    /// Create a registry at a custom location
    pub fn with_root(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            index: RegistryIndex::default(),
            initialized: false,
            config: RegistryConfig::default(),
        }
    }

    /// Create a registry with custom configuration
    pub fn with_config(root: impl Into<PathBuf>, config: RegistryConfig) -> Self {
        Self {
            root: root.into(),
            index: RegistryIndex::default(),
            initialized: false,
            config,
        }
    }

    /// Get the current registry configuration
    pub fn config(&self) -> &RegistryConfig {
        &self.config
    }

    /// Set the registry configuration
    pub fn set_config(&mut self, config: RegistryConfig) {
        self.config = config;
    }

    /// Get path to index file
    fn index_path(&self) -> PathBuf {
        self.root.join("index.json")
    }

    /// Get path to spirits directory
    fn spirits_dir(&self) -> PathBuf {
        self.root.join("spirits")
    }

    /// Get path to cache directory
    fn cache_dir(&self) -> PathBuf {
        self.root.join("cache")
    }

    /// Get path to a spirit's directory
    fn spirit_dir(&self, name: &str) -> PathBuf {
        self.spirits_dir().join(name)
    }

    /// Get path to a spirit version directory
    fn spirit_version_dir(&self, name: &str, version: &str) -> PathBuf {
        self.spirit_dir(name).join(version)
    }

    /// Load index from disk
    async fn load_index(&mut self) -> Result<(), RegistryError> {
        let path = self.index_path();
        if path.exists() {
            let content = fs::read_to_string(&path).await?;
            self.index = serde_json::from_str(&content)?;
        } else {
            self.index = RegistryIndex::new();
        }
        Ok(())
    }

    /// Save index to disk
    async fn save_index(&self) -> Result<(), RegistryError> {
        let content = serde_json::to_string_pretty(&self.index)?;
        fs::write(self.index_path(), content).await?;
        Ok(())
    }

    /// Get current timestamp
    fn now() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    /// Resolve an author's verifying key from the trusted keys directory
    ///
    /// Looks for a key file named `{author}.pub` in the trusted keys directory.
    /// The file should contain the hex-encoded Ed25519 public key.
    ///
    /// # Arguments
    /// * `author` - The author's public key (hex-encoded, 64 characters)
    ///
    /// # Returns
    /// The verifying key if found and valid
    async fn resolve_author_key(&self, author: &str) -> Result<VerifyingKey, RegistryError> {
        // First, try to parse the author field directly as a public key
        // The author field in the manifest is already the hex-encoded public key
        if let Ok(key) = VerifyingKey::from_hex(author) {
            return Ok(key);
        }

        // If that fails, try to look up the key in the trusted keys directory
        if let Some(ref keys_dir) = self.config.trusted_keys_dir {
            let key_path = keys_dir.join(format!("{}.pub", author));
            if key_path.exists() {
                let key_hex = fs::read_to_string(&key_path).await.map_err(|e| {
                    RegistryError::InvalidManifest(format!("Failed to read key file: {}", e))
                })?;
                let key_hex = key_hex.trim();
                return VerifyingKey::from_hex(key_hex).map_err(|e| {
                    RegistryError::InvalidSignature {
                        spirit: author.to_string(),
                        reason: format!("Invalid key format: {}", e),
                    }
                });
            }
        }

        Err(RegistryError::AuthorKeyNotFound {
            author: author.to_string(),
        })
    }

    /// Verify the signature on a manifest
    ///
    /// # Arguments
    /// * `manifest` - The manifest to verify
    ///
    /// # Returns
    /// Ok(()) if signature is valid, or an error otherwise
    async fn verify_manifest_signature(&self, manifest: &Manifest) -> Result<(), RegistryError> {
        let name = &manifest.name;

        // Check if signatures are required
        if self.config.require_signatures {
            // Check if this author is allowed unsigned
            if self
                .config
                .unsigned_allowed_authors
                .contains(&manifest.author)
            {
                // Author is in the allowed list, signature not required
                return Ok(());
            }

            // Signature is required
            if manifest.signature.is_none() {
                return Err(RegistryError::UnsignedSpirit {
                    spirit: name.clone(),
                });
            }
        }

        // If there's a signature, verify it
        if manifest.signature.is_some() {
            // Get the author's verifying key
            let _verifying_key = self.resolve_author_key(&manifest.author).await?;

            // Use the manifest's built-in verify method
            manifest
                .verify()
                .map_err(|e| RegistryError::InvalidSignature {
                    spirit: name.clone(),
                    reason: e.to_string(),
                })?;
        }

        Ok(())
    }

    /// Install from a local directory containing manifest and wasm
    async fn install_from_dir(
        &mut self,
        source_path: &Path,
    ) -> Result<InstalledSpirit, RegistryError> {
        // Validate source path exists
        if !source_path.exists() {
            return Err(RegistryError::InvalidSource(format!(
                "Path does not exist: {}",
                source_path.display()
            )));
        }

        // Look for manifest (try manifest.json first, then manifest.toml)
        let (manifest, _manifest_format) = self.read_manifest(source_path).await?;

        // Check for WASM file
        let wasm_source = source_path.join("spirit.wasm");
        if !wasm_source.exists() {
            return Err(RegistryError::MissingWasm(format!(
                "No spirit.wasm found in {}",
                source_path.display()
            )));
        }

        // Verify signature before proceeding with installation
        self.verify_manifest_signature(&manifest).await?;

        let name = manifest.name.clone();
        let version = manifest.version.to_string();

        // Check if already installed
        if self.index.contains_version(&name, &version) {
            return Err(RegistryError::AlreadyInstalled { name, version });
        }

        // Create target directory
        let target_dir = self.spirit_version_dir(&name, &version);
        fs::create_dir_all(&target_dir).await?;

        // Copy WASM file
        let wasm_target = target_dir.join("spirit.wasm");
        fs::copy(&wasm_source, &wasm_target).await?;

        // Write manifest as JSON (normalized format)
        let manifest_target = target_dir.join("manifest.json");
        let manifest_json = serde_json::to_string_pretty(&manifest)?;
        fs::write(&manifest_target, manifest_json).await?;

        // Copy assets directory if present
        let assets_source = source_path.join("assets");
        if assets_source.exists() && assets_source.is_dir() {
            let assets_target = target_dir.join("assets");
            copy_dir_recursive(&assets_source, &assets_target).await?;
        }

        // Update index
        let now = Self::now();
        let installed = if let Some(existing) = self.index.find_mut(&name) {
            existing.add_version(version.clone());
            existing.clone()
        } else {
            let new_spirit = InstalledSpirit {
                name: name.clone(),
                versions: vec![version.clone()],
                latest: version.clone(),
                installed_at: now,
                source: InstallSource::Local {
                    path: source_path.to_path_buf(),
                },
            };
            self.index.spirits.push(new_spirit.clone());
            new_spirit
        };

        // Create/update 'latest' symlink (Unix only)
        self.update_latest_symlink(&name, &version).await;

        // Persist index
        self.save_index().await?;

        Ok(installed)
    }

    /// Read manifest from source directory
    async fn read_manifest(
        &self,
        source_path: &Path,
    ) -> Result<(Manifest, ManifestFormat), RegistryError> {
        // Try JSON first
        let json_path = source_path.join("manifest.json");
        if json_path.exists() {
            let content = fs::read_to_string(&json_path).await?;
            let manifest: Manifest = serde_json::from_str(&content)
                .map_err(|e| RegistryError::InvalidManifest(format!("JSON parse error: {}", e)))?;
            return Ok((manifest, ManifestFormat::Json));
        }

        // Try TOML
        let toml_path = source_path.join("manifest.toml");
        if toml_path.exists() {
            let content = fs::read_to_string(&toml_path).await?;
            let manifest = Manifest::from_toml(&content)
                .map_err(|e| RegistryError::InvalidManifest(format!("TOML parse error: {}", e)))?;
            return Ok((manifest, ManifestFormat::Toml));
        }

        Err(RegistryError::InvalidManifest(format!(
            "No manifest.json or manifest.toml found in {}",
            source_path.display()
        )))
    }

    /// Update the 'latest' symlink for a spirit
    #[cfg(unix)]
    async fn update_latest_symlink(&self, name: &str, version: &str) {
        let spirit_dir = self.spirit_dir(name);
        let latest_link = spirit_dir.join("latest");

        // Remove existing symlink
        let _ = fs::remove_file(&latest_link).await;

        // Create new symlink
        use std::os::unix::fs::symlink;
        let _ = symlink(version, &latest_link);
    }

    #[cfg(not(unix))]
    async fn update_latest_symlink(&self, _name: &str, _version: &str) {
        // Symlinks not supported on this platform
    }
}

/// Manifest format enumeration
#[derive(Debug, Clone, Copy)]
enum ManifestFormat {
    Json,
    Toml,
}

impl Default for LocalRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// REGISTRY TRAIT IMPLEMENTATION
// ═══════════════════════════════════════════════════════════════════════════

impl Registry for LocalRegistry {
    async fn init(&mut self) -> Result<(), RegistryError> {
        // Create directory structure
        fs::create_dir_all(&self.root).await?;
        fs::create_dir_all(self.spirits_dir()).await?;
        fs::create_dir_all(self.cache_dir()).await?;

        // Load or create index
        let index_exists = self.index_path().exists();
        self.load_index().await?;

        // Save index if it didn't exist (creates the file)
        if !index_exists {
            self.save_index().await?;
        }

        self.initialized = true;

        Ok(())
    }

    async fn install(&mut self, source: &str) -> Result<InstalledSpirit, RegistryError> {
        let path = Path::new(source);
        if path.exists() {
            self.install_from_dir(path).await
        } else if source.starts_with("http://") || source.starts_with("https://") {
            // TODO: Implement remote URL installation
            Err(RegistryError::InvalidSource(
                "Remote URL installation not yet supported".to_string(),
            ))
        } else {
            Err(RegistryError::InvalidSource(format!(
                "Source not found: {}",
                source
            )))
        }
    }

    async fn uninstall(&mut self, name: &str) -> Result<(), RegistryError> {
        let dir = self.spirit_dir(name);
        if dir.exists() {
            fs::remove_dir_all(&dir).await?;
        }

        self.index.spirits.retain(|s| s.name != name);
        self.save_index().await?;

        Ok(())
    }

    async fn uninstall_version(&mut self, name: &str, version: &str) -> Result<(), RegistryError> {
        let dir = self.spirit_version_dir(name, version);
        if dir.exists() {
            fs::remove_dir_all(&dir).await?;
        }

        let new_latest = if let Some(spirit) = self.index.find_mut(name) {
            spirit.remove_version(version);
            if spirit.versions.is_empty() {
                self.index.spirits.retain(|s| s.name != name);
                // Remove spirit directory if empty
                let spirit_dir = self.spirit_dir(name);
                let _ = fs::remove_dir_all(&spirit_dir).await;
                None
            } else {
                Some(spirit.latest.clone())
            }
        } else {
            None
        };

        // Update latest symlink outside of the mutable borrow
        if let Some(latest) = new_latest {
            self.update_latest_symlink(name, &latest).await;
        }

        self.save_index().await?;

        Ok(())
    }

    async fn get(&self, name: &str) -> Result<SpiritSearchResult, RegistryError> {
        let spirit = self
            .index
            .find(name)
            .ok_or_else(|| RegistryError::NotFound(name.to_string()))?;
        self.get_version(name, &spirit.latest).await
    }

    async fn get_version(
        &self,
        name: &str,
        version: &str,
    ) -> Result<SpiritSearchResult, RegistryError> {
        let dir = self.spirit_version_dir(name, version);
        let manifest_path = dir.join("manifest.json");

        if !manifest_path.exists() {
            return Err(RegistryError::VersionNotFound {
                name: name.to_string(),
                version: version.to_string(),
            });
        }

        let content = fs::read_to_string(&manifest_path).await?;
        let manifest: Manifest = serde_json::from_str(&content)?;

        Ok(SpiritSearchResult {
            name: name.to_string(),
            version: version.to_string(),
            manifest,
            path: dir,
        })
    }

    async fn search(&self, query: &SpiritQuery) -> Result<Vec<SpiritSearchResult>, RegistryError> {
        let mut results = Vec::new();

        for spirit in &self.index.spirits {
            // Name filter
            if let Some(ref pattern) = query.name {
                if !spirit.name.to_lowercase().contains(&pattern.to_lowercase()) {
                    continue;
                }
            }

            // Get full manifest for detailed filtering
            if let Ok(result) = self.get(&spirit.name).await {
                // Author filter
                if let Some(ref author) = query.author {
                    if !result
                        .manifest
                        .author
                        .to_lowercase()
                        .contains(&author.to_lowercase())
                    {
                        continue;
                    }
                }

                // Capability filter
                if !query.capabilities.is_empty() {
                    let manifest_caps: Vec<String> = result
                        .manifest
                        .capabilities
                        .iter()
                        .map(|c| format!("{:?}", c))
                        .collect();

                    let has_all = query.capabilities.iter().all(|c| {
                        manifest_caps
                            .iter()
                            .any(|mc| mc.to_lowercase().contains(&c.to_lowercase()))
                    });

                    if !has_all {
                        continue;
                    }
                }

                results.push(result);
            }
        }

        Ok(results)
    }

    async fn list(&self) -> Result<Vec<InstalledSpirit>, RegistryError> {
        Ok(self.index.spirits.clone())
    }

    async fn get_wasm(&self, name: &str, version: Option<&str>) -> Result<Vec<u8>, RegistryError> {
        let result = match version {
            Some(v) => self.get_version(name, v).await?,
            None => self.get(name).await?,
        };

        let wasm_path = result.path.join("spirit.wasm");
        Ok(fs::read(&wasm_path).await?)
    }

    async fn get_manifest(
        &self,
        name: &str,
        version: Option<&str>,
    ) -> Result<Manifest, RegistryError> {
        let result = match version {
            Some(v) => self.get_version(name, v).await?,
            None => self.get(name).await?,
        };
        Ok(result.manifest)
    }

    fn is_installed(&self, name: &str) -> bool {
        self.index.contains(name)
    }

    fn is_version_installed(&self, name: &str, version: &str) -> bool {
        self.index.contains_version(name, version)
    }

    fn root(&self) -> &Path {
        &self.root
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// HELPERS
// ═══════════════════════════════════════════════════════════════════════════

/// Recursively copy a directory
async fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), RegistryError> {
    fs::create_dir_all(dst).await?;

    let mut entries = fs::read_dir(src).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        let dest_path = dst.join(entry.file_name());

        if path.is_dir() {
            Box::pin(copy_dir_recursive(&path, &dest_path)).await?;
        } else {
            fs::copy(&path, &dest_path).await?;
        }
    }

    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn create_test_spirit(
        dir: &Path,
        name: &str,
        version: &str,
    ) -> Result<(), std::io::Error> {
        let manifest = Manifest::new(
            name,
            version.parse().unwrap(),
            "a".repeat(64), // Valid author key
        );

        let manifest_json = serde_json::to_string_pretty(&manifest).unwrap();
        fs::write(dir.join("manifest.json"), manifest_json).await?;

        // Minimal valid WASM module (empty module)
        let wasm_bytes: Vec<u8> = vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00];
        fs::write(dir.join("spirit.wasm"), wasm_bytes).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_init_creates_directories() {
        let temp = TempDir::new().unwrap();
        let mut registry = LocalRegistry::with_root(temp.path());

        registry.init().await.unwrap();

        assert!(temp.path().join("spirits").exists());
        assert!(temp.path().join("cache").exists());
        assert!(temp.path().join("index.json").exists());
    }

    #[tokio::test]
    async fn test_install_from_local_dir() {
        let temp = TempDir::new().unwrap();
        let spirit_dir = temp.path().join("test-spirit");
        fs::create_dir_all(&spirit_dir).await.unwrap();
        create_test_spirit(&spirit_dir, "test-spirit", "0.1.0")
            .await
            .unwrap();

        let registry_dir = temp.path().join("registry");
        let mut registry = LocalRegistry::with_root(&registry_dir);
        registry.init().await.unwrap();

        let installed = registry
            .install(spirit_dir.to_str().unwrap())
            .await
            .unwrap();

        assert_eq!(installed.name, "test-spirit");
        assert_eq!(installed.latest, "0.1.0");
        assert!(registry.is_installed("test-spirit"));
    }

    #[tokio::test]
    async fn test_install_duplicate_fails() {
        let temp = TempDir::new().unwrap();
        let spirit_dir = temp.path().join("test-spirit");
        fs::create_dir_all(&spirit_dir).await.unwrap();
        create_test_spirit(&spirit_dir, "test-spirit", "0.1.0")
            .await
            .unwrap();

        let registry_dir = temp.path().join("registry");
        let mut registry = LocalRegistry::with_root(&registry_dir);
        registry.init().await.unwrap();

        // First install succeeds
        registry
            .install(spirit_dir.to_str().unwrap())
            .await
            .unwrap();

        // Second install fails
        let result = registry.install(spirit_dir.to_str().unwrap()).await;
        assert!(matches!(
            result,
            Err(RegistryError::AlreadyInstalled { .. })
        ));
    }

    #[tokio::test]
    async fn test_get_installed_spirit() {
        let temp = TempDir::new().unwrap();
        let spirit_dir = temp.path().join("hello-spirit");
        fs::create_dir_all(&spirit_dir).await.unwrap();
        create_test_spirit(&spirit_dir, "hello-spirit", "1.0.0")
            .await
            .unwrap();

        let registry_dir = temp.path().join("registry");
        let mut registry = LocalRegistry::with_root(&registry_dir);
        registry.init().await.unwrap();
        registry
            .install(spirit_dir.to_str().unwrap())
            .await
            .unwrap();

        let result = registry.get("hello-spirit").await.unwrap();
        assert_eq!(result.name, "hello-spirit");
        assert_eq!(result.version, "1.0.0");
    }

    #[tokio::test]
    async fn test_uninstall_spirit() {
        let temp = TempDir::new().unwrap();
        let spirit_dir = temp.path().join("remove-me");
        fs::create_dir_all(&spirit_dir).await.unwrap();
        create_test_spirit(&spirit_dir, "remove-me", "0.1.0")
            .await
            .unwrap();

        let registry_dir = temp.path().join("registry");
        let mut registry = LocalRegistry::with_root(&registry_dir);
        registry.init().await.unwrap();
        registry
            .install(spirit_dir.to_str().unwrap())
            .await
            .unwrap();

        assert!(registry.is_installed("remove-me"));

        registry.uninstall("remove-me").await.unwrap();

        assert!(!registry.is_installed("remove-me"));
    }

    #[tokio::test]
    async fn test_list_spirits() {
        let temp = TempDir::new().unwrap();

        // Create two spirits
        let spirit1_dir = temp.path().join("spirit-one");
        fs::create_dir_all(&spirit1_dir).await.unwrap();
        create_test_spirit(&spirit1_dir, "spirit-one", "0.1.0")
            .await
            .unwrap();

        let spirit2_dir = temp.path().join("spirit-two");
        fs::create_dir_all(&spirit2_dir).await.unwrap();
        create_test_spirit(&spirit2_dir, "spirit-two", "0.2.0")
            .await
            .unwrap();

        let registry_dir = temp.path().join("registry");
        let mut registry = LocalRegistry::with_root(&registry_dir);
        registry.init().await.unwrap();
        registry
            .install(spirit1_dir.to_str().unwrap())
            .await
            .unwrap();
        registry
            .install(spirit2_dir.to_str().unwrap())
            .await
            .unwrap();

        let list = registry.list().await.unwrap();
        assert_eq!(list.len(), 2);
    }

    #[tokio::test]
    async fn test_get_wasm() {
        let temp = TempDir::new().unwrap();
        let spirit_dir = temp.path().join("wasm-test");
        fs::create_dir_all(&spirit_dir).await.unwrap();
        create_test_spirit(&spirit_dir, "wasm-test", "0.1.0")
            .await
            .unwrap();

        let registry_dir = temp.path().join("registry");
        let mut registry = LocalRegistry::with_root(&registry_dir);
        registry.init().await.unwrap();
        registry
            .install(spirit_dir.to_str().unwrap())
            .await
            .unwrap();

        let wasm = registry.get_wasm("wasm-test", None).await.unwrap();
        // Check WASM magic number
        assert_eq!(&wasm[0..4], &[0x00, 0x61, 0x73, 0x6d]);
    }

    #[tokio::test]
    async fn test_search_by_name() {
        let temp = TempDir::new().unwrap();
        let spirit_dir = temp.path().join("searchable-spirit");
        fs::create_dir_all(&spirit_dir).await.unwrap();
        create_test_spirit(&spirit_dir, "searchable-spirit", "0.1.0")
            .await
            .unwrap();

        let registry_dir = temp.path().join("registry");
        let mut registry = LocalRegistry::with_root(&registry_dir);
        registry.init().await.unwrap();
        registry
            .install(spirit_dir.to_str().unwrap())
            .await
            .unwrap();

        let query = SpiritQuery::new().with_name("searchable");
        let results = registry.search(&query).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "searchable-spirit");
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // SIGNATURE VERIFICATION TESTS
    // ═══════════════════════════════════════════════════════════════════════════

    use crate::signature::SigningKey;

    /// Create a signed test spirit with a valid signature
    async fn create_signed_spirit(
        dir: &Path,
        name: &str,
        version: &str,
    ) -> Result<(), std::io::Error> {
        // Generate a keypair
        let signing_key = SigningKey::generate();
        let verifying_key = signing_key.verifying_key();
        let author = verifying_key.to_hex();

        let mut manifest = Manifest::new(name, version.parse().unwrap(), author);

        // Sign the manifest
        let signature = manifest
            .sign(&ed25519_dalek::SigningKey::from_bytes(
                &signing_key.to_bytes(),
            ))
            .unwrap();
        manifest.signature = Some(signature);

        let manifest_json = serde_json::to_string_pretty(&manifest).unwrap();
        fs::write(dir.join("manifest.json"), manifest_json).await?;

        // Minimal valid WASM module (empty module)
        let wasm_bytes: Vec<u8> = vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00];
        fs::write(dir.join("spirit.wasm"), wasm_bytes).await?;

        Ok(())
    }

    /// Create a spirit with an invalid signature
    async fn create_invalid_signature_spirit(
        dir: &Path,
        name: &str,
        version: &str,
    ) -> Result<(), std::io::Error> {
        // Generate a keypair for the author
        let signing_key = SigningKey::generate();
        let verifying_key = signing_key.verifying_key();
        let author = verifying_key.to_hex();

        // Generate a different keypair to sign with (wrong key)
        let wrong_signing_key = SigningKey::generate();

        let mut manifest = Manifest::new(name, version.parse().unwrap(), author);

        // Sign with the wrong key
        let signature = manifest
            .sign(&ed25519_dalek::SigningKey::from_bytes(
                &wrong_signing_key.to_bytes(),
            ))
            .unwrap();
        manifest.signature = Some(signature);

        let manifest_json = serde_json::to_string_pretty(&manifest).unwrap();
        fs::write(dir.join("manifest.json"), manifest_json).await?;

        // Minimal valid WASM module (empty module)
        let wasm_bytes: Vec<u8> = vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00];
        fs::write(dir.join("spirit.wasm"), wasm_bytes).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_install_rejects_invalid_signature() {
        let temp = TempDir::new().unwrap();
        let spirit_dir = temp.path().join("invalid-sig-spirit");
        fs::create_dir_all(&spirit_dir).await.unwrap();
        create_invalid_signature_spirit(&spirit_dir, "invalid-sig-spirit", "0.1.0")
            .await
            .unwrap();

        let registry_dir = temp.path().join("registry");
        let mut registry = LocalRegistry::with_root(&registry_dir);
        registry.init().await.unwrap();

        // Installation should fail due to invalid signature
        let result = registry.install(spirit_dir.to_str().unwrap()).await;
        assert!(
            matches!(result, Err(RegistryError::InvalidSignature { .. })),
            "Expected InvalidSignature error, got: {:?}",
            result
        );
    }

    #[tokio::test]
    async fn test_install_accepts_valid_signature() {
        let temp = TempDir::new().unwrap();
        let spirit_dir = temp.path().join("valid-sig-spirit");
        fs::create_dir_all(&spirit_dir).await.unwrap();
        create_signed_spirit(&spirit_dir, "valid-sig-spirit", "0.1.0")
            .await
            .unwrap();

        let registry_dir = temp.path().join("registry");
        let mut registry = LocalRegistry::with_root(&registry_dir);
        registry.init().await.unwrap();

        // Installation should succeed with valid signature
        let result = registry.install(spirit_dir.to_str().unwrap()).await;
        assert!(
            result.is_ok(),
            "Expected successful installation, got: {:?}",
            result
        );

        let installed = result.unwrap();
        assert_eq!(installed.name, "valid-sig-spirit");
        assert_eq!(installed.latest, "0.1.0");
    }

    #[tokio::test]
    async fn test_install_allows_unsigned_when_not_required() {
        let temp = TempDir::new().unwrap();
        let spirit_dir = temp.path().join("unsigned-spirit");
        fs::create_dir_all(&spirit_dir).await.unwrap();
        // create_test_spirit creates an unsigned spirit
        create_test_spirit(&spirit_dir, "unsigned-spirit", "0.1.0")
            .await
            .unwrap();

        let registry_dir = temp.path().join("registry");
        // Default config has require_signatures = false
        let mut registry = LocalRegistry::with_root(&registry_dir);
        registry.init().await.unwrap();

        // Installation should succeed when signatures are not required
        let result = registry.install(spirit_dir.to_str().unwrap()).await;
        assert!(
            result.is_ok(),
            "Expected successful installation, got: {:?}",
            result
        );

        let installed = result.unwrap();
        assert_eq!(installed.name, "unsigned-spirit");
    }

    #[tokio::test]
    async fn test_install_rejects_unsigned_when_required() {
        let temp = TempDir::new().unwrap();
        let spirit_dir = temp.path().join("must-sign-spirit");
        fs::create_dir_all(&spirit_dir).await.unwrap();
        // create_test_spirit creates an unsigned spirit
        create_test_spirit(&spirit_dir, "must-sign-spirit", "0.1.0")
            .await
            .unwrap();

        let registry_dir = temp.path().join("registry");
        // Create config that requires signatures
        let config = RegistryConfig {
            require_signatures: true,
            trusted_keys_dir: None,
            unsigned_allowed_authors: vec![],
        };
        let mut registry = LocalRegistry::with_config(&registry_dir, config);
        registry.init().await.unwrap();

        // Installation should fail because signature is required but missing
        let result = registry.install(spirit_dir.to_str().unwrap()).await;
        assert!(
            matches!(result, Err(RegistryError::UnsignedSpirit { .. })),
            "Expected UnsignedSpirit error, got: {:?}",
            result
        );
    }

    #[tokio::test]
    async fn test_install_allows_unsigned_for_allowed_author() {
        let temp = TempDir::new().unwrap();
        let spirit_dir = temp.path().join("allowed-unsigned");
        fs::create_dir_all(&spirit_dir).await.unwrap();
        // create_test_spirit creates an unsigned spirit with author = "a".repeat(64)
        create_test_spirit(&spirit_dir, "allowed-unsigned", "0.1.0")
            .await
            .unwrap();

        let registry_dir = temp.path().join("registry");
        // Create config that requires signatures but allows this author
        let config = RegistryConfig {
            require_signatures: true,
            trusted_keys_dir: None,
            unsigned_allowed_authors: vec!["a".repeat(64)],
        };
        let mut registry = LocalRegistry::with_config(&registry_dir, config);
        registry.init().await.unwrap();

        // Installation should succeed because this author is in the allowed list
        let result = registry.install(spirit_dir.to_str().unwrap()).await;
        assert!(
            result.is_ok(),
            "Expected successful installation, got: {:?}",
            result
        );

        let installed = result.unwrap();
        assert_eq!(installed.name, "allowed-unsigned");
    }
}
