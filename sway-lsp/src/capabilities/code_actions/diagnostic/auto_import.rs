use crate::{
    capabilities::{
        code_actions::{CodeActionContext, CODE_ACTION_IMPORT_TITLE},
        diagnostic::DiagnosticData,
    },
    core::token::{get_range_from_span, ParsedAstToken, SymbolKind, TypedAstToken},
};
use lsp_types::{
    CodeAction as LspCodeAction, CodeActionKind, CodeActionOrCommand, Position, Range, TextEdit,
    WorkspaceEdit,
};
use serde_json::Value;
use std::{
    cmp::Ordering,
    collections::{BTreeSet, HashMap},
    iter,
};
use sway_core::language::{
    parsed::ImportType,
    ty::{TyConstantDecl, TyDecl, TyFunctionDecl, TyModStatement, TyTypeAliasDecl, TyUseStatement},
    CallPath,
};
use sway_types::{Ident, Spanned};

/// Returns a list of [CodeActionOrCommand] suggestions for inserting a missing import.
pub(crate) fn import_code_action(
    ctx: &CodeActionContext,
    diagnostics: &mut impl Iterator<Item = (Range, DiagnosticData)>,
) -> Option<Vec<CodeActionOrCommand>> {
    // Find a diagnostic that has the attached metadata indicating we should try to suggest an auto-import.
    let symbol_name = diagnostics.find_map(|(_, diag)| diag.unknown_symbol_name)?;

    // Check if there are any matching call paths to import using the name from the diagnostic data.
    let call_paths = get_call_paths_for_name(ctx, &symbol_name)?;

    // Collect the tokens we need to determine where to insert the import statement.
    let mut use_statements = Vec::<TyUseStatement>::new();
    let mut mod_statements = Vec::<TyModStatement>::new();
    let mut program_type_keyword = None;

    ctx.tokens.tokens_for_file(ctx.temp_uri).for_each(|item| {
        if let Some(TypedAstToken::TypedUseStatement(use_stmt)) = &item.value().as_typed() {
            use_statements.push(use_stmt.clone());
        } else if let Some(TypedAstToken::TypedModStatement(mod_stmt)) = &item.value().as_typed() {
            mod_statements.push(mod_stmt.clone());
        } else if item.value().kind == SymbolKind::ProgramTypeKeyword {
            if let Some(ParsedAstToken::Keyword(ident)) = &item.value().as_parsed() {
                program_type_keyword = Some(ident.clone());
            }
        }
    });

    // Create a list of code actions, one for each potential call path.
    let actions = call_paths
        .map(|call_path| {
            let text_edit = get_text_edit(
                &call_path,
                &use_statements,
                &mod_statements,
                &program_type_keyword,
            );
            let changes = HashMap::from([(ctx.uri.clone(), vec![text_edit])]);

            CodeActionOrCommand::CodeAction(LspCodeAction {
                title: format!("{CODE_ACTION_IMPORT_TITLE} `{call_path}`"),
                kind: Some(CodeActionKind::QUICKFIX),
                edit: Some(WorkspaceEdit {
                    changes: Some(changes),
                    ..Default::default()
                }),
                data: Some(Value::String(ctx.uri.to_string())),
                ..Default::default()
            })
        })
        .collect::<Vec<_>>();

    if !actions.is_empty() {
        return Some(actions);
    }

    None
}

