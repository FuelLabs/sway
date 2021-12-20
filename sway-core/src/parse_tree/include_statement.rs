use crate::build_config::BuildConfig;
use crate::error::*;
use crate::parser::Rule;
use crate::span::Span;
use crate::Ident;
use pest::iterators::Pair;

#[derive(Clone, Debug)]
pub struct IncludeStatement {
    pub(crate) alias: Option<Ident>,
    #[allow(dead_code)]
    // this span may be used for errors in the future, although it is not right now.
    span: Span,
    pub(crate) path_span: Span,
}

impl IncludeStatement {
    pub(crate) fn parse_from_pair(
        pair: Pair<Rule>,
        config: Option<&BuildConfig>,
    ) -> CompileResult<Self> {
        let path = config.map(|c| c.path());
        let mut warnings = vec![];
        let mut errors = vec![];
        let span = Span {
            span: pair.as_span(),
            path: path.clone(),
        };
        let mut iter = pair.into_inner();
        let _include_keyword = iter.next();
        let path_to_file_raw = iter.collect::<Vec<_>>();
        let mut alias = None;
        let mut path_span = None;

        for item in path_to_file_raw {
            if item.as_rule() == Rule::file_path {
                path_span = Some(Span {
                    span: item.as_span(),
                    path: path.clone(),
                }.trim());
            } else if item.as_rule() == Rule::alias {
                let alias_parsed = check!(
                    Ident::parse_from_pair(item.into_inner().next().unwrap(), config),
                    continue,
                    warnings,
                    errors
                );
                alias = Some(alias_parsed);
            }
        }

        let path_span = path_span.expect("guaranteed to exist by grammar");
        ok(
            IncludeStatement {
                span,
                alias,
                path_span,
            },
            warnings,
            errors,
        )
    }
}
