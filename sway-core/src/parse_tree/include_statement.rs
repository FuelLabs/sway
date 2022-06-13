use sway_types::{ident::Ident, span::Span};

#[derive(Clone, Debug)]
pub struct IncludeStatement {
    pub(crate) _alias: Option<Ident>,
    #[allow(dead_code)]
    // this span may be used for errors in the future, although it is not right now.
    pub(crate) span: Span,
    pub(crate) _path_span: Span,
}
