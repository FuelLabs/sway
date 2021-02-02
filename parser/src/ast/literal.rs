use crate::parser::Rule;
use crate::CompileError;
use pest::iterators::Pair;

#[derive(Debug)]
pub(crate) enum Literal<'sc> {
    Integer(i64),
    String(&'sc str),
    Boolean(bool),
    Byte(u8),
}

impl<'sc> Literal<'sc> {
    pub(crate) fn parse_from_pair(lit: Pair<'sc, Rule>) -> Result<Self, CompileError<'sc>> {
        let lit_inner = lit.into_inner().next().unwrap();
        let parsed = match lit_inner.as_rule() {
            Rule::integer => Literal::Integer(lit_inner.as_str().parse().map_err(|e| {
                CompileError::Internal(
                    "Called incorrect internal parser on literal type.",
                    lit_inner.into_span(),
                )
            })?),
            Rule::string => {
                // remove opening and closing quotes
                let lit_str = lit_inner.as_str();
                Literal::String(&lit_str[1..lit_str.len() - 1])
            }
            a => {
                eprintln!(
                    "not yet able to parse literal rule {:?} ({:?})",
                    a,
                    lit_inner.as_str()
                );
                return Err(CompileError::Unimplemented(a, lit_inner.as_span()));
            }
        };

        Ok(parsed)
    }
}
