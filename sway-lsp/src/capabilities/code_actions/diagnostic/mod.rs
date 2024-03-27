mod auto_import;
mod qualify;

use crate::capabilities::{code_actions::CodeActionContext, diagnostic::DiagnosticData};
use lsp_types::CodeActionOrCommand;

use self::auto_import::import_code_action;
use self::qualify::qualify_code_action;

/// Returns a list of [CodeActionOrCommand] based on the relevant compiler diagnostics.
pub(crate) fn code_actions(ctx: &CodeActionContext) -> Option<Vec<CodeActionOrCommand>> {
    // Find diagnostics that have attached metadata.
    let diagnostics_with_data = ctx.diagnostics.iter().filter_map(|diag| {
        if let Some(data) = diag.clone().data {
            if let Ok(data) = serde_json::from_value::<DiagnosticData>(data) {
                return Some((diag.range, data));
            }
        }
        None
    });

    import_code_action(ctx, &mut diagnostics_with_data.clone())
        .into_iter()
        .chain(qualify_code_action(ctx, &mut diagnostics_with_data.clone()))
        .reduce(|mut combined, mut curr| {
            combined.append(&mut curr);
            combined
        })
}
