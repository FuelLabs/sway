use crate::build_config::BuildConfig;
use crate::error::*;
use crate::parse_tree::{CallPath, Expression};
use crate::parser::Rule;
use crate::span::Span;
use crate::Ident;
use pest::iterators::Pair;

#[derive(Clone, Debug)]
pub enum UnaryOp {
    Not,
    Ref,
    Deref,
}

impl UnaryOp {
    pub fn parse_from_pair<'sc>(
        pair: Pair<'sc, Rule>,
        config: Option<&BuildConfig>,
    ) -> CompileResult<'sc, Self> {
        use UnaryOp::*;
        match pair.as_str() {
            "!" => ok(Not, Vec::new(), Vec::new()),
            "ref" => ok(Ref, Vec::new(), Vec::new()),
            "deref" => ok(Deref, Vec::new(), Vec::new()),
            _ => {
                let errors = vec![CompileError::Internal(
                    "Attempted to parse unary op from invalid op string.",
                    Span {
                        span: pair.as_span(),
                        path: config.map(|c| c.dir_of_code.clone()),
                    },
                )];
                return err(Vec::new(), errors);
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

    pub fn to_fn_application<'sc>(
        &self,
        arg: Expression<'sc>,
        span: Span<'sc>,
        op_span: Span<'sc>,
    ) -> Expression<'sc> {
        Expression::FunctionApplication {
            name: CallPath {
                prefixes: vec![
                    Ident {
                        primary_name: "std".into(),
                        span: op_span.clone(),
                    },
                    Ident {
                        primary_name: "std".into(),
                        span: op_span.clone(),
                    },
                    Ident {
                        primary_name: "ops".into(),
                        span: op_span.clone(),
                    },
                ],
                suffix: Ident {
                    primary_name: self.to_var_name(),
                    span: op_span,
                },
            },
            arguments: vec![arg],
            span,
        }
    }
}
