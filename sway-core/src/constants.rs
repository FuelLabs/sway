//! Configurable yet nonchanging constants for the compiler.

/// The default extension of [LANGUAGE_NAME] files, e.g. `main.sw`.
pub const DEFAULT_FILE_EXTENSION: &str = "sw";
/// After a large language name change PR, we decided to make this configurable. :)
pub const LANGUAGE_NAME: &str = "Sway";
/// The size, in bytes, of a single word in the FuelVM.
pub const VM_WORD_SIZE: u64 = 8;

// Keywords
pub const INVALID_NAMES: &[&str] = &["storage"];

pub const CONTRACT_CALL_GAS_PARAMETER_NAME: &str = "gas";

pub const CONTRACT_CALL_COINS_PARAMETER_NAME: &str = "coins";
pub const CONTRACT_CALL_COINS_PARAMETER_DEFAULT_VALUE: u64 = 0;

pub const CONTRACT_CALL_ASSET_ID_PARAMETER_NAME: &str = "asset_id";
pub const CONTRACT_CALL_ASSET_ID_PARAMETER_DEFAULT_VALUE: [u8; 32] = [0; 32];

/// The default entry point for scripts and predicates.
pub const DEFAULT_ENTRY_POINT_FN_NAME: &str = "main";
