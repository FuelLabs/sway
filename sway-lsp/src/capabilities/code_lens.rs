use crate::server_state::RunnableMap;
use lsp_types::{CodeLens, Url};
use std::path::PathBuf;

pub fn code_lens(runnables: &RunnableMap, url: &Url) -> Vec<CodeLens> {
    let _p = tracing::trace_span!("code_lens").entered();
    let url_path = PathBuf::from(url.path());

    // Construct code lenses for runnable functions
    let runnables_for_path = runnables.get(&url_path);

    let mut result: Vec<CodeLens> = runnables_for_path
        .map(|runnables| {
            runnables
                .iter()
                .map(|runnable| CodeLens {
                    range: *runnable.range(),
                    command: Some(runnable.command()),
                    data: None,
                })
                .collect()
        })
        .unwrap_or_default();
    // Sort the results
    result.sort_by(|a, b| a.range.start.line.cmp(&b.range.start.line));
    result
}
