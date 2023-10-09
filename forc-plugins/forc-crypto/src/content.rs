use std::{convert::Infallible, fs::read, path::PathBuf, str::FromStr};

#[derive(Clone, Debug, PartialEq)]
pub enum Content {
    Path(PathBuf, Vec<u8>),
    Binary(Vec<u8>),
}

impl Content {
    pub fn from_file_or_binary(input: &str) -> Self {
        let path = PathBuf::from(input);
        match read(&path) {
            Ok(content) => Self::Path(path, content),
            Err(_) => {
                let text = input.trim();
                if let Some(text) = text.strip_prefix("0x") {
                    if let Ok(bin) = hex::decode(text) {
                        return Self::Binary(bin);
                    }
                }
                Self::Binary(text.as_bytes().to_vec())
            }
        }
    }
}

impl FromStr for Content {
    type Err = Infallible;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(Self::from_file_or_binary(s))
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
