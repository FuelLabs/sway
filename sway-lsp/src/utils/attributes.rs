use crate::{
    core::token::{AstToken, TokenMap},
    utils::token::to_ident_key,
};
use sway_core::{
    constants::{DOC_ATTRIBUTE_NAME, STORAGE_PURITY_ATTRIBUTE_NAME},
    AttributesMap, Declaration,
};
use sway_types::Ident;

pub(crate) fn attributes(decl_ident: &Ident, tokens: &TokenMap) -> Option<AttributesMap> {
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

pub(crate) fn doc_attributes(decl_ident: &Ident, tokens: &TokenMap) -> Option<Vec<Ident>> {
    attributes(decl_ident, tokens).and_then(|mut attributes| attributes.remove(DOC_ATTRIBUTE_NAME))
}

pub(crate) fn storage_attributes(decl_ident: &Ident, tokens: &TokenMap) -> Option<Vec<Ident>> {
    attributes(decl_ident, tokens)
        .and_then(|mut attributes| attributes.remove(STORAGE_PURITY_ATTRIBUTE_NAME))
}
