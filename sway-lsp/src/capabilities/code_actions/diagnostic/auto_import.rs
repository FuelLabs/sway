use std::{cmp::Ordering, collections::HashMap};

use crate::{
    capabilities::{code_actions::CodeActionContext, diagnostic::DiagnosticData},
    core::token::{get_range_from_span, AstToken, SymbolKind, TypedAstToken},
};
use lsp_types::{
    CodeAction as LspCodeAction, CodeActionKind, CodeActionOrCommand, Position, Range, TextEdit,
    WorkspaceEdit,
};
use serde_json::Value;
use sway_core::language::{
    parsed::ImportType,
    ty::{TyDecl, TyIncludeStatement, TyUseStatement},
    CallPath,
};
use sway_types::{Ident, Spanned};

use super::CODE_ACTION_IMPORT_TITLE;

/// Returns a list of [CodeActionOrCommand] suggestions for inserting a missing import.
pub(crate) fn import_code_action(
    ctx: &CodeActionContext,
    diagnostics: &mut impl Iterator<Item = DiagnosticData>,
) -> Option<Vec<CodeActionOrCommand>> {
    // Find a diagnostic that has the attached metadata indicating we should try to suggest an auto-import.
    let symbol_name = diagnostics.find_map(|diag| diag.unknown_symbol_name)?;

    // Check if there are any matching call paths to import using the name from the diagnostic data.
    let call_paths = get_call_paths_for_name(ctx, &symbol_name)?;

    // Collect the tokens we need to determine where to insert the import statement.
    let mut use_statements = Vec::<TyUseStatement>::new();
    let mut include_statements = Vec::<TyIncludeStatement>::new();
    let mut program_type_keyword = None;

    ctx.tokens
        .tokens_for_file(ctx.temp_uri)
        .for_each(|(_, token)| {
            if let Some(TypedAstToken::TypedUseStatement(use_stmt)) = token.typed {
                use_statements.push(use_stmt);
            } else if let Some(TypedAstToken::TypedIncludeStatement(include_stmt)) = token.typed {
                include_statements.push(include_stmt);
            } else if token.kind == SymbolKind::ProgramTypeKeyword {
                if let AstToken::Keyword(ident) = token.parsed {
                    program_type_keyword = Some(ident);
                }
            }
        });

    let actions = call_paths
        .map(|call_path| {
            // To determine where to insert the import statement in the file, we try these options and do
            // one of the following, based on the contents of the file.
            //
            // 1. Add the import to an existing import that has the same prefix.
            // 2. Insert the import on a new line relative to existing use statements.
            // 3. Insert the import on a new line after existing mod statements.
            // 4. Insert the import on a new line after the program type statement (e.g. `contract;`)
            // 5. If all else fails, insert it at the beginning of the file.

            let text_edit: TextEdit = get_text_edit_for_group(&call_path, &use_statements)
                .or_else(|| get_text_edit_in_use_block(&call_path, &use_statements))
                .unwrap_or(get_text_edit_fallback(
                    &call_path,
                    &include_statements,
                    &program_type_keyword,
                ));

            let changes = HashMap::from([(ctx.uri.clone(), vec![text_edit])]);

            CodeActionOrCommand::CodeAction(LspCodeAction {
                title: format!("{} `{}`", CODE_ACTION_IMPORT_TITLE, call_path),
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

/// Returns an [Iterator] of [CallPath]s that match the given symbol name.
fn get_call_paths_for_name<'s>(
    ctx: &'s CodeActionContext,
    symbol_name: &'s String,
) -> Option<impl 's + Iterator<Item = CallPath>> {
    let namespace = ctx.namespace.to_owned()?;
    Some(
        ctx.tokens
            .tokens_for_name(symbol_name)
            .filter_map(move |(_, token)| {
                // If the typed token is a declaration, then we can import it.
                match token.typed.as_ref() {
                    Some(TypedAstToken::TypedDeclaration(ty_decl)) => {
                        return match ty_decl {
                            TyDecl::StructDecl(decl) => {
                                let struct_decl = ctx.engines.de().get_struct(&decl.decl_id);
                                let call_path = struct_decl.call_path.to_import_path(&namespace);
                                Some(call_path)
                            }
                            TyDecl::EnumDecl(decl) => {
                                let enum_decl = ctx.engines.de().get_enum(&decl.decl_id);
                                let call_path = enum_decl.call_path.to_import_path(&namespace);
                                Some(call_path)
                            }
                            TyDecl::TraitDecl(decl) => {
                                let trait_decl = ctx.engines.de().get_trait(&decl.decl_id);
                                let call_path = trait_decl.call_path.to_import_path(&namespace);
                                Some(call_path)
                            }
                            _ => None,
                        };
                    }
                    Some(TypedAstToken::TypedFunctionDeclaration(ty_decl)) => {
                        let call_path = ty_decl.call_path.to_import_path(&namespace);
                        Some(call_path)
                    }
                    Some(TypedAstToken::TypedConstantDeclaration(ty_decl)) => {
                        let call_path = ty_decl.call_path.to_import_path(&namespace);
                        Some(call_path)
                    }
                    Some(TypedAstToken::TypedTypeAliasDeclaration(ty_decl)) => {
                        let call_path = ty_decl.call_path.to_import_path(&namespace);
                        Some(call_path)
                    }
                    _ => None,
                }
            }),
    )
}

/// If there is an existing [TyUseStatement] with the same prefix as the given [CallPath], returns a
/// [TextEdit] that inserts the call path into the existing statement. Otherwise, returns [None].
fn get_text_edit_for_group(
    call_path: &CallPath,
    use_statements: &[TyUseStatement],
) -> Option<TextEdit> {
    let group_statement = use_statements.iter().find(|use_stmt| {
        call_path
            .prefixes
            .iter()
            .zip(use_stmt.call_path.iter())
            .all(|(prefix, stmt_prefix)| prefix.as_str() == stmt_prefix.as_str())
    })?;

    let prefix_string = group_statement
        .call_path
        .iter()
        .map(|path| path.as_str())
        .collect::<Vec<_>>()
        .join("::");
    let statement_suffix_string = {
        let name = match &group_statement.import_type {
            ImportType::Star => "*".to_string(),
            ImportType::SelfImport(_) => "self".to_string(),
            ImportType::Item(ident) => ident.to_string(),
        };
        match &group_statement.alias {
            Some(alias) => format!("{} as {}", name, alias),
            None => name,
        }
    };
    let mut suffixes = [statement_suffix_string, call_path.suffix.to_string()];
    suffixes.sort(); // TODO: test this. Is there a better way to sort?
    let suffix_string = suffixes.join(", ");

    Some(TextEdit {
        range: get_range_from_span(&group_statement.span()),
        new_text: format!("use {}::{{{}}};\n", prefix_string, suffix_string),
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
        new_text: format!("use {};\n", call_path),
    })
}

/// Returns a [TextEdit] to insert an import statement either after the last mod statement, after the program
/// type statement, or at the beginning of the file.
fn get_text_edit_fallback(
    call_path: &CallPath,
    include_statements: &[TyIncludeStatement],
    program_type_keyword: &Option<Ident>,
) -> TextEdit {
    let range_line = include_statements
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
        new_text: format!("\nuse {};\n", call_path),
    }
}
