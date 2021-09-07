use crate::build_config::BuildConfig;
use crate::parser::Rule;
use crate::span::Span;
use crate::{
    error::{ok, CompileResult},
    CodeBlock, Expression,
};
use pest::iterators::Pair;

#[derive(Debug, Clone)]
pub struct WhileLoop<'sc> {
    pub(crate) condition: Expression<'sc>,
    pub(crate) body: CodeBlock<'sc>,
}

impl<'sc> WhileLoop<'sc> {
    pub(crate) fn parse_from_pair(
        pair: Pair<'sc, Rule>,
        config: Option<BuildConfig>,
    ) -> CompileResult<'sc, Self> {
        let path = config.clone().map(|c| c.dir_of_code);
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let mut iter = pair.into_inner();
        let _while_keyword = iter.next().unwrap();
        let condition = iter.next().unwrap();
        let body = iter.next().unwrap();
        let whole_block_span = Span {
            span: body.as_span(),
            path,
        };

        let condition = eval2!(
            Expression::parse_from_pair,
            warnings,
            errors,
            condition,
            config,
            Expression::Unit {
                span: Span {
                    span: condition.as_span(),
                    path
                }
            }
        );

        let body = eval2!(
            CodeBlock::parse_from_pair,
            warnings,
            errors,
            body,
            config,
            CodeBlock {
                contents: Default::default(),
                whole_block_span,
                scope: Default::default()
            }
        );

        ok(WhileLoop { condition, body }, warnings, errors)
    }
}
