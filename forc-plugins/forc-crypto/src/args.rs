//! A `forc` plugin for converting a given string or path to their hash.

use std::{
    fs::read,
    io::{self, BufRead},
    path::Path,
};

forc_types::cli_examples! {
    crate::Command {
       [ Hashes an argument with SHA256 => "forc crypto sha256 test" ]
       [ Hashes an argument with Keccak256 => "forc crypto keccak256 test" ]
       [ Hashes a file path with SHA256 => "forc crypto sha256 {file}" ]
       [ Hashes a file path with Keccak256 => "forc crypto keccak256 {file}" ]
    }
}

#[derive(Debug, Clone, clap::Args)]
#[clap(
    version,
    about = "Hashes the argument or file with this algorithm",
    after_help = help(),
)]
pub struct HashArgs {
    /// This argument is optional, it can be either:
    ///
    /// 1. A path to a file. If that is the case, the content of the file is
    ///    loaded
    ///
    /// 2. A binary string encoded as a hex string. If that is the case, the
    ///    hex is decoded and passed as a Vec<u8>
    ///
    /// 3. A string. This is the last option, if the string is "-", "stdin"
    ///    is read instead. Otherwise the raw string is converted to a Vec<u8>
    ///    and passed
    ///
    /// 4. If it is not provided, "stdin" is read
    content_or_filepath: Option<String>,
}

fn checked_read_file<P: AsRef<Path>>(path: &Option<P>) -> Option<Vec<u8>> {
    path.as_ref().map(read)?.ok()
}

fn checked_read_stdin<R: BufRead>(content: &Option<String>, mut stdin: R) -> Option<Vec<u8>> {
    match content.as_ref().map(|x| x.as_str()) {
        Some("-") | None => {
            let mut buffer = Vec::new();
            if stdin.read_to_end(&mut buffer).is_ok() {
                Some(buffer)
            } else {
                Some(vec![])
            }
        }
        _ => None,
    }
}

fn read_as_binary(content: &Option<String>) -> Vec<u8> {
    content
        .as_ref()
        .map(|x| {
            if let Some(hex) = x.trim().strip_prefix("0x") {
                if let Ok(bin) = hex::decode(hex) {
                    bin
                } else {
                    x.as_bytes().to_vec()
                }
            } else {
                x.as_bytes().to_vec()
            }
        })
        .unwrap_or_default()
}

/// Reads the arg and returns a vector of bytes
///
/// These are the rules
///  1. If None, stdin is read.
///  2. If it's a String and it happens to be a file path, its content will be returned
///  3. If it's a String and it is "-", stdin is read
///  4. If the string starts with "0x", it will be treated as a hex string. Only
///     fully valid hex strings are accepted.
///  5. Otherwise the String will be converted to a vector of bytes
pub fn read_content_filepath_or_stdin(arg: Option<String>) -> Vec<u8> {
    match checked_read_file(&arg) {
        Some(bytes) => bytes,
        None => match checked_read_stdin(&arg, io::stdin().lock()) {
            Some(bytes) => bytes,
            None => read_as_binary(&arg),
        },
    }
}

/// The HashArgs takes no or a single argument, it can be either a string or a
/// path to a file. It can be consumed and converted to a Vec<u8> using the From
/// trait.
///
/// This is a wrapper around `read_content_filepath_or_stdin`
impl From<HashArgs> for Vec<u8> {
    fn from(value: HashArgs) -> Self {
        read_content_filepath_or_stdin(value.content_or_filepath)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_checked_read_file() {
        assert!(checked_read_file(&Some("not a file")).is_none());
        assert!(checked_read_file(&Some("Cargo.toml")).is_some());
        assert!(checked_read_file::<String>(&None).is_none());
    }

    #[test]
    fn test_checked_stdin() {
        let stdin = b"I'm a test from stdin";
        assert_eq!(
            None,
            checked_read_stdin(&Some("value".to_owned()), &stdin[..])
        );
        assert_eq!(
            Some(b"I'm a test from stdin".to_vec()),
            checked_read_stdin(&None, &stdin[..])
        );
        assert_eq!(
            Some(b"I'm a test from stdin".to_vec()),
            checked_read_stdin(&Some("-".to_owned()), &stdin[..])
        );
        assert_eq!(None, checked_read_stdin(&Some("".to_owned()), &stdin[..]));
    }

    #[test]
    fn test_read_binary() {
        let x = "      0xff";
        assert_eq!(vec![255u8], read_as_binary(&Some(x.to_owned())));
        let x = "0xFF";
        assert_eq!(vec![255u8], read_as_binary(&Some(x.to_owned())));
        let x = " 0xFf";
        assert_eq!(vec![255u8], read_as_binary(&Some(x.to_owned())));
        let x = " 0xFfx";
        assert_eq!(b" 0xFfx".to_vec(), read_as_binary(&Some(x.to_owned())));
        let x = " some random data\n\n\0";
        assert_eq!(
            b" some random data\n\n\0".to_vec(),
            read_as_binary(&Some(x.to_owned()))
        );
    }
}
