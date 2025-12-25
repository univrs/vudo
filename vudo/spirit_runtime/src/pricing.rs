//! Pricing Models for Spirit Execution
//!
//! Defines credit costs for running Spirits in the VUDO VM.

use serde::{Deserialize, Serialize};

/// Pricing model for Spirit execution credits
///
/// Credits are the unit of resource consumption in VUDO.
/// Each Spirit defines its pricing model in the manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PricingModel {
    /// Base cost to invoke the Spirit (in microcredits)
    #[serde(default = "default_base_cost")]
    pub base_cost: u64,

    /// Cost per fuel unit consumed (in microcredits)
    #[serde(default = "default_per_fuel_cost")]
    pub per_fuel_cost: u64,

    /// Cost per byte of memory used (in microcredits)
    #[serde(default)]
    pub per_memory_byte_cost: u64,

    /// Cost per storage read operation (in microcredits)
    #[serde(default)]
    pub per_storage_read_cost: u64,

    /// Cost per storage write operation (in microcredits)
    #[serde(default)]
    pub per_storage_write_cost: u64,

    /// Cost per network operation (in microcredits)
    #[serde(default)]
    pub per_network_op_cost: u64,

    /// Minimum credits required to start execution
    #[serde(default = "default_min_balance")]
    pub min_balance: u64,
}

fn default_base_cost() -> u64 {
    100 // 100 microcredits base cost
}

fn default_per_fuel_cost() -> u64 {
    1 // 1 microcredit per 1000 fuel units
}

fn default_min_balance() -> u64 {
    1000 // Minimum 1000 microcredits to run
}

impl Default for PricingModel {
    fn default() -> Self {
        Self {
            base_cost: default_base_cost(),
            per_fuel_cost: default_per_fuel_cost(),
            per_memory_byte_cost: 0,
            per_storage_read_cost: 10,
            per_storage_write_cost: 100,
            per_network_op_cost: 50,
            min_balance: default_min_balance(),
        }
    }
}

impl PricingModel {
    /// Create a new pricing model
    pub fn new(base_cost: u64, per_fuel_cost: u64) -> Self {
        Self {
            base_cost,
            per_fuel_cost,
            ..Default::default()
        }
    }

    /// Create a free pricing model (for testing/development)
    pub fn free() -> Self {
        Self {
            base_cost: 0,
            per_fuel_cost: 0,
            per_memory_byte_cost: 0,
            per_storage_read_cost: 0,
            per_storage_write_cost: 0,
            per_network_op_cost: 0,
            min_balance: 0,
        }
    }

    /// Calculate total cost for an execution
    pub fn calculate_cost(&self, metrics: &ExecutionMetrics) -> CreditCost {
        let fuel_cost = (metrics.fuel_consumed * self.per_fuel_cost) / 1000;
        let memory_cost = metrics.peak_memory * self.per_memory_byte_cost;
        let storage_read_cost = metrics.storage_reads as u64 * self.per_storage_read_cost;
        let storage_write_cost = metrics.storage_writes as u64 * self.per_storage_write_cost;
        let network_cost = metrics.network_ops as u64 * self.per_network_op_cost;

        CreditCost {
            base: self.base_cost,
            fuel: fuel_cost,
            memory: memory_cost,
            storage_read: storage_read_cost,
            storage_write: storage_write_cost,
            network: network_cost,
            total: self.base_cost
                + fuel_cost
                + memory_cost
                + storage_read_cost
                + storage_write_cost
                + network_cost,
        }
    }

    /// Check if a balance is sufficient to start execution
    pub fn can_execute(&self, balance: u64) -> bool {
        balance >= self.min_balance
    }

    /// Estimate maximum cost for given resource limits
    pub fn estimate_max_cost(&self, fuel_limit: u64, memory_limit: u64) -> u64 {
        self.base_cost
            + (fuel_limit * self.per_fuel_cost) / 1000
            + memory_limit * self.per_memory_byte_cost
    }
}

/// Breakdown of credit costs for an execution
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CreditCost {
    /// Base invocation cost
    pub base: u64,
    /// Cost from fuel consumption
    pub fuel: u64,
    /// Cost from memory usage
    pub memory: u64,
    /// Cost from storage reads
    pub storage_read: u64,
    /// Cost from storage writes
    pub storage_write: u64,
    /// Cost from network operations
    pub network: u64,
    /// Total cost (sum of all components)
    pub total: u64,
}

