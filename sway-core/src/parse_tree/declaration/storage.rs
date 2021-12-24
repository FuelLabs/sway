use crate::{
    error::*, ident::Ident, parse_tree::Expression, parser::Rule, type_engine::*, BuildConfig, Span,
};
use fuel_pest::iterators::Pair;

#[derive(Debug, Clone)]
/// A declaration of contract storage. Only valid within contract contexts.
/// All values in this struct are mutable and persistent among executions of the same contract deployment.
pub struct StorageDeclaration<'sc> {
    pub fields: Vec<StorageField<'sc>>,
    pub span: Span<'sc>,
}

/// An individual field in a storage declaration.
/// A type annotation _and_ initializer value must be provided. The initializer value must be a
/// constant expression. For now, that basically means just a literal, but as constant folding
/// improves, we can update that.
#[derive(Debug, Clone)]
pub struct StorageField<'sc> {
    pub name: Ident<'sc>,
    pub r#type: TypeInfo,
    pub initializer: Expression<'sc>,
}

impl<'sc> StorageField<'sc> {
    pub(crate) fn parse_from_pair(
        pair: Pair<'sc, Rule>,
        conf: Option<&BuildConfig>,
    ) -> CompileResult<'sc, Self> {
        let mut errors = vec![];
        let mut warnings = vec![];
        let mut iter = pair.into_inner();
        let name = iter.next().expect("guaranteed by grammar");
        let r#type = iter.next().expect("guaranteed by grammar");
        let initializer = iter.next().expect("guaranteed by grammar");

        let name = check!(
            Ident::parse_from_pair(name, conf),
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
        let initializer = check!(
            Expression::parse_from_pair(initializer, conf),
            return err(warnings, errors),
            warnings,
            errors
        );
        ok(
            StorageField {
                name,
                r#type,
                initializer,
            },
            warnings,
            errors,
        )
    }
}

impl<'sc> StorageDeclaration<'sc> {
    pub(crate) fn parse_from_pair(
        pair: Pair<'sc, Rule>,
        config: Option<&BuildConfig>,
    ) -> CompileResult<'sc, Self> {
        debug_assert_eq!(pair.as_rule(), Rule::storage_decl);
        let path = config.map(|c| c.path());
        let mut errors = vec![];
        let mut warnings = vec![];
        let span = Span {
            span: pair.as_span(),
            path,
        };
        let mut iter = pair.into_inner();
        let storage_keyword = iter.next();
        debug_assert_eq!(
            storage_keyword.map(|x| x.as_rule()),
            Some(Rule::storage_keyword)
        );
        let fields_results: Vec<CompileResult<'sc, StorageField>> = iter
            .next()
            .unwrap()
            .into_inner()
            .map(|x| StorageField::parse_from_pair(x, config))
            .collect();
        let mut fields: Vec<StorageField> = Vec::with_capacity(fields_results.len());
        for res in fields_results {
            let ok = check!(res, continue, warnings, errors);
            fields.push(ok);
        }
        ok(StorageDeclaration { fields, span }, warnings, errors)
    }
}
