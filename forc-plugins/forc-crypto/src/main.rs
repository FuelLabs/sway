//! A `forc` plugin for converting a given string or path to their hash.

use anyhow::Result;
use clap::Parser;
use forc_tracing::{init_tracing_subscriber, println_error};
use std::default::Default;
use tracing::info;

mod address;
mod content;
mod keccak256;
mod keygen;
mod sha256;

#[derive(Debug, clap::Args)]
#[clap(author, version, about = "Hashes the argument or file with this hash")]
pub struct HashArgs {
    pub content: content::Content,
}

#[derive(Debug, clap::Args)]
#[clap(
    author,
    version,
    about = "Converts any valid address to all supported formats"
)]
pub struct AddressArgs {
    pub content: String,
}

#[derive(Debug, Parser)]
#[clap(
    name = "forc-crypto",
    about = "Forc plugin for hashing arbitrary data.",
    version
)]
pub enum Command {
    Keccak256(HashArgs),
    Sha256(HashArgs),
    Address(AddressArgs),
    NewKey(keygen::new_key::Arg),
    ParseSecret(keygen::parse_secret::Arg),
}

fn main() {
    init_tracing_subscriber(Default::default());
    if let Err(err) = run() {
        println_error(&format!("{}", err));
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let app = Command::parse();
    let content = match app {
        Command::Keccak256(arg) => hex::encode(keccak256::hash(arg.content)?),
        Command::Sha256(arg) => hex::encode(sha256::hash(arg.content)?),
        Command::Address(arg) => address::dump_address(arg.content)?,
        Command::NewKey(arg) => keygen::new_key::handler(arg)?,
        Command::ParseSecret(arg) => keygen::parse_secret::handler(arg)?,
    };

    info!("{}", content);
    Ok(())
}
