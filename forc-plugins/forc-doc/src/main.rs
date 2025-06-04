use anyhow::{bail, Result};
use clap::Parser;
use forc_doc::{
    cli::Command as DeprecatedCommand, compile_html, get_doc_dir,
    render::constant::INDEX_FILENAME, DocumentationBuilderOptions, ASSETS_DIR_NAME,
};
use include_dir::{include_dir, Dir};
use std::{
    fs,
    path::{PathBuf},
    process::Command as Process,
};

mod workspace;
use workspace::{DocContext, WorkspaceMember};

#[derive(Debug, Parser)]
#[clap(
    name = "forc-doc",
    about = "Generate documentation for Sway projects",
    version
)]
struct Opt {
    /// Output directory for generated documentation
    #[clap(long, short = 'o', default_value = "doc")]
    output_dir: PathBuf,

    /// Include private items in documentation
    #[clap(long)]
    include_private: bool,

    /// Generate documentation for all workspace members
    #[clap(long)]
    workspace: bool,

    /// Path to the project directory
    #[clap(long, short = 'p', default_value = ".")]
    path: PathBuf,

    /// Open the generated docs in a browser
    #[clap(long)]
    open: bool,
}

// Assets directory (CSS, JS, etc.)
static ASSETS_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/src/static.files");
const SWAY_HJS_FILENAME: &str = "highlight.js";

fn main() -> Result<()> {
    let opt = Opt::parse();

    let context = DocContext::detect(&opt.path)?;

    match context {
        DocContext::Package(package_path) => {
            generate_package_docs(&package_path, &opt)?;
        }
        DocContext::Workspace { root, members } => {
            if opt.workspace {
                generate_workspace_docs(&root, &members, &opt)?;
            } else {
                println!("This directory contains a Sway workspace with {} members:", members.len());
                for member in &members {
                    println!("  - {}", member.name);
                }
                println!("\nTo generate documentation for all workspace members, use:");
                println!("  forc doc --workspace");
                println!("\nTo generate documentation for a specific member, navigate to its directory and run forc doc");
                return Ok(());
            }
        }
    }

    Ok(())
}

fn generate_package_docs(package_path: &PathBuf, opt: &Opt) -> Result<()> {
    println!("Generating documentation for package at {}", package_path.display());

    let options = DocumentationBuilderOptions {
        pkg_dir: package_path.clone(),
        output_dir: opt.output_dir.clone(),
        include_private: opt.include_private,
        ..Default::default()
    };

    let (doc_path, pkg_manifest) = compile_html(&options, &get_doc_dir)?;

    copy_assets(&doc_path)?;

    println!("Documentation generated at {}", doc_path.display());

    if opt.open {
        open_docs(&doc_path.join(pkg_manifest.project_name()).join(INDEX_FILENAME))?;
    }

    Ok(())
}

fn generate_workspace_docs(
    workspace_root: &PathBuf,
    members: &[WorkspaceMember],
    opt: &Opt,
) -> Result<()> {
    println!("Generating documentation for workspace with {} members", members.len());

    fs::create_dir_all(&opt.output_dir)?;

    for member in members {
        println!("Generating documentation for {}", member.name);

        let member_output_dir = opt.output_dir.join(&member.name);
        fs::create_dir_all(&member_output_dir)?;

        let member_options = DocumentationBuilderOptions {
            pkg_dir: member.path.clone(),
            output_dir: member_output_dir.clone(),
            include_private: opt.include_private,
            ..Default::default()
        };

        match compile_html(&member_options, &get_doc_dir) {
            Ok((member_doc_path, _)) => {
                copy_assets(&member_doc_path)?;
            }
            Err(e) => {
                eprintln!("Failed to generate documentation for {}: {}", member.name, e);
            }
        }
    }

    generate_workspace_index(workspace_root, members, &opt.output_dir)?;

    if opt.open {
        open_docs(&opt.output_dir.join("index.html"))?;
    }

    println!("Workspace documentation generated at {}", opt.output_dir.display());
    Ok(())
}

fn generate_workspace_index(
    _workspace_root: &PathBuf,
    members: &[WorkspaceMember],
    output_dir: &PathBuf,
) -> Result<()> {
    let index_path = output_dir.join("index.html");

    let html_content = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Workspace Documentation</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 40px; }}
        h1 {{ color: #333; }}
        .member {{ margin: 20px 0; padding: 15px; border: 1px solid #ddd; border-radius: 5px; }}
        .member h3 {{ margin: 0 0 10px 0; }}
        .member a {{ text-decoration: none; color: #0066cc; }}
        .member a:hover {{ text-decoration: underline; }}
    </style>
</head>
<body>
    <h1>Workspace Documentation</h1>
    <p>This workspace contains {} packages:</p>
    {}
</body>
</html>"#,
        members.len(),
        members
            .iter()
            .map(|member| format!(
                r#"<div class="member">
                    <h3><a href="{0}/index.html">{0}</a></h3>
                    <p>Documentation for the {0} package</p>
                </div>"#,
                member.name
            ))
            .collect::<Vec<_>>()
            .join("\n    ")
    );

    fs::write(index_path, html_content)?;
    Ok(())
}

fn copy_assets(doc_path: &PathBuf) -> Result<()> {
    let assets_path = doc_path.join(ASSETS_DIR_NAME);
    fs::create_dir_all(&assets_path)?;

    for file in ASSETS_DIR.files() {
        let asset_path = assets_path.join(file.path());
        fs::write(asset_path, file.contents())?;
    }

    let sway_hjs = include_bytes!("static.files/highlight.js");
    fs::write(assets_path.join(SWAY_HJS_FILENAME), sway_hjs)?;
    Ok(())
}

fn open_docs(path: &PathBuf) -> Result<()> {
    const BROWSER_ENV_VAR: &str = "BROWSER";
    match std::env::var_os(BROWSER_ENV_VAR) {
        Some(browser_var) => {
            let browser = PathBuf::from(browser_var);
            Process::new(&browser).arg(path).status().map_err(|e| {
                anyhow::anyhow!(
                    "Couldn't open docs with {}: {}",
                    browser.to_string_lossy(),
                    e
                )
            })?;
        }
        None => {
            opener::open(path).map_err(|e| anyhow::anyhow!("Couldn't open docs: {}", e))?;
        }
    }
    Ok(())
}
