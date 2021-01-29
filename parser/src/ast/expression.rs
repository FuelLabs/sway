use crate::ast::Literal;

#[derive(Debug)]
pub(crate) enum Expression<'sc> {
    Literal(Literal<'sc>),
}
