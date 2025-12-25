//! Fuel Management for VUDO VM
//!
//! This module implements fuel-based execution metering for VUDO VM, wrapping
//! wasmtime's fuel system with additional tracking and management capabilities.
//!
//! Based on the FuelManagement trait from:
//! `/home/ardeshir/repos/univrs-vudo/ontology/prospective/vudo-vm/traits/execution.dol`
//!
//! Fuel provides:
//! - Deterministic execution bounds (prevents infinite loops)
//! - Fair resource sharing across sandboxes
//! - Cost metering for credit system
//! - Preemptive multitasking support

use thiserror::Error;
use wasmtime::Store;

// ═══════════════════════════════════════════════════════════════════════════
// CONSTANTS
// ═══════════════════════════════════════════════════════════════════════════

/// Default fuel allocation: 1 billion units
///
/// This provides a reasonable balance between:
/// - Allowing substantial computation
/// - Preventing runaway execution
/// - Fair scheduling across multiple sandboxes
pub const DEFAULT_FUEL: u64 = 1_000_000_000;

/// Approximate fuel consumption per second of execution
///
/// This is a rough estimate based on typical WASM instruction throughput.
/// Actual consumption varies based on:
/// - Instruction complexity (i64.add vs. memory.grow)
/// - Host function calls
/// - Memory access patterns
///
/// Benchmark estimate: ~100M instructions/second on modern CPU
pub const FUEL_PER_SECOND: u64 = 100_000_000;

/// Maximum fuel that can be allocated to prevent overflow
pub const MAX_FUEL: u64 = u64::MAX / 2;

// ═══════════════════════════════════════════════════════════════════════════
// ERROR TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// Errors that can occur during fuel management operations
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum FuelError {
    /// Fuel has been exhausted and execution cannot continue
    #[error("fuel exhausted: {consumed} units consumed, {remaining} remaining")]
    Exhausted { consumed: u64, remaining: u64 },

    /// Operation would exceed the maximum allowed fuel limit
    #[error("fuel over limit: requested {requested}, max allowed {max}")]
    OverLimit { requested: u64, max: u64 },

    /// Invalid fuel amount provided (e.g., zero when non-zero expected)
    #[error("invalid fuel amount: {amount} (reason: {reason})")]
    InvalidAmount { amount: u64, reason: String },
}

// ═══════════════════════════════════════════════════════════════════════════
// FUEL MANAGER
// ═══════════════════════════════════════════════════════════════════════════

/// Fuel manager that wraps wasmtime's fuel system with additional tracking
///
/// The FuelManager maintains:
/// - Initial fuel allocation
/// - Current remaining fuel
/// - Total consumed fuel across refueling cycles
///
/// # Example
///
/// ```no_run
/// use vudo_vm::fuel::{FuelManager, DEFAULT_FUEL};
///
/// let mut manager = FuelManager::new(DEFAULT_FUEL);
/// assert_eq!(manager.remaining(), DEFAULT_FUEL);
///
/// // Simulate consumption
/// manager.consume(1000).unwrap();
/// assert_eq!(manager.remaining(), DEFAULT_FUEL - 1000);
/// assert_eq!(manager.total_consumed(), 1000);
///
/// // Check exhaustion
/// assert!(!manager.is_exhausted());
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FuelManager {
    /// Initial fuel allocated when manager was created or last refueled to max
    initial_fuel: u64,

    /// Current remaining fuel available for execution
    remaining_fuel: u64,

    /// Total fuel consumed across all consumption operations
    /// This tracks cumulative usage even across refueling
    consumed_fuel: u64,
}

