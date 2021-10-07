use crate::build_config::BuildConfig;
use crate::parser::Rule;
use crate::span::Span;
use crate::{
    error::{ok, CompileResult},
    CodeBlock, Expression,
};
use pest::iterators::Pair;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct IfStatement<'sc> {
    pub(crate) condition: Expression<'sc>,
    pub(crate) then: CodeBlock<'sc>,
    pub(crate) r#else: Option<CodeBlock<'sc>>
}

impl<'sc> IfStatement<'sc> {
    pub(crate) fn parse_from_pair(
        pair: Pair<'sc, Rule>,
        config: Option<&BuildConfig>,
        docstrings: &mut HashMap<String, String>,
    ) -> CompileResult<'sc, Self> {
        let path = config.map(|c| c.path());
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let mut iter = pair.into_inner();
        let _if_keyword = iter.next().unwrap();
        let condition = iter.next().unwrap();
        let then = iter.next().unwrap();
        let r#else = iter.next();
        let then_span = Span {
            span: then.as_span(),
            path: path.clone(),
        };

        let condition = check!(
            Expression::parse_from_pair(condition.clone(), config, docstrings),
            Expression::Unit {
                span: Span {
                    span: condition.as_span(),
                    path: path.clone()
                }
            },
            warnings,
            errors
        );

        let then = check!(
            CodeBlock::parse_from_pair(then, config, docstrings),
            CodeBlock {
                contents: Default::default(),
                whole_block_span: then_span,
                scope: Default::default()
            },
            warnings,
            errors
        );

        let r#else = if let Some(r#else) = r#else {
            Some(check!(
                CodeBlock::parse_from_pair(r#else.clone(), config, docstrings),
                CodeBlock {
                    contents: Default::default(),
                    whole_block_span: Span {
                        span: r#else.as_span(),
                        path: path.clone()
                    },
                    scope: Default::default()
                },
                warnings,
                errors
            ))
        } else { None };

        ok(IfStatement { condition, then, r#else }, warnings, errors)
    }
}
