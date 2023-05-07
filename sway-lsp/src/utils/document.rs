use crate::error::DirectoryError;
use std::path::PathBuf;
use sway_types::Span;
use tower_lsp::lsp_types::Url;

/// Create a [Url] from a [PathBuf].
pub fn get_url_from_path(path: &PathBuf) -> Result<Url, DirectoryError> {
    Url::from_file_path(path).map_err(|_| DirectoryError::UrlFromPathFailed {
        path: path.to_string_lossy().to_string(),
    })
}

/// Create a [Url] from a [Span].
pub fn get_url_from_span(span: &Span) -> Result<Url, DirectoryError> {
    if let Some(path) = span.path() {
        get_url_from_path(path)
    } else {
        Err(DirectoryError::UrlFromSpanFailed {
            span: span.as_str().to_string(),
        })
    }
}