impl FuelManager {
    /// Creates a new FuelManager with the specified initial fuel
    ///
    /// # Arguments
    ///
    /// * `initial_fuel` - The starting fuel amount (must be <= MAX_FUEL)
    ///
    /// # Returns
    ///
    /// A new FuelManager instance with full fuel
    ///
    /// # Panics
    ///
    /// Panics if `initial_fuel` exceeds `MAX_FUEL`
    ///
    /// # Example
    ///
    /// ```no_run
    /// use vudo_vm::fuel::FuelManager;
    ///
    /// let manager = FuelManager::new(1_000_000);
    /// assert_eq!(manager.remaining(), 1_000_000);
    /// ```
    pub fn new(initial_fuel: u64) -> Self {
        assert!(
            initial_fuel <= MAX_FUEL,
            "initial fuel {} exceeds maximum {}",
            initial_fuel,
            MAX_FUEL
        );

        Self {
            initial_fuel,
            remaining_fuel: initial_fuel,
            consumed_fuel: 0,
        }
    }

    /// Consumes the specified amount of fuel
    ///
    /// This method:
    /// 1. Checks if sufficient fuel remains
    /// 2. Deducts from remaining fuel
    /// 3. Adds to total consumed
    ///
    /// # Arguments
    ///
    /// * `amount` - The fuel to consume
    ///
    /// # Returns
    ///
    /// - `Ok(())` if fuel was successfully consumed
    /// - `Err(FuelError::Exhausted)` if insufficient fuel remains
    /// - `Err(FuelError::InvalidAmount)` if amount is 0
    ///
    /// # Example
    ///
    /// ```no_run
    /// use vudo_vm::fuel::FuelManager;
    ///
    /// let mut manager = FuelManager::new(1000);
    /// manager.consume(100).unwrap();
    /// assert_eq!(manager.remaining(), 900);
    /// ```
    pub fn consume(&mut self, amount: u64) -> Result<(), FuelError> {
        if amount == 0 {
            return Err(FuelError::InvalidAmount {
                amount: 0,
                reason: "cannot consume zero fuel".to_string(),
            });
        }

        if amount > self.remaining_fuel {
            return Err(FuelError::Exhausted {
                consumed: self.consumed_fuel,
                remaining: self.remaining_fuel,
            });
        }

        self.remaining_fuel -= amount;
        self.consumed_fuel += amount;

        Ok(())
    }

    /// Adds fuel back to the manager (refueling)
    ///
    /// This method:
    /// 1. Validates the refuel amount
    /// 2. Ensures total won't exceed MAX_FUEL
    /// 3. Adds to remaining fuel
    ///
    /// Note: Refueling does NOT reset the consumed_fuel counter, which tracks
    /// total historical consumption for billing/metrics purposes.
    ///
    /// # Arguments
    ///
    /// * `amount` - The fuel to add
    ///
    /// # Returns
    ///
    /// - `Ok(())` if fuel was successfully added
    /// - `Err(FuelError::OverLimit)` if refueling would exceed MAX_FUEL
    /// - `Err(FuelError::InvalidAmount)` if amount is 0
    ///
    /// # Example
    ///
    /// ```no_run
    /// use vudo_vm::fuel::FuelManager;
    ///
    /// let mut manager = FuelManager::new(1000);
    /// manager.consume(500).unwrap();
    /// manager.refuel(300).unwrap();
    /// assert_eq!(manager.remaining(), 800);
    /// ```
    pub fn refuel(&mut self, amount: u64) -> Result<(), FuelError> {
        if amount == 0 {
            return Err(FuelError::InvalidAmount {
                amount: 0,
                reason: "cannot refuel with zero fuel".to_string(),
            });
        }

        let new_remaining =
            self.remaining_fuel
                .checked_add(amount)
                .ok_or(FuelError::OverLimit {
                    requested: amount,
                    max: MAX_FUEL,
                })?;

        if new_remaining > MAX_FUEL {
            return Err(FuelError::OverLimit {
                requested: amount,
                max: MAX_FUEL - self.remaining_fuel,
            });
        }

        self.remaining_fuel = new_remaining;

        Ok(())
    }

