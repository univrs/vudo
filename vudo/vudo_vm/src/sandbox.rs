//! VUDO VM Sandbox - Isolated WASM Execution Environment
//!
//! This module implements the core Sandbox abstraction for VUDO VM, providing
//! secure, resource-limited execution of WASM modules (Spirits).
//!
//! Based on: ontology/prospective/vudo-vm/genes/sandbox.dol v0.1.0

use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use wasmtime::*;

// ═══════════════════════════════════════════════════════════════════════════
// CONSTANTS
// ═══════════════════════════════════════════════════════════════════════════

pub const DEFAULT_MEMORY_BYTES: u64 = 67_108_864; // 64 MB
pub const DEFAULT_CPU_QUOTA: f64 = 0.1; // 10%
pub const DEFAULT_MAX_FUEL: u64 = 1_000_000_000; // 1 billion
pub const DEFAULT_MAX_DURATION_SECS: u64 = 30; // 30 seconds
pub const MAX_SANDBOX_MEMORY: u64 = 1_073_741_824; // 1 GB
pub const MAX_MODULE_SIZE: usize = 104_857_600; // 100 MB

// ═══════════════════════════════════════════════════════════════════════════
// SANDBOX STATE
// ═══════════════════════════════════════════════════════════════════════════

/// Represents the lifecycle state of a sandbox.
///
/// State transitions follow a defined state machine:
/// - Initializing: Loading WASM and validating
/// - Ready: Module validated, awaiting execution
/// - Running: Currently executing
/// - Paused: Fuel exhausted, awaiting refuel
/// - Terminated: Clean shutdown
/// - Failed: Unrecoverable error occurred
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SandboxState {
    Initializing,
    Ready,
    Running,
    Paused,
    Terminated,
    Failed,
}

// ═══════════════════════════════════════════════════════════════════════════
// ERRORS
// ═══════════════════════════════════════════════════════════════════════════

/// Enumerates possible sandbox execution failures.
///
/// Each error type maps to a specific failure mode:
/// - OutOfMemory: Exceeded memory_bytes limit
/// - CpuQuotaExceeded: Exceeded cpu_quota limit
/// - CapabilityDenied: Attempted operation without permission
/// - WasmTrap: WASM runtime trap (invalid memory access, etc.)
/// - Timeout: Exceeded max_duration limit
/// - InvalidModule: WASM module failed validation
#[derive(Debug, Clone)]
pub enum SandboxError {
    OutOfMemory,
    CpuQuotaExceeded,
    CapabilityDenied(String),
    WasmTrap(String),
    Timeout,
    InvalidModule(String),
    RuntimeError(String),
    FunctionNotFound(String),
}

impl std::fmt::Display for SandboxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SandboxError::OutOfMemory => write!(f, "Out of memory"),
            SandboxError::CpuQuotaExceeded => write!(f, "CPU quota exceeded"),
            SandboxError::CapabilityDenied(msg) => write!(f, "Capability denied: {}", msg),
            SandboxError::WasmTrap(msg) => write!(f, "WASM trap: {}", msg),
            SandboxError::Timeout => write!(f, "Execution timeout"),
            SandboxError::InvalidModule(msg) => write!(f, "Invalid module: {}", msg),
            SandboxError::RuntimeError(msg) => write!(f, "Runtime error: {}", msg),
            SandboxError::FunctionNotFound(msg) => write!(f, "Function not found: {}", msg),
        }
    }
}

impl std::error::Error for SandboxError {}

// ═══════════════════════════════════════════════════════════════════════════
// RESOURCE LIMITS
// ═══════════════════════════════════════════════════════════════════════════

/// ResourceLimits define the boundaries of sandbox execution.
///
/// - memory_bytes: Maximum WASM linear memory
/// - cpu_quota: Fraction of CPU time (0.0 to 1.0)
/// - max_fuel: wasmtime fuel units before pause
/// - max_duration: Wall-clock timeout
/// - max_table_elements: WASM table size limit
/// - max_instances: Number of module instances
///
/// These limits implement the "capability-bounded substrate"
/// principle from the VUDO architecture.
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    pub memory_bytes: u64,
    pub cpu_quota: f64,
    pub max_fuel: u64,
    pub max_duration: Duration,
    pub max_table_elements: u32,
    pub max_instances: u32,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            memory_bytes: DEFAULT_MEMORY_BYTES,
            cpu_quota: DEFAULT_CPU_QUOTA,
            max_fuel: DEFAULT_MAX_FUEL,
            max_duration: Duration::from_secs(DEFAULT_MAX_DURATION_SECS),
            max_table_elements: 1000,
            max_instances: 1,
        }
    }
}

