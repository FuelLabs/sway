use thiserror::Error;
use tower_lsp::lsp_types::Diagnostic;

#[derive(Debug, Error)]
pub enum LanguageServerError {
    #[error(transparent)]
    DocumentError(#[from] DocumentError),

    #[error("Failed to create build plan. {0}")]
    BuildPlanFailed(anyhow::Error),
    #[error("Failed to compile. {0}")]
    FailedToCompile(anyhow::Error),
    #[error("Failed to parse document. {:?}", diagnostics)]
    FailedToParse { diagnostics: Vec<Diagnostic> },
}

#[derive(Debug, Error)]
pub enum DocumentError {
    #[error("No document found at {:?}", path)]
    DocumentNotFound { path: String },
    #[error("Missing Forc.toml in {:?}", dir)]
    ManifestFileNotFound { dir: String },
    #[error("Document is already stored.")]
    DocumentAlreadyStored { path: String },
}
