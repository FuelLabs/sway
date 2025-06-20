pub mod cli;
pub mod doc;
pub mod render;
pub mod search;

use anyhow::{bail, Result};
use cli::Command;
use doc::Documentation;
use forc_pkg::{
    self as pkg,
    manifest::{GenericManifestFile, ManifestFile},
    PackageManifestFile, Programs,
};
use forc_tracing::println_action_green;
use forc_util::default_output_directory;
use render::RenderedDocumentation;
use std::{
    fs,
    path::{Path, PathBuf},
};
use sway_core::{language::ty::TyProgram, BuildTarget, Engines};
use sway_features::ExperimentalFeatures;

pub const DOC_DIR_NAME: &str = "doc";
pub const ASSETS_DIR_NAME: &str = "static.files";

/// Generate documentation for a given package.
pub fn generate_docs(opts: &Command) -> Result<DocContext> {
    let ctx = DocContext::from_options(opts)?;
    let mut compile_results = compile(&ctx, opts)?.collect::<Vec<_>>();
    compile_html(opts, &ctx, &mut compile_results)?;
    Ok(ctx)
}

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

pub struct DocContext {
    pub manifest: ManifestFile,
    pub pkg_manifest: Box<PackageManifestFile>,
    pub doc_path: PathBuf,
    pub engines: Engines,
    pub build_plan: pkg::BuildPlan,
}

impl DocContext {
    pub fn from_options(opts: &Command) -> Result<Self> {
        // get manifest directory
        let dir = if let Some(ref path) = opts.path {
            PathBuf::from(path)
        } else {
            std::env::current_dir()?
        };
        let manifest = ManifestFile::from_dir(dir)?;
        let ManifestFile::Package(pkg_manifest) = &manifest else {
            bail!("forc-doc does not support workspaces.")
        };
        let pkg_manifest = pkg_manifest.clone();

        // create doc path
        let out_path = default_output_directory(manifest.dir());
        let doc_dir = opts
            .doc_path
            .clone()
            .unwrap_or_else(|| DOC_DIR_NAME.to_string());
        let doc_path = out_path.join(doc_dir);
        if doc_path.exists() {
            std::fs::remove_dir_all(&doc_path)?;
        }
        fs::create_dir_all(&doc_path)?;

        // Build Plan
        let member_manifests = manifest.member_manifests()?;
        let lock_path = manifest.lock_path()?;

        let ipfs_node = opts.ipfs_node.clone().unwrap_or_default();
        let build_plan = pkg::BuildPlan::from_lock_and_manifests(
            &lock_path,
            &member_manifests,
            opts.locked,
            opts.offline,
            &ipfs_node,
        )?;

        Ok(Self {
            manifest,
            pkg_manifest,
            doc_path,
            engines: Engines::default(),
            build_plan,
        })
    }
}

pub fn compile(ctx: &DocContext, opts: &Command) -> Result<impl Iterator<Item = Option<Programs>>> {
    println_action_green(
        "Compiling",
        &format!(
            "{} ({})",
            ctx.pkg_manifest.project_name(),
            ctx.manifest.dir().to_string_lossy()
        ),
    );

    let tests_enabled = opts.document_private_items;
    pkg::check(
        &ctx.build_plan,
        BuildTarget::default(),
        opts.silent,
        None,
        tests_enabled,
        &ctx.engines,
        None,
        &opts.experimental.experimental,
        &opts.experimental.no_experimental,
        sway_core::DbgGeneration::Full,
    )
    .map(|results| results.into_iter().map(|(programs, _handler)| programs))
}

pub fn compile_html(
    opts: &Command,
    ctx: &DocContext,
    compile_results: &mut Vec<Option<Programs>>,
) -> Result<()> {
    let raw_docs = if opts.no_deps {
        let ty_program = compile_results
            .pop()
            .flatten()
            .and_then(|p| p.typed.ok())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "documentation could not be built from manifest located at '{}'",
                    ctx.pkg_manifest.path().display()
                )
            })?;
        build_docs(opts, ctx, &ty_program, &ctx.manifest, &ctx.pkg_manifest)?
    } else {
        let (order, graph, manifest_map) = (
            ctx.build_plan.compilation_order(),
            ctx.build_plan.graph(),
            ctx.build_plan.manifest_map(),
        );
        let mut raw_docs = Documentation(Vec::new());

        for (node, compile_result) in order.iter().zip(compile_results) {
            let id = &graph[*node].id();
            if let Some(pkg_manifest_file) = manifest_map.get(id) {
                let manifest_file = ManifestFile::from_dir(pkg_manifest_file.path())?;
                let ty_program = compile_result
                    .as_ref()
                    .and_then(|programs| programs.typed.clone().ok())
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "documentation could not be built from manifest located at '{}'",
                            pkg_manifest_file.path().display()
                        )
                    })?;

                raw_docs.0.extend(
                    build_docs(opts, ctx, &ty_program, &manifest_file, pkg_manifest_file)?.0,
                );
            }
        }
        raw_docs
    };

    search::write_search_index(&ctx.doc_path, &raw_docs)?;
    Ok(())
}

fn build_docs(
    opts: &Command,
    ctx: &DocContext,
    ty_program: &TyProgram,
    manifest: &ManifestFile,
    pkg_manifest: &PackageManifestFile,
) -> Result<Documentation> {
    let experimental = ExperimentalFeatures::new(
        &pkg_manifest.project.experimental,
        &opts.experimental.experimental,
        &opts.experimental.no_experimental,
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
        &ctx.engines,
        pkg_manifest.project_name(),
        &ty_program,
        opts.document_private_items,
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
        RenderPlan::new(opts.no_deps, opts.document_private_items, &ctx.engines),
        root_attributes,
        &ty_program.kind,
        forc_version,
    )?;

    // write file contents to doc folder
    write_content(rendered_docs, &ctx.doc_path)?;
    println_action_green("Finished", pkg_manifest.project_name());

    Ok(raw_docs)
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
