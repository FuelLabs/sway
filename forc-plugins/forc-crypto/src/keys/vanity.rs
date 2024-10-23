use fuel_crypto::{fuel_types::Address, PublicKey, SecretKey};
use fuels_accounts::wallet::{generate_mnemonic_phrase, DEFAULT_DERIVATION_PATH_PREFIX};
use fuels_core::types::bech32::{Bech32Address, FUEL_BECH32_HRP};
use rayon::iter::{self, Either, ParallelIterator};
use regex::Regex;
use serde_json::json;
use std::{
    error::Error,
    fmt, fs,
    path::PathBuf,
    time::{Duration, Instant},
};
use tokio::runtime::Runtime;

forc_util::cli_examples! {
    crate::Command {
        [ Generate a checksummed vanity address with a given prefix => "forc crypto vanity --starts-with aaa" ]
    }
}

#[derive(Debug, clap::Parser)]
#[clap(
    version,
    about = "Generate a vanity address",
    after_help = "Generate vanity addresses for the Fuel blockchain"
)]
pub struct Arg {
    /// Prefix regex pattern or hex string.
    #[arg(long, value_name = "PATTERN", required_unless_present = "ends_with")]
    pub starts_with: Option<String>,

    /// Suffix regex pattern or hex string.
    #[arg(long, value_name = "PATTERN")]
    pub ends_with: Option<String>,

    /// Timeout in seconds for address generation
    #[arg(long, value_name = "SECONDS")]
    pub timeout: Option<u64>,

    /// Return mnemonic with address (default false)
    #[arg(long)]
    pub mnemonic: bool,

    /// Path to save the generated vanity address to.
    #[arg(long, value_hint = clap::ValueHint::FilePath, value_name = "PATH")]
    pub save_path: Option<PathBuf>,
}

pub fn handler(args: Arg) -> anyhow::Result<serde_json::Value> {
    let Arg {
        starts_with,
        ends_with,
        mnemonic,
        save_path,
        timeout,
    } = args;

    let mut left_exact_hex = None;
    let mut left_regex = None;
    if let Some(prefix) = starts_with {
        match parse_pattern(&prefix, true)? {
            Either::Left(left) => left_exact_hex = Some(left),
            Either::Right(re) => left_regex = Some(re),
        }
    }

    let mut right_exact_hex = None;
    let mut right_regex = None;
    if let Some(suffix) = ends_with {
        match parse_pattern(&suffix, false)? {
            Either::Left(right) => right_exact_hex = Some(right),
            Either::Right(re) => right_regex = Some(re),
        }
    }

    println!("Starting to generate vanity address...");
    let start_time = Instant::now();

    let result = match (left_exact_hex, left_regex, right_exact_hex, right_regex) {
        (Some(left), _, Some(right), _) => {
            let matcher = HexMatcher { left, right };
            find_vanity_address_with_timeout(matcher, mnemonic, timeout)
        }
        (Some(left), _, _, Some(right)) => {
            let matcher = LeftExactRightRegexMatcher { left, right };
            find_vanity_address_with_timeout(matcher, mnemonic, timeout)
        }
        (_, Some(left), _, Some(right)) => {
            let matcher = RegexMatcher { left, right };
            find_vanity_address_with_timeout(matcher, mnemonic, timeout)
        }
        (_, Some(left), Some(right), _) => {
            let matcher = LeftRegexRightExactMatcher { left, right };
            find_vanity_address_with_timeout(matcher, mnemonic, timeout)
        }
        (Some(left), None, None, None) => {
            let matcher = LeftHexMatcher { left };
            find_vanity_address_with_timeout(matcher, mnemonic, timeout)
        }
        (None, None, Some(right), None) => {
            let matcher = RightHexMatcher { right };
            find_vanity_address_with_timeout(matcher, mnemonic, timeout)
        }
        (None, Some(re), None, None) => {
            let matcher = SingleRegexMatcher { re };
            find_vanity_address_with_timeout(matcher, mnemonic, timeout)
        }
        (None, None, None, Some(re)) => {
            let matcher = SingleRegexMatcher { re };
            find_vanity_address_with_timeout(matcher, mnemonic, timeout)
        }
        _ => {
            return Err(VanityAddressError::InvalidPattern(
                "Invalid pattern combination".to_string(),
            )
            .into())
        }
    };

    let (address, secret_key, mnemonic) = result?;

    let duration = start_time.elapsed();
    println!(
        "Successfully found vanity address in {:.3} seconds.\n",
        duration.as_secs_f64()
    );

    let result = if let Some(mnemonic) = mnemonic {
        json!({
            "Address": address.to_string(),
            "PrivateKey": hex::encode(secret_key.as_ref()),
            "Mnemonic": mnemonic,
        })
    } else {
        json!({
            "Address": address.to_string(),
            "PrivateKey": hex::encode(secret_key.as_ref()),
        })
    };

    if let Some(path) = save_path {
        fs::write(path, serde_json::to_string_pretty(&result)?)?;
    }

    Ok(result)
}

