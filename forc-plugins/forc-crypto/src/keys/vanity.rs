use fuel_crypto::{fuel_types::Address, PublicKey, SecretKey};
use fuels_accounts::signers::{
    derivation::DEFAULT_DERIVATION_PATH, private_key::generate_mnemonic_phrase,
};
use fuels_core::types::checksum_address::checksum_encode;
use rayon::iter::{self, Either, ParallelIterator};
use regex::Regex;
use serde_json::json;
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};
use tokio::runtime::Runtime;

forc_util::cli_examples! {
    crate::Command {
        [ Generate a checksummed vanity address with a given prefix => "forc crypto vanity --starts-with \"aaa\"" ]
        [ Generate a checksummed vanity address with a given suffix => "forc crypto vanity --ends-with \"aaa\"" ]
        [ Generate a checksummed vanity address with a given prefix and suffix => "forc crypto vanity --starts-with \"00\" --ends-with \"ff\"" ]
        [ Generate a checksummed vanity address with a given regex pattern => "forc crypto vanity --regex \"^00.*ff$\"" ]
    }
}

fn validate_hex_string(s: &str) -> Result<String, String> {
    if !s.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("Pattern must contain only hex characters (0-9, a-f)".to_string());
    }
    Ok(s.to_string())
}

fn validate_regex_pattern(s: &str) -> Result<String, String> {
    if s.len() > 128 {
        return Err("Regex pattern too long: max 128 characters".to_string());
    }

    if let Err(e) = Regex::new(&format!("(?i){s}")) {
        return Err(format!("Invalid regex pattern: {e}"));
    }

    Ok(s.to_string())
}

#[derive(Debug, clap::Parser)]
#[clap(
    version,
    about = "Generate a vanity address",
    after_help = "Generate vanity addresses for the Fuel blockchain"
)]
pub struct Arg {
    /// Desired hex string prefix for the address
    #[arg(
        long,
        value_name = "HEX_STRING",
        required_unless_present = "ends_with",
        required_unless_present = "regex",
        conflicts_with = "regex",
        value_parser = validate_hex_string,
    )]
    pub starts_with: Option<String>,

    /// Desired hex string suffix for the address
    #[arg(long, value_name = "HEX_STRING", conflicts_with = "regex", value_parser = validate_hex_string)]
    pub ends_with: Option<String>,

    /// Desired regex pattern to match the entire address (case-insensitive)
    #[arg(long, value_name = "PATTERN", conflicts_with = "starts_with", value_parser = validate_regex_pattern)]
    pub regex: Option<String>,

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

impl Arg {
    pub fn validate(&self) -> anyhow::Result<()> {
        let total_length = self.starts_with.as_ref().map_or(0, |s| s.len())
            + self.ends_with.as_ref().map_or(0, |s| s.len());
        if total_length > 64 {
            return Err(anyhow::anyhow!(
                "Combined pattern length exceeds 64 characters"
            ));
        }
        Ok(())
    }
}

pub fn handler(args: Arg) -> anyhow::Result<serde_json::Value> {
    args.validate()?;

    let Arg {
        starts_with,
        ends_with,
        regex,
        mnemonic,
        timeout,
        save_path,
    } = args;

    let matcher = if let Some(pattern) = regex {
        Either::Left(RegexMatcher::new(&pattern)?)
    } else {
        let starts_with = starts_with.as_deref().unwrap_or("");
        let ends_with = ends_with.as_deref().unwrap_or("");
        Either::Right(HexMatcher::new(starts_with, ends_with)?)
    };

    println!("Starting to generate vanity address...");
    let start_time = Instant::now();

    let result = find_vanity_address_with_timeout(matcher, mnemonic, timeout)?;
    let (address, secret_key, mnemonic) = result;

    let duration = start_time.elapsed();
    println!(
        "Successfully found vanity address in {:.3} seconds.\n",
        duration.as_secs_f64()
    );

    let checksum_address = checksum_encode(&address.to_string())?;
    let result = if let Some(mnemonic) = mnemonic {
        json!({
            "Address": checksum_address,
            "PrivateKey": hex::encode(secret_key.as_ref()),
            "Mnemonic": mnemonic,
        })
    } else {
        json!({
            "Address": checksum_address,
            "PrivateKey": hex::encode(secret_key.as_ref()),
        })
    };

    if let Some(path) = save_path {
        std::fs::write(path, serde_json::to_string_pretty(&result)?)?;
    }

    Ok(result)
}

pub trait VanityMatcher: Send + Sync + 'static {
    fn is_match(&self, addr: &Address) -> bool;
}

