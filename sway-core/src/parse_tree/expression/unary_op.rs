use crate::{
    parse_tree::{CallPath, Expression},
    Ident,
};

use sway_types::span::Span;

#[derive(Clone, Debug)]
pub enum UnaryOp {
    Not,
    Ref,
    Deref,
}

impl UnaryOp {
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