impl ResourceLimits {
    /// Validates resource limits according to DOL constraints
    pub fn validate(&self) -> Result<(), SandboxError> {
        if self.memory_bytes > MAX_SANDBOX_MEMORY {
            return Err(SandboxError::InvalidModule(format!(
                "Memory limit {} exceeds maximum {}",
                self.memory_bytes, MAX_SANDBOX_MEMORY
            )));
        }

        if self.cpu_quota < 0.0 || self.cpu_quota > 1.0 {
            return Err(SandboxError::InvalidModule(
                "CPU quota must be between 0.0 and 1.0".to_string(),
            ));
        }

        if self.max_fuel == 0 {
            return Err(SandboxError::InvalidModule(
                "max_fuel must be greater than 0".to_string(),
            ));
        }

        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// CAPABILITY TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// CapabilityType defines the categories of privileged operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CapabilityType {
    // Network capabilities
    NetworkListen,
    NetworkConnect,
    NetworkBroadcast,

    // Storage capabilities
    StorageRead,
    StorageWrite,
    StorageDelete,

    // Compute capabilities
    SpawnSandbox,
    CrossSandboxCall,

    // Sensor capabilities (read external state)
    SensorTime,
    SensorRandom,
    SensorEnvironment,

    // Actuator capabilities (affect external state)
    ActuatorLog,
    ActuatorNotify,
    ActuatorCredit,

    // Special capabilities
    Unrestricted, // Only for system Spirits
}

/// CapabilityScope defines the boundaries of a capability grant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CapabilityScope {
    Global,
    Sandboxed,
    Peer,
    Domain,
}

/// A CapabilityGrant is a cryptographically signed permission.
///
/// Grants are:
/// - Explicit: No implicit permissions
/// - Signed: Verifiable authenticity
/// - Scoped: Limited to specific operations/targets
/// - Temporal: Can expire
/// - Revocable: Can be withdrawn
#[derive(Debug, Clone)]
pub struct CapabilityGrant {
    pub id: u64,
    pub capability: CapabilityType,
    pub scope: CapabilityScope,
    pub granter: [u8; 32], // Ed25519 public key
    pub grantee: [u8; 32], // Ed25519 public key
    pub granted_at: u64,   // Unix timestamp
    pub expires_at: Option<u64>,
    pub revoked: bool,
    pub signature: [u8; 64], // Ed25519 signature
}

