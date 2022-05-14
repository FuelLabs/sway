use crate::{build_config::BuildConfig, error::*, parse_tree::ident};

use sway_types::{ident::Ident, span::Span};

#[derive(Clone, Debug)]
pub struct IncludeStatement {
    pub(crate) alias: Option<Ident>,
    #[allow(dead_code)]
    // this span may be used for errors in the future, although it is not right now.
    pub(crate) span: Span,
    pub(crate) path_span: Span,
}
