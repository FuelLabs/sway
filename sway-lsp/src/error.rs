use thiserror::Error;
use tower_lsp::lsp_types::Diagnostic;

#[derive(Debug, Error)]
pub enum LanguageServerError {
    #[error(transparent)]
    DocumentError(#[from] DocumentError),
    #[error(transparent)]
    DirectoryError(#[from] DirectoryError),

    #[error("Failed to create build plan. {0}")]
    BuildPlanFailed(anyhow::Error),
    #[error("Failed to compile. {0}")]
    FailedToCompile(anyhow::Error),
    #[error("Failed to parse document")]
    FailedToParse { diagnostics: Vec<Diagnostic> },
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DocumentError {
    #[error("No document found at {:?}", path)]
    DocumentNotFound { path: String },
    #[error("Missing Forc.toml in {:?}", dir)]
    ManifestFileNotFound { dir: String },
    #[error("Document is already stored at {:?}", path)]
    DocumentAlreadyStored { path: String },
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
    #[error("Failed to create temp directory")]
    CanonicalizeFailed,
    #[error("Failed to copy workspace contents to temp directory")]
    CopyContentsFailed,

    #[error("Failed to create build plan. {0}")]
    StripPrefixError(std::path::StripPrefixError),

    #[error("Unable to create Url from path {:?}", path)]
    UrlFromPathFailed { path: String },
}
