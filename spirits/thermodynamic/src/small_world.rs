//! Small-World Network Metrics - Generated from DOL
//! Watts-Strogatz metrics for network analysis

use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};

/// Graph edge
#[wasm_bindgen]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Edge {
    pub from_node: i64,
    pub to_node: i64,
}

#[wasm_bindgen]
impl Edge {
    #[wasm_bindgen(constructor)]
    pub fn new(from_node: i64, to_node: i64) -> Self {
        Self { from_node, to_node }
    }
}

/// Graph metrics
#[wasm_bindgen]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphMetrics {
    pub node_count: i64,
    pub edge_count: i64,
    pub clustering: f64,
    pub path_length: f64,
}

#[wasm_bindgen]
impl GraphMetrics {
    #[wasm_bindgen(constructor)]
    pub fn new(node_count: i64, edge_count: i64, clustering: f64, path_length: f64) -> Self {
        Self {
            node_count,
            edge_count,
            clustering,
            path_length,
        }
    }

    /// Average degree of nodes
    pub fn average_degree(&self) -> f64 {
        let n = self.node_count as f64;
        let m = self.edge_count as f64;
        2.0 * m / n
    }
}

/// Small-world network metrics (Watts-Strogatz)
#[wasm_bindgen]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SmallWorldMetrics {
    pub n: i64,           // Node count
    pub m: i64,           // Edge count
    pub k: f64,           // Average degree
    pub clustering: f64,  // Clustering coefficient (C)
    pub path_length: f64, // Average path length (L)
    pub c_random: f64,    // Random graph expected C
    pub l_random: f64,    // Random graph expected L
}

#[wasm_bindgen]
impl SmallWorldMetrics {
    #[wasm_bindgen(constructor)]
    pub fn new(
        n: i64,
        m: i64,
        k: f64,
        clustering: f64,
        path_length: f64,
        c_random: f64,
        l_random: f64,
    ) -> Self {
        Self {
            n,
            m,
            k,
            clustering,
            path_length,
            c_random,
            l_random,
        }
    }

    /// Gamma = C / C_random (clustering relative to random)
    pub fn gamma(&self) -> f64 {
        if self.c_random <= 0.0 {
            return 0.0;
        }
        self.clustering / self.c_random
    }

    /// Lambda = L / L_random (path length relative to random)
    pub fn lambda(&self) -> f64 {
        if self.l_random <= 0.0 {
            return 0.0;
        }
        self.path_length / self.l_random
    }

    /// Sigma = gamma / lambda (small-world coefficient)
    pub fn sigma(&self) -> f64 {
        let g = self.clustering / self.c_random;
        let l = self.path_length / self.l_random;
        if l <= 0.0 {
            return 0.0;
        }
        g / l
    }

    /// Returns 1.0 if small-world (sigma > 1), 0.0 otherwise
    pub fn is_small_world(&self) -> f64 {
        if self.sigma() > 1.0 {
            return 1.0;
        }
        0.0
    }

    /// Human-readable interpretation
    pub fn interpretation(&self) -> String {
        let sigma = self.sigma();
        let mut parts = Vec::new();

        if sigma > 1.0 {
            parts.push(format!("Small-world (sigma = {:.2} > 1)", sigma));
        } else {
            parts.push(format!("Not small-world (sigma = {:.2} <= 1)", sigma));
        }

        if self.clustering > 0.5 {
            parts.push(format!("High clustering (C = {:.3})", self.clustering));
        } else if self.clustering > 0.1 {
            parts.push(format!("Moderate clustering (C = {:.3})", self.clustering));
        } else {
            parts.push(format!("Low clustering (C = {:.3})", self.clustering));
        }

        parts.join("; ")
    }
}

/// Calculate expected clustering for random graph: C_random = k / n
#[wasm_bindgen]
pub fn random_clustering(n: i64, k: f64) -> f64 {
    let node_count = n as f64;
    k / node_count
}

/// Approximate random graph path length (without ln)
#[wasm_bindgen]
pub fn random_path_length_approx(n: i64, k: f64) -> f64 {
    let node_count = n as f64;
    node_count / k
}

/// Calculate random graph path length using libm ln
#[wasm_bindgen]
pub fn random_path_length(n: i64, k: f64) -> f64 {
    use libm::log as ln;
    let node_count = n as f64;
    ln(node_count) / ln(k)
}

/// Calculate small-world sigma coefficient
#[wasm_bindgen]
pub fn calculate_sigma(c: f64, l: f64, c_random: f64, l_random: f64) -> f64 {
    if c_random <= 0.0 || l_random <= 0.0 {
        return 0.0;
    }
    let gamma = c / c_random;
    let lambda = l / l_random;
    if lambda <= 0.0 {
        return 0.0;
    }
    gamma / lambda
}

/// Check if network has small-world properties
#[wasm_bindgen]
pub fn is_small_world_network(sigma: f64, min_clustering: f64, c: f64) -> bool {
    sigma > 1.0 && c >= min_clustering
}

/// Create example small network metrics (N=100, k=6)
#[wasm_bindgen]
pub fn small_network_example() -> SmallWorldMetrics {
    let n = 100;
    let k = 6.0;
    let c = 0.4719;
    let l = 3.684;
    let c_random = random_clustering(n, k);
    let l_random = random_path_length(n, k);

    SmallWorldMetrics::new(n, 300, k, c, l, c_random, l_random)
}

/// Create example Dunbar-sized cluster (N=150)
#[wasm_bindgen]
pub fn dunbar_cluster_example() -> SmallWorldMetrics {
    let n = 150;
    let k = 6.0;
    let c = 0.4219;
    let l = 3.9147;
    let c_random = random_clustering(n, k);
    let l_random = random_path_length(n, k);

    SmallWorldMetrics::new(n, 450, k, c, l, c_random, l_random)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_small_network() {
        let metrics = small_network_example();
        assert!(metrics.sigma() > 1.0, "Small network should have sigma > 1");
        assert!(metrics.is_small_world() > 0.5, "Should be small-world");
    }

    #[test]
    fn test_dunbar_cluster() {
        let metrics = dunbar_cluster_example();
        assert!(metrics.sigma() > 1.0, "Dunbar cluster should have sigma > 1");
    }

    #[test]
    fn test_gamma_lambda() {
        let metrics = small_network_example();
        assert!(metrics.gamma() > 1.0, "Gamma should be > 1 for clustered network");
        assert!(metrics.lambda() > 1.0, "Lambda should be > 1");
    }
}