    /// Returns the amount of fuel currently remaining
    ///
    /// # Example
    ///
    /// ```no_run
    /// use vudo_vm::fuel::FuelManager;
    ///
    /// let mut manager = FuelManager::new(1000);
    /// manager.consume(250).unwrap();
    /// assert_eq!(manager.remaining(), 750);
    /// ```
    pub fn remaining(&self) -> u64 {
        self.remaining_fuel
    }

    /// Checks if fuel is exhausted (remaining == 0)
    ///
    /// # Returns
    ///
    /// `true` if no fuel remains, `false` otherwise
    ///
    /// # Example
    ///
    /// ```no_run
    /// use vudo_vm::fuel::FuelManager;
    ///
    /// let mut manager = FuelManager::new(100);
    /// assert!(!manager.is_exhausted());
    ///
    /// manager.consume(100).unwrap();
    /// assert!(manager.is_exhausted());
    /// ```
    pub fn is_exhausted(&self) -> bool {
        self.remaining_fuel == 0
    }

    /// Returns the total amount of fuel consumed historically
    ///
    /// This value tracks cumulative consumption even across refueling operations,
    /// making it useful for:
    /// - Billing/credit calculations
    /// - Performance metrics
    /// - Resource usage tracking
    ///
    /// # Example
    ///
    /// ```no_run
    /// use vudo_vm::fuel::FuelManager;
    ///
    /// let mut manager = FuelManager::new(1000);
    /// manager.consume(300).unwrap();
    /// manager.refuel(200).unwrap();
    /// manager.consume(100).unwrap();
    ///
    /// assert_eq!(manager.total_consumed(), 400); // 300 + 100
    /// ```
    pub fn total_consumed(&self) -> u64 {
        self.consumed_fuel
    }

    /// Returns the initial fuel amount
    ///
    /// # Example
    ///
    /// ```no_run
    /// use vudo_vm::fuel::FuelManager;
    ///
    /// let manager = FuelManager::new(5000);
    /// assert_eq!(manager.initial_fuel(), 5000);
    /// ```
    pub fn initial_fuel(&self) -> u64 {
        self.initial_fuel
    }

    /// Resets the manager to initial state
    ///
    /// This:
    /// - Restores remaining fuel to initial allocation
    /// - Resets consumed counter to 0
    ///
    /// Use this when restarting execution from a clean state.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use vudo_vm::fuel::FuelManager;
    ///
    /// let mut manager = FuelManager::new(1000);
    /// manager.consume(500).unwrap();
    /// manager.reset();
    ///
    /// assert_eq!(manager.remaining(), 1000);
    /// assert_eq!(manager.total_consumed(), 0);
    /// ```
    pub fn reset(&mut self) {
        self.remaining_fuel = self.initial_fuel;
        self.consumed_fuel = 0;
    }
}

