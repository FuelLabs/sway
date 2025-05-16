use clap::Parser;
use forc_util::{find_manifest_dir, ForcToml, ManifestFile};
use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Debug, Clone, Parser)]
pub struct Args {
    /// Path to the project or workspace root
    #[clap(long, value_parser)]
    pub path: Option<PathBuf>,

    /// Output directory for the generated documentation
    #[clap(long, value_parser)]
    pub output: Option<PathBuf>,

    /// Include private items in the documentation
    #[clap(long)]
    pub include_private: bool,

    /// Open the documentation in a web browser after generation
    #[clap(long)]
    pub open: bool,
}

fn main() {
    let args = Args::parse();

    let manifest_dir = match &args.path {
        Some(path) => path.clone(),
        None => find_manifest_dir(std::env::current_dir().unwrap())
            .expect("Failed to find Forc.toml in current or parent directories"),
    };

    if is_workspace(&manifest_dir) {
        handle_workspace(&manifest_dir, &args);
    } else {
        handle_single_project(&manifest_dir, &args);
    }
}

/// Detect if the path is a workspace by checking for `[workspace]` in Forc.toml
fn is_workspace(path: &Path) -> bool {
    let forc_toml_path = path.join("Forc.toml");
    if let Ok(content) = fs::read_to_string(forc_toml_path) {
        content.contains("[workspace]")
    } else {
        false
    }
}

/// Generate docs for a single project
fn handle_single_project(project_path: &Path, args: &Args) {
    let mut command = Command::new("forc");
    command.arg("doc");

    if let Some(path) = &args.path {
        command.arg("--path").arg(path);
    }

    if let Some(output) = &args.output {
        command.arg("--output").arg(output);
    }

    if args.include_private {
        command.arg("--include-private");
    }

    if args.open {
        command.arg("--open");
    }

    let status = command.current_dir(project_path).status().unwrap();

    if !status.success() {
        eprintln!("❌ Failed to generate docs for project at {:?}", project_path);
    }
}

/// Handle workspace: extract members and build docs for each
fn handle_workspace(workspace_path: &Path, args: &Args) {
    let forc_toml_path = workspace_path.join("Forc.toml");
    let content = fs::read_to_string(&forc_toml_path)
        .expect("Failed to read Forc.toml for workspace");

    let toml: toml::Value = toml::from_str(&content).expect("Invalid TOML format");
    let members = toml
        .get("workspace")
        .and_then(|w| w.get("members"))
        .and_then(|m| m.as_array())
        .expect("Workspace must have 'members' array in Forc.toml");

    let mut generated_projects = vec![];

    for member in members {
        if let Some(member_str) = member.as_str() {
            let member_path = workspace_path.join(member_str);
            let output_dir = args
                .output
                .clone()
                .unwrap_or_else(|| workspace_path.join("docs").join(member_str));

            let mut command = Command::new("forc");
            command.arg("doc");
            command.arg("--path").arg(&member_path);
            command.arg("--output").arg(&output_dir);

            if args.include_private {
                command.arg("--include-private");
            }

            // Don't open browser for each member
            let status = command.current_dir(workspace_path).status().unwrap();

            if status.success() {
                println!("✅ Generated docs for {member_str}");
                generated_projects.push((member_str.to_string(), output_dir));
            } else {
                eprintln!("❌ Failed to generate docs for {member_str}");
            }
        }
    }

    if !generated_projects.is_empty() {
        create_workspace_index(workspace_path, &generated_projects);

        if args.open {
            let index_path = workspace_path.join("docs/index.html");
            let _ = opener::open(index_path);
        }
    }
}

/// Generate a top-level HTML index for the workspace
fn create_workspace_index(workspace_path: &Path, members: &[(String, PathBuf)]) {
    let index_path = workspace_path.join("docs/index.html");
    let mut file = File::create(&index_path).expect("Failed to create workspace index.html");

    writeln!(
        file,
        "<html><head><title>Workspace Documentation</title></head><body><h1>Workspace Documentation</h1><ul>"
    )
    .unwrap();

    for (name, _) in members {
        writeln!(
            file,
            "<li><a href=\"./{name}/index.html\">{name}</a></li>"
        )
        .unwrap();
    }

    writeln!(file, "</ul></body></html>").unwrap();

    println!("📘 Workspace docs index generated at: {:?}", index_path);
}
