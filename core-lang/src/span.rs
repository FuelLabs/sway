use std::{path::PathBuf, sync::Arc};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Span<'sc> {
    pub span: fuel_pest::Span<'sc>,
    pub(crate) path: Option<Arc<PathBuf>>,
}

impl<'sc> Span<'sc> {
    pub fn start(&self) -> usize {
        self.span.start()
    }

    pub fn end(&self) -> usize {
        self.span.end()
    }

    pub fn start_pos(&self) -> fuel_pest::Position {
        self.span.start_pos()
    }

    pub fn end_pos(&self) -> fuel_pest::Position {
        self.span.end_pos()
    }

    pub fn split(&self) -> (fuel_pest::Position<'sc>, fuel_pest::Position<'sc>) {
        self.span.split()
    }

    pub fn str(self) -> String {
        self.span.as_str().to_string()
    }

    pub fn as_str(&self) -> &str {
        self.span.as_str()
    }

    pub fn input(&self) -> &str {
        self.span.input()
    }

    pub fn path(&self) -> String {
        self.path
            .as_deref()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|| "".to_string())
    }
}
