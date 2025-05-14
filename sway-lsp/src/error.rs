use lsp_types::Range;
use swayfmt::FormatterError;
use thiserror::Error;

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
    #[error("Build Plan Cache is empty")]
    BuildPlanCacheIsEmpty,
    #[error("Failed to compile. {0}")]
    FailedToCompile(anyhow::Error),
    #[error("Failed to parse document")]
    FailedToParse,
    #[error("Error formatting document: {0}")]
    FormatError(FormatterError),
    #[error("No Programs were returned from the compiler")]
    ProgramsIsNone,
    #[error("Member program not found in the compiler results")]
    MemberProgramNotFound,
    #[error("Unable to acquire a semaphore permit for parsing")]
    UnableToAcquirePermit,
    #[error("Client is not initialized")]
    ClientNotInitialized,
    #[error("Client request error: {0}")]
    ClientRequestError(String),
    #[error("Global workspace not initialized")]
    GlobalWorkspaceNotInitialized,
    #[error("SyncWorkspace already initialized")]
    SyncWorkspaceAlreadyInitialized,
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
    #[error("File wasn't able to be removed at path {:?} : {:?}", path, err)]
    UnableToRemoveFile { path: String, err: String },

    #[error("Permission denied for path {:?}", path)]
    PermissionDenied { path: String },
    #[error("IO error for path {:?} : {:?}", path, error)]
    IOError { path: String, error: String },
    #[error("Invalid range {:?}", range)]
    InvalidRange { range: Range },
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DirectoryError {
    #[error("Can't find temporary directory")]
    TempDirNotFound,
    #[error("Can't find manifest directory")]
    ManifestDirNotFound,
    #[error("Can't find temporary member directory")]
    TempMemberDirNotFound,
    #[error("Can't extract project name from {:?}", dir)]
    CantExtractProjectName { dir: String },
    #[error("Failed to create hidden .lsp_locks directory: {0}")]
    LspLocksDirFailed(String),
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
    #[error("Unable to create Url from span {:?}", span)]
    UrlFromSpanFailed { span: String },
    #[error("Unable to create path from Url {:?}", url)]
    PathFromUrlFailed { url: String },
    #[error("Unable to create span from path {:?}", path)]
    SpanFromPathFailed { path: String },
    #[error("No program ID found for path {:?}", path)]
    ProgramIdNotFound { path: String },
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RenameError {
    #[error("No token was found in the token map at that position")]
    TokenNotFound,
    #[error("Token is not part of the user's workspace")]
    TokenNotPartOfWorkspace,
    #[error("Keywords and intrinsics are unable to be renamed")]
    SymbolKindNotAllowed,
    #[error("Invalid name {:?}: not an identifier", name)]
    InvalidName { name: String },
    #[error("Identifiers cannot begin with a double underscore, as that naming convention is reserved for compiler intrinsics.")]
    InvalidDoubleUnderscore,
    #[error("The file {:?}: already exists", path)]
    FileAlreadyExists { path: String },
    #[error("The module {:?}: cannot be renamed", path)]
    UnableToRenameModule { path: String },
}
