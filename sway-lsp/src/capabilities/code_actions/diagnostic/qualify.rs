use super::auto_import::get_symbol_paths_for_name;
use crate::capabilities::{
    code_actions::{CodeActionContext, CODE_ACTION_QUALIFY_TITLE},
    diagnostic::DiagnosticData,
};
use lsp_types::{
    CodeAction as LspCodeAction, CodeActionKind, CodeActionOrCommand, Range, TextEdit,
    WorkspaceEdit,
};
use serde_json::Value;
use std::collections::HashMap;

/// Returns a list of [CodeActionOrCommand] suggestions for qualifying an unknown symbol with a path.
pub(crate) fn qualify_code_action(
    ctx: &CodeActionContext,
    diagnostics: &mut impl Iterator<Item = (Range, DiagnosticData)>,
) -> Option<Vec<CodeActionOrCommand>> {
    // Find a diagnostic that has the attached metadata indicating we should try to qualify the path.
    let (symbol_name, range) = diagnostics.find_map(|(range, diag)| {
        let name = diag.unknown_symbol_name?;
        Some((name, range))
    })?;

    // Check if there are any matching symbol paths to import using the name from the diagnostic data.
    let symbol_paths = get_symbol_paths_for_name(ctx, &symbol_name)?;

    // Create a list of code actions, one for each potential symbol path.
    let actions = symbol_paths
        .map(|symbol_path| {
            let text_edit = TextEdit {
                range,
                new_text: format!("{}", symbol_path),
            };

            let changes = HashMap::from([(ctx.uri.clone(), vec![text_edit])]);

            CodeActionOrCommand::CodeAction(LspCodeAction {
                title: format!("{} `{}`", CODE_ACTION_QUALIFY_TITLE, symbol_path),
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
