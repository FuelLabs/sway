use crate::{build_config::BuildConfig, error::*, parse_tree::ident, span, Rule};
use pest::iterators::Pair;

use sway_types::ident::Ident;

#[derive(Debug, Clone)]
pub enum ImportType {
    Star,
    SelfImport,
    Item(Ident),
}

/// A [UseStatement] is a statement that imports something from a module into the local namespace.
#[derive(Debug, Clone)]
pub struct UseStatement {
    pub(crate) call_path: Vec<Ident>,
    pub(crate) import_type: ImportType,
    // If `is_absolute` is true, then this use statement is an absolute path from
    // the project root namespace. If not, then it is relative to the current namespace.
    pub(crate) is_absolute: bool,
    pub(crate) alias: Option<Ident>,
}

impl UseStatement {
    pub(crate) fn parse_from_pair(
        pair: Pair<Rule>,
        config: Option<&BuildConfig>,
    ) -> CompileResult<Vec<Self>> {
        let mut errors = vec![];
        let mut warnings = vec![];
        let stmt = pair.into_inner().next().unwrap();
        let is_absolute = stmt.as_rule() == Rule::absolute_use_statement;
        let mut stmt = stmt.into_inner();
        let _use_keyword = stmt.next();
        let import_path = if is_absolute {
            stmt.clone().nth(1).expect("Guaranteed by grammar")
        } else {
            stmt.clone().next().expect("Guaranteed by grammar")
        };

        let use_statements_buf = check!(
            handle_import_path(import_path, config, is_absolute),
            return err(warnings, errors),
            warnings,
            errors
        );

        ok(use_statements_buf, warnings, errors)
    }
}

fn handle_import_path(
    import_path: Pair<Rule>,
    config: Option<&BuildConfig>,
    is_absolute: bool,
) -> CompileResult<Vec<UseStatement>> {
    let mut errors = vec![];
    let mut warnings = vec![];

    // Populate the call path
    let mut import_path_buf = vec![];
    let mut import_path_vec = import_path.into_inner().collect::<Vec<_>>();
    let mut last_item = import_path_vec.pop().unwrap();

    let mut top_level_alias = None;
    if last_item.as_rule() == Rule::alias {
        let item = last_item.into_inner().nth(1).unwrap();
        let alias_parsed = check!(
            ident::parse_from_pair(item, config),
            return err(warnings, errors),
            warnings,
            errors
        );
        top_level_alias = Some(alias_parsed);
        last_item = import_path_vec.pop().unwrap();
    }

    for item in import_path_vec.into_iter() {
        if item.as_rule() == Rule::ident {
            import_path_buf.push(check!(
                ident::parse_from_pair(item, config),
                return err(warnings, errors),
                warnings,
                errors
            ));
        }
    }

    let mut use_statements_buf = Vec::new();

    if last_item.as_rule() == Rule::import_items {
        // Handle the case where the last item is actually a list of items
        let mut import_items = last_item.into_inner();
        let _path_separator = import_items.next();

        let mut it = import_items.clone();
        while let Some(item) = it.next() {
            // kind of a base case here
            if item.as_rule() == Rule::ident {
                let import_type = ImportType::Item(check!(
                    ident::parse_from_pair(item.clone(), config),
                    return err(warnings, errors),
                    warnings,
                    errors
                ));
                let next = it.clone().next();
                let next_is_alias =
                    next.is_some() && next.clone().unwrap().as_rule() == Rule::alias;
                let mut alias = None;
                if next_is_alias {
                    let next_item = next.clone().unwrap();
                    let alias_item = next_item.into_inner().nth(1).unwrap();
                    let alias_parsed = check!(
                        ident::parse_from_pair(alias_item, config),
                        continue,
                        warnings,
                        errors
                    );
                    alias = Some(alias_parsed);
                    it.next();
                }

                use_statements_buf.push(UseStatement {
                    call_path: import_path_buf.clone(),
                    import_type,
                    is_absolute,
                    alias,
                });
            } else if item.as_rule() == Rule::import_path {
                // recurse - get the statement buffers and append
                let use_statements_buf_local = check!(
                    handle_import_path(item, config, is_absolute),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                for mut use_statement in use_statements_buf_local {
                    let mut new_call_path = Vec::new();
                    for ident in import_path_buf.clone() {
                        new_call_path.push(ident);
                    }
                    for ident in use_statement.call_path.clone() {
                        new_call_path.push(ident);
                    }
                    use_statement.call_path = new_call_path;
                    use_statements_buf.push(use_statement);
                }
            }
        }
    } else {
        // Handle the case where the last item is just an individual item
        let import_type = match last_item.as_rule() {
            Rule::star => {
                // Check that a star import does not have an alias
                if top_level_alias.is_some() {
                    errors.push(CompileError::AsteriskWithAlias {
                        span: span::Span::from_pest(last_item.as_span(), config.map(|c| c.path())),
                    });
                }
                ImportType::Star
            }
            Rule::self_keyword => ImportType::SelfImport,
            Rule::ident => ImportType::Item(check!(
                ident::parse_from_pair(last_item, config),
                return err(warnings, errors),
                warnings,
                errors
            )),
            _ => unreachable!(),
        };

        use_statements_buf.push(UseStatement {
            call_path: import_path_buf.clone(),
            import_type,
            is_absolute,
            alias: top_level_alias,
        });
    }

    ok(use_statements_buf, warnings, errors)
}
