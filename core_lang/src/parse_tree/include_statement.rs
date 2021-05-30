use crate::error::*;
use pest::Span;
use pest::iterators::Pair;
use crate::Ident;
use crate::parser::Rule;

#[derive(Clone, Debug)]
pub struct IncludeStatement <'sc> { 
    file_path: Vec<Ident<'sc>>,
    alias: Option<Ident<'sc>>,
    span: Span<'sc>
}

impl <'sc> IncludeStatement <'sc> {
    pub(crate) fn parse_from_pair(pair: Pair<'sc, Rule>) -> CompileResult<'sc, Self> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let span = pair.as_span();
        let mut iter = pair.into_inner();
        let _include_keyword = iter.next();
        let path_to_file_raw = iter.collect::<Vec<_>>();
        let mut alias = None;
        let mut file_path: Vec<Ident<'sc>> = Vec::with_capacity(path_to_file_raw.len());
        for item in path_to_file_raw {
            if item.as_rule() == Rule::ident {
                let ident = eval!(Ident::parse_from_pair, warnings, errors, item, continue);
                file_path.push(ident);
            } else if item.as_rule() == Rule::alias {
                let alias_parsed = eval!(Ident::parse_from_pair, warnings, errors, item.into_inner().next().unwrap(), continue);
                alias = Some(alias_parsed);
            }
        }
        ok(IncludeStatement { span, file_path, alias }, warnings, errors)
    }
}