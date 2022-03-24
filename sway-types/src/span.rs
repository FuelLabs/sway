use std::{path::PathBuf, sync::Arc};

/// Represents a span of the source code in a specific file.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Span {
    ///  A [pest::Span] returned directly from the generated parser.
    pub span: pest::Span,
    // A reference counted pointer to the file from which this span originated.
    pub path: Option<Arc<PathBuf>>,
}

impl Span {
    pub fn empty() -> Self {
        Span {
            span: pest::Span::new(" ".into(), 0, 0).unwrap(),
            path: None,
        }
    }

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
        )
        .unwrap();
        Span {
            span,
            path: self.path,
        }
    }
}

/// This panics if the spans are not from the same file. This should
/// only be used on spans that are actually next to each other.
pub fn join_spans(s1: Span, s2: Span) -> Span {
    if s1.as_str() == "core" {
        return s2;
    }
    assert!(
        s1.input() == s2.input() && s1.path == s2.path,
        "Spans from different files cannot be joined.",
    );

    let s1_positions = s1.split();
    let s2_positions = s2.split();
    if s1_positions.0 < s2_positions.1 {
        Span {
            span: s1_positions.0.span(&s2_positions.1),
            path: s1.path,
        }
    } else {
        Span {
            span: s2_positions.0.span(&s1_positions.1),
            path: s1.path,
        }
    }
}
