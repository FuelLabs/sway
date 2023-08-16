use serde::Serialize;

use crate::SourceId;

use {
    lazy_static::lazy_static,
    std::{cmp, fmt, hash::Hash, sync::Arc},
};

lazy_static! {
    static ref DUMMY_SPAN: Span = Span::new(Arc::from(""), 0, 0, None).unwrap();
}

pub struct Position {
    input: Arc<str>,
    pos: usize,
}

impl Position {
    pub fn new(input: Arc<str>, pos: usize) -> Option<Position> {
        input.clone().get(pos..).map(|_| Position { input, pos })
    }
    #[inline]
    pub fn line_col(&self) -> (usize, usize) {
        if self.pos > self.input.len() {
            panic!("position out of bounds");
        }

        let mut pos = self.pos;
        // Position's pos is always a UTF-8 border.
        let slice = &self.input[..pos];
        let mut chars = slice.chars().peekable();

        let mut line_col = (1, 1);

        while pos != 0 {
            match chars.next() {
                Some('\r') => {
                    if let Some(&'\n') = chars.peek() {
                        chars.next();

                        if pos == 1 {
                            pos -= 1;
                        } else {
                            pos -= 2;
                        }

                        line_col = (line_col.0 + 1, 1);
                    } else {
                        pos -= 1;
                        line_col = (line_col.0, line_col.1 + 1);
                    }
                }
                Some('\n') => {
                    pos -= 1;
                    line_col = (line_col.0 + 1, 1);
                }
                Some(c) => {
                    pos -= c.len_utf8();
                    line_col = (line_col.0, line_col.1 + 1);
                }
                None => unreachable!(),
            }
        }

        line_col
    }
}

/// Represents a span of the source code in a specific file.
#[derive(Clone, Ord, PartialOrd)]
pub struct Span {
    // The original source code.
    src: Arc<str>,
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

impl Serialize for Span {
    // Serialize a tuple two fields: `start` and `end`.
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeTuple;

        let mut state = serializer.serialize_tuple(2)?;
        state.serialize_element(&self.start)?;
        state.serialize_element(&self.end)?;
        state.end()
    }
}

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

    pub fn new(src: Arc<str>, start: usize, end: usize, source: Option<SourceId>) -> Option<Span> {
        let _ = src.get(start..end)?;
        Some(Span {
            src,
            start,
            end,
            source_id: source,
        })
    }

    pub fn from_string(source: String) -> Span {
        let len = source.len();
        Span::new(Arc::from(source), 0, len, None).unwrap()
    }

    pub fn src(&self) -> &Arc<str> {
        &self.src
    }

    pub fn start(&self) -> usize {
        self.start
    }

    pub fn end(&self) -> usize {
        self.end
    }

    pub fn source_id(&self) -> Option<&SourceId> {
        self.source_id.as_ref()
    }

    pub fn start_pos(&self) -> Position {
        Position::new(self.src.clone(), self.start).unwrap()
    }

    pub fn end_pos(&self) -> Position {
        Position::new(self.src.clone(), self.end).unwrap()
    }

    pub fn split(&self) -> (Position, Position) {
        let start = self.start_pos();
        let end = self.end_pos();
        (start, end)
    }

    pub fn str(self) -> String {
        self.as_str().to_owned()
    }

    pub fn as_str(&self) -> &str {
        &self.src[self.start..self.end]
    }

    pub fn input(&self) -> &str {
        &self.src
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
        let char = self.src[self.end..].chars().next()?;
        Some(Span {
            src: self.src.clone(),
            source_id: self.source_id,
            start: self.end,
            end: self.end + char.len_utf8(),
        })
    }

    /// This panics if the spans are not from the same file. This should
    /// only be used on spans that are actually next to each other.
    pub fn join(s1: Span, s2: Span) -> Span {
        assert!(
            Arc::ptr_eq(&s1.src, &s2.src) && s1.source_id == s2.source_id,
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
            .reduce(Span::join)
            .unwrap_or_else(Span::dummy)
    }

    /// Returns the line and column start and end.
    pub fn line_col(&self) -> (LineCol, LineCol) {
        (
            self.start_pos().line_col().into(),
            self.end_pos().line_col().into(),
        )
    }
}

impl fmt::Debug for Span {
    #[cfg(not(feature = "no-span-debug"))]
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("Span")
            .field("src (ptr)", &self.src.as_ptr())
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

#[derive(Clone, Copy, Debug)]
pub struct LineCol {
    pub line: usize,
    pub col: usize,
}

impl From<(usize, usize)> for LineCol {
    fn from(o: (usize, usize)) -> Self {
        LineCol {
            line: o.0,
            col: o.1,
        }
    }
}
