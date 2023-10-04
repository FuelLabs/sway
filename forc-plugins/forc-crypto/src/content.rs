use anyhow::{anyhow, Result};
use std::{
    fs::read,
    path::PathBuf,
    str::{from_utf8, FromStr},
};

#[derive(Clone, Debug, PartialEq)]
pub enum Content {
    Path(PathBuf, Vec<u8>),
    Binary(Vec<u8>),
}

impl Content {
    pub fn from_hex_or_utf8(input: Vec<u8>) -> Result<Vec<u8>> {
        if let Ok(text) = from_utf8(&input) {
            let text = text.trim();
            if let Some(text) = text.strip_prefix("0x") {
                if let Ok(bin) = hex::decode(text) {
                    return Ok(bin);
                }
            }
            Ok(text.as_bytes().to_vec())
        } else {
            Err(anyhow!(
                "{:?} is not a valid UTF-8 string nor a valid hex string",
                input
            ))
        }
    }
}

impl FromStr for Content {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let path = PathBuf::from(s);
        match read(&path) {
            Ok(content) => Ok(Content::Path(path, Self::from_hex_or_utf8(content)?)),
            Err(_) => Ok(Content::Binary(Self::from_hex_or_utf8(
                s.as_bytes().to_vec(),
            )?)),
        }
    }
}

impl AsRef<[u8]> for Content {
    fn as_ref(&self) -> &[u8] {
        match self {
            Content::Path(_, content) => content.as_ref(),
            Content::Binary(raw) => raw.as_ref(),
        }
    }
}