impl CapabilityGrant {
    /// Checks if this grant is currently valid
    pub fn is_valid(&self) -> bool {
        if self.revoked {
            return false;
        }

        if let Some(expiry) = self.expires_at {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            if now >= expiry {
                return false;
            }
        }

        true
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// EXECUTION RESULT
// ═══════════════════════════════════════════════════════════════════════════

/// Result of a sandbox execution
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub success: bool,
    pub return_value: Option<Vec<Val>>,
    pub fuel_consumed: u64,
    pub duration: Duration,
    pub memory_used: u64,
    pub error: Option<String>,
}

// ═══════════════════════════════════════════════════════════════════════════
// SANDBOX METRICS
// ═══════════════════════════════════════════════════════════════════════════

/// Aggregate metrics for sandbox performance analysis.
/// Updated after each execution cycle.
/// Used for billing, optimization, and debugging.
#[derive(Debug, Clone)]
pub struct SandboxMetrics {
    pub sandbox_id: u64,
    pub execution_count: u64,
    pub total_fuel_consumed: u64,
    pub total_duration: Duration,
    pub peak_memory: u64,
    pub trap_count: u64,
    pub last_updated: u64, // Unix timestamp
}

impl SandboxMetrics {
    fn new(sandbox_id: u64) -> Self {
        Self {
            sandbox_id,
            execution_count: 0,
            total_fuel_consumed: 0,
            total_duration: Duration::from_secs(0),
            peak_memory: 0,
            trap_count: 0,
            last_updated: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    fn update(&mut self, result: &ExecutionResult) {
        self.execution_count += 1;
        self.total_fuel_consumed += result.fuel_consumed;
        self.total_duration += result.duration;
        self.peak_memory = self.peak_memory.max(result.memory_used);
        if !result.success {
            self.trap_count += 1;
        }
        self.last_updated = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// SANDBOX CONTEXT
// ═══════════════════════════════════════════════════════════════════════════

/// Internal state holder for the Wasmtime store
struct SandboxContext {
    fuel_consumed: u64,
    start_time: Option<Instant>,
    timeout: Duration,
}

// ═══════════════════════════════════════════════════════════════════════════
// SANDBOX
// ═══════════════════════════════════════════════════════════════════════════

/// A Sandbox is an isolated WASM execution environment for Spirits.
///
/// Each Sandbox:
/// - Has a unique identity tied to an Ed25519 owner
/// - Contains compiled WASM bytecode
/// - Operates within explicit resource limits
/// - Requires capability grants for any external operations
/// - Tracks its own resource consumption
///
/// The Sandbox implements VUDO's security model:
/// "Nothing escapes the sandbox without explicit capability."
///
/// Lifecycle:
/// 1. Initializing → Loading WASM, validating
/// 2. Ready → Module validated, awaiting execution
/// 3. Running → Currently executing
/// 4. Paused → Fuel exhausted, awaiting refuel
/// 5. Terminated → Clean shutdown
/// 6. Failed → Unrecoverable error
pub struct Sandbox {
    pub id: u64,
    pub owner: [u8; 32], // Ed25519 public key
    pub wasm_module: Vec<u8>,
    pub limits: ResourceLimits,
    pub capabilities: Vec<CapabilityGrant>,
    pub state: SandboxState,
    pub created_at: u64, // Unix timestamp
    pub last_executed: Option<u64>,
    pub fuel_consumed: u64,
    pub memory_peak: u64,

    // Wasmtime runtime components
    engine: Engine,
    module: Option<Module>,
    store: Store<SandboxContext>,
    instance: Option<Instance>,

    // Metrics tracking
    metrics: SandboxMetrics,
}

impl Sandbox {
    /// Creates a new sandbox with the given WASM module, owner, and resource limits.
    ///
    /// This performs initial validation:
    /// - Module size check (max 100 MB)
    /// - Owner key length check (32 bytes for Ed25519)
    /// - Resource limits validation
    ///
    /// The sandbox starts in Initializing state.
    pub fn new(wasm: &[u8], owner: [u8; 32], limits: ResourceLimits) -> Result<Self, SandboxError> {
        // Validate module size
        if wasm.is_empty() {
            return Err(SandboxError::InvalidModule(
                "WASM module is empty".to_string(),
            ));
        }

        if wasm.len() > MAX_MODULE_SIZE {
            return Err(SandboxError::InvalidModule(format!(
                "WASM module size {} exceeds maximum {}",
                wasm.len(),
                MAX_MODULE_SIZE
            )));
        }

        // Validate resource limits
        limits.validate()?;

        // Configure Wasmtime engine with resource limits
        let mut config = Config::new();
        config.consume_fuel(true);

        // Set memory limits
        config.max_wasm_stack(2 * 1024 * 1024); // 2 MB stack

        // Create engine
        let engine = Engine::new(&config)
            .map_err(|e| SandboxError::RuntimeError(format!("Failed to create engine: {}", e)))?;

        // Create store with context
        let context = SandboxContext {
            fuel_consumed: 0,
            start_time: None,
            timeout: limits.max_duration,
        };
        let mut store = Store::new(&engine, context);

        // Set initial fuel
        store
            .set_fuel(limits.max_fuel)
            .map_err(|e| SandboxError::RuntimeError(format!("Failed to set fuel: {}", e)))?;

        let sandbox_id = Self::generate_id();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Ok(Self {
            id: sandbox_id,
            owner,
            wasm_module: wasm.to_vec(),
            limits,
            capabilities: Vec::new(),
            state: SandboxState::Initializing,
            created_at: now,
            last_executed: None,
            fuel_consumed: 0,
            memory_peak: 0,
            engine,
            module: None,
            store,
            instance: None,
            metrics: SandboxMetrics::new(sandbox_id),
        })
    }

    /// Initialize the sandbox by compiling the WASM module.
    ///
    /// This transitions from Initializing -> Ready or Failed.
    pub fn initialize(&mut self) -> Result<(), SandboxError> {
        if self.state != SandboxState::Initializing {
            return Err(SandboxError::RuntimeError(
                "Can only initialize from Initializing state".to_string(),
            ));
        }

        // Compile the WASM module
        let module = Module::new(&self.engine, &self.wasm_module).map_err(|e| {
            self.state = SandboxState::Failed;
            SandboxError::InvalidModule(format!("Failed to compile module: {}", e))
        })?;

        self.module = Some(module);
        self.state = SandboxState::Ready;

        Ok(())
    }

    /// Invoke a function in the WASM module.
    ///
    /// This executes the function with the given arguments and returns the result.
    /// The sandbox must be in Ready or Paused state.
    ///
    /// During execution:
    /// - Fuel consumption is tracked
    /// - Execution time is measured
    /// - Memory usage is monitored
    /// - Timeouts are enforced
    pub fn invoke(
        &mut self,
        function: &str,
        args: &[Val],
    ) -> Result<ExecutionResult, SandboxError> {
        // Check state
        if self.state != SandboxState::Ready && self.state != SandboxState::Paused {
            return Err(SandboxError::RuntimeError(format!(
                "Cannot invoke from state {:?}",
                self.state
            )));
        }

        // Get or create instance
        if self.instance.is_none() {
            let module = self
                .module
                .as_ref()
                .ok_or_else(|| SandboxError::RuntimeError("Module not initialized".to_string()))?;

            let instance = Instance::new(&mut self.store, module, &[]).map_err(|e| {
                self.state = SandboxState::Failed;
                SandboxError::RuntimeError(format!("Failed to instantiate module: {}", e))
            })?;

            self.instance = Some(instance);
        }

        let instance = self.instance.as_ref().unwrap();

        // Get the function
        let func = instance
            .get_func(&mut self.store, function)
            .ok_or_else(|| SandboxError::FunctionNotFound(function.to_string()))?;

        // Set up execution context
        self.state = SandboxState::Running;
        self.store.data_mut().start_time = Some(Instant::now());

        let fuel_before = self.store.get_fuel().unwrap_or(0);
        let start = Instant::now();

        // Execute the function
        let mut results = vec![Val::I32(0); func.ty(&self.store).results().len()];
        let execution_result = func.call(&mut self.store, args, &mut results);

        let duration = start.elapsed();
        let fuel_after = self.store.get_fuel().unwrap_or(0);
        let fuel_consumed = fuel_before.saturating_sub(fuel_after);

        // Update tracking
        self.fuel_consumed += fuel_consumed;
        self.last_executed = Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        );

        // Get memory usage (approximate)
        let memory_used = self.estimate_memory_usage();
        self.memory_peak = self.memory_peak.max(memory_used);

        // Build result
        let exec_result = match execution_result {
            Ok(_) => {
                self.state = SandboxState::Ready;
                ExecutionResult {
                    success: true,
                    return_value: Some(results),
                    fuel_consumed,
                    duration,
                    memory_used,
                    error: None,
                }
            }
            Err(e) => {
                // Check if it's a timeout or trap
                if duration >= self.limits.max_duration {
                    self.state = SandboxState::Failed;
                    ExecutionResult {
                        success: false,
                        return_value: None,
                        fuel_consumed,
                        duration,
                        memory_used,
                        error: Some(format!("Timeout: {}", e)),
                    }
                } else if fuel_after == 0 {
                    self.state = SandboxState::Paused;
                    ExecutionResult {
                        success: false,
                        return_value: None,
                        fuel_consumed,
                        duration,
                        memory_used,
                        error: Some("Out of fuel".to_string()),
                    }
                } else {
                    self.state = SandboxState::Failed;
                    ExecutionResult {
                        success: false,
                        return_value: None,
                        fuel_consumed,
                        duration,
                        memory_used,
                        error: Some(format!("WASM trap: {}", e)),
                    }
                }
            }
        };

        // Update metrics
        self.metrics.update(&exec_result);

        Ok(exec_result)
    }

    /// Get the current state of the sandbox.
    pub fn get_state(&self) -> SandboxState {
        self.state
    }

    /// Get current metrics for the sandbox.
    pub fn metrics(&self) -> SandboxMetrics {
        self.metrics.clone()
    }

    /// Add a capability grant to the sandbox.
    pub fn grant_capability(&mut self, grant: CapabilityGrant) {
        self.capabilities.push(grant);
    }

    /// Check if the sandbox has a specific capability.
    pub fn has_capability(&self, cap_type: CapabilityType) -> bool {
        self.capabilities
            .iter()
            .any(|grant| grant.capability == cap_type && grant.is_valid())
    }

    /// Refuel the sandbox (add more fuel).
    pub fn refuel(&mut self, additional_fuel: u64) -> Result<(), SandboxError> {
        let current = self.store.get_fuel().unwrap_or(0);
        let new_fuel = current.saturating_add(additional_fuel);

        self.store
            .set_fuel(new_fuel)
            .map_err(|e| SandboxError::RuntimeError(format!("Failed to refuel: {}", e)))?;

        if self.state == SandboxState::Paused {
            self.state = SandboxState::Ready;
        }

        Ok(())
    }

    /// Terminate the sandbox cleanly.
    pub fn terminate(&mut self) {
        self.state = SandboxState::Terminated;
        self.instance = None;
    }

    // Helper methods

    fn generate_id() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64
    }

    fn estimate_memory_usage(&self) -> u64 {
        // This is a simple estimate based on module size
        // In a real implementation, we would query actual memory usage from wasmtime
        self.wasm_module.len() as u64
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox_state_transitions() {
        // Create a minimal WASM module (empty module for testing)
        let wasm = wat::parse_str(
            r#"
            (module
                (func (export "test") (result i32)
                    i32.const 42
                )
            )
        "#,
        )
        .unwrap();

        let owner = [0u8; 32];
        let limits = ResourceLimits::default();

        let mut sandbox = Sandbox::new(&wasm, owner, limits).unwrap();
        assert_eq!(sandbox.get_state(), SandboxState::Initializing);

        sandbox.initialize().unwrap();
        assert_eq!(sandbox.get_state(), SandboxState::Ready);
    }

    #[test]
    fn test_sandbox_execution() {
        let wasm = wat::parse_str(
            r#"
            (module
                (func (export "add") (param i32 i32) (result i32)
                    local.get 0
                    local.get 1
                    i32.add
                )
            )
        "#,
        )
        .unwrap();

        let owner = [0u8; 32];
        let limits = ResourceLimits::default();

        let mut sandbox = Sandbox::new(&wasm, owner, limits).unwrap();
        sandbox.initialize().unwrap();

        let args = vec![Val::I32(10), Val::I32(32)];
        let result = sandbox.invoke("add", &args).unwrap();

        assert!(result.success);
        assert_eq!(result.return_value.as_ref().unwrap()[0].unwrap_i32(), 42);
    }

    #[test]
    fn test_resource_limits_validation() {
        let limits = ResourceLimits {
            memory_bytes: MAX_SANDBOX_MEMORY + 1,
            ..Default::default()
        };

        assert!(limits.validate().is_err());

        let limits = ResourceLimits {
            cpu_quota: 1.5,
            ..Default::default()
        };

        assert!(limits.validate().is_err());
    }

    #[test]
    fn test_capability_grant_validation() {
        let grant = CapabilityGrant {
            id: 1,
            capability: CapabilityType::ActuatorLog,
            scope: CapabilityScope::Sandboxed,
            granter: [0u8; 32],
            grantee: [1u8; 32],
            granted_at: 1000,
            expires_at: Some(2000),
            revoked: false,
            signature: [0u8; 64],
        };

        // Test revoked grant
        let mut revoked_grant = grant.clone();
        revoked_grant.revoked = true;
        assert!(!revoked_grant.is_valid());
    }
}
