use crate::build_config::BuildConfig;
use crate::error::*;
use crate::span;
use crate::Ident;
use crate::Rule;
use pest::iterators::Pair;

#[derive(Debug, Clone)]
pub enum ImportType<'sc> {
    Star,
    Item(Ident<'sc>),
}

/// A [UseStatement] is a statement that imports a module from a source file.
#[derive(Debug, Clone)]
pub struct UseStatement<'sc> {
    pub(crate) call_path: Vec<Ident<'sc>>,
    pub(crate) import_type: ImportType<'sc>,
    // If `is_absolute` is true, then this use statement is an absolute path from
    // the project root namespace. If not, then it is relative to the current namespace.
    pub(crate) is_absolute: bool,
    pub(crate) alias: Option<Ident<'sc>>,
}

impl<'sc> UseStatement<'sc> {
    pub(crate) fn parse_from_pair(
        pair: Pair<'sc, Rule>,
        config: Option<&BuildConfig>,
    ) -> CompileResult<'sc, Self> {
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
        let mut import_path_buf = vec![];
        let mut import_path_vec = import_path.into_inner().collect::<Vec<_>>();
        let last_item = import_path_vec.pop().unwrap();
        let import_type = match last_item.as_rule() {
            Rule::star => ImportType::Star,
            Rule::ident => ImportType::Item(check!(
                Ident::parse_from_pair(last_item, config),
                return err(warnings, errors),
                warnings,
                errors
            )),
            _ => unreachable!(),
        };

        for item in import_path_vec.into_iter() {
            if item.as_rule() == Rule::star {
                errors.push(CompileError::NonFinalAsteriskInPath {
                    span: span::Span {
                        span: item.as_span(),
                        path: config.map(|c| c.path()),
                    },
                });
                continue;
            } else if item.as_rule() == Rule::ident {
                import_path_buf.push(check!(
                    Ident::parse_from_pair(item, config),
                    return err(warnings, errors),
                    warnings,
                    errors
                ));
            }
        }

        let mut alias = None;
        for item in stmt {
            if item.as_rule() == Rule::alias {
                let item = item.into_inner().nth(1).unwrap();
                let alias_parsed = check!(
                    Ident::parse_from_pair(item, config),
                    continue,
                    warnings,
                    errors
                );
                alias = Some(alias_parsed);
            }
        }

        ok(
            UseStatement {
                call_path: import_path_buf,
                import_type,
                is_absolute,
                alias,
            },
            Vec::new(),
            Vec::new(),
        )
    }
}
