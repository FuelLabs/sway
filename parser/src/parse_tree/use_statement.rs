use crate::error::CompileError;
use crate::error::*;
use crate::Rule;
use pest::iterators::Pair;

#[derive(Debug, Clone)]
pub(crate) struct UseStatement<'sc> {
    root: &'sc str,
    path: Vec<&'sc str>,
}

impl<'sc> UseStatement<'sc> {
    pub(crate) fn parse_from_pair(pair: Pair<'sc, Rule>) -> CompileResult<'sc, Self> {
        let mut stmt = pair.into_inner();
        let _use_keyword = stmt.next();
        let import_path = stmt.next().unwrap();
        let mut path_iter = import_path
            .into_inner()
            .filter(|x| x.as_rule() != Rule::path_separator)
            .map(|x| x.as_str());
        let root = path_iter.next().unwrap();
        let path = path_iter.collect();
        ok(UseStatement { root, path }, Vec::new(), Vec::new())
    }
}
