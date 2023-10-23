mod auto_import;

use crate::capabilities::{code_actions::CodeActionContext, diagnostic::DiagnosticData};
use lsp_types::CodeActionOrCommand;

use self::auto_import::import_code_action;

use super::CODE_ACTION_IMPORT_TITLE;

/// Returns a list of [CodeActionOrCommand] based on the relavent compiler diagnostics.
pub(crate) fn code_actions(ctx: &CodeActionContext) -> Option<Vec<CodeActionOrCommand>> {
    // Find diagnostics that have attached metadata.
    let diagnostics_with_data = ctx.diagnostics.iter().filter_map(|diag| {
        if let Some(data) = diag.clone().data {
            return serde_json::from_value::<DiagnosticData>(data).ok();
        }
        None
    });

    import_code_action(ctx, &mut diagnostics_with_data.clone())
        .into_iter()
        .reduce(|mut combined, mut curr| {
            combined.append(&mut curr);
            combined
        })
}
