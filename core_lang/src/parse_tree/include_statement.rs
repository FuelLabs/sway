use crate::error::*;
use pest::Span;
use pest::iterators::Pair;
use crate::Ident;
use crate::parser::Rule;

#[derive(Clone, Debug)]
pub struct IncludeStatement <'sc>{ 
    span: Span<'sc>
}

impl <'sc> IncludeStatement <'sc> {
    pub(crate) fn parse_from_pair(pair: Pair<'sc, Rule>) -> CompileResult<'sc, Self> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let mut iter = pair.into_inner();
        let _include_keyword = iter.next();
        let path_to_file_raw = iter.collect::<Vec<_>>();
        let mut path_to_file: Vec<Ident<'sc>> = Vec::with_capacity(path_to_file_raw.len());
        for item in path_to_file_raw {
            let ident = eval!(Ident::parse_from_pair, warnings, errors, item, continue);
            path_to_file.push(ident);
        }
        todo!()
    }
}