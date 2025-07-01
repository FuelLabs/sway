use anyhow::{bail, Result};
use clap::Parser;
use forc_doc::{generate_docs, render::INDEX_FILENAME, Command, ASSETS_DIR_NAME, DocResult};
use include_dir::{include_dir, Dir};
use std::{
    process::Command as Process,
    {fs, path::PathBuf},
};

pub fn main() -> Result<()> {
    let build_instructions = Command::parse();
    let (doc_path, doc_result) = generate_docs(&build_instructions)?;

    // CSS, icons and logos
    static ASSETS_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/src/static.files");
    let assets_path = doc_path.join(ASSETS_DIR_NAME);
    fs::create_dir_all(&assets_path)?;
    for file in ASSETS_DIR.files() {
        let asset_path = assets_path.join(file.path());
        fs::write(asset_path, file.contents())?;
    }
    // Sway syntax highlighting file
    const SWAY_HJS_FILENAME: &str = "highlight.js";
    let sway_hjs = std::include_bytes!("static.files/highlight.js");
    fs::write(assets_path.join(SWAY_HJS_FILENAME), sway_hjs)?;

    // check if the user wants to open the doc in the browser
    // if opening in the browser fails, attempt to open using a file explorer
    if build_instructions.open {
        const BROWSER_ENV_VAR: &str = "BROWSER";
        let path = match &doc_result {
            DocResult::Workspace { .. } => {
                // For workspaces, open the root index.html
                doc_path.join(INDEX_FILENAME)
            }
            DocResult::Package(pkg_manifest) => {
                // For single packages, open the package-specific index
                doc_path
                    .join(pkg_manifest.project_name())
                    .join(INDEX_FILENAME)
            }
        };
        let default_browser_opt = std::env::var_os(BROWSER_ENV_VAR);
        match default_browser_opt {
            Some(def_browser) => {
                let browser = PathBuf::from(def_browser);
                if let Err(e) = Process::new(&browser).arg(path).status() {
                    bail!(
                        "Couldn't open docs with {}: {}",
                        browser.to_string_lossy(),
                        e
                    );
                }
            }
            None => {
                if let Err(e) = opener::open(&path) {
                    bail!("Couldn't open docs: {}", e);
                }
            }
        }
    }
    Ok(())
}
