//! WASM exports with wasm-bindgen annotations
//! This module wraps the DOL-generated types for browser use

use wasm_bindgen::prelude::*;
use crate::{
    EnergySystemMetrics as CoreEnergySystemMetrics,
    SmallWorldMetrics as CoreSmallWorldMetrics,
    create_solar_pv_eroei as core_create_solar_pv_eroei,
    solar_system_example as core_solar_system_example,
    hyphal_node_energy as core_hyphal_node_energy,
    hyphal_network_example as core_hyphal_network_example,
    max_supported_nodes as core_max_supported_nodes,
    random_clustering as core_random_clustering,
    random_path_length as core_random_path_length,
    calculate_sigma as core_calculate_sigma,
    dunbar_cluster_example as core_dunbar_cluster_example,
};

// EROEI Functions

#[wasm_bindgen]
pub fn create_solar_pv_eroei() -> f64 {
    core_create_solar_pv_eroei()
}

#[wasm_bindgen]
pub fn hyphal_node_energy(num_nodes: i64, power_per_node_w: f64) -> f64 {
    core_hyphal_node_energy(num_nodes, power_per_node_w)
}

#[wasm_bindgen]
pub fn max_supported_nodes(solar_mw: f64, node_power_w: f64) -> i64 {
    core_max_supported_nodes(solar_mw, node_power_w)
}

// Small-World Functions

#[wasm_bindgen]
pub fn random_clustering(n: i64, k: f64) -> f64 {
    core_random_clustering(n, k)
}

#[wasm_bindgen]
pub fn random_path_length(n: i64, k: f64) -> f64 {
    core_random_path_length(n, k)
}

#[wasm_bindgen]
pub fn calculate_sigma(c: f64, l: f64, c_random: f64, l_random: f64) -> f64 {
    core_calculate_sigma(c, l, c_random, l_random)
}

// WASM wrapper for EnergySystemMetrics
#[wasm_bindgen]
pub struct EnergySystemMetrics {
    inner: CoreEnergySystemMetrics,
}

#[wasm_bindgen]
impl EnergySystemMetrics {
    #[wasm_bindgen(getter)]
    pub fn total_output_kwh(&self) -> f64 {
        self.inner.total_output_kwh
    }

    #[wasm_bindgen(getter)]
    pub fn total_input_kwh(&self) -> f64 {
        self.inner.total_input_kwh
    }

    #[wasm_bindgen(getter)]
    pub fn component_count(&self) -> i64 {
        self.inner.component_count
    }

    pub fn system_eroei(&self) -> f64 {
        self.inner.system_eroei()
    }

    pub fn net_energy(&self) -> f64 {
        self.inner.net_energy()
    }

    pub fn is_viable(&self) -> f64 {
        self.inner.is_viable()
    }

    pub fn viability_level(&self) -> i64 {
        self.inner.viability_level()
    }

    pub fn viability_assessment(&self) -> i64 {
        self.inner.viability_assessment()
    }
}

#[wasm_bindgen]
pub fn solar_system_example() -> EnergySystemMetrics {
    EnergySystemMetrics {
        inner: core_solar_system_example(),
    }
}

#[wasm_bindgen]
pub fn hyphal_network_example(num_nodes: i64) -> EnergySystemMetrics {
    EnergySystemMetrics {
        inner: core_hyphal_network_example(num_nodes),
    }
}

// WASM wrapper for SmallWorldMetrics
#[wasm_bindgen]
pub struct SmallWorldMetrics {
    inner: CoreSmallWorldMetrics,
}

#[wasm_bindgen]
impl SmallWorldMetrics {
    #[wasm_bindgen(getter)]
    pub fn n(&self) -> i64 {
        self.inner.n
    }

    #[wasm_bindgen(getter)]
    pub fn m(&self) -> i64 {
        self.inner.m
    }

    #[wasm_bindgen(getter)]
    pub fn k(&self) -> f64 {
        self.inner.k
    }

    #[wasm_bindgen(getter)]
    pub fn clustering(&self) -> f64 {
        self.inner.clustering
    }

    #[wasm_bindgen(getter)]
    pub fn path_length(&self) -> f64 {
        self.inner.path_length
    }

    #[wasm_bindgen(getter)]
    pub fn c_random(&self) -> f64 {
        self.inner.c_random
    }

    #[wasm_bindgen(getter)]
    pub fn l_random(&self) -> f64 {
        self.inner.l_random
    }

    pub fn gamma(&self) -> f64 {
        self.inner.gamma()
    }

    pub fn lambda(&self) -> f64 {
        self.inner.lambda()
    }

    pub fn sigma(&self) -> f64 {
        self.inner.sigma()
    }

    pub fn is_small_world(&self) -> f64 {
        self.inner.is_small_world()
    }

    pub fn interpretation(&self) -> i64 {
        self.inner.interpretation()
    }
}

#[wasm_bindgen]
pub fn dunbar_cluster_example() -> SmallWorldMetrics {
    SmallWorldMetrics {
        inner: core_dunbar_cluster_example(),
    }
}
