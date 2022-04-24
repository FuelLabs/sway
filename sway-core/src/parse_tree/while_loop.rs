use crate::{
    build_config::BuildConfig,
    error::{ok, CompileResult, ParserLifter},
    error_recovery_exp,
    parser::Rule,
    CodeBlock, Expression,
};

use sway_types::span::Span;

use pest::iterators::Pair;

/// A parsed while loop. Contains the `condition`, which is defined from an [Expression], and the `body` from a [CodeBlock].
#[derive(Debug, Clone)]
pub struct WhileLoop {
    pub condition: Expression,
    pub body: CodeBlock,
}

impl WhileLoop {
    pub(crate) fn parse_from_pair(
        pair: Pair<Rule>,
        config: Option<&BuildConfig>,
    ) -> CompileResult<ParserLifter<Self>> {
        let path = config.map(|c| c.path());
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let mut iter = pair.into_inner();
        let _while_keyword = iter.next().unwrap();
        let condition = iter.next().unwrap();
        let body = iter.next().unwrap();
        let whole_block_span = Span::from_pest(body.as_span(), path.clone());

        let condition_result = check!(
            Expression::parse_from_pair(condition.clone(), config),
            ParserLifter::empty(error_recovery_exp(Span::from_pest(
                condition.as_span(),
                path
            ))),
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
        let while_loop = WhileLoop {
            condition: condition_result.value,
            body,
        };

        ok(
            ParserLifter {
                var_decls: condition_result.var_decls,
                value: while_loop,
            },
            warnings,
            errors,
        )
    }
}
