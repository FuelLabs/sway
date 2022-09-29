mod cli;
use anyhow::Result;
use cli::Command;
use forc_pkg::{self as pkg, ManifestFile};
use std::{
    collections::BTreeMap,
    {fs, path::PathBuf},
};
use sway_core::{
    declaration_engine::*, AstNode, AstNodeContent, Attribute, AttributeKind, AttributesMap,
    CompileResult, Declaration, ParseProgram, ParseSubmodule, TypedAstNodeContent,
    TypedDeclaration, TypedProgram,
};
use sway_types::{Ident, Spanned};

#[derive(Eq, PartialEq, Ord, PartialOrd)]
enum DescriptorType {
    Struct,
    Enum,
    Trait,
}

#[derive(Eq, PartialEq, Ord, PartialOrd)]
struct Descriptor {
    ty: DescriptorType,
    name: Ident,
}

impl From<&Declaration> for Descriptor {
    fn from(o: &Declaration) -> Self {
        use Declaration::*;
        use DescriptorType::*;
        match o {
            StructDeclaration(ref decl) => Descriptor {
                ty: Struct,
                name: decl.name.clone(),
            },
            _ => todo!(),
        }
    }
}
impl From<&TypedDeclaration> for Descriptor {
    fn from(o: &TypedDeclaration) -> Self {
        use DescriptorType::*;
        use TypedDeclaration::*;
        match o {
            StructDeclaration(ref decl) => Descriptor {
                ty: Struct,
                name: de_get_struct(decl.clone(), &decl.span())
                    .unwrap()
                    .name
                    .clone(),
            },
            _ => todo!(),
        }
    }
}
type TypeInformation = TypedDeclaration;
type Documentation = BTreeMap<Descriptor, (Vec<Attribute>, TypeInformation)>;

struct HTMLString(String);

struct RenderedDocumentation {
    file_contents: HTMLString,
    file_name: String,
}

/// Main method for `forc doc`.
pub fn main() -> Result<()> {
    let Command {
        manifest_path,
        open: open_result,
        offline_mode: offline,
        silent_mode,
        locked,
        no_deps,
    } = todo!();

    let dir = if let Some(ref path) = manifest_path {
        PathBuf::from(path)
    } else {
        std::env::current_dir()?
    };
    let manifest = ManifestFile::from_dir(&dir)?;

    // check if the out path exists
    let out_path = PathBuf::from(&manifest.dir()).join("out");
    let doc_path = out_path.join("doc");
    if out_path.try_exists().is_err() {
        // create the out path
        fs::create_dir_all(&out_path)?;
        if doc_path.try_exists().is_err() {
            fs::create_dir(&doc_path)?;
        }
    }
    // gather docs

    let plan = pkg::BuildPlan::from_lock_and_manifest(&manifest, locked, offline)?;
    // compile the program and extract the docs
    let compilation = pkg::check(&plan, silent_mode)?;
    let docs = get_compiled_docs(&compilation, no_deps);

    // render docs to HTML
    todo!("render");

    // write to outfile
    todo!("write to outfile");

    // check if the user wants to open the doc in the browser
    if open_result {
        todo!("open in browser");
    }

    Ok(())
}

fn attributes_map(ast_node: &AstNode) -> Option<Vec<AttributesMap>> {
    match ast_node.content.clone() {
        AstNodeContent::Declaration(decl) => match decl {
            Declaration::EnumDeclaration(decl) => {
                let mut attr_map = vec![decl.attributes];
                for variant in decl.variants {
                    attr_map.push(variant.attributes)
                }

                Some(attr_map)
            }
            Declaration::FunctionDeclaration(decl) => {
                let attr_map = vec![decl.attributes];

                Some(attr_map)
            }
            Declaration::StructDeclaration(decl) => {
                let mut attr_map = vec![decl.attributes];
                for field in decl.fields {
                    attr_map.push(field.attributes)
                }

                Some(attr_map)
            }
            Declaration::ConstantDeclaration(decl) => {
                let attr_map = vec![decl.attributes];

                Some(attr_map)
            }
            Declaration::StorageDeclaration(decl) => {
                let mut attr_map = vec![decl.attributes];
                for field in decl.fields {
                    attr_map.push(field.attributes)
                }

                Some(attr_map)
            }
            Declaration::TraitDeclaration(decl) => {
                let mut attr_map = vec![decl.attributes];
                for method in decl.methods {
                    attr_map.push(method.attributes)
                }

                Some(attr_map)
            }
            Declaration::ImplTrait(decl) => {
                let mut attr_map = Vec::new();
                for method in decl.functions {
                    attr_map.push(method.attributes)
                }

                Some(attr_map)
            }
            Declaration::AbiDeclaration(decl) => {
                let mut attr_map = Vec::new();
                for method in decl.methods {
                    attr_map.push(method.attributes)
                }

                Some(attr_map)
            }
            Declaration::ImplSelf(decl) => {
                let mut attr_map = Vec::new();
                for method in decl.functions {
                    attr_map.push(method.attributes)
                }

                Some(attr_map)
            }
            _ => None,
        },
        _ => None,
    }
}
/// Wrapper for `Vec<Attribute>` to use `collect()` method.
struct Attributes(Vec<Attribute>);
impl std::iter::FromIterator<std::option::Option<Vec<Attribute>>> for Attributes {
    fn from_iter<T: IntoIterator<Item = std::option::Option<Vec<Attribute>>>>(iter: T) -> Self {
        let mut c = Vec::new();

        for attrs in iter.into_iter().flatten() {
            for attr in attrs {
                c.push(attr)
            }
        }
        Self(c)
    }
}
fn doc_attributes(ast_node: &AstNode) -> Vec<Attribute> {
    attributes_map(ast_node)
        .map(|attributes| {
            let attr_map = attributes
                .iter()
                .map(|attr| attr.clone().remove(&AttributeKind::Doc))
                .collect::<Attributes>();
            match attr_map {
                Attributes(c) => c,
            }
        })
        .unwrap_or_default()
}
fn extract_submodule_docs(submodule: &ParseSubmodule, docs: &mut Vec<Vec<Attribute>>) {
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
) -> Documentation {
    let mut docs: Documentation = Default::default();

    // Here we must consolidate the typed ast and the parsed annotations, as the docstrings
    // are not preserved in the typed ast.
    if let Some((parsed_program, Some(typed_program))) = &compilation.value {
        for ast_node in &typed_program.root.all_nodes {
            // first, populate the descriptors and type information (decl).
            if let TypedAstNodeContent::Declaration(ref decl) = ast_node.content {
                let mut entry = docs
                    .entry(Descriptor::from(decl))
                    .or_insert((Vec::new(), decl.clone()));
                entry.1 = decl.clone();
            }
        }
        // then, grab the docstrings
        for ast_node in &parsed_program.root.tree.root_nodes {
            if let AstNodeContent::Declaration(ref decl) = ast_node.content {
                let docstrings = doc_attributes(&ast_node);
                if let Some(entry) = docs.get_mut(&Descriptor::from(decl)) {
                    entry.0 = docstrings;
                } else {
                    // this could be invalid in the case of a partial compilation. TODO audit this
                    panic!("Invariant violated: we shouldn't have parsed stuff that isnt in the typed tree");
                }
            }
        }
        if !no_deps {
            while let Some((_, submodule)) = parsed_program.root.submodules.first() {
                todo!()
                //                extract_submodule_docs(submodule, &mut docs);
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
//
