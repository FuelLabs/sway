mod cli;
mod descriptor;
mod doc;
mod render;

use anyhow::{bail, Result};
use clap::Parser;
use cli::Command;
use pkg::manifest::ManifestFile;
use std::{
    process::Command as Process,
    {fs, path::PathBuf},
};

use crate::{
    doc::{Document, Documentation},
    render::{RenderedDocument, RenderedDocumentation},
};
use forc_pkg::{self as pkg};
use forc_util::default_output_directory;

/// Main method for `forc doc`.
pub fn main() -> Result<()> {
    let Command {
        manifest_path,
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

    // check if the out path exists
    let project_name = &pkg_manifest.project.name;
    let out_path = default_output_directory(manifest.dir());
    let doc_path = out_path.join("doc");
    fs::create_dir_all(&doc_path)?;

    // compile the program and extract the docs
    let member_manifests = manifest.member_manifests()?;
    let lock_path = manifest.lock_path()?;
    let plan =
        pkg::BuildPlan::from_lock_and_manifests(&lock_path, &member_manifests, locked, offline)?;
    let compilation = pkg::check(&plan, silent)?;
    let raw_docs: Documentation = Document::from_ty_program(&compilation, no_deps)?;
    // render docs to HTML
    let rendered_docs: RenderedDocumentation =
        RenderedDocument::from_raw_docs(&raw_docs, project_name);

    // write to outfile
    for doc in rendered_docs {
        let mut doc_path = doc_path.clone();
        for prefix in doc.module_prefix {
            doc_path.push(prefix);
        }

        fs::create_dir_all(&doc_path)?;
        doc_path.push(doc.file_name);
        fs::write(&doc_path, doc.file_contents.0.as_bytes())?;
    }

    // check if the user wants to open the doc in the browser
    // if opening in the browser fails, attempt to open using a file explorer
    if open_result {
        let path = doc_path.join("all.html");
        let default_browser_opt = std::env::var_os("BROWSER");
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
