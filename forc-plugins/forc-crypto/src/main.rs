//! A `forc` plugin for converting a given string or path to their hash.

use anyhow::Result;
use clap::Parser;
use forc_tracing::{init_tracing_subscriber, println_error};
use std::{
    default::Default,
    fs::read,
    io::{self, Read},
    path::PathBuf,
};
use tracing::info;

mod address;
mod content;
mod keccak256;
mod keygen;
mod sha256;

#[derive(Debug, clap::Args)]
#[clap(
    author,
    version,
    about = "Converts any valid address to all supported formats"
)]
pub struct AddressArgs {
    pub content: String,
}

#[derive(Debug, Clone, clap::Args)]
#[clap(author, version, about = "Hashes the argument or file with this hash")]
pub struct HashArgs {
    content_or_filepath: Option<String>,
}

fn read_stdin() -> Vec<u8> {
    let mut buffer = Vec::new();
    if io::stdin().lock().read_to_end(&mut buffer).is_ok() {
        buffer
    } else {
        vec![]
    }
}

/// The HashArgs takes no or a single argument, it can be either a string or a
/// path to a file. It can be consumed and converted to a Vec<u8> using the From
/// trait.
///
/// The usage is as follows:
///  1. Zero or one argument is accepted
///  2. If no argument is passed, `stdin` is being read
///  3. The argument will be checked to be a file path, if it is the content
///     will be loaded from the file
///  4. Otherwise, the content is treated as a string
///  5. If the string is "-", `stdin` is being read
///  6. If the string starts with "0x", it will be treated as a hex string. Only
///     fully valid hex strings are accepted.
///  7. Any other string, or any malformed hex string will be treated as a
///     vector of bytes
impl From<HashArgs> for Vec<u8> {
    fn from(value: HashArgs) -> Self {
        if let Some(content_or_filepath) = value.content_or_filepath {
            let path = PathBuf::from(&content_or_filepath);
            match read(path) {
                Ok(bytes) => bytes,
                Err(_) => {
                    let text = content_or_filepath.trim();
                    if text == "-" {
                        return read_stdin();
                    }
                    if let Some(text) = text.strip_prefix("0x") {
                        if let Ok(bin) = hex::decode(text) {
                            return bin;
                        }
                    }
                    text.as_bytes().to_vec()
                }
            }
        } else {
            read_stdin()
        }
    }
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
        Command::Keccak256(arg) => hex::encode(keccak256::hash(arg)?),
        Command::Sha256(arg) => hex::encode(sha256::hash(arg)?),
        Command::Address(arg) => address::dump_address(arg.content)?,
        Command::NewKey(arg) => keygen::new_key::handler(arg)?,
        Command::ParseSecret(arg) => keygen::parse_secret::handler(arg)?,
    };

    info!("{}", content);
    Ok(())
}
