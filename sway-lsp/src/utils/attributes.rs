#![allow(dead_code)]
use crate::{
    core::token::{AstToken, TokenMap},
    utils::token::to_ident_key,
};
use std::collections::HashMap;
use sway_ast::AttributeDecl;
use sway_core::{
    constants::{DOC_ATTRIBUTE_NAME, STORAGE_PURITY_ATTRIBUTE_NAME},
    Declaration,
};
use sway_types::Ident;

#[derive(Clone, Debug)]
pub struct Attribute {
    pub name: Ident,
    pub args: Vec<Ident>,
}

pub(crate) fn attribute_decls(decl_ident: &Ident, tokens: &TokenMap) -> Option<Vec<AttributeDecl>> {
    if let Some(item) = tokens.get(&to_ident_key(decl_ident)) {
        let token = item.value();
        return match &token.parsed {
            AstToken::Declaration(declaration) => match declaration {
                Declaration::EnumDeclaration(decl) => Some(decl.attributes.clone()),
                Declaration::FunctionDeclaration(decl) => Some(decl.attributes.clone()),
                Declaration::StructDeclaration(decl) => Some(decl.attributes.clone()),
                Declaration::ConstantDeclaration(decl) => Some(decl.attributes.clone()),
                Declaration::StorageDeclaration(decl) => Some(decl.attributes.clone()),
                _ => None,
            },
            AstToken::StorageField(field) => Some(field.attributes.clone()),
            AstToken::StructField(field) => Some(field.attributes.clone()),
            AstToken::TraitFn(trait_fn) => Some(trait_fn.attributes.clone()),
            AstToken::EnumVariant(variant) => Some(variant.attributes.clone()),
            _ => None,
        };
    }
    None
}

pub(crate) fn doc_attributes(decl_ident: &Ident, tokens: &TokenMap) -> Option<Vec<Attribute>> {
    attribute_decls(decl_ident, tokens)
        .and_then(|attribute_list| attributes_to_map(&attribute_list).remove(DOC_ATTRIBUTE_NAME))
}

pub(crate) fn storage_attributes(decl_ident: &Ident, tokens: &TokenMap) -> Option<Vec<Attribute>> {
    attribute_decls(decl_ident, tokens).and_then(|attribute_list| {
        attributes_to_map(&attribute_list).remove(STORAGE_PURITY_ATTRIBUTE_NAME)
    })
}

pub(crate) fn attributes_to_map(attribute_list: &[AttributeDecl]) -> HashMap<String, Vec<Attribute>> {
    let mut attrs_map: HashMap<String, Vec<Attribute>> = HashMap::new();
    for attr_decl in attribute_list {
        let attr = attr_decl.attribute.get();
        let args = attr
            .args
            .as_ref()
            .map(|parens| parens.get().into_iter().cloned().collect())
            .unwrap_or_else(Vec::new);

        let attribute = Attribute {
            name: attr.name.clone(),
            args,
        };
        let name = attr.name.as_str();
        match attrs_map.get_mut(name) {
            Some(old_args) => {
                old_args.push(attribute);
            }
            None => {
                attrs_map.insert(name.to_string(), vec![attribute]);
            }
        }
    }
    attrs_map
}
