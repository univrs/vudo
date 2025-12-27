//! Spirit Runtime - Package management for VUDO Spirits
//!
//! This crate provides Spirit manifest parsing, versioning, dependency resolution,
//! registry management, and pricing models for the VUDO ecosystem.
//!
//! # Overview
//!
//! A Spirit is a packaged WASM module with:
//! - Manifest metadata (name, version, author, capabilities)
//! - Compiled WASM bytecode
//! - Ed25519 signatures for authenticity
//! - Pricing information for execution credits
//!
//! # Registry
//!
//! The registry system manages Spirit installation, discovery, and versioning:
//!
//! ```ignore
//! use spirit_runtime::registry::{LocalRegistry, Registry};
//!
//! let mut registry = LocalRegistry::new();
//! registry.init().await?;
//! registry.install("./my-spirit/").await?;
//! ```
//!
//! # Example
//!
//! ```ignore
//! use spirit_runtime::{Manifest, SemVer};
//!
//! let manifest = Manifest::from_toml(toml_content)?;
//! assert!(manifest.validate()?);
//! ```

pub mod dependency;
pub mod manifest;
pub mod pricing;
pub mod registry;
pub mod signature;
pub mod version;

pub use dependency::{Dependency, DependencyResolver};
pub use manifest::{Capability, Manifest, ManifestBuilder, ManifestError};
pub use pricing::{CreditCost, PricingModel};
pub use registry::{LocalRegistry, QueryBuilder, Registry, RegistryError};
pub use signature::{KeyPair, Signature, SignatureError, SigningKey, VerifyingKey};
pub use version::SemVer;
