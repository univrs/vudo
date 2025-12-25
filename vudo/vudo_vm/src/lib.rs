//! VUDO VM - Secure WebAssembly Sandbox Runtime
//!
//! This crate provides a secure, capability-based WebAssembly execution environment
//! with strict resource limits and fine-grained permissions control.
//!
//! # Overview
//!
//! The VUDO VM implements a sandbox that enforces:
//! - Memory limits
//! - CPU quotas (fuel-based)
//! - Time-based execution limits
//! - Capability-based access control
//! - Table element limits
//! - Instance limits
//!
//! # Example
//!
//! ```ignore
//! use vudo_vm::{Sandbox, ResourceLimits};
//!
//! let limits = ResourceLimits::default();
//! let sandbox = Sandbox::new(limits)?;
//! ```

pub mod capability;
pub mod error;
pub mod fuel;
pub mod host;
pub mod limits;
pub mod sandbox;

pub use error::SandboxError;
pub use limits::ResourceLimits;

// Re-export capability types for convenience
pub use capability::{
    CapabilityGrant, CapabilityScope, CapabilitySet, CapabilityType, MINIMAL_CAPABILITIES,
    NETWORK_SPIRIT_CAPABILITIES, SYSTEM_SPIRIT_CAPABILITIES,
};

// Re-export host interface types for convenience
pub use host::{HostCallResult, HostInterface, InMemoryStorage, LogLevel, StorageBackend};