impl CreditCost {
    /// Create a zero cost
    pub fn zero() -> Self {
        Self::default()
    }
}

/// Execution metrics used for pricing calculation
#[derive(Debug, Clone, Default)]
pub struct ExecutionMetrics {
    /// Fuel units consumed
    pub fuel_consumed: u64,
    /// Peak memory usage in bytes
    pub peak_memory: u64,
    /// Number of storage read operations
    pub storage_reads: u32,
    /// Number of storage write operations
    pub storage_writes: u32,
    /// Number of network operations
    pub network_ops: u32,
}

impl ExecutionMetrics {
    /// Create new metrics
    pub fn new() -> Self {
        Self::default()
    }

    /// Record fuel consumption
    pub fn record_fuel(&mut self, amount: u64) {
        self.fuel_consumed += amount;
    }

    /// Record memory usage (updates peak if higher)
    pub fn record_memory(&mut self, bytes: u64) {
        self.peak_memory = self.peak_memory.max(bytes);
    }

    /// Record a storage read
    pub fn record_storage_read(&mut self) {
        self.storage_reads += 1;
    }

    /// Record a storage write
    pub fn record_storage_write(&mut self) {
        self.storage_writes += 1;
    }

    /// Record a network operation
    pub fn record_network_op(&mut self) {
        self.network_ops += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pricing_default() {
        let pricing = PricingModel::default();
        assert_eq!(pricing.base_cost, 100);
        assert_eq!(pricing.per_fuel_cost, 1);
        assert_eq!(pricing.min_balance, 1000);
    }

    #[test]
    fn test_pricing_free() {
        let pricing = PricingModel::free();
        assert_eq!(pricing.base_cost, 0);
        assert_eq!(pricing.per_fuel_cost, 0);
        assert!(pricing.can_execute(0));
    }

    #[test]
    fn test_calculate_cost() {
        let pricing = PricingModel::new(100, 1);

        let metrics = ExecutionMetrics {
            fuel_consumed: 10000, // 10 credits worth of fuel
            peak_memory: 0,
            storage_reads: 0,
            storage_writes: 0,
            network_ops: 0,
        };

        let cost = pricing.calculate_cost(&metrics);
        assert_eq!(cost.base, 100);
        assert_eq!(cost.fuel, 10); // 10000 / 1000 * 1
        assert_eq!(cost.total, 110);
    }

    #[test]
    fn test_calculate_cost_with_storage() {
        let pricing = PricingModel {
            per_storage_read_cost: 10,
            per_storage_write_cost: 100,
            ..PricingModel::default()
        };

        let metrics = ExecutionMetrics {
            fuel_consumed: 0,
            peak_memory: 0,
            storage_reads: 5,
            storage_writes: 2,
            network_ops: 0,
        };

        let cost = pricing.calculate_cost(&metrics);
        assert_eq!(cost.storage_read, 50);
        assert_eq!(cost.storage_write, 200);
    }

    #[test]
    fn test_can_execute() {
        let pricing = PricingModel::default();

        assert!(!pricing.can_execute(500));
        assert!(pricing.can_execute(1000));
        assert!(pricing.can_execute(2000));
    }

    #[test]
    fn test_execution_metrics() {
        let mut metrics = ExecutionMetrics::new();

        metrics.record_fuel(1000);
        metrics.record_fuel(500);
        assert_eq!(metrics.fuel_consumed, 1500);

        metrics.record_memory(1024);
        metrics.record_memory(512);
        assert_eq!(metrics.peak_memory, 1024); // Peak, not cumulative

        metrics.record_storage_read();
        metrics.record_storage_read();
        metrics.record_storage_write();
        assert_eq!(metrics.storage_reads, 2);
        assert_eq!(metrics.storage_writes, 1);
    }

    #[test]
    fn test_estimate_max_cost() {
        let pricing = PricingModel::new(100, 1);

        let max_cost = pricing.estimate_max_cost(1_000_000, 0);
        assert_eq!(max_cost, 100 + 1000); // base + fuel
    }
}
