use crate::span::Span;

/// This panics if the spans are not from the same file. This should
/// only be used on spans that are actually next to each other.
pub fn join_spans<'sc>(s1: Span<'sc>, s2: Span<'sc>) -> Span<'sc> {
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
