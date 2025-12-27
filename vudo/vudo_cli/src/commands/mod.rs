//! VUDO CLI Commands
//!
//! This module contains all the command implementations for the VUDO CLI.

pub mod build;
pub mod check;
pub mod doc;
pub mod dol;
pub mod fmt;
pub mod info;
pub mod new;
pub mod pack;
pub mod publish;
pub mod run;
pub mod search;
pub mod sign;
pub mod summon;
pub mod test;
pub mod upgrade;

// Re-export Args structs for convenience
pub use build::BuildArgs;
pub use check::CheckArgs;
pub use doc::DocArgs;
pub use dol::DolArgs;
pub use fmt::FmtArgs;
pub use info::InfoArgs;
pub use new::NewArgs;
pub use pack::PackArgs;
pub use publish::PublishArgs;
pub use run::RunArgs;
pub use search::SearchArgs;
pub use sign::SignArgs;
pub use summon::SummonArgs;
pub use test::TestArgs;
pub use upgrade::UpgradeArgs;
