pub mod address;
pub mod args;
pub mod keccak256;
pub mod keys;
pub mod sha256;

pub(crate) fn help() -> &'static str {
    Box::leak(
        format!(
            "EXAMPLES:\n{}{}{}{}{}{}",
            args::examples(),
            address::examples(),
            keys::new_key::examples(),
            keys::parse_secret::examples(),
            keys::get_public_key::examples(),
            keys::vanity::examples(),
        )
        .into_boxed_str(),
    )
}

/// Forc plugin for hashing arbitrary data
#[derive(Debug, clap::Parser)]
#[clap(
    name = "forc-crypto",
    after_help = help(),
    version
)]
pub enum Command {
    Keccak256(args::HashArgs),
    Sha256(args::HashArgs),
    Address(address::Args),
    GetPublicKey(keys::get_public_key::Arg),
    NewKey(keys::new_key::Arg),
    ParseSecret(keys::parse_secret::Arg),
    Vanity(keys::vanity::Arg),
}
