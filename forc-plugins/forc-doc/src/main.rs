mod cli;
use anyhow::Result;
use clap::Parser;
use cli::Command;
use forc_pkg::{self as pkg, ManifestFile};
use std::{
    collections::BTreeMap,
    io::prelude::*,
    {fs, path::PathBuf},
};
use sway_core::{
    declaration_engine::*, semantic_analysis::TypedSubmodule, AstNode, AstNodeContent, Attribute,
    AttributeKind, AttributesMap, CompileResult, Declaration, ParseProgram, ParseSubmodule,
    TypedAstNodeContent, TypedDeclaration, TypedProgram,
};
use sway_types::{Ident, Spanned};

#[derive(Eq, PartialEq, Ord, PartialOrd, Debug)]
enum DescriptorType {
    Struct,
    Enum,
    Trait,
    Abi,
    Storage,
    ImplSelfDesc,
    ImplTraitDesc,
    Function,
    Const,
}
impl DescriptorType {
    pub fn to_name(&self) -> &'static str {
        use DescriptorType::*;
        match self {
            Struct => "struct",
            Enum => "enum",
            Trait => "trait",
            Abi => "abi",
            Storage => "storage",
            ImplSelfDesc => "impl_self",
            ImplTraitDesc => "impl_trait",
            Function => "function",
            Const => "const",
        }
    }
}

#[derive(Eq, PartialEq, Ord, PartialOrd)]
enum Descriptor {
    Documentable {
        ty: DescriptorType,
        name: Option<Ident>,
    },
    NonDocumentable,
}

impl Descriptor {
    pub fn to_file_name(&self) -> Option<String> {
        use Descriptor::*;
        match self {
            NonDocumentable => None,
            Documentable { ty, name } => {
                let name_str = match name {
                    Some(name) => name.as_str(),
                    None => ty.to_name(),
                };
                Some(format!("{}.{}.html", ty.to_name(), name_str))
            }
        }
    }
}

impl From<&Declaration> for Descriptor {
    fn from(o: &Declaration) -> Self {
        use Declaration::*;
        use DescriptorType::*;
        match o {
            StructDeclaration(ref decl) => Descriptor::Documentable {
                ty: Struct,
                name: Some(decl.name.clone()),
            },
            EnumDeclaration(ref decl) => Descriptor::Documentable {
                ty: Enum,
                name: Some(decl.name.clone()),
            },
            TraitDeclaration(ref decl) => Descriptor::Documentable {
                ty: Trait,
                name: Some(decl.name.clone()),
            },
            AbiDeclaration(ref decl) => Descriptor::Documentable {
                ty: Abi,
                name: Some(decl.name.clone()),
            },
            StorageDeclaration(_) => Descriptor::Documentable {
                ty: Storage,
                name: None, // no ident
            },
            ImplSelf(_) => Descriptor::Documentable {
                ty: ImplSelfDesc,
                name: None, // no ident
            },
            ImplTrait(ref decl) => Descriptor::Documentable {
                ty: ImplTraitDesc,
                name: Some(decl.trait_name.suffix.clone()),
            },
            FunctionDeclaration(ref decl) => Descriptor::Documentable {
                ty: Function,
                name: Some(decl.name.clone()),
            },
            ConstantDeclaration(ref decl) => Descriptor::Documentable {
                ty: Const,
                name: Some(decl.name.clone()),
            },
            _ => Descriptor::NonDocumentable,
        }
    }
}
impl From<&TypedDeclaration> for Descriptor {
    fn from(o: &TypedDeclaration) -> Self {
        use DescriptorType::*;
        use TypedDeclaration::*;
        match o {
            StructDeclaration(ref decl) => Descriptor::Documentable {
                ty: Struct,
                name: Some(de_get_struct(decl.clone(), &decl.span()).unwrap().name),
            },
            EnumDeclaration(ref decl) => Descriptor::Documentable {
                ty: Enum,
                name: Some(de_get_enum(decl.clone(), &decl.span()).unwrap().name),
            },
            TraitDeclaration(ref decl) => Descriptor::Documentable {
                ty: Trait,
                name: Some(de_get_trait(decl.clone(), &decl.span()).unwrap().name),
            },
            AbiDeclaration(ref decl) => Descriptor::Documentable {
                ty: Abi,
                name: Some(de_get_abi(decl.clone(), &decl.span()).unwrap().name),
            },
            StorageDeclaration(_) => Descriptor::Documentable {
                ty: Storage,
                name: None,
            },
            ImplTrait(ref decl) => Descriptor::Documentable {
                ty: ImplTraitDesc,
                name: Some(
                    de_get_impl_trait(decl.clone(), &decl.span())
                        .unwrap()
                        .trait_name
                        .suffix,
                ),
            },
            FunctionDeclaration(ref decl) => Descriptor::Documentable {
                ty: Function,
                name: Some(de_get_function(decl.clone(), &decl.span()).unwrap().name),
            },
            ConstantDeclaration(ref decl) => Descriptor::Documentable {
                ty: Const,
                name: Some(de_get_constant(decl.clone(), &decl.span()).unwrap().name),
            },
            _ => Descriptor::NonDocumentable,
        }
    }
}
type TypeInformation = TypedDeclaration;
type Documentation = BTreeMap<Descriptor, (Vec<Attribute>, TypeInformation)>;

