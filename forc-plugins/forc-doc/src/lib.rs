pub mod doc;
pub mod render;
pub mod search;

use anyhow::{bail, Result};
use clap::Parser;
use doc::{module::ModuleInfo, Documentation};
use forc_pkg::{
    self as pkg,
    manifest::{GenericManifestFile, ManifestFile},
    source::IPFSNode,
    PackageManifestFile, Programs,
};
use forc_tracing::println_action_green;
use forc_util::default_output_directory;
use render::{index::WorkspaceIndex, RenderedDocumentation, HTMLString, Renderable};
use std::{
    fs,
    path::{Path, PathBuf},
};
use sway_core::{language::ty::{TyProgram, TyProgramKind}, BuildTarget, Engines};
use sway_features::ExperimentalFeatures;

pub const DOC_DIR_NAME: &str = "doc";
pub const ASSETS_DIR_NAME: &str = "static.files";

forc_util::cli_examples! {
    crate::Command {
        [ Build the docs for a project in the current path => "forc doc"]
        [ Build the docs for a project in the current path and open it in the browser => "forc doc --open" ]
        [ Build the docs for a project located in another path => "forc doc --path {path}" ]
        [ Build the docs for the current project exporting private types => "forc doc --document-private-items" ]
        [ Build the docs offline without downloading any dependencies => "forc doc --offline" ]
    }
}

/// Forc plugin for building a Sway package's documentation
#[derive(Debug, Parser, Default)]
#[clap(
    name = "forc-doc",
    after_help = help(),
    version
)]
pub struct Command {
    /// Path to the project.
    ///
    /// If not specified, current working directory will be used.
    #[clap(short, long, alias = "manifest-path")]
    pub path: Option<String>,
    /// Include non-public items in the documentation.
    #[clap(long)]
    pub document_private_items: bool,
    /// Open the docs in a browser after building them.
    #[clap(long)]
    pub open: bool,
    /// Offline mode, prevents Forc from using the network when managing dependencies.
    /// Meaning it will only try to use previously downloaded dependencies.
    #[clap(long)]
    pub offline: bool,
    /// Requires that the Forc.lock file is up-to-date. If the lock file is missing, or it
    /// needs to be updated, Forc will exit with an error.
    #[clap(long)]
    pub locked: bool,
    /// Do not build documentation for dependencies.
    #[clap(long)]
    pub no_deps: bool,
    /// The IPFS Node to use for fetching IPFS sources.
    ///
    /// Possible values: FUEL, PUBLIC, LOCAL, <GATEWAY_URL>
    #[clap(long)]
    pub ipfs_node: Option<IPFSNode>,
    /// The path to the documentation output directory.
    ///
    /// If not specified, the default documentation output directory will be used.
    #[clap(long)]
    pub doc_path: Option<String>,
    #[clap(flatten)]
    pub experimental: sway_features::CliFields,
    /// Silent mode. Don't output any warnings or errors to the command line.
    #[clap(long, short = 's', action)]
    pub silent: bool,
}

/// Result of documentation generation, either for a single package or a workspace.
#[derive(Debug, Clone)]
pub enum DocResult {
    Package(Box<PackageManifestFile>),
    Workspace { name: String, libraries: Vec<String> },
}

/// Generate documentation for a given package or workspace.
pub fn generate_docs(opts: &Command) -> Result<(PathBuf, DocResult)> {
    let ctx = DocContext::from_options(opts)?;
    let mut compile_results = compile(&ctx, opts)?.collect::<Vec<_>>();
    let doc_result = compile_html(opts, &ctx, &mut compile_results)?;
    Ok((ctx.doc_path, doc_result))
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
    pub pkg_manifest: Option<Box<PackageManifestFile>>,
    pub doc_path: PathBuf,
    pub engines: Engines,
    pub build_plan: pkg::BuildPlan,
    pub is_workspace: bool,
    pub workspace_name: String,
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
        
        // Get workspace name for later use
        let workspace_name = std::env::current_dir()?
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("workspace")
            .to_string();
        
        // Handle Package vs Workspace manifests  
        let is_workspace = matches!(&manifest, ManifestFile::Workspace(_));
        
        // Get package manifest for single packages (None for workspaces)
        let pkg_manifest = match &manifest {
            ManifestFile::Package(pkg_manifest) => Some(pkg_manifest.clone()),
            ManifestFile::Workspace(_) => None,
        };

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
        
        // Check for empty workspaces
        if is_workspace && member_manifests.is_empty() {
            bail!("Workspace contains no members");
        }

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
            is_workspace,
            workspace_name,
        })
    }
}

