//! Configurable yet nonchanging constants for the compiler.

/// The default extension of [LANGUAGE_NAME] files, e.g. `main.sw`.
pub const DEFAULT_FILE_EXTENSION: &str = "sw";
/// After a large language name change PR, we decided to make this configurable. :)
pub const LANGUAGE_NAME: &str = "Sway";
/// The size, in bytes, of a single word in the FuelVM.
pub const VM_WORD_SIZE: u64 = 8;

pub const CONTRACT_CALL_GAS_PARAMETER_NAME: &str = "gas";

pub const CONTRACT_CALL_COINS_PARAMETER_NAME: &str = "coins";
pub const CONTRACT_CALL_COINS_PARAMETER_DEFAULT_VALUE: u64 = 0;

pub const CONTRACT_CALL_ASSET_ID_PARAMETER_NAME: &str = "asset_id";
pub const CONTRACT_CALL_ASSET_ID_PARAMETER_DEFAULT_VALUE: [u8; 32] = [0; 32];

/// The default entry point for scripts and predicates.
pub const DEFAULT_ENTRY_POINT_FN_NAME: &str = "main";

/// The default prefix for the compiler generated names of tuples
pub const TUPLE_NAME_PREFIX: &str = "__tuple_";

// The default prefix for the compiler generated names of struct fields
pub const DESTRUCTURE_PREFIX: &str = "__destructure_";

/// The default prefix for the compiler generated names of match
pub const MATCH_RETURN_VAR_NAME_PREFIX: &str = "__match_return_var_name_";

/// The valid attribute strings related to storage and purity.
pub const STORAGE_PURITY_ATTRIBUTE_NAME: &str = "storage";
pub const STORAGE_PURITY_READ_NAME: &str = "read";
pub const STORAGE_PURITY_WRITE_NAME: &str = "write";

/// The valid attribute strings related to inline.
pub const INLINE_ATTRIBUTE_NAME: &str = "inline";
pub const INLINE_NEVER_NAME: &str = "never";
pub const INLINE_ALWAYS_NAME: &str = "always";

/// The valid attribute strings related to documentation.
pub const DOC_ATTRIBUTE_NAME: &str = "doc";

/// The attribute used for Sway in-language unit tests.
pub const TEST_ATTRIBUTE_NAME: &str = "test";

/// The valid attribute string used for payable functions.
pub const PAYABLE_ATTRIBUTE_NAME: &str = "payable";

/// The list of valid attributes.
pub const VALID_ATTRIBUTE_NAMES: &[&str] = &[
    STORAGE_PURITY_ATTRIBUTE_NAME,
    DOC_ATTRIBUTE_NAME,
    TEST_ATTRIBUTE_NAME,
    INLINE_ATTRIBUTE_NAME,
    PAYABLE_ATTRIBUTE_NAME,
];
