pub mod defaults;
pub mod parameters;
pub mod program_type;

/// The suffix that helps identify the file which contains the hash of the binary file created when
/// scripts are built.
pub const SWAY_BIN_HASH_SUFFIX: &str = "-bin-hash";

/// The suffix that helps identify the file which contains the root hash of the binary file created
/// when predicates are built.
pub const SWAY_BIN_ROOT_SUFFIX: &str = "-bin-root";
