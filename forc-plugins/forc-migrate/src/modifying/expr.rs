use sway_ast::{
    keywords::{DotToken, Token},
    Expr, Parens, PathExprSegment, Punctuated,
};
use sway_types::{Ident, Span};

use crate::assert_insert_span;

use super::New;

impl New {
    /// Creates an [Expr] representing a call to a non-generic method with the name `method_name`.
    /// The method does not accepts any arguments.
    pub(crate) fn method_call<S: AsRef<str> + ?Sized>(
        insert_span: Span,
        target: Expr,
        method_name: &S,
    ) -> Expr {
        assert_insert_span!(insert_span);

        Expr::MethodCall {
            target: Box::new(target),
            dot_token: DotToken::new(insert_span.clone()),
            path_seg: PathExprSegment {
                name: Ident::new_with_override(method_name.as_ref().into(), insert_span.clone()),
                generics_opt: None,
            },
            contract_args_opt: None,
            args: Parens {
                inner: Punctuated {
                    value_separator_pairs: vec![],
                    final_value_opt: None,
                },
                span: insert_span,
            },
        }
    }
}
