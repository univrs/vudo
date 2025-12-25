//! Resource limits for sandboxed WebAssembly execution.
//!
//! This module defines the resource constraints that can be applied to
//! WebAssembly modules running in the VUDO VM sandbox.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Resource limits for a sandboxed WebAssembly execution environment.
///
/// These limits enforce security and resource constraints on WebAssembly
/// modules to prevent abuse and ensure fair resource allocation.
///
/// # Fields
///
/// - `memory_bytes`: Maximum linear memory size in bytes
/// - `cpu_quota`: CPU time quota (fuel-based metering)
/// - `max_fuel`: Maximum fuel units for execution
/// - `max_duration`: Maximum wall-clock execution time
/// - `max_table_elements`: Maximum number of table elements
/// - `max_instances`: Maximum number of module instances
///
/// # Example
///
/// ```
/// use vudo_vm::ResourceLimits;
///
/// let limits = ResourceLimits {
///     memory_bytes: 64 * 1024 * 1024, // 64 MB
///     cpu_quota: 1_000_000,
///     max_fuel: 1_000_000,
///     max_duration: std::time::Duration::from_secs(30),
///     max_table_elements: 10_000,
///     max_instances: 10,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum linear memory allocation in bytes.
    ///
    /// This limit applies to the total WebAssembly linear memory that can be
    /// allocated by a module. Attempts to grow memory beyond this limit will
    /// result in a `SandboxError::OutOfMemory`.
    pub memory_bytes: usize,

    /// CPU quota in fuel units.
    ///
    /// This is used for CPU time metering. Each WebAssembly instruction
    /// consumes a certain amount of fuel. When the quota is exhausted,
    /// execution is terminated with `SandboxError::CpuQuotaExceeded`.
    pub cpu_quota: u64,

    /// Maximum fuel units that can be consumed during execution.
    ///
    /// This provides an additional layer of fuel-based limiting beyond
    /// the CPU quota. Typically set to the same value as `cpu_quota`.
    pub max_fuel: u64,

    /// Maximum wall-clock duration for execution.
    ///
    /// If execution takes longer than this duration, it will be terminated
    /// with `SandboxError::Timeout`. This prevents modules from hanging
    /// or executing indefinitely.
    pub max_duration: Duration,

    /// Maximum number of elements in WebAssembly tables.
    ///
    /// This limits the size of function tables and other table types,
    /// preventing excessive memory usage through table allocations.
    pub max_table_elements: u32,

    /// Maximum number of module instances that can be created.
    ///
    /// This prevents resource exhaustion through excessive instantiation
    /// of WebAssembly modules.
    pub max_instances: u32,
}

impl Default for ResourceLimits {
    /// Creates default resource limits suitable for general-purpose sandboxing.
    ///
    /// # Default Values
    ///
    /// - `memory_bytes`: 16 MB
    /// - `cpu_quota`: 1,000,000 fuel units
    /// - `max_fuel`: 1,000,000 fuel units
    /// - `max_duration`: 10 seconds
    /// - `max_table_elements`: 10,000 elements
    /// - `max_instances`: 10 instances
    fn default() -> Self {
        Self {
            memory_bytes: 16 * 1024 * 1024, // 16 MB
            cpu_quota: 1_000_000,
            max_fuel: 1_000_000,
            max_duration: Duration::from_secs(10),
            max_table_elements: 10_000,
            max_instances: 10,
        }
    }
}

impl ResourceLimits {
    /// Creates a new `ResourceLimits` with custom values.
    ///
    /// # Arguments
    ///
    /// * `memory_bytes` - Maximum linear memory in bytes
    /// * `cpu_quota` - CPU quota in fuel units
    /// * `max_fuel` - Maximum fuel units
    /// * `max_duration` - Maximum execution duration
    /// * `max_table_elements` - Maximum table elements
    /// * `max_instances` - Maximum module instances
    ///
    /// # Example
    ///
    /// ```
    /// use vudo_vm::ResourceLimits;
    /// use std::time::Duration;
    ///
    /// let limits = ResourceLimits::new(
    ///     32 * 1024 * 1024,
    ///     2_000_000,
    ///     2_000_000,
    ///     Duration::from_secs(30),
    ///     20_000,
    ///     20,
    /// );
    /// ```
    pub fn new(
        memory_bytes: usize,
        cpu_quota: u64,
        max_fuel: u64,
        max_duration: Duration,
        max_table_elements: u32,
        max_instances: u32,
    ) -> Self {
        Self {
            memory_bytes,
            cpu_quota,
            max_fuel,
            max_duration,
            max_table_elements,
            max_instances,
        }
    }

    /// Creates resource limits for testing with very permissive values.
    ///
    /// # Warning
    ///
    /// These limits are NOT suitable for production use and should only
    /// be used in controlled testing environments.
    ///
    /// # Test Values
    ///
    /// - `memory_bytes`: 1 GB
    /// - `cpu_quota`: 100,000,000 fuel units
    /// - `max_fuel`: 100,000,000 fuel units
    /// - `max_duration`: 300 seconds (5 minutes)
    /// - `max_table_elements`: 1,000,000 elements
    /// - `max_instances`: 1,000 instances
    pub fn for_testing() -> Self {
        Self {
            memory_bytes: 1024 * 1024 * 1024, // 1 GB
            cpu_quota: 100_000_000,
            max_fuel: 100_000_000,
            max_duration: Duration::from_secs(300),
            max_table_elements: 1_000_000,
            max_instances: 1_000,
        }
    }

    /// Creates very restrictive resource limits for untrusted code.
    ///
    /// These limits are suitable for running untrusted WebAssembly modules
    /// where security and resource protection are paramount.
    ///
    /// # Restrictive Values
    ///
    /// - `memory_bytes`: 1 MB
    /// - `cpu_quota`: 100,000 fuel units
    /// - `max_fuel`: 100,000 fuel units
    /// - `max_duration`: 1 second
    /// - `max_table_elements`: 100 elements
    /// - `max_instances`: 1 instance
    pub fn restrictive() -> Self {
        Self {
            memory_bytes: 1024 * 1024, // 1 MB
            cpu_quota: 100_000,
            max_fuel: 100_000,
            max_duration: Duration::from_secs(1),
            max_table_elements: 100,
            max_instances: 1,
        }
    }
}
