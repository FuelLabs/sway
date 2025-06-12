pub mod cli;
pub mod doc;
pub mod render;
pub mod search;
pub mod tests;

use anyhow::{bail, Result};
use cli::Command;
use doc::Documentation;
use forc_pkg as pkg;
use forc_pkg::{
    manifest::{GenericManifestFile, ManifestFile},
    PackageManifestFile,
};
use forc_tracing::println_action_green;
use forc_util::default_output_directory;
use render::RenderedDocumentation;
use std::sync::Arc;
use std::{
    fs,
    path::{Path, PathBuf},
};
use sway_core::{
    language::ty::{TyProgram, TyProgramKind},
    BuildTarget, Engines,
};
use sway_features::ExperimentalFeatures;

pub const ASSETS_DIR_NAME: &str = "static.files";

/// Information passed to the render phase to get TypeInfo, CallPath or visibility for type anchors.
#[derive(Clone)]
pub struct RenderPlan<'e> {
    no_deps: bool,
    document_private_items: bool,
    engines: &'e Engines,
}

impl<'e> RenderPlan<'e> {
    pub fn new(
        no_deps: bool,
        document_private_items: bool,
        engines: &'e Engines,
    ) -> RenderPlan<'e> {
        Self {
            no_deps,
            document_private_items,
            engines,
        }
    }
}

pub struct ProgramInfo<'a> {
    pub ty_program: Arc<TyProgram>,
    pub engines: &'a Engines,
    pub manifest: &'a ManifestFile,
    pub pkg_manifest: &'a PackageManifestFile,
    pub members: &'a Vec<String>,
}

pub fn compile_html(
    build_instructions: &Command,
    get_doc_dir: &dyn Fn(&Command) -> String,
) -> Result<(PathBuf, Box<PackageManifestFile>)> {
    // get manifest directory
    let dir = if let Some(ref path) = build_instructions.path {
        PathBuf::from(path)
    } else {
        std::env::current_dir()?
    };
    let manifest = ManifestFile::from_dir(dir.clone())?;

    let (members, pkg_manifest) = match manifest.clone() {
        ManifestFile::Package(f) => (Vec::<String>::new(), f),
        ManifestFile::Workspace(f) => {
            // Get members
            let members = f
                .members()
                .map(|p| p.to_string_lossy().to_string())
                .collect::<Vec<_>>();

            // Extract the first package manifest from the workspace
            let member_pkgs = f.member_pkg_manifests()?;
            let collected_pkgs: Result<Vec<_>, _> = member_pkgs.collect();

            (members, Box::new(collected_pkgs?[0].clone()))
        }
    };

    // create doc path
    let out_path = default_output_directory(manifest.dir());
    let doc_dir = get_doc_dir(build_instructions);
    let doc_path = out_path.join(doc_dir);
    if doc_path.exists() {
        std::fs::remove_dir_all(&doc_path)?;
    }
    fs::create_dir_all(&doc_path)?;

    println_action_green(
        "Compiling",
        &format!(
            "{} ({})",
            pkg_manifest.project_name(),
            manifest.dir().to_string_lossy()
        ),
    );

    let member_manifests = manifest.member_manifests()?;
    let lock_path = manifest.lock_path()?;

    let ipfs_node = build_instructions.ipfs_node.clone().unwrap_or_default();
    let plan = pkg::BuildPlan::from_lock_and_manifests(
        &lock_path,
        &member_manifests,
        build_instructions.locked,
        build_instructions.offline,
        &ipfs_node,
    )?;

    let engines = Engines::default();
    let tests_enabled = build_instructions.document_private_items;
    let mut compile_results = pkg::check(
        &plan,
        BuildTarget::default(),
        build_instructions.silent,
        None,
        tests_enabled,
        &engines,
        None,
        &build_instructions.experimental.experimental,
        &build_instructions.experimental.no_experimental,
        sway_core::DbgGeneration::Full,
    )?;

    let raw_docs = if build_instructions.no_deps {
        let Some(ty_program) = compile_results
            .pop()
            .and_then(|(programs, _handler)| programs)
            .and_then(|p| p.typed.ok())
        else {
            bail! {
                "documentation could not be built from manifest located at '{}'",
                pkg_manifest.path().display()
            }
        };
        let program_info = ProgramInfo {
            ty_program,
            engines: &engines,
            manifest: &manifest,
            pkg_manifest: &pkg_manifest,
            members: &members,
        };
        build_docs(program_info, &doc_path, build_instructions)?
    } else {
        let order = plan.compilation_order();
        let graph = plan.graph();
        let manifest_map = plan.manifest_map();
        let mut raw_docs = Documentation(Vec::new());

        for (node, (compile_result, _handler)) in order.iter().zip(compile_results) {
            let id = &graph[*node].id();
            if let Some(pkg_manifest_file) = manifest_map.get(id) {
                let manifest_file = ManifestFile::from_dir(pkg_manifest_file.path())?;
                let Some(ty_program) = compile_result.and_then(|programs| programs.typed.ok())
                else {
                    bail!(
                        "documentation could not be built from manifest located at '{}'",
                        pkg_manifest_file.path().display()
                    )
                };
                let program_info = ProgramInfo {
                    ty_program,
                    engines: &engines,
                    manifest: &manifest_file,
                    pkg_manifest: pkg_manifest_file,
                    members: &members,
                };
                raw_docs
                    .0
                    .extend(build_docs(program_info, &doc_path, build_instructions)?.0);
            }
        }
        raw_docs
    };
    search::write_search_index(&doc_path, &raw_docs)?;

    Ok((doc_path, pkg_manifest.to_owned()))
}

