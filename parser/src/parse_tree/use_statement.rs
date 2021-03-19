use crate::error::*;
use crate::Ident;
use crate::Rule;
use pest::iterators::Pair;

use super::CallPath;

#[derive(Debug, Clone)]
pub(crate) enum ImportType<'sc> {
    Star,
    Item(Ident<'sc>),
}

#[derive(Debug, Clone)]
pub(crate) struct UseStatement<'sc> {
    pub(crate) call_path: Vec<Ident<'sc>>,
    pub(crate) import_type: ImportType<'sc>,
}

impl<'sc> UseStatement<'sc> {
    pub(crate) fn parse_from_pair(pair: Pair<'sc, Rule>) -> CompileResult<'sc, Self> {
        let mut errors = vec![];
        let mut warnings = vec![];
        let mut stmt = pair.into_inner();
        let _use_keyword = stmt.next();
        let import_path = stmt.next().unwrap();
        let mut import_path_buf = vec![];
        let mut import_path_vec = import_path.into_inner().collect::<Vec<_>>();
        let last_item = import_path_vec.pop().unwrap();
        let import_type = match last_item.as_rule() {
            Rule::star => ImportType::Star,
            Rule::ident => ImportType::Item(eval!(
                Ident::parse_from_pair,
                warnings,
                errors,
                last_item,
                return err(warnings, errors)
            )),
            _ => unreachable!(),
        };

        for item in import_path_vec.into_iter() {
            if item.as_rule() == Rule::ident {
                import_path_buf.push(eval!(
                    Ident::parse_from_pair,
                    warnings,
                    errors,
                    item,
                    return err(warnings, errors)
                ));
            }
        }
        ok(
            UseStatement {
                call_path: import_path_buf,
                import_type,
            },
            Vec::new(),
            Vec::new(),
        )
    }
}
