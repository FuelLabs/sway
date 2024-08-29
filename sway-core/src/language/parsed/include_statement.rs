use sway_types::{span::Span, Ident};

use crate::language::Visibility;

#[derive(Clone, Debug, PartialEq)]
pub struct IncludeStatement {
    // this span may be used for errors in the future, although it is not right now.
    pub span: Span,
    pub mod_name: Ident,
    pub visibility: Visibility,
}
