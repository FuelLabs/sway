use crate::{
    build_config::BuildConfig,
    error::*,
    parse_tree::{CallPath, Expression},
    parser::Rule,
    Ident,
};

use indexmap::IndexSet;
use sway_types::span::Span;

use pest::iterators::Pair;

#[derive(Clone, Debug)]
pub enum UnaryOp {
    Not,
    Ref,
    Deref,
}

impl UnaryOp {
    pub fn parse_from_pair(pair: Pair<Rule>, config: Option<&BuildConfig>) -> CompileResult<Self> {
        use UnaryOp::*;
        match pair.as_str() {
            "!" => ok(Not, IndexSet::new(), IndexSet::new()),
            "ref" => ok(Ref, IndexSet::new(), IndexSet::new()),
            "deref" => ok(Deref, IndexSet::new(), IndexSet::new()),
            _ => {
                let errors = IndexSet::from([CompileError::Internal(
                    "Attempted to parse unary op from invalid op string.",
                    Span {
                        span: pair.as_span(),
                        path: config.map(|c| c.path()),
                    },
                )]);
                err(IndexSet::new(), errors)
            }
        }
    }

    fn to_var_name(&self) -> &'static str {
        use UnaryOp::*;
        match self {
            Ref => "ref",
            Deref => "deref",
            Not => "not",
        }
    }

    pub fn to_fn_application(&self, arg: Expression, span: Span, op_span: Span) -> Expression {
        Expression::FunctionApplication {
            type_arguments: vec![],
            name: CallPath {
                prefixes: vec![
                    Ident::new_with_override("core", op_span.clone()),
                    Ident::new_with_override("ops", op_span.clone()),
                ],
                suffix: Ident::new_with_override(self.to_var_name(), op_span),
                is_absolute: false,
            },
            arguments: vec![arg],
            span,
        }
    }
}