pub struct HexMatcher {
    prefix: String,
    suffix: String,
}

impl HexMatcher {
    pub fn new(prefix: &str, suffix: &str) -> anyhow::Result<Self> {
        Ok(Self {
            prefix: prefix.to_lowercase(),
            suffix: suffix.to_lowercase(),
        })
    }
}

impl VanityMatcher for HexMatcher {
    fn is_match(&self, addr: &Address) -> bool {
        let hex_addr = hex::encode(addr.as_ref()).to_lowercase();
        hex_addr.starts_with(&self.prefix) && hex_addr.ends_with(&self.suffix)
    }
}

pub struct RegexMatcher {
    re: Regex,
}

impl RegexMatcher {
    pub fn new(pattern: &str) -> anyhow::Result<Self> {
        let re = Regex::new(&format!("(?i){pattern}"))?;
        Ok(Self { re })
    }
}

impl VanityMatcher for RegexMatcher {
    fn is_match(&self, addr: &Address) -> bool {
        let addr = hex::encode(addr.as_ref());
        self.re.is_match(&addr)
    }
}

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub fn find_vanity_address_with_timeout(
    matcher: Either<RegexMatcher, HexMatcher>,
    use_mnemonic: bool,
    timeout_secs: Option<u64>,
) -> anyhow::Result<(Address, SecretKey, Option<String>)> {
    let should_stop = Arc::new(AtomicBool::new(false));
    let should_stop_clone = should_stop.clone();

    let generate_wallet = move || {
        let breakpoint = if use_mnemonic { 1_000 } else { 100_000 };
        let start = Instant::now();
        let attempts = std::sync::atomic::AtomicUsize::new(0);

        wallet_generator(use_mnemonic)
            .find_any(|result| {
                // Check if we should stop due to timeout
                if should_stop.load(Ordering::Relaxed) {
                    return true; // This will cause find_any to return the current result
                }

                let current = attempts.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                if current != 0 && current.is_multiple_of(breakpoint) {
                    let elapsed = start.elapsed().as_secs_f64();
                    let rate = current as f64 / elapsed;
                    println!("└─ tried {current} addresses ({rate:.2} addresses/sec)...");
                }

                if let Ok((addr, _, _)) = result {
                    match &matcher {
                        Either::Left(regex_matcher) => regex_matcher.is_match(addr),
                        Either::Right(hex_matcher) => hex_matcher.is_match(addr),
                    }
                } else {
                    false
                }
            })
            .ok_or_else(|| anyhow::anyhow!("No matching address found"))?
    };

    let Some(secs) = timeout_secs else {
        return generate_wallet();
    };

    Runtime::new()?.block_on(async {
        let generation_task = tokio::task::spawn_blocking(generate_wallet);

        tokio::select! {
            result = generation_task => {
                match result {
                    Ok(wallet_result) => wallet_result,
                    Err(_) => Err(anyhow::anyhow!("No matching address found")),
                }
            }
            _ = tokio::time::sleep(Duration::from_secs(secs)) => {
                // Signal all threads to stop
                should_stop_clone.store(true, Ordering::Relaxed);
                // Wait a short time for threads to notice the stop signal
                tokio::time::sleep(Duration::from_millis(100)).await;
                Err(anyhow::anyhow!("Vanity address generation timed out after {} seconds", secs))
            }
        }
    })
}

#[inline]
fn wallet_generator(
    use_mnemonic: bool,
) -> impl ParallelIterator<Item = anyhow::Result<(Address, SecretKey, Option<String>)>> {
    iter::repeat(()).map(move |()| generate_wallet(use_mnemonic))
}

