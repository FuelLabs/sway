use pest;
use std::path::PathBuf;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub(crate) struct Span<'sc> {
    pub(crate) span: pest::Span<'sc>,
    pub(crate) path: Option<PathBuf>,
}

impl<'sc> Span<'sc> {
    pub(crate) fn start(&self) -> usize {
        self.span.start()
    }

    pub(crate) fn end(&self) -> usize {
        self.span.end()
    }

    pub(crate) fn start_pos(&self) -> pest::Position {
        self.span.start_pos()
    }

    pub(crate) fn end_pos(&self) -> pest::Position {
        self.span.end_pos()
    }

    pub(crate) fn split(&self) -> (pest::Position<'sc>, pest::Position<'sc>) {
        self.span.split()
    }

    pub(crate) fn as_str(&self) -> &str {
        self.span.as_str()
    }
}
