use pest::Span;

pub(crate) fn join_spans<'sc>(s1: Span<'sc>, s2: Span<'sc>) -> Span<'sc> {
    let s1_positions = s1.split();
    let s2_positions = s2.split();
    if s1_positions.0 < s2_positions.1 {
        s1_positions.0.span(&s2_positions.1)
    } else {
        s2_positions.0.span(&s1_positions.1)
    }
}
