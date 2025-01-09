use std::{path::PathBuf, sync::Arc};

use lsp_types::{CodeLens, Url};

use crate::{capabilities::runnable, core::session::Session};

pub fn code_lens(session: &Arc<Session>, url: &Url) -> Vec<CodeLens> {
    dbg!();
    let _p = tracing::trace_span!("code_lens").entered();
    dbg!();
    let url_path = PathBuf::from(url.path());
    dbg!(url_path.display().to_string());

    // Construct code lenses for runnable functions
    let runnables_for_path = session.runnables.get(&url_path);
    dbg!(runnables_for_path.is_some());

    let mut result: Vec<CodeLens> = runnables_for_path
        .map(|runnables| {
            dbg!(runnables.len());
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
