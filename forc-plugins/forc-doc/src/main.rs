mod cli;
mod descriptor;
mod doc;
mod render;

use crate::{
    doc::{Document, Documentation},
    render::{RenderedDocument, RenderedDocumentation, ALL_DOC_FILENAME},
};
use anyhow::{bail, Result};
use clap::Parser;
use cli::Command;
use forc_pkg as pkg;
use forc_util::default_output_directory;
use include_dir::{include_dir, Dir};
use pkg::manifest::ManifestFile;
use std::{
    process::Command as Process,
    {fs, path::PathBuf},
};
use sway_core::TypeEngine;

/// Main method for `forc doc`.
pub fn main() -> Result<()> {
    let Command {
        manifest_path,
        document_private_items,
        open: open_result,
        offline,
        silent,
        locked,
        no_deps,
    } = Command::parse();

    // get manifest directory
    let dir = if let Some(ref path) = manifest_path {
        PathBuf::from(path)
    } else {
        std::env::current_dir()?
    };
    let manifest = ManifestFile::from_dir(&dir)?;
    let pkg_manifest = if let ManifestFile::Package(pkg_manifest) = &manifest {
        pkg_manifest
    } else {
        bail!("forc-doc does not support workspaces.")
    };

    // create doc path
    const DOC_DIR_NAME: &str = "doc";
    let project_name = &pkg_manifest.project.name;
    let out_path = default_output_directory(manifest.dir());
    let doc_path = out_path.join(DOC_DIR_NAME);
    fs::create_dir_all(&doc_path)?;

    // compile the program and extract the docs
    let member_manifests = manifest.member_manifests()?;
    let lock_path = manifest.lock_path()?;
    let plan =
        pkg::BuildPlan::from_lock_and_manifests(&lock_path, &member_manifests, locked, offline)?;
    let type_engine = TypeEngine::default();
    let compilation = pkg::check(&plan, silent, &type_engine)?
        .pop()
        .expect("there is guaranteed to be at least one elem in the vector");
    let raw_docs: Documentation =
        Document::from_ty_program(&compilation, no_deps, document_private_items)?;
    // render docs to HTML
    let rendered_docs: RenderedDocumentation =
        RenderedDocument::from_raw_docs(&raw_docs, project_name);

    // write contents to outfile
    for doc in rendered_docs {
        let mut doc_path = doc_path.clone();
        for prefix in doc.module_prefix {
            doc_path.push(prefix);
        }

        fs::create_dir_all(&doc_path)?;
        doc_path.push(doc.file_name);
        fs::write(&doc_path, doc.file_contents.0.as_bytes())?;
    }
    // CSS, icons and logos
    static ASSETS_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/src/assets");
    const ASSETS_DIR_NAME: &str = "assets";
    let assets_path = doc_path.join(ASSETS_DIR_NAME);
    fs::create_dir_all(&assets_path)?;
    for file in ASSETS_DIR.files() {
        let asset_path = assets_path.join(file.path());
        fs::write(&asset_path, file.contents())?;
    }
    // Sway syntax highlighting file
    const SWAY_HJS_FILENAME: &str = "sway.js";
    let sway_hjs = std::include_bytes!("../../../scripts/highlightjs/sway.js");
    fs::write(assets_path.join(SWAY_HJS_FILENAME), sway_hjs)?;

    // check if the user wants to open the doc in the browser
    // if opening in the browser fails, attempt to open using a file explorer
    if open_result {
        const BROWSER_ENV_VAR: &str = "BROWSER";
        let path = doc_path.join(ALL_DOC_FILENAME);
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
