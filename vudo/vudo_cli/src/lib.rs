//! VUDO CLI library: the parts of `vudo` other crates (and node-side tests)
//! need to drive exactly what the CLI does.

pub mod node;

/// The manifest and key types [`node::SignedPackage`] is built from.
pub use spirit_runtime;