/// Returns an [Iterator] of [CallPath]s that match the given symbol name. The [CallPath]s are sorted
/// alphabetically.
pub(crate) fn get_call_paths_for_name<'s>(
    ctx: &'s CodeActionContext,
    symbol_name: &'s String,
) -> Option<impl 's + Iterator<Item = CallPath>> {
    let mut call_paths = ctx
        .tokens
        .tokens_for_name(symbol_name)
        .filter_map(move |item| {
            // If the typed token is a declaration, then we can import it.
            match item.value().as_typed().as_ref() {
                Some(TypedAstToken::TypedDeclaration(ty_decl)) => match ty_decl {
                    TyDecl::StructDecl(decl) => {
                        let struct_decl = ctx.engines.de().get_struct(&decl.decl_id);
                        let call_path = struct_decl
                            .call_path
                            .to_import_path(ctx.engines, ctx.namespace);
                        Some(call_path)
                    }
                    TyDecl::EnumDecl(decl) => {
                        let enum_decl = ctx.engines.de().get_enum(&decl.decl_id);
                        let call_path = enum_decl
                            .call_path
                            .to_import_path(ctx.engines, ctx.namespace);
                        Some(call_path)
                    }
                    TyDecl::TraitDecl(decl) => {
                        let trait_decl = ctx.engines.de().get_trait(&decl.decl_id);
                        let call_path = trait_decl
                            .call_path
                            .to_import_path(ctx.engines, ctx.namespace);
                        Some(call_path)
                    }
                    TyDecl::FunctionDecl(decl) => {
                        let function_decl = ctx.engines.de().get_function(&decl.decl_id);
                        let call_path = function_decl
                            .call_path
                            .to_import_path(ctx.engines, ctx.namespace);
                        Some(call_path)
                    }
                    TyDecl::ConstantDecl(decl) => {
                        let constant_decl = ctx.engines.de().get_constant(&decl.decl_id);
                        let call_path = constant_decl
                            .call_path
                            .to_import_path(ctx.engines, ctx.namespace);
                        Some(call_path)
                    }
                    TyDecl::TypeAliasDecl(decl) => {
                        let type_alias_decl = ctx.engines.de().get_type_alias(&decl.decl_id);
                        let call_path = type_alias_decl
                            .call_path
                            .to_import_path(ctx.engines, ctx.namespace);
                        Some(call_path)
                    }
                    _ => None,
                },
                Some(TypedAstToken::TypedFunctionDeclaration(TyFunctionDecl {
                    call_path, ..
                })) => {
                    let call_path = call_path.to_import_path(ctx.engines, ctx.namespace);
                    Some(call_path)
                }
                Some(TypedAstToken::TypedConstantDeclaration(TyConstantDecl {
                    call_path, ..
                }))
                | Some(TypedAstToken::TypedTypeAliasDeclaration(TyTypeAliasDecl {
                    call_path,
                    ..
                })) => {
                    let call_path = call_path.to_import_path(ctx.engines, ctx.namespace);
                    Some(call_path)
                }
                _ => None,
            }
        })
        .collect::<Vec<_>>();
    call_paths.sort();
    Some(call_paths.into_iter())
}

/// Returns a [TextEdit] to insert an import statement for the given [CallPath] in the appropriate location in the file.
///
/// To determine where to insert the import statement in the file, we try these options and do
/// one of the following, based on the contents of the file.
///
/// 1. Add the import to an existing import that has the same prefix.
/// 2. Insert the import on a new line relative to existing use statements.
/// 3. Insert the import on a new line after existing mod statements.
/// 4. Insert the import on a new line after the program type statement (e.g. `contract;`)
/// 5. If all else fails, insert it at the beginning of the file.
fn get_text_edit(
    call_path: &CallPath,
    use_statements: &[TyUseStatement],
    mod_statements: &[TyModStatement],
    program_type_keyword: &Option<Ident>,
) -> TextEdit {
    get_text_edit_for_group(call_path, use_statements)
        .or_else(|| get_text_edit_in_use_block(call_path, use_statements))
        .unwrap_or(get_text_edit_fallback(
            call_path,
            mod_statements,
            program_type_keyword,
        ))
}

/// Returns a [TextEdit] that inserts the call path into the existing statement if there is an
/// existing [TyUseStatement] with the same prefix as the given [CallPath]. Otherwise, returns [None].
fn get_text_edit_for_group(
    call_path: &CallPath,
    use_statements: &[TyUseStatement],
) -> Option<TextEdit> {
    let group_statements = use_statements.iter().filter(|use_stmt| {
        call_path
            .prefixes
            .iter()
            .zip(use_stmt.call_path.iter())
            .all(|(prefix, stmt_prefix)| prefix.as_str() == stmt_prefix.as_str())
    });

    let mut group_statement_span = None;
    let mut suffixes = group_statements
        .filter_map(|stmt| {
            // Set the group statement span if it hasn't been set yet. If it has been set, filter out
            // any statements that aren't part of the same import group.
            if group_statement_span.is_none() {
                group_statement_span = Some(stmt.span());
            } else if group_statement_span != Some(stmt.span()) {
                return None;
            }

            let name = match &stmt.import_type {
                ImportType::Star => "*".to_string(),
                ImportType::SelfImport(_) => "self".to_string(),
                ImportType::Item(ident) => ident.to_string(),
            };
            match &stmt.alias {
                Some(alias) => Some(format!("{name} as {alias}")),
                None => Some(name),
            }
        })
        .chain(iter::once(call_path.suffix.to_string()))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    // If there were no imports with the same prefix, return None. Otherwise, build the text edit response.
    group_statement_span.map(|span| {
        suffixes.sort();
        let suffix_string = suffixes.join(", ");
        let prefix_string = call_path
            .prefixes
            .iter()
            .map(sway_types::BaseIdent::as_str)
            .collect::<Vec<_>>()
            .join("::");

        TextEdit {
            range: get_range_from_span(&span.clone()),
            new_text: format!("use {prefix_string}::{{{suffix_string}}};"),
        }
    })
}

