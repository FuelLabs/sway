#![allow(dead_code)]
use crate::core::token::{AstToken, Token};
use sway_core::{language::parse_tree::Declaration, Attribute, AttributeKind, AttributesMap};

pub(crate) fn attributes_map(token: &Token) -> Option<AttributesMap> {
    match &token.parsed {
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
    }
}

pub(crate) fn doc_attributes(token: &Token) -> Option<Vec<Attribute>> {
    attributes_map(token).and_then(|mut attributes| attributes.remove(&AttributeKind::Doc))
}

pub(crate) fn storage_attributes(token: &Token) -> Option<Vec<Attribute>> {
    attributes_map(token).and_then(|mut attributes| attributes.remove(&AttributeKind::Storage))
}
