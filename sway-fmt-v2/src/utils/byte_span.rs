use std::cmp::Ordering;

use sway_types::Span;

/// A stripped down version of sway-types::src::Span
#[derive(PartialEq, Eq, Debug, Clone, Default)]
pub struct ByteSpan {
    // The byte position in the string of the start of the span.
    pub start: usize,
    // The byte position in the string of the end of the span.
    pub end: usize,
}

impl From<Span> for ByteSpan {
    /// Takes `start` and `end` from `sway::types::Span` and constructs a `ByteSpan`
    fn from(span: Span) -> Self {
        ByteSpan {
            start: span.start(),
            end: span.end(),
        }
    }
}

impl ByteSpan {
    pub fn len(&self) -> usize {
        self.end - self.start
    }
}

impl Ord for ByteSpan {
    fn cmp(&self, other: &Self) -> Ordering {
        // If the starting position is the same encapsulatig span (i.e, wider one) should come
        // first
        match self.start.cmp(&other.start) {
            Ordering::Equal => other.end.cmp(&self.end),
            ord => ord,
        }
    }
}

impl PartialOrd for ByteSpan {
    fn partial_cmp(&self, other: &ByteSpan) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// While searching for a structure we need the possible places that structure can be placed inside the code
/// `leaf_spans` collects all field spans so that we can check in between them.
pub trait LeafSpans {
    fn leaf_spans(&self) -> Vec<ByteSpan>;
}

#[cfg(test)]
mod tests {
    use super::ByteSpan;
    #[test]
    fn test_byte_span_ordering() {
        let first_span = ByteSpan { start: 2, end: 6 };
        let second_span = ByteSpan { start: 2, end: 4 };
        let third_span = ByteSpan { start: 4, end: 7 };

        let mut vec = vec![second_span.clone(), third_span.clone(), first_span.clone()];
        vec.sort();

        assert_eq!(vec[0], first_span);
        assert_eq!(vec[1], second_span);
        assert_eq!(vec[2], third_span);
    }
}
