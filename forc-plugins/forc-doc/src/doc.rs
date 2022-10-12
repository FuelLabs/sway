use crate::descriptor::Descriptor;
use std::collections::BTreeMap;
use sway_core::{
    language::parsed::{AstNode, AstNodeContent, Declaration, ParseProgram, ParseSubmodule},
    language::ty::TyDeclaration,
    semantic_analysis::TySubmodule,
    Attribute, AttributeKind, AttributesMap, CompileResult, TyAstNodeContent, TyProgram,
};

type TypeInformation = TyDeclaration;
pub(crate) type Documentation = BTreeMap<Descriptor, (Vec<Attribute>, TypeInformation)>;

/// Gather [Documentation] from the [CompileResult].
pub(crate) fn get_compiled_docs(
    compilation: &CompileResult<(ParseProgram, Option<TyProgram>)>,
    no_deps: bool,
) -> Documentation {
    let mut docs: Documentation = Default::default();

    // Here we must consolidate the typed ast and the parsed annotations, as the docstrings
    // are not preserved in the typed ast.
    if let Some((parse_program, Some(typed_program))) = &compilation.value {
        for ast_node in &typed_program.root.all_nodes {
            // first, populate the descriptors and type information (decl).
            if let TyAstNodeContent::Declaration(ref decl) = ast_node.content {
                let mut entry = docs
                    .entry(Descriptor::from_typed_decl(decl, vec![]))
                    .or_insert((Vec::new(), decl.clone()));
                entry.1 = decl.clone();
            }
        }
        // then, grab the docstrings
        for ast_node in &parse_program.root.tree.root_nodes {
            if let AstNodeContent::Declaration(ref decl) = ast_node.content {
                let docstrings = doc_attributes(ast_node);
                if let Some(entry) = docs.get_mut(&Descriptor::from_decl(decl, vec![])) {
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
                let module_prefix = vec![];
                extract_typed_submodule(typed_submodule, &mut docs, &module_prefix);
            }
            for (_, ref parse_submodule) in &parse_program.root.submodules {
                let module_prefix = vec![];
                extract_parse_submodule(parse_submodule, &mut docs, &module_prefix);
            }
        }
    }

    docs
}
fn extract_typed_submodule(
    typed_submodule: &TySubmodule,
    docs: &mut Documentation,
    module_prefix: &Vec<String>,
) {
    let mut new_submodule_prefix = module_prefix.clone();
    new_submodule_prefix.push(typed_submodule.library_name.as_str().to_string());
    for ast_node in &typed_submodule.module.all_nodes {
        // first, populate the descriptors and type information (decl).
        if let TyAstNodeContent::Declaration(ref decl) = ast_node.content {
            let mut entry = docs
                .entry(Descriptor::from_typed_decl(
                    decl,
                    new_submodule_prefix.clone(),
                ))
                .or_insert((Vec::new(), decl.clone()));
            entry.1 = decl.clone();
        }
    }
    // if there is another submodule we need to go a level deeper
    if let Some((_, submodule)) = typed_submodule.module.submodules.first() {
        extract_typed_submodule(submodule, docs, &new_submodule_prefix);
    }
}
fn extract_parse_submodule(
    parse_submodule: &ParseSubmodule,
    docs: &mut Documentation,
    module_prefix: &Vec<String>,
) {
    let mut new_submodule_prefix = module_prefix.clone();
    new_submodule_prefix.push(parse_submodule.library_name.as_str().to_string());

    for ast_node in &parse_submodule.module.tree.root_nodes {
        if let AstNodeContent::Declaration(ref decl) = ast_node.content {
            let docstrings = doc_attributes(ast_node);
            if let Some(entry) =
                docs.get_mut(&Descriptor::from_decl(decl, new_submodule_prefix.clone()))
            {
                entry.0 = docstrings;
            } else {
                // this could be invalid in the case of a partial compilation. TODO audit this
                panic!("Invariant violated: we shouldn't have parsed stuff that isnt in the typed tree");
            }
        }
    }
    // if there is another submodule we need to go a level deeper
    if let Some((_, submodule)) = parse_submodule.module.submodules.first() {
        extract_parse_submodule(submodule, docs, &new_submodule_prefix);
    }
}

// Wrapper for `Vec<Attribute>` to use `collect()` method.
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
