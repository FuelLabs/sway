use crate::SourceId;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::{
    cmp,
    fmt::{self, Display},
    hash::Hash,
    sync::Arc,
};

lazy_static! {
    static ref DUMMY_SPAN: Span = Span::new(
        Source {
            text: Arc::from(""),
            line_starts: Arc::new(vec![])
        },
        0,
        0,
        None
    )
    .unwrap();
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq, Serialize, Deserialize)]
pub struct Source {
    pub text: Arc<str>,
    pub line_starts: Arc<Vec<usize>>,
}

impl Source {
    pub fn new(text: &str) -> Self {
        Self {
            text: Arc::from(text),
            line_starts: Arc::new(
                text.char_indices()
                    .filter(|x| x.1 == '\n')
                    .map(|x| x.0)
                    .collect(),
            ),
        }
    }

    pub fn line_col(&self, position: usize) -> LineCol {
        if position > self.text.len() || self.text.is_empty() {
            LineCol { line: 0, col: 0 }
        } else {
            let (line, line_start) = match self.line_starts.binary_search(&position) {
                Ok(line) | Err(line) => (line, self.line_starts.get(line)),
            };
            line_start.map_or(LineCol { line: 0, col: 0 }, |line_start| LineCol {
                line,
                col: position - line_start,
            })
        }
    }
}

impl From<&str> for Source {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

pub struct Position {
    line_col: LineCol,
}

impl Position {
    pub fn new(input: &str, pos: usize) -> Option<Position> {
        let line_col = Self::calc_line_col(input, pos);
        input.get(pos..).map(|_| Position { line_col })
    }

    pub fn line_col(&self) -> LineCol {
        self.line_col
    }

    pub fn calc_line_col(input: &str, pos: usize) -> LineCol {
        assert!(pos <= input.len(), "position out of bounds");

        // This is performance critical, so we use bytecount instead of a naive implementation.
        let newlines_up_to_pos = bytecount::count(&input.as_bytes()[..pos], b'\n');
        let line = newlines_up_to_pos + 1;

        // Find the last newline character before the position
        let last_newline_pos = match input[..pos].rfind('\n') {
            Some(pos) => pos + 1, // Start after the newline
            None => 0,            // If no newline, start is at the beginning
        };

        // Column number should start from 1, not 0
        let col = pos - last_newline_pos + 1;
        LineCol { line, col }
    }
}

/// Represents a span of the source code in a specific file.
#[derive(Clone, Ord, PartialOrd, Serialize, Deserialize)]
pub struct Span {
    // The original source code.
    src: Source,
    // The byte position in the string of the start of the span.
    start: usize,
    // The byte position in the string of the end of the span.
    end: usize,
    // A reference counted pointer to the file from which this span originated.
    source_id: Option<SourceId>,
}

impl Hash for Span {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.start.hash(state);
        self.end.hash(state);
        self.source_id.hash(state);
    }
}

impl PartialEq for Span {
    fn eq(&self, other: &Self) -> bool {
        self.start == other.start && self.end == other.end && self.source_id == other.source_id
    }
}

impl Eq for Span {}

impl From<Span> for std::ops::Range<usize> {
    fn from(value: Span) -> Self {
        Self {
            start: value.start,
            end: value.end,
        }
    }
}

impl Span {
    pub fn dummy() -> Span {
        DUMMY_SPAN.clone()
    }

    pub fn new(src: Source, start: usize, end: usize, source: Option<SourceId>) -> Option<Span> {
        let _ = src.text.get(start..end)?;
        Some(Span {
            src,
            start,
            end,
            source_id: source,
        })
    }

    /// Creates an empty [Span], means a span whose [Span::start] and [Span::end] are the same.
    /// The resulting empty [Span] will point to the start of the provided `span` and
    /// be in the same file.
    pub fn empty_at_start(span: &Span) -> Span {
        Span::new(
            span.src().clone(),
            span.start(),
            span.start(),
            span.source_id().copied(),
        )
        .expect("the existing `span` is a valid `Span`")
    }

    /// Creates an empty [Span], means a span whose [Span::start] and [Span::end] are the same.
    /// The resulting empty [Span] will point to the end of the provided `span` and
    /// be in the same file.
    pub fn empty_at_end(span: &Span) -> Span {
        Span::new(
            span.src().clone(),
            span.end(),
            span.end(),
            span.source_id().copied(),
        )
        .expect("the existing `span` is a valid `Span`")
    }

