use crate::{error::*, parse_tree::ident, parser::Rule, type_engine::*, BuildConfig};

use sway_types::{ident::Ident, span::Span};

use pest::iterators::Pair;

#[derive(Debug, Clone)]
/// A declaration of contract storage. Only valid within contract contexts.
/// All values in this struct are mutable and persistent among executions of the same contract deployment.
pub struct StorageDeclaration {
    pub fields: Vec<StorageField>,
    pub span: Span,
}

/// An individual field in a storage declaration.
/// A type annotation _and_ initializer value must be provided. The initializer value must be a
/// constant expression. For now, that basically means just a literal, but as constant folding
/// improves, we can update that.
#[derive(Debug, Clone)]
pub struct StorageField {
    pub name: Ident,
    pub r#type: TypeInfo,
}

impl StorageField {
    pub(crate) fn parse_from_pair(
        pair: Pair<Rule>,
        conf: Option<&BuildConfig>,
    ) -> CompileResult<ParserLifter<Self>> {
        let mut errors = vec![];
        let mut warnings = vec![];
        let mut iter = pair.into_inner();
        let name = iter.next().expect("guaranteed by grammar");
        let r#type = iter.next().expect("guaranteed by grammar");

        let name = check!(
            ident::parse_from_pair(name, conf),
            return err(warnings, errors),
            warnings,
            errors
        );
        let r#type = check!(
            TypeInfo::parse_from_pair(r#type, conf),
            return err(warnings, errors),
            warnings,
            errors
        );
        let res = StorageField { name, r#type };
        ok(
            ParserLifter {
                var_decls: vec![],
                value: res,
            },
            warnings,
            errors,
        )
    }
}

impl StorageDeclaration {
    pub(crate) fn parse_from_pair(
        pair: Pair<Rule>,
        config: Option<&BuildConfig>,
    ) -> CompileResult<ParserLifter<Self>> {
        debug_assert_eq!(pair.as_rule(), Rule::storage_decl);
        let path = config.map(|c| c.path());
        let mut errors = vec![];
        let mut warnings = vec![];
        let span = Span::from_pest(pair.as_span(), path);
        let mut iter = pair.into_inner();
        let storage_keyword = iter.next();
        debug_assert_eq!(
            storage_keyword.map(|x| x.as_rule()),
            Some(Rule::storage_keyword)
        );
        let fields_results: Vec<CompileResult<ParserLifter<StorageField>>> = iter
            .next()
            .unwrap()
            .into_inner()
            .map(|x| StorageField::parse_from_pair(x, config))
            .collect();
        let mut fields: Vec<StorageField> = Vec::with_capacity(fields_results.len());
        let mut var_decls = vec![];
        for res in fields_results {
            let mut ok = check!(res, continue, warnings, errors);
            fields.push(ok.value);
            var_decls.append(&mut ok.var_decls);
        }
        let res = StorageDeclaration { fields, span };
        ok(
            ParserLifter {
                var_decls,
                value: res,
            },
            warnings,
            errors,
        )
    }
}
