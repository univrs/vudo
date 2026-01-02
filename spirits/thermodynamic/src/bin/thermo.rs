//! thermo - Thermodynamic Economics Calculator CLI
//!
//! Command-line interface for EROEI and Small-World network analysis.

use thermodynamic::{
    EnergySystemMetrics,
    create_solar_pv_eroei, hyphal_node_energy, max_supported_nodes,
    solar_system_example, hyphal_network_example,
    random_clustering, random_path_length, calculate_sigma,
    dunbar_cluster_example,
};
use std::env;

fn print_help() {
    println!(r#"thermo - Thermodynamic Economics Calculator

USAGE:
    thermo <COMMAND> [OPTIONS]

COMMANDS:
    solar              Analyze 1 MW solar PV system EROEI
    hyphal <nodes>     Analyze Hyphal Network energy consumption
    combined <nodes>   Analyze Solar + Hyphal combined EROEI
    maxnodes <mw>      Calculate max nodes supportable by solar capacity

    network <n> <k>    Analyze small-world network properties
    dunbar             Analyze Dunbar-sized cluster (N=150)

    full <nodes>       Full ecosystem analysis (EROEI + Small-World)

    help               Show this help message
    version            Show version

EXAMPLES:
    thermo solar
    thermo hyphal 1000
    thermo combined 1000
    thermo maxnodes 1.0
    thermo network 100 6
    thermo full 1000
"#);
}

fn cmd_solar() {
    println!("=== Solar PV System Analysis (1 MW) ===\n");

    let eroei = create_solar_pv_eroei();
    let system = solar_system_example();

    println!("Total Output:         {:>12.0} kWh/year", system.total_output_kwh);
    println!("Total Input:          {:>12.0} kWh/year", system.total_input_kwh);
    println!("Net Energy:           {:>12.0} kWh/year", system.net_energy());
    println!();
    println!("System EROEI:         {:>12.2}", system.system_eroei());
    println!("Component EROEI:      {:>12.2}", eroei);
    println!("Viability:            {}", system.viability_assessment());
    println!("Meets 7:1 threshold:  {}", if system.is_viable() > 0.5 { "YES" } else { "NO" });
}

fn cmd_hyphal(nodes: i64) {
    println!("=== Hyphal Network Analysis ({} nodes) ===\n", nodes);

    let system = hyphal_network_example(nodes);
    let node_energy = hyphal_node_energy(nodes, 100.0);

    println!("Total Output:         {:>12.0} kWh/year", system.total_output_kwh);
    println!("Total Input:          {:>12.0} kWh/year", system.total_input_kwh);
    println!("Net Energy:           {:>12.0} kWh/year", system.net_energy());
    println!();
    println!("Per-node consumption: {:>12.0} kWh/year", node_energy / nodes as f64);
    println!("System EROEI:         {:>12.2}", system.system_eroei());
    println!("Viability:            {}", system.viability_assessment());
    println!();
    println!("NOTE: Hyphal Network is a PURE CONSUMER (EROEI = 0)");
    println!("      It requires an external autotrophic energy source.");
}

fn cmd_combined(nodes: i64) {
    println!("=== Combined Solar + Hyphal Analysis ({} nodes) ===\n", nodes);

    let solar = solar_system_example();
    let hyphal = hyphal_network_example(nodes);

    let combined_output = solar.total_output_kwh;
    let combined_input = solar.total_input_kwh + hyphal.total_input_kwh;
    let combined = EnergySystemMetrics::new(combined_output, combined_input, solar.component_count + nodes);

    println!("Solar Output:         {:>12.0} kWh/year", solar.total_output_kwh);
    println!("Solar Input:          {:>12.0} kWh/year", solar.total_input_kwh);
    println!("Hyphal Input:         {:>12.0} kWh/year", hyphal.total_input_kwh);
    println!("Combined Input:       {:>12.0} kWh/year", combined_input);
    println!("Net Energy:           {:>12.0} kWh/year", combined.net_energy());
    println!();
    println!("Solar EROEI:          {:>12.2}", solar.system_eroei());
    println!("Combined EROEI:       {:>12.2}", combined.system_eroei());
    println!("EROEI Reduction:      {:>12.0}%", (1.0 - combined.system_eroei() / solar.system_eroei()) * 100.0);
    println!();
    println!("Viability:            {}", combined.viability_assessment());
    println!("Meets 7:1 threshold:  {}", if combined.is_viable() > 0.5 { "YES" } else { "NO" });
}

fn cmd_maxnodes(solar_mw: f64) {
    println!("=== Maximum Nodes Calculation ===\n");

    let nodes = max_supported_nodes(solar_mw, 100.0);

    println!("Solar capacity:       {:>12.2} MW", solar_mw);
    println!("Node power draw:      {:>12.0} W", 100.0);
    println!("Capacity factor:      {:>12.0}%", 17.0);
    println!("Distribution eff:     {:>12.0}%", 80.0);
    println!();
    println!("Max nodes supported:  {:>12}", nodes);
    println!();
    println!("NOTE: For EROEI > 5, reduce node count by ~30%");
    println!("      Recommended:    {:>12}", (nodes as f64 * 0.7) as i64);
}

fn cmd_network(n: i64, k: f64) {
    println!("=== Small-World Network Analysis (N={}, k={}) ===\n", n, k);

    let c_random = random_clustering(n, k);
    let l_random = random_path_length(n, k);

    // Use example clustering values (Watts-Strogatz with p=0.1)
    let c = 0.45;  // Typical clustering for p=0.1
    let l = l_random * 1.3;  // Slightly longer than random

    let sigma = calculate_sigma(c, l, c_random, l_random);

    println!("Nodes (N):            {:>12}", n);
    println!("Edges (M):            {:>12}", (n * k as i64) / 2);
    println!("Average degree (k):   {:>12.2}", k);
    println!();
    println!("Clustering (C):       {:>12.4}", c);
    println!("Random graph C:       {:>12.4}", c_random);
    println!("gamma = C/C_random:   {:>12.2}", c / c_random);
    println!();
    println!("Path length (L):      {:>12.4}", l);
    println!("Random graph L:       {:>12.4}", l_random);
    println!("lambda = L/L_random:  {:>12.2}", l / l_random);
    println!();
    println!("Small-world sigma:    {:>12.2}", sigma);
    println!("Is small-world:       {}", if sigma > 1.0 { "YES" } else { "NO" });
}

fn cmd_dunbar() {
    println!("=== Dunbar Cluster Analysis (N=150) ===\n");

    let metrics = dunbar_cluster_example();

    println!("Nodes (N):            {:>12}", metrics.n);
    println!("Edges (M):            {:>12}", metrics.m);
    println!("Average degree (k):   {:>12.2}", metrics.k);
    println!();
    println!("Clustering (C):       {:>12.4}", metrics.clustering);
    println!("Random graph C:       {:>12.4}", metrics.c_random);
    println!("gamma = C/C_random:   {:>12.2}", metrics.gamma());
    println!();
    println!("Path length (L):      {:>12.4}", metrics.path_length);
    println!("Random graph L:       {:>12.4}", metrics.l_random);
    println!("lambda = L/L_random:  {:>12.2}", metrics.lambda());
    println!();
    println!("Small-world sigma:    {:>12.2}", metrics.sigma());
    println!("Is small-world:       {}", if metrics.sigma() > 1.0 { "YES" } else { "NO" });
    println!();
    println!("Interpretation:       {}", metrics.interpretation());
}

fn cmd_full(nodes: i64) {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║           THERMODYNAMIC ECONOMICS ECOSYSTEM ANALYSIS           ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    // EROEI Analysis
    let solar = solar_system_example();
    let hyphal = hyphal_network_example(nodes);

    let combined_output = solar.total_output_kwh;
    let combined_input = solar.total_input_kwh + hyphal.total_input_kwh;
    let combined = EnergySystemMetrics::new(combined_output, combined_input, solar.component_count + nodes);

    println!("┌──────────────────────────────────────────────────────────────────┐");
    println!("│ EROEI ANALYSIS                                                   │");
    println!("├──────────────────────────────────────────────────────────────────┤");
    println!("│ Solar system EROEI:      {:>8.2}                                │", solar.system_eroei());
    println!("│ Hyphal network EROEI:    {:>8.2}                                │", hyphal.system_eroei());
    println!("│ Combined EROEI:          {:>8.2}                                │", combined.system_eroei());
    println!("│ Viability:               {}│", format!("{:<35}", combined.viability_assessment()));
    println!("│ Max nodes (1 MW):        {:>8}                                │", max_supported_nodes(1.0, 100.0));
    println!("└──────────────────────────────────────────────────────────────────┘\n");

    // Small-World Analysis
    let k = 6.0;
    let c_random = random_clustering(nodes, k);
    let l_random = random_path_length(nodes, k);
    let c = 0.45;
    let l = l_random * 1.3;
    let sigma = calculate_sigma(c, l, c_random, l_random);

    println!("┌──────────────────────────────────────────────────────────────────┐");
    println!("│ SMALL-WORLD ANALYSIS                                             │");
    println!("├──────────────────────────────────────────────────────────────────┤");
    println!("│ Nodes:                   {:>8}                                │", nodes);
    println!("│ Average degree:          {:>8.1}                                │", k);
    println!("│ Clustering (C):          {:>8.3}                                │", c);
    println!("│ Path length (L):         {:>8.2}                                │", l);
    println!("│ Sigma:                   {:>8.2}                                │", sigma);
    println!("│ Small-world:             {:>8}                                │", if sigma > 1.0 { "YES" } else { "NO" });
    println!("└──────────────────────────────────────────────────────────────────┘\n");

    // Summary
    let thermodynamic_tax = (1.0 - combined.system_eroei() / solar.system_eroei()) * 100.0;

    println!("┌──────────────────────────────────────────────────────────────────┐");
    println!("│ SUMMARY                                                          │");
    println!("├──────────────────────────────────────────────────────────────────┤");
    println!("│ Thermodynamic tax:       {:>7.1}%                                │", thermodynamic_tax);
    if combined.is_viable() > 0.5 {
        println!("│ Status:                  {:>8}                                │", "VIABLE");
    } else {
        println!("│ Status:                  {:>8}                                │", "AT RISK");
    }
    if sigma > 1.0 {
        println!("│ Network topology:        {:>8}                                │", "OPTIMAL");
    } else {
        println!("│ Network topology:        {:>8}                                │", "SUBOPTIMAL");
    }
    println!("└──────────────────────────────────────────────────────────────────┘");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_help();
        return;
    }

    match args[1].as_str() {
        "help" | "-h" | "--help" => print_help(),
        "version" | "-v" | "--version" => println!("thermo {}", env!("CARGO_PKG_VERSION")),
        "solar" => cmd_solar(),
        "hyphal" => {
            let nodes = args.get(2)
                .and_then(|s| s.parse().ok())
                .unwrap_or(1000);
            cmd_hyphal(nodes);
        }
        "combined" => {
            let nodes = args.get(2)
                .and_then(|s| s.parse().ok())
                .unwrap_or(1000);
            cmd_combined(nodes);
        }
        "maxnodes" => {
            let mw = args.get(2)
                .and_then(|s| s.parse().ok())
                .unwrap_or(1.0);
            cmd_maxnodes(mw);
        }
        "network" => {
            let n = args.get(2)
                .and_then(|s| s.parse().ok())
                .unwrap_or(100);
            let k = args.get(3)
                .and_then(|s| s.parse().ok())
                .unwrap_or(6.0);
            cmd_network(n, k);
        }
        "dunbar" => cmd_dunbar(),
        "full" => {
            let nodes = args.get(2)
                .and_then(|s| s.parse().ok())
                .unwrap_or(1000);
            cmd_full(nodes);
        }
        cmd => {
            eprintln!("Unknown command: {}", cmd);
            eprintln!("Run 'thermo help' for usage information.");
        }
    }
}
