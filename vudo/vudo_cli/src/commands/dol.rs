//! `vudo dol` - Enter the DOL REPL (interactive mode)

use anyhow::Result;
use clap::Args;
use vudo_repl::{Repl, ReplConfig};

use crate::config::VudoConfig;

#[derive(Args, Debug)]
pub struct DolArgs {
    /// DOL file to load on startup
    #[arg(short, long)]
    pub load: Option<String>,

    /// Don't display the welcome banner
    #[arg(long)]
    pub no_banner: bool,
}

pub async fn execute(args: DolArgs, _config: &VudoConfig) -> Result<()> {
    let repl_config = ReplConfig {
        show_banner: !args.no_banner,
        load_file: args.load,
    };

    let mut repl = Repl::new(repl_config)?;
    repl.run()?;

    Ok(())
}
