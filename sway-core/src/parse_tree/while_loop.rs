use crate::build_config::BuildConfig;
use crate::parser::Rule;
use crate::span::Span;
use crate::{
    error::{ok, CompileResult},
    CodeBlock, Expression,
};
use pest::iterators::Pair;

/// A parsed while loop. Contains the `condition`, which is defined from an [Expression], and the `body` from a [CodeBlock].
#[derive(Debug, Clone)]
pub struct WhileLoop {
    pub(crate) condition: Expression,
    pub(crate) body: CodeBlock,
}

impl WhileLoop {
    pub(crate) fn parse_from_pair(
        pair: Pair<Rule>,
        config: Option<&BuildConfig>,
    ) -> CompileResult<Self> {
        let path = config.map(|c| c.path());
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let mut iter = pair.into_inner();
        let _while_keyword = iter.next().unwrap();
        let condition = iter.next().unwrap();
        let body = iter.next().unwrap();
        let whole_block_span = Span {
            span: body.as_span(),
            path: path.clone(),
        };

        let condition = check!(
            Expression::parse_from_pair(condition.clone(), config),
            Expression::Tuple {
                fields: vec![],
                span: Span {
                    span: condition.as_span(),
                    path,
                }
            },
            warnings,
            errors
        );

        let body = check!(
            CodeBlock::parse_from_pair(body, config),
            CodeBlock {
                contents: Default::default(),
                whole_block_span,
            },
            warnings,
            errors
        );

        ok(WhileLoop { condition, body }, warnings, errors)
    }
}
