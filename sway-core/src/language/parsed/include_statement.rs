use sway_types::span::Span;

#[derive(Clone, Debug)]
pub struct IncludeStatement {
    // this span may be used for errors in the future, although it is not right now.
    pub(crate) _span: Span,
    pub(crate) _mod_name_span: Span,
}
