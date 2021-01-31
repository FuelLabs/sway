use crate::ast::Literal;

#[derive(Debug)]
pub(crate) enum Expression<'sc> {
    Literal(Literal<'sc>),
    FunctionApplication {
        name: &'sc str,
        arguments: Vec<Expression<'sc>>,
    },
    VariableExpression {
        name: &'sc str,
    },
}
