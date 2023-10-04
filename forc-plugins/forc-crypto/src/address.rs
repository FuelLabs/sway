use anyhow::anyhow;
use fuel_crypto::fuel_types::Address;
use fuels_core::types::bech32::Bech32Address;
use std::str::{from_utf8, FromStr};

/// Takes a valid address in any supported format and returns them in all
/// supported format. This is meant to be a tool that can be used to convert any
/// address format to all other formats
pub fn dump_address<T: AsRef<[u8]>>(data: T) -> anyhow::Result<String> {
    let bytes_32: Result<[u8; 32], _> = data.as_ref().try_into();
    let (bech32, addr) = match bytes_32 {
        Ok(bytes) => (
            Bech32Address::from(Address::from(bytes)),
            Address::from(bytes),
        ),
        Err(_) => handle_string_conversion(data)?,
    };

    Ok(format!("Bench32: {}\nAddress: 0x{}\n", bech32, addr))
}

fn handle_string_conversion<T: AsRef<[u8]>>(data: T) -> anyhow::Result<(Bech32Address, Address)> {
    let addr = from_utf8(data.as_ref())?;
    if let Ok(bech32) = Bech32Address::from_str(addr) {
        Ok((bech32.clone(), Address::from(bech32)))
    } else if let Ok(addr) = Address::from_str(addr) {
        Ok((Bech32Address::from(addr), addr))
    } else {
        Err(anyhow!("{} cannot be parsed to a valid address", addr))
    }
}