fn generate_wallet(use_mnemonic: bool) -> anyhow::Result<(Address, SecretKey, Option<String>)> {
    let mut rng = rand::thread_rng();

    let (private_key, mnemonic) = if use_mnemonic {
        let mnemonic = generate_mnemonic_phrase(&mut rng, 24)?;
        let private_key =
            SecretKey::new_from_mnemonic_phrase_with_path(&mnemonic, DEFAULT_DERIVATION_PATH)?;
        (private_key, Some(mnemonic))
    } else {
        (SecretKey::random(&mut rng), None)
    };

    let public = PublicKey::from(&private_key);
    let hashed = public.hash();
    let address = Address::from(*hashed);

    Ok((address, private_key, mnemonic))
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    // Helper function to parse args and get validation errors
    fn parse_args(args: Vec<&str>) -> Result<Arg, String> {
        let args =
            Arg::try_parse_from(std::iter::once("test").chain(args)).map_err(|e| e.to_string())?;
        args.validate().map_err(|e| e.to_string())?;
        Ok(args)
    }

    #[test]
    fn test_invalid_hex_characters() {
        let result = parse_args(vec!["--starts-with", "xyz"]);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "error: invalid value 'xyz' for '--starts-with <HEX_STRING>': Pattern must contain only hex characters (0-9, a-f)\n\nFor more information, try '--help'.\n");
    }

    #[test]
    fn test_pattern_too_long() {
        let result = parse_args(vec![
            "--starts-with",
            &"a".repeat(32),
            "--ends-with",
            &"b".repeat(33),
        ]);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Combined pattern length exceeds 64 characters"
        );
    }

    #[test]
    fn test_invalid_regex_syntax() {
        let result = parse_args(vec!["--regex", "["]);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "error: invalid value '[' for '--regex <PATTERN>': Invalid regex pattern: regex parse error:\n    (?i)[\n        ^\nerror: unclosed character class\n\nFor more information, try '--help'.\n");
    }

    #[test]
    fn test_regex_too_long() {
        let result = parse_args(vec!["--regex", &"a".repeat(129)]);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "error: invalid value 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa' for '--regex <PATTERN>': Regex pattern too long: max 128 characters\n\nFor more information, try '--help'.\n");
    }

    #[test]
    fn test_conflicting_args() {
        let result = parse_args(vec!["--starts-with", "aa", "--regex", "^aa"]);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "error: the argument '--starts-with <HEX_STRING>' cannot be used with '--regex <PATTERN>'\n\nUsage: test --starts-with <HEX_STRING>\n\nFor more information, try '--help'.\n");
    }

    #[test]
    fn test_timeout_respected() {
        // This pattern should take a long time to generate
        let args = parse_args(vec!["--starts-with", "fffffffffffff", "--timeout", "1"]).unwrap();

        let result = handler(args);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Vanity address generation timed out after 1 seconds"
        );
    }

    // Test actual functionality with minimal patterns
    #[test]
    fn test_valid_short_prefix() {
        let args = parse_args(vec!["--starts-with", "a"]).unwrap();
        let result = handler(args).unwrap();
        let address = result["Address"].as_str().unwrap();
        assert!(
            address.to_lowercase().starts_with("0xa"),
            "Address should start with 'a'"
        );
    }

    #[test]
    fn test_valid_short_suffix() {
        let args = parse_args(vec!["--ends-with", "a"]).unwrap();
        let result = handler(args).unwrap();
        let address = result["Address"].as_str().unwrap();
        assert!(
            address.to_lowercase().ends_with('a'),
            "Address should end with 'a'"
        );
    }

    #[test]
    fn test_both_prefix_and_suffix() {
        let args = parse_args(vec!["--starts-with", "a", "--ends-with", "b"]).unwrap();
        let result = handler(args).unwrap();
        let address = result["Address"].as_str().unwrap().to_lowercase();
        assert!(address.starts_with("0xa"), "Address should start with 'a'");
        assert!(address.ends_with('b'), "Address should end with 'b'");
    }

    #[test]
    fn test_simple_regex() {
        let args = parse_args(vec!["--regex", "^a.*b$"]).unwrap();
        let result = handler(args).unwrap();
        let address = result["Address"].as_str().unwrap().to_lowercase();
        assert!(address.starts_with("0xa"), "Address should start with 'a'");
        assert!(address.ends_with('b'), "Address should end with 'b'");
    }

    #[test]
    fn test_simple_regex_uppercase() {
        let args = parse_args(vec!["--regex", "^A.*B$"]).unwrap();
        let result = handler(args).unwrap();
        let address = result["Address"].as_str().unwrap().to_lowercase();
        assert!(address.starts_with("0xa"), "Address should start with 'a'");
        assert!(address.ends_with('b'), "Address should end with 'b'");
    }

    #[test]
    fn test_mnemonic_generation() {
        let args = parse_args(vec!["--starts-with", "a", "--mnemonic"]).unwrap();
        let result = handler(args).unwrap();

        assert!(result["Mnemonic"].is_string(), "Mnemonic should be present");
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
        assert!(
            address.to_lowercase().starts_with("0xa"),
            "Address should start with 'a'"
        );
    }

    #[test]
    fn test_save_path() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let args = parse_args(vec![
            "--starts-with",
            "a",
            "--save-path",
            tmp.path().to_str().unwrap(),
        ])
        .unwrap();

        handler(args).unwrap();

        assert!(tmp.path().exists(), "File should exist");
        let content = std::fs::read_to_string(tmp.path()).unwrap();
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
}
