use std::{path::{PathBuf, Path}, fmt, collections::HashSet};
use semver::Version;
use sway_core::{language::parsed::TreeType, fuel_prelude::fuel_tx};
use thiserror::Error as ThisError;

use crate::{manifest::ManifestFile, DepKind, source::path};

#[derive(ThisError, Debug)]
pub enum ForcPkgError {
    #[error("could not find manifest at path: {0}")]
    MissingManifest(PathBuf),
    #[error("failed to serialize lock file: {0}")]
    FailedToSerializeLockFile(String),
    #[error("failed to write lock file: {0}")]
    FailedToWriteLockFile(String),
    #[error("the lock file {0:?} needs to be updated (Cause: {1}) but --locked was passed to prevent this.")]
    LockFileNeedsChange(PathBuf, String),
    #[error("graph contains no project node")]
    NoProjectNodeInGraph,
    #[error("graph contains more than one project node")]
    MultipleProjectNodesInGraph,
    #[error("{0} requires forc version {1} but current version is {2}\nUpdate the toolchain by following: https://fuellabs.github.io/sway/v{2}/introduction/installation.html")]
    OutdatedForcVersion(String, Version, Version),
    #[error("couldn't find manifest file for {0}")]
    ManifestFileNotFound(String),
    #[error("failed to construct path for dependency {0}: {1}")]
    DependencyPathConstructionFailed(String, PathBuf),
    #[error("no entry in parent manifest")]
    NoEntryInParentManifest,
    #[error("dependency node's source does not match manifest entry")]
    DependencySourceDoesNotMatchManifestEntry,
    #[error("\"{0}\" is declared as a {1} dependency, but it is actually a {2}")]
    ProgramTypeMismatch(String, DepKind, TreeType),
    #[error("dependency name {0:?} must match manifest project name {1:?} \
    unless `package` = {1:?}` is specified in the dependency declaration")]
    DependencyNameDoesNotMatchManifestProjectName(String, String),
    #[error("no dependency or patch with name {0:?} in manifest of {1:?}")]
    MissingDependencyOrPatch(String, String),
    #[error("cannot find dependency in the workspace")]
    MissingDependencyInWorkspace,
    #[error("dependency cycle detected: {0:?}")]
    DependencyCycleDetected(String),
    #[error("invalid `path_root` for path dependency package {0}")]
    InvalidPathRoot(String),
    #[error("failed to find path root: `path` dependency \"{0}\" has no parent")]
    FailedToFindPathRoot(path::Pinned),
    #[error("failed to source dependency {0}")]
    FailedToSourceDependency(String),
    #[error("dependency of {0} named {1} is invalid: {2}")]
    InvalidDependency(String, String, String),
    #[error("failed to compile {0}")]
    FailedToCompile(String),
    #[error("invalid test argument(s) for test: {0}")]
    InvalidTestArguments(String),
    #[error("missing span for test function")]
    MissingSpanForTestFunction,
    #[error("cannot find package in the workspace")]
    MissingPackageInWorkspace,
    #[error("there are conflicting salt declarations for contract dependency named: {0}\nDeclared salts: {1:?}")]
    ConflictingSaltDeclarations(String, HashSet<fuel_tx::Salt>),
    #[error("unable to check sway program: build plan contains no packages")]
    UnableToCheck,
    #[error("could not find `{0}` in `{1}` or any parent directory")]
    MissingFileInPathOrParents(String, PathBuf),
    #[error("parsing {0} failed: \n{1}")]
    ParsingFailed(String, String),
    #[error("failed to get current dir, reason: {0}")]
    FailedToGetCurrentDir(std::io::Error),
    #[error("failed to canonicalize path {0}, reason: {1}")]
    FailedToCanonicalize(PathBuf, std::io::Error),
    #[error("failed to read manifest at {0}: {1}")]
    FailedToReadManifest(PathBuf, std::io::Error),
    #[error("failed to parse manifest, reason: {0}.")]
    FailedToParseManifest(String),
    #[error("failed to validate path from entry field {0} in Forc manifest file.")]
    FailedToValidateFromEntry(String),
    #[error("name validation failed, reason: {0}")]
    FailedToValidateName(String)

}

#[derive(ThisError, Debug, Clone)]
/// A warning is an unexpected situation happened during forc-pkg execution which can be handled by
/// forc-pkg internally, but should be addressed by the user as a good practice.
pub enum ForcPkgWarning {
    #[error("  Warning: unused manifest key: {0}")]
    UnusedManifestKey(String),
    #[error("Lock file did not exist")]
    MissingLockFile,
    #[error("Invalid lock: {0}")]
    InvalidLockFile(String),
    #[error("Lock file did not match manifest")]
    LockDidNotMatchManifest,
    #[error("You specified both `{0}` and `release` profiles. Using the `release` profile")]
    ConflictingProfileDeclarationsWithRelase(String),
    #[error("Provided profile option {0} is not present in the manifest file. Using the default profile.")]
    MissingProfile(String)
}
