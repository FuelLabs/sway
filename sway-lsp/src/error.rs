use swayfmt::FormatterError;
use thiserror::Error;

use crate::capabilities::diagnostic::Diagnostics;

#[derive(Debug, Error)]
pub enum LanguageServerError {
    // Inherited errors
    #[error(transparent)]
    DocumentError(#[from] DocumentError),
    #[error(transparent)]
    DirectoryError(#[from] DirectoryError),
    #[error(transparent)]
    RenameError(#[from] RenameError),

    // Top level errors
    #[error("Failed to create build plan. {0}")]
    BuildPlanFailed(anyhow::Error),
    #[error("Failed to compile. {0}")]
    FailedToCompile(anyhow::Error),
    #[error("Failed to parse document")]
    FailedToParse { diagnostics: Diagnostics },
    #[error("Error formatting document: {0}")]
    FormatError(FormatterError),
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DocumentError {
    #[error("No document found at {:?}", path)]
    DocumentNotFound { path: String },
    #[error("Missing Forc.toml in {:?}", dir)]
    ManifestFileNotFound { dir: String },
    #[error("Cannot get member manifest files for the manifest at {:?}", dir)]
    MemberManifestsFailed { dir: String },
    #[error("Cannot get lock file path for the manifest at {:?}", dir)]
    ManifestsLockPathFailed { dir: String },
    #[error("Document is already stored at {:?}", path)]
    DocumentAlreadyStored { path: String },
    #[error("File wasn't able to be created at path {:?} : {:?}", path, err)]
    UnableToCreateFile { path: String, err: String },
    #[error("Unable to write string to file at {:?} : {:?}", path, err)]
    UnableToWriteFile { path: String, err: String },
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DirectoryError {
    #[error("Can't find temporary directory")]
    TempDirNotFound,
    #[error("Can't find manifest directory")]
    ManifestDirNotFound,
    #[error("Can't extract project name from {:?}", dir)]
    CantExtractProjectName { dir: String },
    #[error("Failed to create temp directory")]
    TempDirFailed,
    #[error("Failed to canonicalize path")]
    CanonicalizeFailed,
    #[error("Failed to copy workspace contents to temp directory")]
    CopyContentsFailed,
    #[error("Failed to create build plan. {0}")]
    StripPrefixError(std::path::StripPrefixError),
    #[error("Unable to create Url from path {:?}", path)]
    UrlFromPathFailed { path: String },
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RenameError {
    #[error("No token was found in the token map at that position")]
    TokenNotFound,
    #[error("Token is not part of the users workspace")]
    TokenNotPartOfWorkspace,
    #[error("Keywords and instrinsics are unable to be renamed")]
    UnableToRenameKeyword,
    #[error("Invalid name {:?}: not an identifier", name)]
    InvalidName { name: String },
    #[error("Identifiers cannot begin with a double underscore, as that naming convention is reserved for compiler intrinsics.")]
    InvalidDoubleUnderscore,
}
