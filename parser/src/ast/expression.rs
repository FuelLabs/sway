use crate::ast::Literal;

pub(crate) enum Expression<'sc> {
    Literal(Literal<'sc>),
}
