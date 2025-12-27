//! Spirit Registry System
//!
//! This module provides a complete package management system for Spirit WASM modules.
//! It supports installing, searching, and managing Spirits with version control.
//!
//! # Architecture
//!
//! The registry uses a trait-based design to support multiple backends:
//!
//! - [`Registry`] - Core trait defining registry operations
//! - [`LocalRegistry`] - Filesystem-based implementation (default)
//!
//! # Directory Structure
//!
//! The local registry stores spirits in a well-defined structure:
//!
//! ```text
//! ~/.vudo/registry/
//! ├── index.json           # Registry index
//! └── spirits/
//!     └── {name}/
//!         ├── latest -> {version}  # Symlink to latest version
//!         └── {version}/
//!             ├── manifest.json
//!             └── spirit.wasm
//! ```
//!
//! # Example Usage
//!
//! ```ignore
//! use spirit_runtime::registry::{LocalRegistry, Registry, QueryBuilder};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Initialize the registry
//!     let mut registry = LocalRegistry::new();
//!     registry.init().await?;
//!
//!     // Install a spirit from local path
//!     let spirit = registry.install("./my-spirit/").await?;
//!     println!("Installed: {} v{}", spirit.name, spirit.latest);
//!
//!     // Search for spirits
//!     let query = QueryBuilder::new()
//!         .name("hello")
//!         .capability("SensorTime")
//!         .build();
//!     let results = registry.search(&query).await?;
//!
//!     // Get WASM bytes for execution
//!     let wasm = registry.get_wasm("hello-world", None).await?;
//!
//!     Ok(())
//! }
//! ```

mod local;
mod search;
mod traits;
mod types;

// Re-export primary types
pub use local::LocalRegistry;
pub use search::{compare_versions, filter_by_capability, matches_name_pattern, sort_results};
pub use search::{QueryBuilder, SortBy, SortOrder};
pub use traits::{Registry, RegistryExt};
pub use types::{
    InstallSource, InstalledSpirit, RegistryError, RegistryIndex, SpiritQuery, SpiritSearchResult,
};