pub fn find_vanity_address_with_timeout<T: VanityMatcher>(
    matcher: T,
    use_mnemonic: bool,
    timeout_secs: Option<u64>,
) -> anyhow::Result<(Address, SecretKey, Option<String>)> {
    let generate_wallet = move || {
        let breakpoint = if use_mnemonic { 1_000 } else { 100_000 };
        let start = Instant::now();
        let attempts = std::sync::atomic::AtomicUsize::new(0); // atomic for parallel iteration
        wallet_generator(use_mnemonic)
            .find_any(|result| {
                let current = attempts.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                if current != 0 && current % breakpoint == 0 {
                    let elapsed = start.elapsed().as_secs_f64();
                    let rate = current as f64 / elapsed;
                    println!(
                        "└─ tried {} addresses ({:.2} addresses/sec)...",
                        current, rate
                    );
                }

                if let Ok((addr, _, _)) = result {
                    matcher.is_match(addr)
                } else {
                    false
                }
            })
            .ok_or(VanityAddressError::GenerationFailed)?
    };

    let Some(secs) = timeout_secs else {
        return generate_wallet();
    };

    // Run the async timeout logic in the runtime
    Runtime::new()?.block_on(async {
        let generation_task = tokio::task::spawn_blocking(generate_wallet);

        let abort_handle = generation_task.abort_handle();
        let timeout_duration = Duration::from_secs(secs);

        tokio::select! {
            result = generation_task => {
                match result {
                    Ok(wallet_result) => wallet_result.map_err(Into::<anyhow::Error>::into),
                    Err(_e) => Err(VanityAddressError::GenerationFailed.into()),
                }
            }
            _ = tokio::time::sleep(timeout_duration) => {
                abort_handle.abort();
                Err(VanityAddressError::Timeout(secs).into())
            }
        }
    })
}

/// Returns an infinite parallel iterator which yields addresses.
#[inline]
fn wallet_generator(
    use_mnemonic: bool,
) -> impl ParallelIterator<Item = anyhow::Result<(Address, SecretKey, Option<String>)>> {
    iter::repeat(()).map(move |()| generate_wallet(use_mnemonic))
}

/// Generates an address, secret key, and optionally a mnemonic.
fn generate_wallet(use_mnemonic: bool) -> anyhow::Result<(Address, SecretKey, Option<String>)> {
    let mut rng = rand::thread_rng();

    let (private_key, mnemonic) = if use_mnemonic {
        let mnemonic = generate_mnemonic_phrase(&mut rng, 24)?;
        let account_ix = 0;
        let derivation_path = format!("{DEFAULT_DERIVATION_PATH_PREFIX}/{account_ix}'/0/0");
        let private_key =
            SecretKey::new_from_mnemonic_phrase_with_path(&mnemonic, &derivation_path)?;
        (private_key, Some(mnemonic))
    } else {
        (SecretKey::random(&mut rng), None)
    };

    let public = PublicKey::from(&private_key);
    let hashed = public.hash();
    let address = Bech32Address::new(FUEL_BECH32_HRP, hashed);

    Ok((address.into(), private_key, mnemonic))
}

pub trait VanityMatcher: Send + Sync + 'static {
    fn is_match(&self, addr: &Address) -> bool;
}

