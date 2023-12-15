//! A `forc` plugin for converting a given string or path to their hash.

use anyhow::Result;
use atty::Stream;
use clap::Parser;
use forc_tracing::{init_tracing_subscriber, println_error};
use std::{
    default::Default,
    io::{stdin, stdout, Read, Write},
};
use termion::screen::IntoAlternateScreen;

mod address;
mod args;
mod keccak256;
mod keygen;
mod sha256;

const ABOUT: &str = "Forc plugin for hashing arbitrary data.";

fn after_long_help() -> &'static str {
    Box::leak(
        format!(
            "EXAMPLES:\n{}{}{}{}",
            args::examples(),
            address::examples(),
            keygen::new_key::examples(),
            keygen::parse_secret::examples(),
        )
        .into_boxed_str(),
    )
}

#[derive(Debug, Parser)]
#[clap(
    name = "forc-crypto",
    about = ABOUT,
    after_long_help = after_long_help(),
    version
)]
pub enum Command {
    Keccak256(args::HashArgs),
    Sha256(args::HashArgs),
    Address(address::Args),
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
        Command::Keccak256(arg) => keccak256::hash(arg)?,
        Command::Sha256(arg) => sha256::hash(arg)?,
        Command::Address(arg) => address::dump_address(arg.address)?,
        Command::NewKey(arg) => keygen::new_key::handler(arg)?,
        Command::ParseSecret(arg) => keygen::parse_secret::handler(arg)?,
    };

    display_output(content)
}

fn wait_for_keypress() {
    let mut single_key = [0u8];
    stdin().read_exact(&mut single_key).unwrap();
}

fn has_sensible_info<T>(message: &T) -> bool
where
    T: serde::Serialize,
{
    match serde_json::to_value(message) {
        Ok(serde_json::Value::Object(map)) => map.get("secret").is_some(),
        _ => false,
    }
}

pub(crate) fn display_output<T>(message: T) -> anyhow::Result<()>
where
    T: serde::Serialize,
{
    if atty::is(Stream::Stdout) {
        let text = serde_yaml::to_string(&message).expect("valid string");
        if has_sensible_info(&message) {
            let mut screen = stdout().into_alternate_screen()?;
            writeln!(screen, "{text}",)?;
            screen.flush()?;
            println!("### Do not share or lose this private key! Press any key to exit. ###");
            wait_for_keypress();
        } else {
            println!("{text}");
        }
    } else {
        print!("{}", serde_json::to_string(&message).expect("valid json"));
    }
    Ok(())
}
