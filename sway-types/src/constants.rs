//! Configurable yet non-changing constants for the compiler.

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

/// The valid attribute strings related to storage and purity.
pub const STORAGE_PURITY_ATTRIBUTE_NAME: &str = "storage";
pub const STORAGE_PURITY_READ_NAME: &str = "read";
pub const STORAGE_PURITY_WRITE_NAME: &str = "write";

/// The valid attribute strings related to inline.
pub const INLINE_ATTRIBUTE_NAME: &str = "inline";
pub const INLINE_NEVER_NAME: &str = "never";
pub const INLINE_ALWAYS_NAME: &str = "always";

/// The valid attribute strings related to documentation control.
pub const DOC_ATTRIBUTE_NAME: &str = "doc";

/// The valid attribute strings related to documentation comments.
pub const DOC_COMMENT_ATTRIBUTE_NAME: &str = "doc-comment";

/// The attribute used for Sway in-language unit tests.
pub const TEST_ATTRIBUTE_NAME: &str = "test";

/// The valid attribute string used for payable functions.
pub const PAYABLE_ATTRIBUTE_NAME: &str = "payable";

/// The valid attribute strings related to allow.
pub const ALLOW_ATTRIBUTE_NAME: &str = "allow";
pub const ALLOW_DEAD_CODE_NAME: &str = "dead_code";
pub const ALLOW_DEPRECATED_NAME: &str = "deprecated";

/// The valid attribute strings related to conditional compilation.
pub const CFG_ATTRIBUTE_NAME: &str = "cfg";
pub const CFG_TARGET_ARG_NAME: &str = "target";
pub const CFG_PROGRAM_TYPE_ARG_NAME: &str = "program_type";
pub const CFG_EXPERIMENTAL_NEW_ENCODING: &str = "experimental_new_encoding";

pub const DEPRECATED_ATTRIBUTE_NAME: &str = "deprecated";

pub const NAMESPACE_ATTRIBUTE_NAME: &str = "namespace";

pub const FALLBACK_ATTRIBUTE_NAME: &str = "fallback";

/// The list of valid attributes.
pub const VALID_ATTRIBUTE_NAMES: &[&str] = &[
    STORAGE_PURITY_ATTRIBUTE_NAME,
    DOC_ATTRIBUTE_NAME,
    DOC_COMMENT_ATTRIBUTE_NAME,
    TEST_ATTRIBUTE_NAME,
    INLINE_ATTRIBUTE_NAME,
    PAYABLE_ATTRIBUTE_NAME,
    ALLOW_ATTRIBUTE_NAME,
    CFG_ATTRIBUTE_NAME,
    DEPRECATED_ATTRIBUTE_NAME,
    NAMESPACE_ATTRIBUTE_NAME,
    FALLBACK_ATTRIBUTE_NAME,
];

pub const CORE: &str = "core";
pub const STD: &str = "std";
pub const PRELUDE: &str = "prelude";
