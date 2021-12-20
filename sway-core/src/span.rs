use std::{path::PathBuf, sync::Arc};

/// Represents a span of the source code in a specific file.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Span {
    ///  A [pest::Span] returned directly from the generated parser.
    pub span: pest::Span,
    // A reference counted pointer to the file from which this span originated.
    pub(crate) path: Option<Arc<PathBuf>>,
}

impl Span {
    pub fn start(&self) -> usize {
        self.span.start()
    }

    pub fn end(&self) -> usize {
        self.span.end()
    }

    pub fn start_pos(&self) -> pest::Position {
        self.span.start_pos()
    }

    pub fn end_pos(&self) -> pest::Position {
        self.span.end_pos()
    }

    pub fn split(&self) -> (pest::Position, pest::Position) {
        self.span.clone().split()
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

    pub fn trim(self) -> Span {
        let start_delta = self.as_str().len() - self.as_str().trim_start().len();
        let end_delta = self.as_str().len() - self.as_str().trim_end().len();
        let span = pest::Span::new(
            self.span.input().clone(),
            self.span.start() + start_delta,
            self.span.end() - end_delta,
        ).unwrap();
        Span {
            span,
            path: self.path,
        }
    }
}
