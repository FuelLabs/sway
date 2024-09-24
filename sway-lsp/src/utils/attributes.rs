#![allow(dead_code)]
use crate::core::token::{ParsedAstToken, Token};
use sway_core::{language::parsed::Declaration, transform, Engines};

pub fn attributes_map<F>(engines: &Engines, token: &Token, mut callback: F)
where
    F: FnMut(&transform::AttributesMap),
{
    match &token.parsed {
        ParsedAstToken::Declaration(declaration) => match declaration {
            Declaration::EnumDeclaration(decl_id) => {
                let decl = engines.pe().get_enum(decl_id);
                callback(&decl.attributes);
            }
            Declaration::FunctionDeclaration(decl_id) => {
                let decl = engines.pe().get_function(decl_id);
                callback(&decl.attributes);
            }
            Declaration::StructDeclaration(decl_id) => {
                let decl = engines.pe().get_struct(decl_id);
                callback(&decl.attributes);
            }
            Declaration::ConstantDeclaration(decl_id) => {
                let decl = engines.pe().get_constant(decl_id);
                callback(&decl.attributes);
            }
            Declaration::StorageDeclaration(decl_id) => {
                let decl = engines.pe().get_storage(decl_id);
                callback(&decl.attributes);
            }
            Declaration::AbiDeclaration(decl_id) => {
                let decl = engines.pe().get_abi(decl_id);
                callback(&decl.attributes);
            }
            _ => {}
        },
        ParsedAstToken::StorageField(field) => callback(&field.attributes),
        ParsedAstToken::StructField(field) => callback(&field.attributes),
        ParsedAstToken::TraitFn(decl_id) => {
            let decl = engines.pe().get_trait_fn(decl_id);
            callback(&decl.attributes);
        }
        ParsedAstToken::EnumVariant(variant) => callback(&variant.attributes),
        _ => {}
    }
}

pub fn doc_comment_attributes<F>(engines: &Engines, token: &Token, mut callback: F)
where
    F: FnMut(&[transform::Attribute]),
{
    attributes_map(engines, token, |attribute_map| {
        let attrs = attribute_map
            .get(&transform::AttributeKind::DocComment)
            .map(Vec::as_slice);
        if let Some(attrs) = attrs {
            callback(attrs);
        }
    });
}

pub fn storage_attributes<F>(engines: &Engines, token: &Token, callback: F)
where
    F: Fn(&[transform::Attribute]),
{
    attributes_map(engines, token, |attribute_map| {
        let attrs = attribute_map
            .get(&transform::AttributeKind::Storage)
            .map(Vec::as_slice);
        if let Some(attrs) = attrs {
            callback(attrs);
        }
    });
}
