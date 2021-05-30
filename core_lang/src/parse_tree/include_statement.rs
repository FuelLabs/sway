use crate::error::*;
use pest::Span;
use pest::iterators::Pair;
use crate::parser::Rule;

#[derive(Clone, Debug)]
pub struct IncludeStatement <'sc>{ 
    span: Span<'sc>
}

impl <'sc> IncludeStatement <'sc> {
    pub(crate) fn parse_from_pair(pair: Pair<'sc, Rule>) -> CompileResult<'sc, Self> {
        todo!()
    }
}