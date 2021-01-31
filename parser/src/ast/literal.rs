use crate::parser::Rule;
use crate::CompileError;
use pest::iterators::Pair;

#[derive(Debug)]
pub(crate) enum Literal<'sc> {
    Integer(i64),
    String(&'sc str),
    Boolean(bool),
}

impl Literal<'_> {
    pub(crate) fn parse_from_pair<'sc>(lit: Pair<'sc, Rule>) -> Result<Self, CompileError<'sc>> {
        let lit_inner = lit.into_inner().next().unwrap();
        let parsed = match lit_inner.as_rule() {
            Rule::integer => Literal::Integer(lit_inner.as_str().parse().map_err(|e| {
                CompileError::Internal(
                    "Called incorrect internal parser on literal type.",
                    lit_inner.into_span(),
                )
            })?),
            _ => todo!(),
        };

        Ok(parsed)
    }
}
