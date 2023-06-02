use crate::{
    doc::Documentation,
    render::{constant::INDEX_FILENAME, RenderedDocumentation},
};
use anyhow::{bail, Result};
use clap::Parser;
use cli::Command;
use colored::*;
use forc_pkg as pkg;
use forc_util::default_output_directory;
use include_dir::{include_dir, Dir};
use pkg::{manifest::ManifestFile, PackageManifestFile};
use std::{
    path::Path,
    process::Command as Process,
    {fs, path::PathBuf},
};
use sway_core::{
    decl_engine::DeclEngine, language::ty::TyProgram, query_engine::QueryEngine, BuildTarget,
    Engines, TypeEngine,
};

mod cli;
mod doc;
mod render;

pub(crate) const ASSETS_DIR_NAME: &str = "static.files";

/// Information passed to the render phase to get TypeInfo, CallPath or visibility for type anchors.
#[derive(Clone)]
struct RenderPlan<'e> {
    document_private_items: bool,
    type_engine: &'e TypeEngine,
    decl_engine: &'e DeclEngine,
}
impl<'e> RenderPlan<'e> {
    fn new(
        document_private_items: bool,
        type_engine: &'e TypeEngine,
        decl_engine: &'e DeclEngine,
    ) -> RenderPlan<'e> {
        Self {
            document_private_items,
            type_engine,
            decl_engine,
        }
    }
}
struct ProgramInfo<'a> {
    ty_program: TyProgram,
    type_engine: &'a TypeEngine,
    decl_engine: &'a DeclEngine,
    manifest: &'a ManifestFile,
    pkg_manifest: &'a PackageManifestFile,
}

/// Main method for `forc doc`.
pub fn main() -> Result<()> {
    let build_instructions = Command::parse();

    // get manifest directory
    let dir = if let Some(ref path) = build_instructions.manifest_path {
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
    let out_path = default_output_directory(manifest.dir());
    let doc_path = out_path.join(DOC_DIR_NAME);
    if doc_path.exists() {
        std::fs::remove_dir_all(&doc_path)?;
    }
    fs::create_dir_all(&doc_path)?;

    println!(
        "   {} {} ({})",
        "Compiling".bold().yellow(),
        pkg_manifest.project_name(),
        manifest.dir().to_string_lossy()
    );

    let member_manifests = manifest.member_manifests()?;
    let lock_path = manifest.lock_path()?;
    let plan = pkg::BuildPlan::from_lock_and_manifests(
        &lock_path,
        &member_manifests,
        build_instructions.locked,
        build_instructions.offline,
    )?;

    let type_engine = TypeEngine::default();
    let decl_engine = DeclEngine::default();
    let query_engine = QueryEngine::default();
    let engines = Engines::new(&type_engine, &decl_engine, &query_engine);
    let tests_enabled = build_instructions.document_private_items;
    let mut compile_results = pkg::check(
        &plan,
        BuildTarget::default(),
        build_instructions.silent,
        tests_enabled,
        engines,
    )?;

    if !build_instructions.no_deps {
        let order = plan.compilation_order();
        let graph = plan.graph();
        let manifest_map = plan.manifest_map();

        for (node, compile_result) in order.iter().zip(compile_results) {
            let id = &graph[*node].id();

            if let Some(pkg_manifest_file) = manifest_map.get(id) {
                let manifest_file = ManifestFile::from_dir(pkg_manifest_file.path())?;
                let ty_program = match compile_result.value.and_then(|programs| programs.typed) {
                    Some(ty_program) => ty_program,
                    _ => bail!(
                        "documentation could not be built from manifest located at '{}'",
                        pkg_manifest_file.path().display()
                    ),
                };
                let program_info = ProgramInfo {
                    ty_program,
                    type_engine: &type_engine,
                    decl_engine: &decl_engine,
                    manifest: &manifest_file,
                    pkg_manifest: pkg_manifest_file,
                };

                build_docs(program_info, &doc_path, &build_instructions)?;
            }
        }
    } else {
        let ty_program = match compile_results
            .pop()
            .and_then(|compilation| compilation.value)
            .and_then(|programs| programs.typed)
        {
            Some(ty_program) => ty_program,
            _ => bail!(
                "documentation could not be built from manifest located at '{}'",
                pkg_manifest.path().display()
            ),
        };
        let program_info = ProgramInfo {
            ty_program,
            type_engine: &type_engine,
            decl_engine: &decl_engine,
            manifest: &manifest,
            pkg_manifest,
        };
        build_docs(program_info, &doc_path, &build_instructions)?;
    }

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
        let path = doc_path
            .join(pkg_manifest.project_name())
            .join(INDEX_FILENAME);
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

fn build_docs(
    program_info: ProgramInfo,
    doc_path: &Path,
    build_instructions: &Command,
) -> Result<()> {
    let Command {
        document_private_items,
        ..
    } = *build_instructions;
    let ProgramInfo {
        ty_program,
        type_engine,
        decl_engine,
        manifest,
        pkg_manifest,
    } = program_info;

    println!(
        "    {} documentation for {} ({})",
        "Building".bold().yellow(),
        pkg_manifest.project_name(),
        manifest.dir().to_string_lossy()
    );

    let raw_docs = Documentation::from_ty_program(
        decl_engine,
        pkg_manifest.project_name(),
        &ty_program,
        document_private_items,
    )?;
    let root_attributes =
        (!ty_program.root.attributes.is_empty()).then_some(ty_program.root.attributes);
    let forc_version = pkg_manifest
        .project
        .forc_version
        .as_ref()
        .map(|ver| format!("Forc v{}.{}.{}", ver.major, ver.minor, ver.patch));
    // render docs to HTML
    let rendered_docs = RenderedDocumentation::from_raw_docs(
        raw_docs,
        RenderPlan::new(document_private_items, type_engine, decl_engine),
        root_attributes,
        ty_program.kind,
        forc_version,
    )?;

    // write file contents to doc folder
    write_content(rendered_docs, doc_path)?;
    println!("    {}", "Finished".bold().yellow());

    Ok(())
}

fn write_content(rendered_docs: RenderedDocumentation, doc_path: &Path) -> Result<()> {
    for doc in rendered_docs.0 {
        let mut doc_path = doc_path.to_path_buf();
        for prefix in doc.module_info.module_prefixes {
            doc_path.push(prefix)
        }

        fs::create_dir_all(&doc_path)?;
        doc_path.push(doc.html_filename);
        fs::write(&doc_path, doc.file_contents.0.as_bytes())?;
    }

    Ok(())
}
