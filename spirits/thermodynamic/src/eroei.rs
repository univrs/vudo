//! EROEI Calculator - Generated from DOL
//! Energy Return on Energy Invested calculations for thermodynamic economics

use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};

/// Energy component with EROEI accounting
#[wasm_bindgen]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnergyComponent {
    pub name_id: i64,
    pub component_type: i64,
    pub energy_output_kwh_year: f64,
    pub energy_input_kwh_year: f64,
    pub embodied_energy_kwh: f64,
    pub lifespan_years: f64,
}

#[wasm_bindgen]
impl EnergyComponent {
    #[wasm_bindgen(constructor)]
    pub fn new(
        name_id: i64,
        component_type: i64,
        energy_output_kwh_year: f64,
        energy_input_kwh_year: f64,
        embodied_energy_kwh: f64,
        lifespan_years: f64,
    ) -> Self {
        Self {
            name_id,
            component_type,
            energy_output_kwh_year,
            energy_input_kwh_year,
            embodied_energy_kwh,
            lifespan_years,
        }
    }

    /// Embodied energy amortized over lifespan
    pub fn annualized_embodied(&self) -> f64 {
        self.embodied_energy_kwh / self.lifespan_years
    }

    /// Total annual energy input including amortized embodied
    pub fn total_annual_input(&self) -> f64 {
        let amortized = self.embodied_energy_kwh / self.lifespan_years;
        self.energy_input_kwh_year + amortized
    }

    /// EROEI for this component
    pub fn eroei(&self) -> f64 {
        let input = self.energy_input_kwh_year + (self.embodied_energy_kwh / self.lifespan_years);
        if input <= 0.0 {
            return 999999.0;
        }
        self.energy_output_kwh_year / input
    }
}

/// Energy system metrics aggregator
#[wasm_bindgen]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnergySystemMetrics {
    pub total_output_kwh: f64,
    pub total_input_kwh: f64,
    pub component_count: i64,
}

#[wasm_bindgen]
impl EnergySystemMetrics {
    #[wasm_bindgen(constructor)]
    pub fn new(total_output_kwh: f64, total_input_kwh: f64, component_count: i64) -> Self {
        Self {
            total_output_kwh,
            total_input_kwh,
            component_count,
        }
    }

    /// System-wide EROEI
    pub fn system_eroei(&self) -> f64 {
        if self.total_input_kwh <= 0.0 {
            return 0.0;
        }
        self.total_output_kwh / self.total_input_kwh
    }

    /// Net energy available for useful work
    pub fn net_energy(&self) -> f64 {
        self.total_output_kwh - self.total_input_kwh
    }

    /// Returns 1.0 if viable (EROEI >= 7), 0.0 otherwise
    pub fn is_viable(&self) -> f64 {
        let eroei = self.total_output_kwh / self.total_input_kwh;
        if eroei >= 7.0 {
            return 1.0;
        }
        0.0
    }

    /// Viability level: 0=Non-viable, 1=Subsistence, 2=Critical, 3=Marginal, 4=Good, 5=Excellent
    pub fn viability_level(&self) -> i64 {
        let eroei = self.total_output_kwh / self.total_input_kwh;
        if eroei >= 20.0 {
            return 5;
        }
        if eroei >= 12.0 {
            return 4;
        }
        if eroei >= 7.0 {
            return 3;
        }
        if eroei >= 5.0 {
            return 2;
        }
        if eroei >= 3.0 {
            return 1;
        }
        0
    }

    /// Viability assessment string
    pub fn viability_assessment(&self) -> String {
        match self.viability_level() {
            5 => "Excellent - Supports complex technology development".to_string(),
            4 => "Good - Supports education, healthcare, R&D".to_string(),
            3 => "Marginal - Can maintain infrastructure".to_string(),
            2 => "Critical - Basic industrial activity only".to_string(),
            1 => "Subsistence - Basic agriculture only".to_string(),
            _ => "Non-viable - Cannot sustain society".to_string(),
        }
    }
}

/// Returns EROEI for 1 MW solar PV system
#[wasm_bindgen]
pub fn create_solar_pv_eroei() -> f64 {
    let output = 1500000.0;
    let operational = 10000.0;
    let annualized_embodied = 60000.0;
    let total_input = operational + annualized_embodied;
    output / total_input
}

/// Returns annual energy consumption in kWh for Hyphal network nodes
#[wasm_bindgen]
pub fn hyphal_node_energy(num_nodes: i64, power_per_node_w: f64) -> f64 {
    let hours_per_year = 8760.0;
    let node_count = num_nodes as f64;
    node_count * power_per_node_w * hours_per_year / 1000.0
}

/// Calculate max nodes supportable by solar capacity
#[wasm_bindgen]
pub fn max_supported_nodes(solar_mw: f64, node_power_w: f64) -> i64 {
    let capacity_factor = 0.17;
    let distribution_efficiency = 0.80;
    let hours_per_year = 8760.0;

    let annual_kwh = solar_mw * 1000.0 * capacity_factor * hours_per_year;
    let usable_kwh = annual_kwh * distribution_efficiency;
    let per_node_kwh = node_power_w * hours_per_year / 1000.0;

    (usable_kwh / per_node_kwh) as i64
}

/// Create example solar system metrics
#[wasm_bindgen]
pub fn solar_system_example() -> EnergySystemMetrics {
    // 1 MW solar + battery + distribution
    EnergySystemMetrics::new(
        4132500.0,  // Total output kWh/year
        135333.0,   // Total input kWh/year
        3,          // 3 components
    )
}

/// Create hyphal network metrics (pure consumer)
#[wasm_bindgen]
pub fn hyphal_network_example(num_nodes: i64) -> EnergySystemMetrics {
    let node_energy = hyphal_node_energy(num_nodes, 100.0);
    let network_overhead = node_energy * 0.2;

    EnergySystemMetrics::new(
        0.0,                             // No output
        node_energy + network_overhead,  // All consumption
        num_nodes,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solar_eroei() {
        let eroei = create_solar_pv_eroei();
        assert!(eroei > 20.0, "Solar EROEI should be > 20");
    }

    #[test]
    fn test_max_nodes() {
        let nodes = max_supported_nodes(1.0, 100.0);
        // With 17% capacity factor and 80% distribution efficiency:
        // 1 MW * 0.17 * 0.80 * 8760h = 1,191,360 kWh usable
        // 100W * 8760h = 876 kWh per node
        // ~1360 nodes supported
        assert!(nodes > 1000, "1 MW should support > 1000 nodes at 100W each");
        assert!(nodes < 2000, "Calculation should give ~1360 nodes");
    }

    #[test]
    fn test_viability() {
        let solar = solar_system_example();
        assert_eq!(solar.viability_level(), 5);

        let hyphal = hyphal_network_example(1000);
        assert_eq!(hyphal.viability_level(), 0);
    }
}
