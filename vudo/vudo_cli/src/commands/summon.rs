//! `vudo summon` - Summon a Spirit on a DOLOS node by name or content hash
//!
//! Posts `{name | hash, entry, args}` to the node's `POST /api/spirits`. The
//! node resolves the name in the DHT, fetches the package from a provider
//! through its verified path and runs it; this command reports the run and
//! fails unless it completed (Spirit Hosting H2d-3).

use anyhow::Result;
use clap::Args;
use colored::*;

use crate::config::VudoConfig;
use vudo_cli::node::{self, Summon, Target};

#[derive(Args, Debug)]
pub struct SummonArgs {
    /// Spirit name `<author key hex>/<name>@<version>`, or its content hash
    pub spirit: String,

    /// Node API base URL, e.g. http://127.0.0.1:8080
    #[arg(long, default_value = "http://127.0.0.1:8080")]
    pub node: String,

    /// Exported function to invoke
    #[arg(short, long, default_value = "main")]
    pub entry: String,

    /// Integer arguments (i64), comma separated
    #[arg(short, long, value_delimiter = ',', allow_hyphen_values = true)]
    pub args: Vec<i64>,

    /// Fuel budget (may lower the manifest's, never raise it)
    #[arg(long)]
    pub fuel: Option<u64>,
}

pub async fn execute(args: SummonArgs, _config: &VudoConfig) -> Result<()> {
    println!(
        "{} Spirit: {}",
        "Summoning".green().bold(),
        args.spirit.cyan()
    );
    let target: Target = args.spirit.parse()?;
    println!("  {} {}", "Node:".cyan(), args.node);

    let instance = node::summon(
        &args.node,
        &Summon {
            target,
            entry: args.entry,
            args: args.args,
            fuel: args.fuel,
        },
    )
    .await?;

    println!("  {} {}", "Instance:".cyan(), instance.id);
    println!(
        "  {} {}@{}",
        "Spirit:".cyan(),
        instance.name,
        instance.version
    );
    println!("  {} {}", "Fuel consumed:".cyan(), instance.fuel_consumed);
    println!(
        "  {} {}",
        "Result:".cyan(),
        serde_json::Value::from(instance.result.clone())
    );
    println!(
        "\n{} {} ran on the node",
        "✓".green().bold(),
        instance.entry
    );

    Ok(())
}
