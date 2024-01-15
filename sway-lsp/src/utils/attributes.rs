#![allow(dead_code)]
use crate::core::token::{AstToken, Token};
use sway_core::{language::parsed::Declaration, transform, Engines};

pub fn attributes_map<F>(engines: &Engines, token: &Token, mut callback: F)
where
    F: FnMut(&transform::AttributesMap),
{
    match &token.parsed {
        AstToken::Declaration(declaration) => match declaration {
            Declaration::EnumDeclaration(decl) => callback(&decl.attributes),
            Declaration::FunctionDeclaration(decl_id) => {
                let decl = engines.pe().get_function(decl_id);
                callback(&decl.attributes)
            }
            Declaration::StructDeclaration(decl_id) => {
                let decl = engines.pe().get_struct(decl_id);
                callback(&decl.attributes)
            }
            Declaration::ConstantDeclaration(decl) => callback(&decl.attributes),
            Declaration::StorageDeclaration(decl) => callback(&decl.attributes),
            Declaration::AbiDeclaration(decl) => callback(&decl.attributes),
            _ => {}
        },
        AstToken::StorageField(field) => callback(&field.attributes),
        AstToken::StructField(field) => callback(&field.attributes),
        AstToken::TraitFn(trait_fn) => callback(&trait_fn.attributes),
        AstToken::EnumVariant(variant) => callback(&variant.attributes),
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
