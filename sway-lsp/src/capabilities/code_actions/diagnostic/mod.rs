use std::{cmp::Ordering, collections::HashMap, iter};

use crate::{
    capabilities::{code_actions::CodeActionContext, diagnostic::DiagnosticData},
    core::{
        session,
        token::{get_range_from_span, AstToken, Token, TypedAstToken},
    },
};
use lsp_types::{
    CodeAction as LspCodeAction, CodeActionKind, CodeActionOrCommand, Position, Range, TextEdit,
    WorkspaceEdit,
};
use serde_json::Value;
use sway_core::{
    fuel_prelude::fuel_vm::call,
    language::{
        parsed::{ImportType, TreeType},
        ty::{TyDecl, TyUseStatement},
    },
    Namespace,
};
use sway_types::{BaseIdent, Ident, Spanned};

use super::CODE_ACTION_IMPORT_TITLE;

pub(crate) fn code_actions(
    ctx: &CodeActionContext,
    namespace: &Option<Namespace>,
) -> Option<Vec<CodeActionOrCommand>> {
    if let Some(namespace) = namespace {
        return import_code_action(ctx, namespace);
    }
    None
}

/// Returns a [CodeActionOrCommand] for the given code action.
fn import_code_action(
    ctx: &CodeActionContext,
    namespace: &Namespace,
) -> Option<Vec<CodeActionOrCommand>> {
    if let Some(diag_data) = ctx.diagnostics.iter().find_map(|diag| {
        let data = diag.clone().data?;
        serde_json::from_value::<DiagnosticData>(data).ok()
    }) {
        // Check if there is a type to import using the name from the diagnostic data.
        let call_paths = ctx
            .tokens
            .tokens_for_name(&diag_data.name_to_import)
            .filter_map(|(ident, token)| {
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
                    _ => return None,
                }
            });

        // eprintln!("before res1");
        // let res1 = ctx
        //     .tokens
        //     .tokens_for_file(ctx.temp_uri)
        //     .filter_map(|(ident, token)| {
        //         eprintln!("1 foreach ident: {:?}", ident);
        //         if let Some(TypedAstToken::TypedUseStatement(use_stmt)) = token.typed {
        //             eprintln!("use_stmt: {:?}", use_stmt);
        //             return Some("hi");
        //         }

        //         None
        //     });

        // let (range_line, prefix) = ctx
        // let res = ctx
        //     .tokens
        //     .tokens_for_file(ctx.temp_uri)
        //     // .filter_map(|(ident, token)| {
        //     .reduce(|(acc_ident, acc_token), (ident, token)| {
        //         // if line > acc.0 {
        //         //     (line, prefix)
        //         // } else {
        //         //     acc
        //         // }

        //         // eprintln!("2 foreach ident: {:?}", ident);

        //         if let Some(TypedAstToken::TypedUseStatement(use_stmt)) = token.typed {
        //             eprintln!("use_stmt: {:?}", use_stmt);
        //             // return Some((use_stmt.span.end_pos().line_col().0, ""));
        //             // todo: sort
        //             return (ident, token);
        //         }

        //         // if let AstToken::Keyword(_) = token.parsed {
        //         //     if ident.name == "use" {
        //         //         return Some((ident.range.start.line, ""));
        //         //     } else if ["mod", "contract", "script", "library", "predicate"]
        //         //         .contains(&ident.name.as_str())
        //         //     {
        //         //         return Some((ident.range.end.line + 1, "\n"));
        //         //     }
        //         // }

        //         return (acc_ident, acc_token);
        //     });
        // .reduce(
        //     |acc, (line, prefix)| {
        //         if line > acc.0 {
        //             (line, prefix)
        //         } else {
        //             acc
        //         }
        //     },
        // )
        // .unwrap_or((1, "\n"));

        let actions = call_paths
            .filter_map(|call_path| {
                let mut use_statements = Vec::<TyUseStatement>::new();
                let mut keywords = Vec::<Ident>::new();

                ctx.tokens
                    .tokens_for_file(ctx.temp_uri)
                    .for_each(|(_, token)| {
                        if let Some(TypedAstToken::TypedUseStatement(use_stmt)) = token.typed {
                            use_statements.push(use_stmt);
                        }
                        if let AstToken::Keyword(ident) = token.parsed {
                            keywords.push(ident);
                        }
                    });

                let text_edit: TextEdit = {
                    // First, check if this import can be added to an existing use statement.
                    let group_statement = use_statements.iter().find(|use_stmt| {
                        call_path
                            .prefixes
                            .iter()
                            .zip(use_stmt.call_path.iter())
                            .all(|(prefix, stmt_prefix)| prefix.as_str() == stmt_prefix.as_str())
                    });

                    if let Some(statement) = group_statement {
                        let prefix_string = statement
                            .call_path
                            .iter()
                            .map(|path| path.as_str())
                            .collect::<Vec<_>>()
                            .join("::");
                        let statement_suffix_string = {
                            let name = match &statement.import_type {
                                ImportType::Star => "*".to_string(),
                                ImportType::SelfImport(_) => "self".to_string(),
                                ImportType::Item(ident) => ident.to_string(),
                            };
                            match &statement.alias {
                                Some(alias) => format!("{} as {}", name, alias.to_string()),
                                None => name,
                            }
                        };
                        let mut suffixes = [statement_suffix_string, call_path.suffix.to_string()];
                        suffixes.sort(); //todo
                        let suffix_string = suffixes.join(", ");

                        TextEdit {
                            range: get_range_from_span(&statement.span()), // todo: fix
                            new_text: format!("use {}::{{{}}};\n", prefix_string, suffix_string),
                        }
                    } else {
                        // Find the best position in the file to insert a new use statement.
                        // First, check if it can be inserted relative to existing use statements.
                        if !use_statements.is_empty() {
                            let after_statement = use_statements
                                .iter()
                                .reduce(|acc, curr| {
                                    if call_path.span().as_str().cmp(curr.span().as_str())
                                        == Ordering::Greater
                                        && curr.span().as_str().cmp(acc.span().as_str())
                                            == Ordering::Greater
                                    {
                                        return curr;
                                    }
                                    return acc;
                                })
                                .unwrap();

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

                            TextEdit {
                                range: Range::new(
                                    Position::new(range_line, 0),
                                    Position::new(range_line, 0),
                                ),
                                new_text: format!("use {};\n", call_path),
                            }
                        } else {
                            // Otherwise, insert it at the top of the file, after any mod statements.
                            let range_line = keywords
                                .iter()
                                .filter_map(|kw| {
                                    if kw.as_str() == "mod" {
                                        return Some(get_range_from_span(&kw.span()).end.line + 1);
                                    }
                                    None
                                })
                                .max()
                                .or_else(|| {
                                    keywords.iter().find_map(|kw| {
                                        if ["mod", "contract", "script", "library", "predicate"]
                                            .contains(&kw.as_str())
                                        {
                                            return Some(
                                                get_range_from_span(&kw.span()).end.line + 1,
                                            );
                                        }
                                        None
                                    })
                                })
                                .unwrap_or(1);

                            TextEdit {
                                range: Range::new(
                                    Position::new(range_line, 0),
                                    Position::new(range_line, 0),
                                ),
                                new_text: format!("\nuse {};\n", call_path),
                            }
                        }
                    }
                };

                let changes = HashMap::from([(ctx.uri.clone(), vec![text_edit])]);

                Some(CodeActionOrCommand::CodeAction(LspCodeAction {
                    title: format!("{} `{}`", CODE_ACTION_IMPORT_TITLE, call_path),
                    kind: Some(CodeActionKind::QUICKFIX),
                    edit: Some(WorkspaceEdit {
                        changes: Some(changes),
                        ..Default::default()
                    }),
                    data: Some(Value::String(ctx.uri.to_string())),
                    ..Default::default()
                }))
            })
            .collect::<Vec<_>>();

        if !actions.is_empty() {
            return Some(actions);
        }
    }
    None
}