struct HexMatcher {
    left: Vec<u8>,
    right: Vec<u8>,
}

impl VanityMatcher for HexMatcher {
    fn is_match(&self, addr: &Address) -> bool {
        let bytes = addr.as_ref();
        bytes.starts_with(&self.left)
            && bytes[bytes.len() - self.right.len()..]
                .iter()
                .zip(self.right.iter())
                .all(|(a, b)| a.eq_ignore_ascii_case(b))
    }
}

struct LeftHexMatcher {
    left: Vec<u8>,
}

impl VanityMatcher for LeftHexMatcher {
    fn is_match(&self, addr: &Address) -> bool {
        let bytes = addr.as_ref();
        bytes.starts_with(&self.left)
    }
}

struct RightHexMatcher {
    right: Vec<u8>,
}

impl VanityMatcher for RightHexMatcher {
    fn is_match(&self, addr: &Address) -> bool {
        let bytes = addr.as_ref();
        bytes[bytes.len() - self.right.len()..]
            .iter()
            .zip(self.right.iter())
            .all(|(a, b)| a.eq_ignore_ascii_case(b))
    }
}

struct LeftExactRightRegexMatcher {
    left: Vec<u8>,
    right: Regex,
}

impl VanityMatcher for LeftExactRightRegexMatcher {
    fn is_match(&self, addr: &Address) -> bool {
        let bytes = addr.as_ref();
        bytes.starts_with(&self.left) && self.right.is_match(&hex::encode(bytes))
    }
}

struct LeftRegexRightExactMatcher {
    left: Regex,
    right: Vec<u8>,
}

impl VanityMatcher for LeftRegexRightExactMatcher {
    fn is_match(&self, addr: &Address) -> bool {
        let bytes = addr.as_ref();
        self.left.is_match(&hex::encode(bytes))
            && bytes[bytes.len() - self.right.len()..]
                .iter()
                .zip(self.right.iter())
                .all(|(a, b)| a.eq_ignore_ascii_case(b))
    }
}

struct SingleRegexMatcher {
    re: Regex,
}

impl VanityMatcher for SingleRegexMatcher {
    fn is_match(&self, addr: &Address) -> bool {
        let addr = hex::encode(addr.as_ref());
        self.re.is_match(&addr)
    }
}

struct RegexMatcher {
    left: Regex,
    right: Regex,
}

impl VanityMatcher for RegexMatcher {
    fn is_match(&self, addr: &Address) -> bool {
        let addr = hex::encode(addr.as_ref());
        self.left.is_match(&addr) && self.right.is_match(&addr)
    }
}

fn parse_pattern(pattern: &str, is_start: bool) -> anyhow::Result<Either<Vec<u8>, Regex>> {
    if pattern.len() > 64 {
        return Err(VanityAddressError::InvalidPattern(
            "Pattern too long: max 64 characters".to_string(),
        )
        .into());
    }

    if let Ok(decoded) = hex::decode(pattern) {
        Ok(Either::Left(decoded))
    } else {
        // Check if the pattern contains only valid hex characters
        if !pattern
            .chars()
            .all(|c| c.is_ascii_hexdigit() || c == '^' || c == '$')
        {
            return Err(VanityAddressError::InvalidPattern(format!("pattern: {}", pattern)).into());
        }

        // Case-insensitive regex pattern
        let (prefix, suffix) = if is_start {
            ("^(?i)", "")
        } else {
            ("(?i)", "$")
        };
        Ok(Either::Right(Regex::new(&format!(
            "{prefix}{pattern}{suffix}"
        ))?))
    }
}

// Custom error type for vanity address generation
#[derive(Debug)]
pub enum VanityAddressError {
    Timeout(u64),
    GenerationFailed,
    InvalidPattern(String),
}

impl fmt::Display for VanityAddressError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Timeout(secs) => write!(
                f,
                "Vanity address generation timed out after {} seconds",
                secs
            ),
            Self::GenerationFailed => write!(f, "No matching address found"),
            Self::InvalidPattern(msg) => write!(f, "Invalid pattern: {}", msg),
        }
    }
}