pub fn compile(ctx: &DocContext, opts: &Command) -> Result<impl Iterator<Item = Option<Programs>>> {
    if ctx.is_workspace {
        println_action_green(
            "Compiling", 
            &format!("workspace ({})", ctx.manifest.dir().to_string_lossy())
        );
    } else if let Some(ref pkg_manifest) = ctx.pkg_manifest {
        println_action_green(
            "Compiling",
            &format!(
                "{} ({})",
                pkg_manifest.project_name(),
                ctx.manifest.dir().to_string_lossy()
            ),
        );
    }

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
) -> Result<DocResult> {
    let mut documented_libraries = Vec::new();

    let raw_docs = if opts.no_deps {
        if let Some(ref pkg_manifest) = ctx.pkg_manifest {
            // Single package mode
            let Some(ty_program) = compile_results
                .pop()
                .and_then(|programs| programs)
                .and_then(|p| p.typed.ok())
            else {
                bail! {
                    "documentation could not be built from manifest located at '{}'",
                    pkg_manifest.path().display()
                }
            };
            
            // Only document if it's a library
            if matches!(ty_program.kind, TyProgramKind::Library { .. }) {
                documented_libraries.push(pkg_manifest.project_name().to_string());
                build_docs(opts, ctx, &ty_program, &ctx.manifest, pkg_manifest)?
            } else {
                bail!(
                    "forc-doc only supports libraries. '{}' is not a library.",
                    pkg_manifest.project_name()
                );
            }
        } else {
            // Workspace mode with no_deps
            bail!("--no-deps flag is not meaningful for workspaces");
        }
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

                // Only document libraries
                if matches!(ty_program.kind, TyProgramKind::Library { .. }) {
                    documented_libraries.push(pkg_manifest_file.project_name().to_string());
                    raw_docs.0.extend(
                        build_docs(opts, ctx, &ty_program, &manifest_file, pkg_manifest_file)?.0,
                    );
                }
            }
        }
        raw_docs
    };
    
    // Create workspace index if this is a workspace
    if ctx.is_workspace && !documented_libraries.is_empty() {
        create_workspace_index(&ctx.doc_path, &documented_libraries, &ctx.engines, &ctx.workspace_name)?;
    }
    
    search::write_search_index(&ctx.doc_path, &raw_docs)?;

    let result = if ctx.is_workspace {
        DocResult::Workspace {
            name: ctx.workspace_name.clone(),
            libraries: documented_libraries,
        }
    } else if let Some(ref pkg_manifest) = ctx.pkg_manifest {
        DocResult::Package(pkg_manifest.clone())
    } else {
        unreachable!("Should have either workspace or package")
    };

    Ok(result)
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
        ty_program,
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

fn create_workspace_index(doc_path: &Path, documented_libraries: &[String], engines: &Engines, workspace_name: &str) -> Result<()> {
    // Create a workspace module info with the actual directory name
    let workspace_info = ModuleInfo::from_ty_module(
        vec![workspace_name.to_string()], // Use actual workspace name
        None,
    );
    
    // Create the workspace index
    let workspace_index = WorkspaceIndex::new(
        workspace_info,
        documented_libraries.to_vec(),
    );
    
    // Render using the existing infrastructure
    let render_plan = RenderPlan::new(false, false, engines);
    let rendered_content = workspace_index.render(render_plan)?;
    let html_content = HTMLString::from_rendered_content(rendered_content)?;
    
    fs::write(doc_path.join("index.html"), html_content.0.as_bytes())?;
    Ok(())
}
