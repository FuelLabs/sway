#![allow(dead_code)]
use crate::core::token::{AstToken, Token};
use sway_core::{language::parsed::Declaration, transform};

pub(crate) fn attributes_map(token: &Token) -> Option<&transform::AttributesMap> {
    match &token.parsed {
        AstToken::Declaration(declaration) => match declaration {
            Declaration::EnumDeclaration(decl) => Some(&decl.attributes),
            Declaration::FunctionDeclaration(decl) => Some(&decl.attributes),
            Declaration::StructDeclaration(decl) => Some(&decl.attributes),
            Declaration::ConstantDeclaration(decl) => Some(&decl.attributes),
            Declaration::StorageDeclaration(decl) => Some(&decl.attributes),
            _ => None,
        },
        AstToken::StorageField(field) => Some(&field.attributes),
        AstToken::StructField(field) => Some(&field.attributes),
        AstToken::TraitFn(trait_fn) => Some(&trait_fn.attributes),
        AstToken::EnumVariant(variant) => Some(&variant.attributes),
        _ => None,
    }
}

pub(crate) fn doc_attributes(token: &Token) -> Option<&[transform::Attribute]> {
    attributes_map(token)
        .and_then(|attributes| attributes.get(&transform::AttributeKind::Doc))
        .map(Vec::as_slice)
}

pub(crate) fn storage_attributes(token: &Token) -> Option<&[transform::Attribute]> {
    attributes_map(token)
        .and_then(|attributes| attributes.get(&transform::AttributeKind::Storage))
        .map(Vec::as_slice)
}
