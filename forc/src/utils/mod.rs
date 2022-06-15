pub mod defaults;
pub mod parameters;
pub mod program_type;

/// The `forc` crate version formatted with the `v` prefix. E.g. "v1.2.3".
///
/// This git tag is used during `Manifest` construction to pin the version of the implicit `std`
/// dependency to the `forc` version.
pub const SWAY_GIT_TAG: &str = concat!("v", clap::crate_version!());

/// The suffix that helps identify the file which contains the hash of the binary file created when
/// scripts are built.
pub const SWAY_BIN_HASH_SUFFIX: &str = "-bin-hash";
