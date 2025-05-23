#![allow(dead_code)]
use crate::core::token::{ParsedAstToken, Token, TokenAstNode, TypedAstToken};
use sway_core::{
    language::{parsed::Declaration, ty},
    transform, Engines,
};

/// Gets attributes from typed token, falling back to parsed AST node if needed.
/// Callback can be used to retrieve doc comment attributes or storage attributes.
pub fn attributes<F>(engines: &Engines, token: &Token, mut callback: F)
where
    F: FnMut(&transform::Attributes),
{
    match &token.ast_node {
        TokenAstNode::Typed(typed_token) => match typed_token {
            TypedAstToken::TypedDeclaration(decl) => match decl {
                ty::TyDecl::EnumDecl(ty::EnumDecl { decl_id, .. }) => {
                    let enum_decl = engines.de().get_enum(decl_id);
                    callback(&enum_decl.attributes);
                }
                ty::TyDecl::StructDecl(ty::StructDecl { decl_id, .. }) => {
                    let struct_decl = engines.de().get_struct(decl_id);
                    callback(&struct_decl.attributes);
                }
                ty::TyDecl::StorageDecl(ty::StorageDecl { decl_id, .. }) => {
                    let storage_decl = engines.de().get_storage(decl_id);
                    callback(&storage_decl.attributes);
                }
                ty::TyDecl::AbiDecl(ty::AbiDecl { decl_id, .. }) => {
                    let abi_decl = engines.de().get_abi(decl_id);
                    callback(&abi_decl.attributes);
                }
                _ => {}
            },
            TypedAstToken::TypedFunctionDeclaration(fn_decl) => {
                callback(&fn_decl.attributes);
            }
            TypedAstToken::TypedConstantDeclaration(constant) => {
                callback(&constant.attributes);
            }
            TypedAstToken::TypedStorageField(field) => {
                callback(&field.attributes);
            }
            TypedAstToken::TypedStructField(field) => {
                callback(&field.attributes);
            }
            TypedAstToken::TypedTraitFn(trait_fn) => {
                callback(&trait_fn.attributes);
            }
            TypedAstToken::TypedEnumVariant(variant) => {
                callback(&variant.attributes);
            }
            TypedAstToken::TypedConfigurableDeclaration(configurable) => {
                callback(&configurable.attributes);
            }
            TypedAstToken::TypedTraitTypeDeclaration(trait_type) => {
                callback(&trait_type.attributes);
            }
            TypedAstToken::TypedTypeAliasDeclaration(type_alias) => {
                callback(&type_alias.attributes);
            }
            _ => {}
        },
        TokenAstNode::Parsed(parsed_token) => match &parsed_token {
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
        },
    }
}

pub fn doc_comment_attributes<F>(engines: &Engines, token: &Token, mut callback: F)
where
    F: FnMut(&[&transform::Attribute]),
{
    attributes(engines, token, |attributes| {
        let attrs = attributes
            .of_kind(transform::AttributeKind::DocComment)
            .collect::<Vec<_>>();
        if !attrs.is_empty() {
            callback(attrs.as_slice());
        }
    });
}

pub fn storage_attributes<F>(engines: &Engines, token: &Token, callback: F)
where
    F: Fn(&[&transform::Attribute]),
{
    attributes(engines, token, |attributes| {
        let attrs = attributes
            .of_kind(transform::AttributeKind::Storage)
            .collect::<Vec<_>>();
        if !attrs.is_empty() {
            callback(attrs.as_slice());
        }
    });
}
