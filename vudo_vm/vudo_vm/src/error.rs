//! Error types for the VUDO VM sandbox.
//!
//! This module defines all possible error conditions that can occur during
//! WebAssembly module execution within the sandbox environment.

use thiserror::Error;

/// Errors that can occur during sandbox execution.
///
/// These errors represent both resource limit violations and runtime failures
/// that may occur when executing WebAssembly modules in the VUDO VM.
#[derive(Error, Debug)]
pub enum SandboxError {
    /// The WebAssembly module exceeded its memory allocation limit.
    ///
    /// This occurs when a module attempts to allocate more memory than
    /// allowed by the configured `ResourceLimits::memory_bytes`.
    #[error("Out of memory: allocation exceeded limit")]
    OutOfMemory,

    /// The WebAssembly module exceeded its CPU quota.
    ///
    /// This occurs when the fuel consumption exceeds the configured
    /// `ResourceLimits::cpu_quota` or `ResourceLimits::max_fuel`.
    #[error("CPU quota exceeded: consumed {consumed}, limit {limit}")]
    CpuQuotaExceeded {
        /// The amount of fuel consumed by the module.
        consumed: u64,
        /// The maximum fuel allowed by the quota.
        limit: u64,
    },

    /// A capability check failed during execution.
    ///
    /// This occurs when a module attempts to perform an operation
    /// without the required capability permission.
    #[error("Capability denied: {capability} - {reason}")]
    CapabilityDenied {
        /// The capability that was required for the operation.
        capability: String,
        /// The reason why the capability check failed.
        reason: String,
    },

    /// A WebAssembly trap occurred during execution.
    ///
    /// This represents any runtime error within the WebAssembly module,
    /// such as division by zero, unreachable instruction, etc.
    #[error("WASM trap: {0}")]
    WasmTrap(String),

    /// The WebAssembly module execution exceeded the time limit.
    ///
    /// This occurs when execution takes longer than the configured
    /// `ResourceLimits::max_duration`.
    #[error("Execution timeout: exceeded {max_duration:?}")]
    Timeout {
        /// The maximum duration allowed for execution.
        max_duration: std::time::Duration,
    },

    /// The WebAssembly module is invalid or could not be loaded.
    ///
    /// This occurs during module validation or instantiation when the
    /// module binary is malformed or contains unsupported features.
    #[error("Invalid module: {0}")]
    InvalidModule(String),
}

/// Result type for sandbox operations.
///
/// This is a convenience type alias for operations that may return a `SandboxError`.
pub type Result<T> = std::result::Result<T, SandboxError>;
