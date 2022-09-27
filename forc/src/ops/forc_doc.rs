use crate::cli::DocCommand;
use anyhow::Result;
use forc_pkg::{self as pkg, ManifestFile};
use std::path::{Path, PathBuf};
use sway_core::{
    AstNode, AstNodeContent, Attribute, AttributeKind, AttributesMap, CompileResult, Declaration,
    ParseProgram, ParseSubmodule, TypedProgram,
};

/// Main method for `forc doc`.
pub fn doc(command: DocCommand) -> Result<()> {
    let DocCommand {
        manifest_path,
        open: open_result,
        offline_mode: offline,
        silent_mode,
        locked,
        no_deps,
    } = command;

    let dir = if let Some(ref path) = manifest_path {
        PathBuf::from(path)
    } else {
        std::env::current_dir()?
    };
    let manifest = ManifestFile::from_dir(&dir)?;
    let plan = pkg::BuildPlan::from_lock_and_manifest(&manifest, locked, offline)?;

    let compilation = pkg::check(&plan, silent_mode)?;
    let _docs = get_compiled_docs(&compilation, no_deps);
    let out_path = Path::new(&dir).join("out");
    if out_path.try_exists().is_err() {
        // create the out path
    }

    // check if the user wants to open the doc in the browser
    if open_result {}

    Ok(())
}

fn attributes_map(ast_node: &AstNode) -> Option<AttributesMap> {
    match ast_node.content.clone() {
        AstNodeContent::Declaration(decl) => match decl {
            Declaration::EnumDeclaration(decl) => Some(decl.attributes),
            Declaration::FunctionDeclaration(decl) => Some(decl.attributes),
            Declaration::StructDeclaration(decl) => Some(decl.attributes),
            Declaration::ConstantDeclaration(decl) => Some(decl.attributes),
            Declaration::StorageDeclaration(decl) => Some(decl.attributes),
            _ => None,
        },
        _ => None,
    }
}
fn doc_attributes(ast_node: &AstNode) -> Option<Vec<Attribute>> {
    attributes_map(ast_node).and_then(|mut attributes| attributes.remove(&AttributeKind::Doc))
}
fn extract_submodule_docs(submodule: &ParseSubmodule, docs: &mut Vec<Option<Vec<Attribute>>>) {
    for ast_node in &submodule.module.tree.root_nodes {
        docs.push(doc_attributes(ast_node));
    }
    if !submodule.module.submodules.is_empty() {
        while let Some((_, submodule)) = submodule.module.submodules.first() {
            extract_submodule_docs(submodule, docs);
        }
    }
}
fn get_compiled_docs(
    compilation: &CompileResult<(ParseProgram, Option<TypedProgram>)>,
    no_deps: bool,
) -> Vec<Option<Vec<Attribute>>> {
    let mut docs = vec![];

    if let Some((parse_program, _)) = &compilation.value {
        for ast_node in &parse_program.root.tree.root_nodes {
            docs.push(doc_attributes(ast_node));
        }
        if !no_deps {
            while let Some((_, submodule)) = parse_program.root.submodules.first() {
                extract_submodule_docs(submodule, &mut docs);
            }
        }
    }

    docs
}

// From Cargo Doc:
//
// pub fn doc(ws: &Workspace<'_>, options: &DocOptions) -> CargoResult<()> {
//     // compile the workspace documentation and store it in `target/doc`
//     let compilation = ops::compile(ws, &options.compile_opts)?;

//     // check if the user wants to open the doc in the browser
//     if options.open_result {
//         let name = &compilation
//             .root_crate_names
//             .get(0)
//             .ok_or_else(|| anyhow::anyhow!("no crates with documentation"))?;
//         let kind = options.compile_opts.build_config.single_requested_kind()?;
//         let path = compilation.root_output[&kind]
//             .with_file_name("doc")
//             .join(&name)
//             .join("index.html");
//         if path.exists() {
//             let config_browser = {
//                 let cfg: Option<PathAndArgs> = ws.config().get("doc.browser")?;
//                 cfg.map(|path_args| (path_args.path.resolve_program(ws.config()), path_args.args))
//             };

//             let mut shell = ws.config().shell();
//             shell.status("Opening", path.display())?;
//             open_docs(&path, &mut shell, config_browser)?;
//         }
//     }

//     Ok(())
// }

// fn open_docs(
//     path: &Path,
//     shell: &mut Shell,
//     config_browser: Option<(PathBuf, Vec<String>)>,
// ) -> Result<()> {
//     let browser =
//         config_browser.or_else(|| Some((PathBuf::from(std::env::var_os("BROWSER")?), Vec::new())));

//     match browser {
//         Some((browser, initial_args)) => {
//             if let Err(e) = Command::new(&browser).args(initial_args).arg(path).status() {
//                 shell.warn(format!(
//                     "Couldn't open docs with {}: {}",
//                     browser.to_string_lossy(),
//                     e
//                 ))?;
//             }
//         }
//         None => {
//             if let Err(e) = opener::open(&path) {
//                 bail!("couldn't open docs: {e}");
//             }
//         }
//     };

//     Ok(())
// }
