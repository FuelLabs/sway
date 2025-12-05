use anyhow::anyhow;
use fuel_crypto::fuel_types::Address;
use serde_json::json;
use std::str::{from_utf8, FromStr};

forc_types::cli_examples! {
    crate::Command {
        [ Convert an address to another format => "forc crypto address fuel12e0xwx34nfp7jrzvn9mp5qkac3yvp7h8fx37ghl7klf82vv2wkys6wd523" ]
    }
}

#[derive(Debug, clap::Args)]
#[clap(
    version,
    about = "Converts an address to another format",
    after_help = help(),
)]
pub struct Args {
    /// The address to convert. It can be either a valid address in hex format
    pub address: String,
}

/// Takes a valid address in any supported format and returns them in all
/// supported format. This is meant to be a tool that can be used to convert any
/// address format to all other formats
pub fn dump_address<T: AsRef<[u8]>>(data: T) -> anyhow::Result<serde_json::Value> {
    let bytes_32: Result<[u8; 32], _> = data.as_ref().try_into();
    let addr = match bytes_32 {
        Ok(bytes) => Address::from(bytes),
        Err(_) => handle_string_conversion(data)?,
    };

    Ok(json!({
        "Address": addr.to_string(),
    }))
}

fn handle_string_conversion<T: AsRef<[u8]>>(data: T) -> anyhow::Result<Address> {
    let addr = from_utf8(data.as_ref())?;
    Address::from_str(addr).map_err(|_| anyhow!("{} cannot be parsed to a valid address", addr))
}
