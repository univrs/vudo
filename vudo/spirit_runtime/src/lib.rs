//! Spirit Runtime - Package management for VUDO Spirits
//!
//! This crate provides Spirit manifest parsing, versioning, dependency resolution,
//! and pricing models for the VUDO ecosystem.
//!
//! # Overview
//!
//! A Spirit is a packaged WASM module with:
//! - Manifest metadata (name, version, author, capabilities)
//! - Compiled WASM bytecode
//! - Ed25519 signatures for authenticity
//! - Pricing information for execution credits
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
pub mod version;

pub use dependency::{Dependency, DependencyResolver};
pub use manifest::{Capability, Manifest, ManifestError};
pub use pricing::{CreditCost, PricingModel};
pub use version::SemVer;
