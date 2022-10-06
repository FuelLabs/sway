mod cli;
mod descriptor;
mod doc;
mod render;

use anyhow::Result;
use clap::Parser;
use cli::Command;
use std::{
    io::prelude::*,
    {fs, path::PathBuf},
};

use crate::{doc::get_compiled_docs, render::RenderedDocumentation};
use forc_pkg::{self as pkg, ManifestFile};

/// Main method for `forc doc`.
pub fn main() -> Result<()> {
    let Command {
        manifest_path,
        open: open_result,
        offline_mode: offline,
        silent_mode,
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

    // check if the out path exists
    let out_path = PathBuf::from(&manifest.dir()).join("out");
    let doc_path = out_path.join("doc");
    if !out_path.try_exists().unwrap_or(false) {
        // create the out path
        fs::create_dir_all(&doc_path)?;
    }

    // compile the program and extract the docs
    let plan = pkg::BuildPlan::from_lock_and_manifest(&manifest, locked, offline)?;
    let compilation = pkg::check(&plan, silent_mode)?;
    let docs = get_compiled_docs(&compilation, no_deps);
    // render docs to HTML
    let rendered = RenderedDocumentation::render(&docs);

    // write to outfile
    for entry in rendered {
        let mut doc_path = doc_path.clone();
        for prefix in entry.module_prefix {
            doc_path.push(prefix);
        }

        fs::create_dir_all(&doc_path)?;
        doc_path.push(entry.file_name);
        let mut file = fs::File::create(doc_path)?;
        file.write_all(dbg!(entry.file_contents.0).as_bytes())?;
    }

    // check if the user wants to open the doc in the browser
    if open_result {
        todo!("open in browser");
    }

    Ok(())
}
