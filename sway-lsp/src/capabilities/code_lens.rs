use std::{path::PathBuf, sync::Arc};

use lsp_types::{CodeLens, Url};

use crate::core::session::Session;

pub fn code_lens(session: &Arc<Session>, url: &Url) -> Vec<CodeLens> {
    let url_path = PathBuf::from(url.path());

    // Construct code lenses for runnable functions
    let runnables_for_path = session.runnables.get(&url_path);
    runnables_for_path
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
        .unwrap_or_default()
}