impl Default for FuelManager {
    /// Creates a FuelManager with DEFAULT_FUEL
    fn default() -> Self {
        Self::new(DEFAULT_FUEL)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// WASMTIME INTEGRATION
// ═══════════════════════════════════════════════════════════════════════════

/// Configures a wasmtime Store with the specified fuel amount
///
/// This function:
/// 1. Enables fuel consumption in the store
/// 2. Sets the initial fuel to the specified amount
///
/// # Arguments
///
/// * `store` - Mutable reference to a wasmtime Store
/// * `fuel` - The fuel amount to allocate
///
/// # Type Parameters
///
/// * `T` - The store's user data type
///
/// # Example
///
/// ```no_run
/// use wasmtime::{Engine, Store};
/// use vudo_vm::fuel::{configure_store_with_fuel, DEFAULT_FUEL};
///
/// let engine = Engine::default();
/// let mut store = Store::new(&engine, ());
/// configure_store_with_fuel(&mut store, DEFAULT_FUEL);
/// ```
pub fn configure_store_with_fuel<T>(store: &mut Store<T>, fuel: u64) {
    // Enable fuel consumption tracking
    store.set_fuel(fuel).expect("failed to set fuel");
}

/// Synchronizes a FuelManager with a wasmtime Store's fuel state
///
/// This function:
/// 1. Reads the current fuel from the store
/// 2. Calculates fuel consumed since last sync
/// 3. Updates the FuelManager's state
///
/// Call this periodically or after execution to keep the FuelManager in sync
/// with wasmtime's actual fuel consumption.
///
/// # Arguments
///
/// * `store` - Reference to a wasmtime Store
/// * `manager` - Mutable reference to a FuelManager
///
/// # Returns
///
/// The amount of fuel consumed since last sync
///
/// # Example
///
/// ```no_run
/// use wasmtime::{Engine, Store};
/// use vudo_vm::fuel::{FuelManager, sync_fuel_from_store, configure_store_with_fuel};
///
/// let engine = Engine::default();
/// let mut store = Store::new(&engine, ());
/// let mut manager = FuelManager::new(1000);
///
/// configure_store_with_fuel(&mut store, manager.remaining());
/// // ... execute some WASM ...
/// let consumed = sync_fuel_from_store(&store, &mut manager);
/// ```
pub fn sync_fuel_from_store<T>(store: &Store<T>, manager: &mut FuelManager) -> u64 {
    // Get current fuel from store
    let store_fuel = store.get_fuel().expect("fuel not enabled in store");

    // Calculate how much was consumed
    let previous_remaining = manager.remaining();

    if store_fuel < previous_remaining {
        let consumed = previous_remaining - store_fuel;
        // Update manager to match store's state
        manager.consume(consumed).ok(); // Ignore errors, we're syncing
        consumed
    } else {
        // No consumption or fuel was added
        0
    }
}

/// Applies a FuelManager's state to a wasmtime Store
///
/// This sets the store's fuel to match the manager's remaining fuel.
/// Use this after refueling the manager to update the store.
///
/// # Arguments
///
/// * `store` - Mutable reference to a wasmtime Store
/// * `manager` - Reference to a FuelManager
///
/// # Example
///
/// ```no_run
/// use wasmtime::{Engine, Store};
/// use vudo_vm::fuel::{FuelManager, apply_fuel_to_store};
///
/// let engine = Engine::default();
/// let mut store = Store::new(&engine, ());
/// let manager = FuelManager::new(5000);
///
/// apply_fuel_to_store(&mut store, &manager);
/// assert_eq!(store.get_fuel().unwrap(), 5000);
/// ```
pub fn apply_fuel_to_store<T>(store: &mut Store<T>, manager: &FuelManager) {
    store
        .set_fuel(manager.remaining())
        .expect("failed to apply fuel to store");
}

// ═══════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_fuel_manager() {
        let manager = FuelManager::new(1000);
        assert_eq!(manager.initial_fuel(), 1000);
        assert_eq!(manager.remaining(), 1000);
        assert_eq!(manager.total_consumed(), 0);
        assert!(!manager.is_exhausted());
    }

    #[test]
    fn test_default_fuel_manager() {
        let manager = FuelManager::default();
        assert_eq!(manager.initial_fuel(), DEFAULT_FUEL);
        assert_eq!(manager.remaining(), DEFAULT_FUEL);
    }

    #[test]
    fn test_consume_fuel() {
        let mut manager = FuelManager::new(1000);

        manager.consume(300).unwrap();
        assert_eq!(manager.remaining(), 700);
        assert_eq!(manager.total_consumed(), 300);

        manager.consume(200).unwrap();
        assert_eq!(manager.remaining(), 500);
        assert_eq!(manager.total_consumed(), 500);
    }

    #[test]
    fn test_consume_exhausts_fuel() {
        let mut manager = FuelManager::new(100);

        manager.consume(100).unwrap();
        assert_eq!(manager.remaining(), 0);
        assert!(manager.is_exhausted());

        let result = manager.consume(1);
        assert!(matches!(result, Err(FuelError::Exhausted { .. })));
    }