/// If there are existing [TyUseStatement]s, returns a [TextEdit] to insert the new import statement on the
/// line above or below an existing statement, ordered alphabetically.
fn get_text_edit_in_use_block(
    call_path: &CallPath,
    use_statements: &[TyUseStatement],
) -> Option<TextEdit> {
    let after_statement = use_statements.iter().reduce(|acc, curr| {
        if call_path.span().as_str().cmp(curr.span().as_str()) == Ordering::Greater
            && curr.span().as_str().cmp(acc.span().as_str()) == Ordering::Greater
        {
            return curr;
        }
        acc
    })?;

    let after_range = get_range_from_span(&after_statement.span());
    let range_line = if call_path
        .span()
        .as_str()
        .cmp(after_statement.span().as_str())
        == Ordering::Greater
    {
        after_range.end.line + 1
    } else {
        after_range.start.line
    };

    Some(TextEdit {
        range: Range::new(Position::new(range_line, 0), Position::new(range_line, 0)),
        new_text: format!("use {call_path};\n"),
    })
}

/// Returns a [TextEdit] to insert an import statement either after the last mod statement, after the program
/// type statement, or at the beginning of the file.
fn get_text_edit_fallback(
    call_path: &CallPath,
    mod_statements: &[TyModStatement],
    program_type_keyword: &Option<Ident>,
) -> TextEdit {
    let range_line = mod_statements
        .iter()
        .map(|stmt| stmt.span())
        .reduce(|acc, span| {
            if span > acc {
                return span;
            }
            acc
        })
        .map(|span| get_range_from_span(&span).end.line + 1)
        .unwrap_or(
            program_type_keyword
                .clone()
                .map(|keyword| get_range_from_span(&keyword.span()).end.line + 1)
                .unwrap_or(1),
        );
    TextEdit {
        range: Range::new(Position::new(range_line, 0), Position::new(range_line, 0)),
        new_text: format!("\nuse {call_path};\n"),
    }
}

#[cfg(test)]
mod tests {
    use sway_core::language::Visibility;
    use sway_types::{span::Source, Span};

    use super::*;

    fn assert_text_edit(text_edit: TextEdit, expected_range: Range, expected_text: String) {
        assert_eq!(text_edit.range, expected_range);
        assert_eq!(text_edit.new_text, expected_text);
    }

    fn get_mock_call_path(prefixes: Vec<&str>, suffix: &str) -> CallPath {
        CallPath {
            prefixes: get_mock_prefixes(prefixes),
            suffix: Ident::new_no_span(suffix.to_string()),
            callpath_type: sway_core::language::CallPathType::Full,
        }
    }

    fn get_mock_prefixes(prefixes: Vec<&str>) -> Vec<Ident> {
        prefixes
            .into_iter()
            .map(|p| Ident::new(Span::from_string(p.into())))
            .collect()
    }

    fn get_prefixes_from_src(src: &Source, prefixes: Vec<&str>) -> Vec<Ident> {
        prefixes
            .into_iter()
            .filter_map(|p| get_ident_from_src(src, p))
            .collect()
    }

    fn get_span_from_src(src: &Source, text: &str) -> Option<Span> {
        let start = src.text.find(text)?;
        let end = start + text.len();
        Span::new(src.clone(), start, end, None)
    }

    fn get_ident_from_src(src: &Source, name: &str) -> Option<Ident> {
        let span = get_span_from_src(src, name)?;
        Some(Ident::new(span))
    }

    fn get_use_stmt_from_src(
        src: &Source,
        prefixes: Vec<&str>,
        import_type: ImportType,
        text: &str,
    ) -> TyUseStatement {
        TyUseStatement {
            call_path: get_prefixes_from_src(src, prefixes),
            span: get_span_from_src(src, text).unwrap(),
            import_type,
            is_relative_to_package_root: false,
            alias: None,
        }
    }

    fn get_mod_stmt_from_src(src: &Source, mod_name: &str, text: &str) -> TyModStatement {
        TyModStatement {
            span: get_span_from_src(src, text).unwrap(),
            mod_name: get_ident_from_src(src, mod_name).unwrap(),
            visibility: Visibility::Private,
        }
    }

