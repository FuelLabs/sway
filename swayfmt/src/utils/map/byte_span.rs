use std::cmp::Ordering;
use sway_ast::{
    attribute::Annotated,
    brackets::{Parens, SquareBrackets},
    keywords::{
        AddToken, ColonToken, CommaToken, ForToken, ForwardSlashToken, RightArrowToken,
        SemicolonToken,
    },
    Braces, TypeField,
};
use sway_parse::Parse;
use sway_types::{Ident, Span, Spanned};

/// This represents the beginning of the file and if during searching we found STARTING_BYTE_SPAN, a custom logic is needed.
/// Because if there are comments in between at the beginning before program kind, we will be searching between {start: 0, end:0} to {start:0, end:x}.
/// Searching in that range would never return a comment since the way we order ByteSpans ensures that encapsulating spans are always coming later than the smaller ones.
pub(crate) const STARTING_BYTE_SPAN: ByteSpan = ByteSpan { start: 0, end: 0 };

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

impl<T> LeafSpans for Braces<T>
where
    T: LeafSpans + Clone,
{
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = Vec::new();
        let mut opening_brace_span = ByteSpan::from(self.span());
        opening_brace_span.end = opening_brace_span.start + 1;
        // Add opening brace's ByteSpan
        collected_spans.push(opening_brace_span);
        // Add T's collected ByteSpan
        collected_spans.append(&mut self.clone().into_inner().leaf_spans());
        let mut closing_brace_span = ByteSpan::from(self.span());
        closing_brace_span.start = closing_brace_span.end - 1;
        // Add closing brace's ByteSpan
        collected_spans.push(closing_brace_span);
        collected_spans
    }
}

impl<T> LeafSpans for Parens<T>
where
    T: LeafSpans + Clone,
{
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = Vec::new();
        let mut opening_paren_span = ByteSpan::from(self.span());
        opening_paren_span.end = opening_paren_span.start + 1;
        // Add opening paren's span
        collected_spans.push(opening_paren_span);
        // Add T's collected ByteSpan
        collected_spans.append(&mut self.clone().into_inner().leaf_spans());
        let mut closing_paren_span = ByteSpan::from(self.span());
        closing_paren_span.start = closing_paren_span.end - 1;
        // Add closing paren's ByteSpan
        collected_spans.push(closing_paren_span);
        collected_spans
    }
}

impl<T> LeafSpans for SquareBrackets<T>
where
    T: LeafSpans + Clone,
{
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = Vec::new();
        let mut opening_bracket_span = ByteSpan::from(self.span());
        opening_bracket_span.end = opening_bracket_span.start + 1;
        // Add opening bracket's span
        collected_spans.push(opening_bracket_span);
        // Add T's collected ByteSpan
        collected_spans.append(&mut self.clone().into_inner().leaf_spans());
        let mut closing_bracket_span = ByteSpan::from(self.span());
        closing_bracket_span.start = closing_bracket_span.end - 1;
        // Add closing bracket's ByteSpan
        collected_spans.push(closing_bracket_span);
        collected_spans
    }
}

impl<T, P> LeafSpans for (T, P)
where
    T: LeafSpans,
    P: LeafSpans,
{
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = self.0.leaf_spans();
        collected_spans.append(&mut self.1.leaf_spans());
        collected_spans
    }
}
impl<T> LeafSpans for Vec<T>
where
    T: LeafSpans,
{
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = Vec::new();
        for t in self {
            collected_spans.append(&mut t.leaf_spans());
        }
        collected_spans
    }
}
impl<T> LeafSpans for Annotated<T>
where
    T: LeafSpans + Parse,
{
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = self.attribute_list.leaf_spans();
        collected_spans.append(&mut self.value.leaf_spans());
        collected_spans
    }
}

impl LeafSpans for Ident {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        vec![ByteSpan::from(self.span())]
    }
}
impl LeafSpans for CommaToken {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        vec![(ByteSpan::from(self.span()))]
    }
}

impl LeafSpans for TypeField {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = vec![ByteSpan::from(self.name.span())];
        collected_spans.push(ByteSpan::from(self.colon_token.span()));
        collected_spans.append(&mut self.ty.leaf_spans());
        collected_spans
    }
}

impl LeafSpans for AddToken {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        vec![ByteSpan::from(self.span())]
    }
}

impl LeafSpans for SemicolonToken {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        vec![ByteSpan::from(self.span())]
    }
}

impl LeafSpans for ColonToken {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        vec![ByteSpan::from(self.span())]
    }
}

impl LeafSpans for RightArrowToken {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        vec![ByteSpan::from(self.span())]
    }
}

impl LeafSpans for ForToken {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        vec![ByteSpan::from(self.span())]
    }
}

impl LeafSpans for ForwardSlashToken {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        vec![ByteSpan::from(self.span())]
    }
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