    #[test]
    fn test_consume_zero_is_error() {
        let mut manager = FuelManager::new(1000);
        let result = manager.consume(0);
        assert!(matches!(result, Err(FuelError::InvalidAmount { .. })));
    }

    #[test]
    fn test_refuel() {
        let mut manager = FuelManager::new(1000);

        manager.consume(600).unwrap();
        assert_eq!(manager.remaining(), 400);

        manager.refuel(300).unwrap();
        assert_eq!(manager.remaining(), 700);
        assert_eq!(manager.total_consumed(), 600); // Consumed doesn't reset
    }

    #[test]
    fn test_refuel_zero_is_error() {
        let mut manager = FuelManager::new(1000);
        let result = manager.refuel(0);
        assert!(matches!(result, Err(FuelError::InvalidAmount { .. })));
    }

    #[test]
    fn test_refuel_over_limit() {
        let mut manager = FuelManager::new(MAX_FUEL - 100);
        let result = manager.refuel(200);
        assert!(matches!(result, Err(FuelError::OverLimit { .. })));
    }

    #[test]
    fn test_reset() {
        let mut manager = FuelManager::new(1000);

        manager.consume(400).unwrap();
        manager.refuel(200).unwrap();
        assert_eq!(manager.remaining(), 800);
        assert_eq!(manager.total_consumed(), 400);

        manager.reset();
        assert_eq!(manager.remaining(), 1000);
        assert_eq!(manager.total_consumed(), 0);
    }

    #[test]
    fn test_conservation_law() {
        // Verify fuel conservation: consumed + remaining should track properly
        let mut manager = FuelManager::new(1000);

        manager.consume(300).unwrap();
        assert_eq!(manager.consumed_fuel + manager.remaining_fuel, 1000);

        manager.consume(200).unwrap();
        assert_eq!(manager.consumed_fuel + manager.remaining_fuel, 1000);

        manager.refuel(100).unwrap();
        // After refuel, consumed + remaining > initial (refueled state)
        assert_eq!(manager.remaining(), 600);
        assert_eq!(manager.total_consumed(), 500);
    }

    #[test]
    #[should_panic(expected = "exceeds maximum")]
    fn test_new_with_excessive_fuel_panics() {
        FuelManager::new(MAX_FUEL + 1);
    }

    #[test]
    fn test_wasmtime_integration() {
        use wasmtime::{Engine, Store};

        let engine = Engine::default();
        let mut store = Store::new(&engine, ());
        let manager = FuelManager::new(5000);

        configure_store_with_fuel(&mut store, manager.remaining());
        assert_eq!(store.get_fuel().unwrap(), 5000);

        apply_fuel_to_store(&mut store, &manager);
        assert_eq!(store.get_fuel().unwrap(), 5000);
    }

    #[test]
    fn test_sync_fuel_from_store() {
        use wasmtime::{Engine, Store};

        let engine = Engine::default();
        let mut store = Store::new(&engine, ());
        let mut manager = FuelManager::new(1000);

        configure_store_with_fuel(&mut store, manager.remaining());

        // Simulate consumption by manually setting lower fuel
        store.set_fuel(700).unwrap();

        let consumed = sync_fuel_from_store(&store, &mut manager);
        assert_eq!(consumed, 300);
        assert_eq!(manager.remaining(), 700);
        assert_eq!(manager.total_consumed(), 300);
    }

    #[test]
    fn test_fuel_error_display() {
        let err = FuelError::Exhausted {
            consumed: 1000,
            remaining: 0,
        };
        assert_eq!(
            err.to_string(),
            "fuel exhausted: 1000 units consumed, 0 remaining"
        );

        let err = FuelError::OverLimit {
            requested: 500,
            max: 100,
        };
        assert_eq!(
            err.to_string(),
            "fuel over limit: requested 500, max allowed 100"
        );

        let err = FuelError::InvalidAmount {
            amount: 0,
            reason: "cannot be zero".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "invalid fuel amount: 0 (reason: cannot be zero)"
        );
    }
}
