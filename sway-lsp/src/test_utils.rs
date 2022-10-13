use std::{env, path::PathBuf};

use tower_lsp::lsp_types::Url;

pub fn sway_workspace_dir() -> PathBuf {
    env::current_dir().unwrap().parent().unwrap().to_path_buf()
}

pub fn get_absolute_path(path: &str) -> String {
    sway_workspace_dir().join(path).to_str().unwrap().into()
}

pub fn get_url(absolute_path: &str) -> Url {
    Url::parse(&format!("file://{}", &absolute_path)).expect("expected URL")
}
