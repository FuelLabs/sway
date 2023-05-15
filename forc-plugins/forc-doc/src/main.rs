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
use pkg::manifest::{Dependency, ManifestFile};
use std::{
    collections::BTreeMap,
    path::Path,
    process::Command as Process,
    sync::Arc,
    {fs, path::PathBuf},
};
use sway_core::{decl_engine::DeclEngine, BuildTarget, Engines, TypeEngine};

mod cli;
mod doc;
mod render;

/// Information passed to the render phase to get TypeInfo, CallPath or visibility for type anchors.
#[derive(Clone)]
struct RenderPlan {
    document_private_items: bool,
    type_engine: Arc<TypeEngine>,
    decl_engine: Arc<DeclEngine>,
}
impl RenderPlan {
    fn new(
        document_private_items: bool,
        type_engine: Arc<TypeEngine>,
        decl_engine: Arc<DeclEngine>,
    ) -> RenderPlan {
        Self {
            document_private_items,
            type_engine,
            decl_engine,
        }
    }
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
    fs::create_dir_all(&doc_path)?;

    // build core documentation
    let project_name = pkg_manifest.project_name();
    let forc_version = pkg_manifest
        .project
        .forc_version
        .as_ref()
        .map(|ver| format!("Forc v{}.{}.{}", ver.major, ver.minor, ver.patch));
    build_docs(
        &manifest,
        &doc_path,
        &build_instructions,
        project_name,
        forc_version,
    )?;

    if !build_instructions.no_deps {
        build_deps(&pkg_manifest.dependencies, &doc_path, &build_instructions)?;
    }

    // CSS, icons and logos
    static ASSETS_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/src/assets");
    const ASSETS_DIR_NAME: &str = "assets";
    let assets_path = doc_path.join(ASSETS_DIR_NAME);
    fs::create_dir_all(&assets_path)?;
    for file in ASSETS_DIR.files() {
        let asset_path = assets_path.join(file.path());
        fs::write(asset_path, file.contents())?;
    }
    // Sway syntax highlighting file
    const SWAY_HJS_FILENAME: &str = "highlight.js";
    let sway_hjs = std::include_bytes!("assets/highlight.js");
    fs::write(assets_path.join(SWAY_HJS_FILENAME), sway_hjs)?;

    // check if the user wants to open the doc in the browser
    // if opening in the browser fails, attempt to open using a file explorer
    if build_instructions.open {
        const BROWSER_ENV_VAR: &str = "BROWSER";
        let path = doc_path.join(project_name).join(INDEX_FILENAME);
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
    manifest: &ManifestFile,
    doc_path: &Path,
    build_instructions: &Command,
    project_name: &str,
    forc_version: Option<String>,
) -> Result<()> {
    let Command {
        document_private_items,
        offline,
        silent,
        locked,
        no_deps,
        ..
    } = *build_instructions;

    println!(
        "   {} {project_name} ({})",
        "Compiling".bold().yellow(),
        manifest.dir().to_string_lossy()
    );

    // compile the program and extract the docs
    let member_manifests = manifest.member_manifests()?;
    let lock_path = manifest.lock_path()?;
    let plan =
        pkg::BuildPlan::from_lock_and_manifests(&lock_path, &member_manifests, locked, offline)?;
    let type_engine = TypeEngine::default();
    let decl_engine = DeclEngine::default();
    let engines = Engines::new(&type_engine, &decl_engine);
    let tests_enabled = true;
    let typed_program = match pkg::check(
        &plan,
        BuildTarget::default(),
        silent,
        tests_enabled,
        engines,
        true,
    )?
    .pop()
    .and_then(|compilation| compilation.value)
    .and_then(|programs| programs.typed)
    {
        Some(typed_program) => typed_program,
        _ => bail!("CompileResult returned None"),
    };

    println!(
        "    {} {project_name} documentation",
        "Building".bold().yellow()
    );

    let raw_docs = Documentation::from_ty_program(
        &decl_engine,
        project_name,
        &typed_program,
        no_deps,
        document_private_items,
    )?;
    let root_attributes =
        (!typed_program.root.attributes.is_empty()).then_some(typed_program.root.attributes);
    let program_kind = typed_program.kind;
    // render docs to HTML
    let rendered_docs = RenderedDocumentation::from_raw_docs(
        raw_docs,
        RenderPlan::new(
            document_private_items,
            Arc::from(type_engine),
            Arc::from(decl_engine),
        ),
        root_attributes,
        program_kind,
        forc_version,
    )?;

    // write file contents to doc folder
    write_content(rendered_docs, doc_path)?;
    println!("    {}", "Finished".bold().yellow());

    Ok(())
}

fn build_deps(
    dependencies: &Option<BTreeMap<String, Dependency>>,
    doc_path: &Path,
    build_instructions: &Command,
) -> Result<()> {
    if let Some(deps) = dependencies {
        for (dep_name, dep) in deps {
            if let Dependency::Detailed(dep_details) = dep {
                if let Some(path) = &dep_details.path {
                    let dep_manifest = ManifestFile::from_dir(&PathBuf::from(path))?;
                    let dep_pkg_manifest =
                        if let ManifestFile::Package(pkg_manifest) = &dep_manifest {
                            pkg_manifest
                        } else {
                            bail!("forc-doc does not support workspaces.")
                        };
                    let project_name = dep_pkg_manifest.project_name();
                    let forc_version = dep_pkg_manifest
                        .project
                        .forc_version
                        .as_ref()
                        .map(|ver| format!("Forc v{}.{}.{}", ver.major, ver.minor, ver.patch));
                    build_docs(
                        &dep_manifest,
                        doc_path,
                        build_instructions,
                        project_name,
                        forc_version,
                    )?;
                } else {
                    println!("a path variable was not set for {dep_name}, which is currently the only supported option.")
                }
            } else {
                println!("{dep_name} is a simple format dependency,\nsimple format dependencies don't specify a path to a manfiest file and are unsupported at this time.")
            }
        }
    }

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
