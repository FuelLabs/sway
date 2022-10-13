use crate::descriptor::Descriptor;
use anyhow::Result;
use std::collections::BTreeMap;
use sway_core::{
    declaration_engine::{
        de_get_abi, de_get_constant, de_get_enum, de_get_function, de_get_impl_trait,
        de_get_storage, de_get_struct, de_get_trait,
    },
    language::parsed::{AstNodeContent, Declaration, ParseProgram, ParseSubmodule},
    language::ty::TyDeclaration,
    semantic_analysis::TySubmodule,
    Attribute, AttributeKind, AttributesMap, CompileResult, TyAstNode, TyAstNodeContent, TyProgram,
};
use sway_types::Spanned;

type TypeInformation = TyDeclaration;
pub(crate) type Documentation = BTreeMap<Descriptor, (Vec<Attribute>, TypeInformation)>;

/// Gather [Documentation] from the [CompileResult].
pub(crate) fn get_compiled_docs(
    compilation: &CompileResult<(ParseProgram, Option<TyProgram>)>,
    no_deps: bool,
) -> Result<Documentation> {
    let mut docs: Documentation = Default::default();
    if let Some((parse_program, Some(typed_program))) = &compilation.value {
        for ast_node in &typed_program.root.all_nodes {
            // first, populate the descriptors and type information (decl).
            if let TyAstNodeContent::Declaration(ref decl) = ast_node.content {
                let mut entry = docs
                    .entry(Descriptor::from_typed_decl(decl, vec![]))
                    .or_insert((Vec::new(), decl.clone()));
                entry.1 = decl.clone();
                let docstrings = doc_attributes(ast_node)?;
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

    Ok(docs)
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
) -> Result<()> {
    let mut new_submodule_prefix = module_prefix.clone();
    new_submodule_prefix.push(parse_submodule.library_name.as_str().to_string());

    for ast_node in &parse_submodule.module.tree.root_nodes {
        if let AstNodeContent::Declaration(ref decl) = ast_node.content {
            let docstrings = doc_attributes(ast_node)?;
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

    Ok(())
}
// Collect the AttributesMaps from a TyAstNode.
fn attributes_map(ast_node: &TyAstNode) -> Result<Option<Vec<AttributesMap>>> {
    match ast_node.content.clone() {
        TyAstNodeContent::Declaration(ty_decl) => match ty_decl {
            TyDeclaration::EnumDeclaration(decl_id) => {
                let decl = de_get_enum(decl_id.clone(), &decl_id.span())?;
                let mut attr_map = vec![decl.attributes];
                for variant in decl.variants {
                    // TODO: add attr from variants
                }

                Ok(Some(attr_map))
            }
            TyDeclaration::FunctionDeclaration(decl_id) => {
                let decl = de_get_function(decl_id.clone(), &decl_id.span())?;
                let attr_map = vec![decl.attributes];

                Ok(Some(attr_map))
            }
            TyDeclaration::StructDeclaration(decl_id) => {
                let decl = de_get_struct(decl_id.clone(), &decl_id.span())?;
                let mut attr_map = vec![decl.attributes];
                // TODO: We must think about how to hanlde field docstrings since they belong
                // to specific fields and not the struct declaration itself. Here we are
                // just collecting them as if they are.
                for field in decl.fields {
                    attr_map.push(field.attributes)
                }

                Ok(Some(attr_map))
            }
            TyDeclaration::ConstantDeclaration(decl) => {
                // TODO: add in attributes for consts
                let decl = de_get_constant(decl.clone(), &decl.span())?;
                let attr_map = vec![decl.attributes];

                Ok(Some(attr_map))
            }
            TyDeclaration::StorageDeclaration(decl_id) => {
                let decl = de_get_storage(decl_id.clone(), &decl_id.span())?;
                let mut attr_map = vec![decl.attributes];
                for field in decl.fields {
                    attr_map.push(field.attributes)
                }

                Ok(Some(attr_map))
            }
            TyDeclaration::TraitDeclaration(decl_id) => {
                // TODO: add attributes for traits
                let decl = de_get_trait(decl_id.clone(), &decl_id.span())?;
                let mut attr_map = vec![decl.attributes];
                for method in decl.methods {
                    attr_map.push(method.attributes)
                }

                Ok(Some(attr_map))
            }
            TyDeclaration::ImplTrait(decl_id) => {
                let decl = de_get_impl_trait(decl_id.clone(), &decl_id.span())?;
                let mut attr_map = Vec::new();
                for method in decl.functions {
                    attr_map.push(method.attributes)
                }

                Ok(Some(attr_map))
            }
            TyDeclaration::AbiDeclaration(decl) => {
                // TODO: add attributes for abi
                let decl = de_get_abi(decl.clone(), &decl.span())?;
                let mut attr_map = Vec::new();
                for method in decl.methods {
                    attr_map.push(method.attributes)
                }

                Ok(Some(attr_map))
            }
            _ => Ok(None),
        },
        _ => Ok(None),
    }
}
// Gather all Attributes from the AttributesMap.
fn doc_attributes(ast_node: &TyAstNode) -> Result<Vec<Attribute>> {
    let result = Vec::new();
    if let Some(attributes_map) = attributes_map(ast_node)? {
        for hashmap in attributes_map {
            if let Some(attributes) = hashmap.get(&AttributeKind::Doc) {
                for attribute in attributes {
                    result.push(*attribute)
                }
            }
        }
    }

    Ok(result)
}
