//! Thermodynamic Economics Spirit
//!
//! EROEI and Small-World calculators for thermodynamic economics.
//! Generated from DOL schemas and compiled to native + WASM targets.

use wasm_bindgen::prelude::*;

pub mod eroei;
pub mod small_world;

// Re-export main types for convenience
pub use eroei::{
    EnergyComponent,
    EnergySystemMetrics,
    create_solar_pv_eroei,
    hyphal_node_energy,
    max_supported_nodes,
    solar_system_example,
    hyphal_network_example,
};

pub use small_world::{
    Edge,
    GraphMetrics,
    SmallWorldMetrics,
    random_clustering,
    random_path_length,
    calculate_sigma,
    is_small_world_network,
    small_network_example,
    dunbar_cluster_example,
};

/// Initialize WASM module (called automatically)
#[wasm_bindgen(start)]
pub fn init() {
    // Module initialization complete
}

/// Version of the thermodynamic Spirit
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Run complete analysis (EROEI + Small-World) and return summary
#[wasm_bindgen]
pub fn analyze_ecosystem(
    solar_mw: f64,
    node_count: i64,
    node_power_w: f64,
    network_degree: f64,
    clustering: f64,
    path_length: f64,
) -> String {
    // EROEI analysis
    let solar = solar_system_example();
    let hyphal = hyphal_network_example(node_count);

    let combined_output = solar.total_output_kwh;
    let combined_input = solar.total_input_kwh + hyphal.total_input_kwh;
    let combined = EnergySystemMetrics::new(combined_output, combined_input, solar.component_count + node_count);

    let max_nodes = max_supported_nodes(solar_mw, node_power_w);

    // Small-world analysis
    let c_random = random_clustering(node_count, network_degree);
    let l_random = random_path_length(node_count, network_degree);
    let sigma = calculate_sigma(clustering, path_length, c_random, l_random);

    format!(
        r#"=== Thermodynamic Economics Analysis ===

EROEI Analysis:
  Solar system EROEI: {:.2}
  Hyphal network EROEI: {:.2}
  Combined EROEI: {:.2}
  Viability: {}
  Max nodes (1 MW): {}

Small-World Analysis:
  Nodes: {}
  Degree: {:.1}
  Clustering: {:.3}
  Path length: {:.2}
  Sigma: {:.2}
  Small-world: {}
"#,
        solar.system_eroei(),
        hyphal.system_eroei(),
        combined.system_eroei(),
        combined.viability_assessment(),
        max_nodes,
        node_count,
        network_degree,
        clustering,
        path_length,
        sigma,
        if sigma > 1.0 { "YES" } else { "NO" }
    )
}