fn build_docs(
    program_info: ProgramInfo,
    doc_path: &Path,
    build_instructions: &Command,
) -> Result<Documentation> {
    let Command {
        document_private_items,
        no_deps,
        experimental,
        ..
    } = build_instructions;

    let ProgramInfo {
        ty_program,
        engines,
        manifest,
        pkg_manifest,
        members,
    } = program_info;

    match ty_program.kind.clone() {
        TyProgramKind::Library { .. } => {
            let experimental = ExperimentalFeatures::new(
                &pkg_manifest.project.experimental,
                &experimental.experimental,
                &experimental.no_experimental,
            )
            .map_err(|err| anyhow::anyhow!("{err}"))?;

            println_action_green(
                "Building",
                &format!(
                    "documentation for {} ({})",
                    pkg_manifest.project_name(),
                    manifest.dir().to_string_lossy()
                ),
            );

            let raw_docs = Documentation::from_ty_program(
                engines,
                pkg_manifest.project_name(),
                &ty_program,
                *document_private_items,
                experimental,
            )?;
            let root_attributes = (!ty_program.root_module.attributes.is_empty())
                .then_some(ty_program.root_module.attributes.clone());
            let forc_version = pkg_manifest
                .project
                .forc_version
                .as_ref()
                .map(|ver| format!("Forc v{}.{}.{}", ver.major, ver.minor, ver.patch));
            // render docs to HTML
            let rendered_docs = RenderedDocumentation::from_raw_docs(
                raw_docs.clone(),
                RenderPlan::new(*no_deps, *document_private_items, engines),
                root_attributes,
                &ty_program.kind,
                forc_version,
                members.clone(),
            )?;

            // write file contents to doc folder
            write_content(rendered_docs, doc_path)?;
            println_action_green("Finished", pkg_manifest.project_name());

            Ok(raw_docs)
        }
        _ => bail!(
            "forc-doc only supports libraries. '{}' is not a library.",
            pkg_manifest.project_name()
        ),
    }
}

fn write_content(rendered_docs: RenderedDocumentation, doc_path: &Path) -> Result<()> {
    for doc in rendered_docs.0 {
        let mut doc_path = doc_path.to_path_buf();
        for prefix in doc.module_info.module_prefixes {
            doc_path.push(prefix);
        }
        fs::create_dir_all(&doc_path)?;
        doc_path.push(doc.html_filename);
        fs::write(&doc_path, doc.file_contents.0.as_bytes())?;
    }
    Ok(())
}

const DOC_DIR_NAME: &str = "doc";
pub fn get_doc_dir(_build_instructions: &Command) -> String {
    DOC_DIR_NAME.into()
}
