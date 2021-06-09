use crate::error::*;
use crate::parser::Rule;
use crate::Ident;
use pest::iterators::Pair;
use pest::Span;

#[derive(Clone, Debug)]
pub struct IncludeStatement<'sc> {
    pub(crate) file_path: &'sc str,
    pub(crate) alias: Option<Ident<'sc>>,
    span: Span<'sc>,
    pub(crate) path_span: Span<'sc>,
}

impl<'sc> IncludeStatement<'sc> {
    pub(crate) fn parse_from_pair(pair: Pair<'sc, Rule>) -> CompileResult<'sc, Self> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let span = pair.as_span();
        let mut iter = pair.into_inner();
        let _include_keyword = iter.next();
        let path_to_file_raw = iter.collect::<Vec<_>>();
        let mut alias = None;
        let mut file_path = None;
        let mut path_span = None;

        for item in path_to_file_raw {
            if item.as_rule() == Rule::file_path {
                file_path = Some(item.as_str().trim());
                path_span = Some(item.as_span());
            } else if item.as_rule() == Rule::alias {
                let alias_parsed = eval!(
                    Ident::parse_from_pair,
                    warnings,
                    errors,
                    item.into_inner().next().unwrap(),
                    continue
                );
                alias = Some(alias_parsed);
            }
        }

        let file_path = file_path.expect("guaranteed to exist by grammar");
        let path_span = path_span.expect("guaranteed to exist by grammar");
        ok(
            IncludeStatement {
                span,
                file_path,
                alias,
                path_span,
            },
            warnings,
            errors,
        )
    }
}