    pub fn from_string(source: String) -> Span {
        let len = source.len();
        Span::new(Source::new(&source), 0, len, None).unwrap()
    }

    pub fn src(&self) -> &Source {
        &self.src
    }

    pub fn source_id(&self) -> Option<&SourceId> {
        self.source_id.as_ref()
    }

    pub fn start(&self) -> usize {
        self.start
    }

    pub fn end(&self) -> usize {
        self.end
    }

    pub fn start_pos(&self) -> LineCol {
        self.src.line_col(self.start)
    }

    pub fn end_pos(&self) -> LineCol {
        self.src.line_col(self.end)
    }

    /// Returns an empty [Span] that points to the start of `self`.
    pub fn start_span(&self) -> Span {
        Self::empty_at_start(self)
    }

    /// Returns an empty [Span] that points to the end of `self`.
    pub fn end_span(&self) -> Span {
        Self::empty_at_end(self)
    }

    pub fn str(self) -> String {
        self.as_str().to_owned()
    }

    pub fn as_str(&self) -> &str {
        &self.src.text[self.start..self.end]
    }

    pub fn input(&self) -> &str {
        &self.src.text
    }

    pub fn trim(self) -> Span {
        let start_delta = self.as_str().len() - self.as_str().trim_start().len();
        let end_delta = self.as_str().len() - self.as_str().trim_end().len();
        Span {
            src: self.src,
            start: self.start + start_delta,
            end: self.end - end_delta,
            source_id: self.source_id,
        }
    }

    /// Creates a new span that points to very next char of the current span.
    ///
    /// ```ignore
    /// let
    ///    ^ <- span returned
    /// ^^^  <- original span
    /// ```
    pub fn next_char_utf8(&self) -> Option<Span> {
        let char = self.src.text[self.end..].chars().next()?;
        Some(Span {
            src: self.src.clone(),
            source_id: self.source_id,
            start: self.end,
            end: self.end + char.len_utf8(),
        })
    }

    /// This panics if the spans are not from the same file. This should
    /// only be used on spans that are actually next to each other.
    pub fn join(s1: Span, s2: &Span) -> Span {
        assert!(
            Arc::ptr_eq(&s1.src.text, &s2.src.text) && s1.source_id == s2.source_id,
            "Spans from different files cannot be joined.",
        );

        Span {
            src: s1.src,
            start: cmp::min(s1.start, s2.start),
            end: cmp::max(s1.end, s2.end),
            source_id: s1.source_id,
        }
    }

    pub fn join_all(spans: impl IntoIterator<Item = Span>) -> Span {
        spans
            .into_iter()
            .reduce(|s1: Span, s2: Span| Span::join(s1, &s2))
            .unwrap_or_else(Span::dummy)
    }

    /// Returns the line and column start and end.
    pub fn line_col(&self) -> LineColRange {
        LineColRange {
            start: self.start_pos(),
            end: self.end_pos(),
        }
    }

    pub fn is_dummy(&self) -> bool {
        self.eq(&DUMMY_SPAN)
    }

    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    /// Returns true if `self` contains `other`.
    pub fn contains(&self, other: &Span) -> bool {
        Arc::ptr_eq(&self.src.text, &other.src.text)
            && self.source_id == other.source_id
            && self.start <= other.start
            && self.end >= other.end
    }
}

impl fmt::Debug for Span {
    #[cfg(not(feature = "no-span-debug"))]
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("Span")
            .field("src (ptr)", &self.src.text.as_ptr())
            .field("source_id", &self.source_id)
            .field("start", &self.start)
            .field("end", &self.end)
            .field("as_str()", &self.as_str())
            .finish()
    }
    #[cfg(feature = "no-span-debug")]
    fn fmt(&self, _fmt: &mut fmt::Formatter) -> fmt::Result {
        Ok(())
    }
}

pub trait Spanned {
    fn span(&self) -> Span;
}

impl<T: Spanned> Spanned for Box<T> {
    fn span(&self) -> Span {
        (**self).span()
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct LineCol {
    pub line: usize,
    pub col: usize,
}

pub struct LineColRange {
    pub start: LineCol,
    pub end: LineCol,
}

impl Display for LineColRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("({}, {})", self.start, self.end))
    }
}

impl Display for LineCol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("line {}:{}", self.line, self.col))
    }
}