impl Error for VanityAddressError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_start_or_end() {
        let args = Arg {
            starts_with: None,
            ends_with: None,
            mnemonic: false,
            save_path: None,
            timeout: None,
        };
        let result = handler(args);
        assert_eq!(
            result.err().unwrap().to_string(),
            "Invalid pattern: Invalid pattern combination",
        );
    }

    #[test]
    fn test_pattern_too_long() {
        let args = Arg {
            starts_with: Some("a".repeat(65)),
            ends_with: None,
            mnemonic: false,
            save_path: None,
            timeout: None,
        };
        let result = handler(args);
        assert_eq!(
            result.err().unwrap().to_string(),
            "Invalid pattern: Pattern too long: max 64 characters",
        );
    }

    #[test]
    fn test_find_simple_vanity_start() {
        let args = Arg {
            starts_with: Some("00".to_string()),
            ends_with: None,
            mnemonic: false,
            save_path: None,
            timeout: None,
        };
        let result = handler(args).unwrap();
        let address = result["Address"].as_str().unwrap();
        assert!(address.starts_with("00"), "Address should start with '00'");
    }

    #[test]
    fn test_find_simple_vanity_end() {
        let args = Arg {
            starts_with: None,
            ends_with: Some("00".to_string()),
            mnemonic: false,
            save_path: None,
            timeout: None,
        };
        let result = handler(args).unwrap();
        let address = result["Address"].as_str().unwrap();
        assert!(address.ends_with("00"), "Address should end with '00'");
    }

    #[test]
    fn test_both_start_and_end() {
        let args = Arg {
            starts_with: Some("a".to_string()),
            ends_with: Some("b".to_string()),
            mnemonic: false,
            save_path: None,
            timeout: None,
        };
        let result = handler(args).unwrap();
        let address = result["Address"].as_str().unwrap();
        assert!(address.starts_with('a'), "Address should start with 'a'");
        assert!(address.ends_with('b'), "Address should end with 'b'");
    }

    #[test]
    fn test_both_start_and_end_case_insensitive() {
        // checksummed addresses cannot start or end with capital letters
        let args = Arg {
            starts_with: Some("A".to_string()),
            ends_with: Some("B".to_string()),
            mnemonic: false,
            save_path: None,
            timeout: None,
        };
        let result = handler(args).unwrap();
        let address = result["Address"].as_str().unwrap();
        assert!(address.starts_with('a'), "Address should start with 'a'");
        assert!(address.ends_with('b'), "Address should end with 'b'");
    }

    #[test]
    fn test_mnemonic_generation() {
        let args = Arg {
            starts_with: Some("a".to_string()),
            ends_with: None,
            mnemonic: true,
            save_path: None,
            timeout: None,
        };
        let result = handler(args).unwrap();
        assert!(
            result.get("Mnemonic").is_some(),
            "Mnemonic should be present"
        );
        assert_eq!(
            result["Mnemonic"]
                .as_str()
                .unwrap()
                .split_whitespace()
                .count(),
            24,
            "Mnemonic should have 24 words"
        );

        let address = result["Address"].as_str().unwrap();
        assert!(address.starts_with('a'), "Address should start with 'a'");
    }

    #[test]
    fn test_save_path() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let args = Arg {
            starts_with: Some("00".to_string()),
            ends_with: None,
            mnemonic: false,
            save_path: Some(tmp.path().to_path_buf()),
            timeout: None,
        };
        handler(args).unwrap();
        assert!(tmp.path().exists(), "File should exist");
        let content = fs::read_to_string(tmp.path()).unwrap();
        let saved_result: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert!(
            saved_result["Address"].is_string(),
            "Saved result should contain an Address"
        );
        assert!(
            saved_result["PrivateKey"].is_string(),
            "Saved result should contain a PrivateKey"
        );
    }

    #[test]
    fn test_invalid_hex_pattern() {
        let args = Arg {
            starts_with: Some("X".to_string()),
            ends_with: None,
            mnemonic: false,
            save_path: None,
            timeout: None,
        };
        let result = handler(args);
        assert!(
            result.is_err(),
            "Handler should fail with invalid hex pattern"
        );
        assert_eq!(
            result.unwrap_err().to_string(),
            "Invalid pattern: pattern: X"
        );
    }
}
