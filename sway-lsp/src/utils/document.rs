use crate::error::DirectoryError;
use std::path::PathBuf;
use sway_types::{SourceEngine, Span};
use tower_lsp::lsp_types::Url;

/// Create a [Url] from a [PathBuf].
pub fn get_url_from_path(path: &PathBuf) -> Result<Url, DirectoryError> {
    Url::from_file_path(path).map_err(|_| DirectoryError::UrlFromPathFailed {
        path: path.to_string_lossy().to_string(),
    })
}

/// Create a [PathBuf] from a [Url].
pub fn get_path_from_url(url: &Url) -> Result<PathBuf, DirectoryError> {
    url.to_file_path()
        .map_err(|_| DirectoryError::PathFromUrlFailed {
            url: url.to_string(),
        })
}

/// Create a [Url] from a [Span].
pub fn get_url_from_span(source_engine: &SourceEngine, span: &Span) -> Result<Url, DirectoryError> {
    if let Some(source_id) = span.source_id() {
        let path = source_engine.get_path(source_id);
        get_url_from_path(&path)
    } else {
        Err(DirectoryError::UrlFromSpanFailed {
            span: span.as_str().to_string(),
        })
    }
}
