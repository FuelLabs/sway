use sway_ast::{
    literal::{LitBool, LitBoolType},
    Literal,
};
use sway_types::Span;

use crate::assert_insert_span;

use super::New;

impl New {
    /// Creates a [Literal] representing bool `value`.
    pub(crate) fn literal_bool(insert_span: Span, value: bool) -> Literal {
        assert_insert_span!(insert_span);

        Literal::Bool(LitBool {
            span: insert_span,
            kind: if value {
                LitBoolType::True
            } else {
                LitBoolType::False
            },
        })
    }
}
