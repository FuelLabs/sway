use crate::parser::Rule;
use crate::{
    error::{ok, CompileResult},
    CodeBlock, Expression,
};
use pest::iterators::Pair;

#[derive(Debug, Clone)]
pub(crate) struct WhileLoop<'sc> {
    pub(crate) condition: Expression<'sc>,
    pub(crate) body: CodeBlock<'sc>,
}

impl<'sc> WhileLoop<'sc> {
    pub(crate) fn parse_from_pair(pair: Pair<'sc, Rule>) -> CompileResult<'sc, Self> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let mut iter = pair.into_inner();
        let _while_keyword = iter.next().unwrap();
        let condition = iter.next().unwrap();
        let body = iter.next().unwrap();

        let condition = eval!(
            Expression::parse_from_pair,
            warnings,
            errors,
            condition,
            Expression::Unit {
                span: condition.as_span()
            }
        );

        let body = eval!(
            CodeBlock::parse_from_pair,
            warnings,
            errors,
            body,
            CodeBlock {
                contents: Default::default(),
                scope: Default::default()
            }
        );
        ok(WhileLoop { condition, body }, warnings, errors)
    }
}