    #[test]
    fn get_text_edit_existing_import() {
        let src = Source::new(
            r#"contract;

use a:b:C;
use b:c:*;
"#,
        );
        let new_call_path = get_mock_call_path(vec!["a", "b"], "D");
        let use_statements = vec![
            get_use_stmt_from_src(
                &src,
                Vec::from(["a", "b"]),
                ImportType::Item(get_ident_from_src(&src, "C").unwrap()),
                "use a:b:C;",
            ),
            get_use_stmt_from_src(&src, Vec::from(["b", "c"]), ImportType::Star, "use b:c:*;"),
        ];

        let mod_statements = vec![];
        let program_type_keyword = get_ident_from_src(&src, "contract");

        let expected_range = Range::new(Position::new(2, 0), Position::new(2, 10));
        let expected_text = "use a::b::{C, D};".into();

        let text_edit = get_text_edit(
            &new_call_path,
            &use_statements,
            &mod_statements,
            &program_type_keyword,
        );
        assert_text_edit(text_edit, expected_range, expected_text);
    }

    #[test]
    fn get_text_edit_new_import() {
        let src = Source::new(
            r#"predicate;

use b:c:*;
"#,
        );
        let new_call_path = get_mock_call_path(vec!["a", "b"], "C");
        let use_statements = vec![get_use_stmt_from_src(
            &src,
            Vec::from(["b", "c"]),
            ImportType::Star,
            "use b:c:*;",
        )];

        let mod_statements = vec![];
        let program_type_keyword = get_ident_from_src(&src, "predicate");

        let expected_range = Range::new(Position::new(2, 0), Position::new(2, 0));
        let expected_text = "use a::b::C;\n".into();

        let text_edit = get_text_edit(
            &new_call_path,
            &use_statements,
            &mod_statements,
            &program_type_keyword,
        );
        assert_text_edit(text_edit, expected_range, expected_text);
    }

    #[test]
    fn get_text_edit_existing_group_import() {
        let src = Source::new(
            r#"contract;

use b:c:{D, F};
"#,
        );
        let new_call_path = get_mock_call_path(vec!["b", "c"], "E");
        let use_statements = vec![
            get_use_stmt_from_src(
                &src,
                Vec::from(["b", "c"]),
                ImportType::Item(get_ident_from_src(&src, "D").unwrap()),
                "use b:c:{D, F};",
            ),
            get_use_stmt_from_src(
                &src,
                Vec::from(["b", "c"]),
                ImportType::Item(get_ident_from_src(&src, "F").unwrap()),
                "use b:c:{D, F};",
            ),
        ];

        let mod_statements = vec![];
        let program_type_keyword = get_ident_from_src(&src, "contract");

        let expected_range = Range::new(Position::new(2, 0), Position::new(2, 15));
        let expected_text = "use b::c::{D, E, F};".into();

        let text_edit = get_text_edit(
            &new_call_path,
            &use_statements,
            &mod_statements,
            &program_type_keyword,
        );
        assert_text_edit(text_edit, expected_range, expected_text);
    }

    #[test]
    fn get_text_edit_after_mod() {
        let src = Source::new(
            r#"library;

mod my_module;
pub mod zz_module;
"#,
        );
        let new_call_path = get_mock_call_path(vec!["b", "c"], "D");
        let use_statements = vec![];

        let mod_statements = vec![
            get_mod_stmt_from_src(&src, "my_module", "mod my_module;"),
            get_mod_stmt_from_src(&src, "zz_module", "pub mod zz_module"),
        ];
        let program_type_keyword = get_ident_from_src(&src, "library");

        let expected_range = Range::new(Position::new(4, 0), Position::new(4, 0));
        let expected_text = "\nuse b::c::D;\n".into();

        let text_edit = get_text_edit(
            &new_call_path,
            &use_statements,
            &mod_statements,
            &program_type_keyword,
        );
        assert_text_edit(text_edit, expected_range, expected_text);
    }

    #[test]
    fn get_text_edit_after_program() {
        let src = Source::new(
            r#"script;

const HI: u8 = 0;
"#,
        );
        let new_call_path = get_mock_call_path(vec!["b", "c"], "D");
        let use_statements = vec![];

        let mod_statements = vec![];
        let program_type_keyword = get_ident_from_src(&src, "script");

        let expected_range = Range::new(Position::new(1, 0), Position::new(1, 0));
        let expected_text = "\nuse b::c::D;\n".into();

        let text_edit = get_text_edit(
            &new_call_path,
            &use_statements,
            &mod_statements,
            &program_type_keyword,
        );
        assert_text_edit(text_edit, expected_range, expected_text);
    }
}
