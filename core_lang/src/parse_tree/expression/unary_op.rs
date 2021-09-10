use crate::build_config::BuildConfig;
use crate::error::*;
use crate::parser::Rule;
use crate::span;
use pest::iterators::Pair;

#[derive(Clone, Debug)]
pub enum UnaryOp {
    Not,
    Ref,
    Deref,
}

impl UnaryOp {
    pub fn parse_from_pair<'sc>(
        pair: Pair<'sc, Rule>,
        config: Option<BuildConfig>,
    ) -> CompileResult<'sc, Option<Self>> {
        use UnaryOp::*;
        match pair.as_str() {
            "!" => ok(Some(Not), Vec::new(), Vec::new()),
            "ref" => ok(Some(Ref), Vec::new(), Vec::new()),
            "deref" => ok(Some(Deref), Vec::new(), Vec::new()),
            _ => {
                let errors = vec![CompileError::Internal(
                    "Attempted to parse unary op from invalid op string.",
                    span::Span {
                        span: pair.as_span(),
                        path: config.map(|c| c.dir_of_code),
                    },
                )];
                return err(Vec::new(), errors);
            }
        }
    }
}
