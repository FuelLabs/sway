use clap::Parser;

/// Determine the predicate-root for a predicate forc package.
#[derive(Debug, Default, Parser)]
#[clap(bin_name = "forc predicate-root", version)]
pub struct Command {}