impl RenderedDocumentation {
    pub fn render(raw: &Documentation) -> Vec<RenderedDocumentation> {
        let mut buf: Vec<RenderedDocumentation> = Default::default();
        for (desc, (_docs, _ty)) in raw {
            let file_name = match desc.to_file_name() {
                Some(x) => x,
                None => continue,
            };
            if let Descriptor::Documentable { ty, name } = desc {
                let name_str = match name {
                    Some(name) => name.as_str(),
                    None => ty.to_name(),
                };
                buf.push(Self {
                    file_contents: HTMLString(format!("Docs for {:?} {:?}", name_str, ty)),
                    file_name,
                })
            }
        }
        buf
    }
}

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
    } = Command::parse();

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
    // gather docs

    let plan = pkg::BuildPlan::from_lock_and_manifest(&manifest, locked, offline)?;
    // compile the program and extract the docs
    let compilation = pkg::check(&plan, silent_mode)?;
    let docs = get_compiled_docs(&compilation, no_deps);

    // render docs to HTML
    let rendered = RenderedDocumentation::render(&docs);

    // write to outfile
    for entry in rendered {
        let mut doc_path = doc_path.clone();
        doc_path.push(entry.file_name);
        let mut file = fs::File::create(doc_path)?;
        file.write_all(entry.file_contents.0.as_bytes())?;
    }

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
fn extract_typed_submodule(typed_submodule: &TypedSubmodule, docs: &mut Documentation) {
    for ast_node in &typed_submodule.module.all_nodes {
        // first, populate the descriptors and type information (decl).
        if let TypedAstNodeContent::Declaration(ref decl) = ast_node.content {
            let mut entry = docs
                .entry(Descriptor::from(decl))
                .or_insert((Vec::new(), decl.clone()));
            entry.1 = decl.clone();
        }
    }
    // if there is another submodule we need to go a level deeper
    if let Some((_, submodule)) = typed_submodule.module.submodules.first() {
        extract_typed_submodule(submodule, docs);
    }
}
fn extract_parse_submodule(parse_submodule: &ParseSubmodule, docs: &mut Documentation) {
    for ast_node in &parse_submodule.module.tree.root_nodes {
        if let AstNodeContent::Declaration(ref decl) = ast_node.content {
            let docstrings = doc_attributes(ast_node);
            if let Some(entry) = docs.get_mut(&Descriptor::from(decl)) {
                entry.0 = docstrings;
            } else {
                // this could be invalid in the case of a partial compilation. TODO audit this
                panic!("Invariant violated: we shouldn't have parsed stuff that isnt in the typed tree");
            }
        }
    }
    // if there is another submodule we need to go a level deeper
    if let Some((_, submodule)) = parse_submodule.module.submodules.first() {
        extract_parse_submodule(submodule, docs);
    }
}
fn get_compiled_docs(
    compilation: &CompileResult<(ParseProgram, Option<TypedProgram>)>,
    no_deps: bool,
) -> Documentation {
    let mut docs: Documentation = Default::default();

    // Here we must consolidate the typed ast and the parsed annotations, as the docstrings
    // are not preserved in the typed ast.
    if let Some((parse_program, Some(typed_program))) = &compilation.value {
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
        for ast_node in &parse_program.root.tree.root_nodes {
            if let AstNodeContent::Declaration(ref decl) = ast_node.content {
                let docstrings = doc_attributes(ast_node);
                if let Some(entry) = docs.get_mut(&Descriptor::from(decl)) {
                    entry.0 = docstrings;
                } else {
                    // this could be invalid in the case of a partial compilation. TODO audit this
                    panic!("Invariant violated: we shouldn't have parsed stuff that isnt in the typed tree");
                }
            }
        }

        if !no_deps
            && !typed_program.root.submodules.is_empty()
            && !parse_program.root.submodules.is_empty()
        {
            // this is the same process as before but for dependencies
            for (_, ref typed_submodule) in &typed_program.root.submodules {
                extract_typed_submodule(typed_submodule, &mut docs);
            }
            for (_, ref parse_submodule) in &parse_program.root.submodules {
                extract_parse_submodule(parse_submodule, &mut docs);
            }
        }
    }

    docs
}
