use anyhow::{anyhow, Result};
use fuel_crypto::{fuel_types::Address, PublicKey, SecretKey};
use fuels_accounts::wallet::{generate_mnemonic_phrase, DEFAULT_DERIVATION_PATH_PREFIX};
use fuels_core::types::bech32::{Bech32Address, FUEL_BECH32_HRP};
use rayon::iter::{self, Either, ParallelIterator};
use regex::Regex;
use serde_json::json;
use std::{fs, path::PathBuf, time::Instant};

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

    /// Return mnemonic with address (default false)
    #[arg(long)]
    pub mnemonic: bool,

    /// Path to save the generated vanity address to.
    #[arg(long, value_hint = clap::ValueHint::FilePath, value_name = "PATH")]
    pub save_path: Option<PathBuf>,
}

pub fn handler(args: Arg) -> Result<serde_json::Value> {
    let Arg {
        starts_with,
        ends_with,
        mnemonic,
        save_path,
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

    let (address, secret_key, mnemonic) =
        match (left_exact_hex, left_regex, right_exact_hex, right_regex) {
            (Some(left), _, Some(right), _) => {
                let matcher = HexMatcher { left, right };
                find_vanity_address(matcher, mnemonic)
            }
            (Some(left), _, _, Some(right)) => {
                let matcher = LeftExactRightRegexMatcher { left, right };
                find_vanity_address(matcher, mnemonic)
            }
            (_, Some(left), _, Some(right)) => {
                let matcher = RegexMatcher { left, right };
                find_vanity_address(matcher, mnemonic)
            }
            (_, Some(left), Some(right), _) => {
                let matcher = LeftRegexRightExactMatcher { left, right };
                find_vanity_address(matcher, mnemonic)
            }
            (Some(left), None, None, None) => {
                let matcher = LeftHexMatcher { left };
                find_vanity_address(matcher, mnemonic)
            }
            (None, None, Some(right), None) => {
                let matcher = RightHexMatcher { right };
                find_vanity_address(matcher, mnemonic)
            }
            (None, Some(re), None, None) => {
                let matcher = SingleRegexMatcher { re };
                find_vanity_address(matcher, mnemonic)
            }
            (None, None, None, Some(re)) => {
                let matcher = SingleRegexMatcher { re };
                find_vanity_address(matcher, mnemonic)
            }
            _ => return Err(anyhow!("Invalid pattern combination")),
        }?;

    let duration = start_time.elapsed();
    println!(
        "Successfully found vanity address in {:.3} seconds.",
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

/// Generates random wallets until `matcher` matches the wallet address, returning the wallet.
pub fn find_vanity_address<T: VanityMatcher>(
    matcher: T,
    use_mnemonic: bool,
) -> Result<(Address, SecretKey, Option<String>)> {
    wallet_generator(use_mnemonic)
        .find_any(|result| {
            if let Ok((addr, _, _)) = result {
                matcher.is_match(addr)
            } else {
                false
            }
        })
        .ok_or_else(|| anyhow!("Failed to generate a matching address"))?
}

/// Returns an infinite parallel iterator which yields addresses.
#[inline]
fn wallet_generator(
    use_mnemonic: bool,
) -> impl ParallelIterator<Item = Result<(Address, SecretKey, Option<String>)>> {
    iter::repeat(()).map(move |()| generate_wallet(use_mnemonic))
}

/// Generates an address, secret key, and optionally a mnemonic.
fn generate_wallet(use_mnemonic: bool) -> Result<(Address, SecretKey, Option<String>)> {
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

pub trait VanityMatcher: Send + Sync {
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

fn parse_pattern(pattern: &str, is_start: bool) -> Result<Either<Vec<u8>, Regex>> {
    if let Ok(decoded) = hex::decode(pattern) {
        if decoded.len() > 32 {
            return Err(anyhow!("Hex pattern must be less than 32 bytes"));
        }
        Ok(Either::Left(decoded))
    } else {
        // Check if the pattern contains only valid hex characters
        if !pattern
            .chars()
            .all(|c| c.is_ascii_hexdigit() || c == '^' || c == '$')
        {
            return Err(anyhow!("Invalid hex pattern: {}", pattern));
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

