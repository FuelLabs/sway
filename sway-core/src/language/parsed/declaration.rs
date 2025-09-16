mod abi;
mod configurable;
mod const_generic;
mod constant;
mod r#enum;
pub mod function;
mod impl_trait;
mod storage;
mod r#struct;
mod r#trait;
mod type_alias;
mod variable;

use std::fmt;

pub use abi::*;
pub use configurable::*;
pub use const_generic::*;
pub use constant::*;
pub use function::*;
pub use impl_trait::*;
pub use r#enum::*;
pub use r#struct::*;
pub use r#trait::*;
pub use storage::*;
use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
};
use sway_types::{Ident, Span, Spanned};
pub use type_alias::*;
pub use variable::*;

use crate::{
    decl_engine::{
        parsed_engine::{ParsedDeclEngine, ParsedDeclEngineGet},
        parsed_id::ParsedDeclId,
        DeclEngineGetParsedDeclId,
    },
    engine_threading::{
        DebugWithEngines, DisplayWithEngines, EqWithEngines, PartialEqWithEngines,
        PartialEqWithEnginesContext,
    },
    language::Visibility,
    Engines,
};

#[derive(Debug, Clone)]
pub enum Declaration {
    VariableDeclaration(ParsedDeclId<VariableDeclaration>),
    FunctionDeclaration(ParsedDeclId<FunctionDeclaration>),
    TraitDeclaration(ParsedDeclId<TraitDeclaration>),
    StructDeclaration(ParsedDeclId<StructDeclaration>),
    EnumDeclaration(ParsedDeclId<EnumDeclaration>),
    EnumVariantDeclaration(EnumVariantDeclaration),
    ImplSelfOrTrait(ParsedDeclId<ImplSelfOrTrait>),
    AbiDeclaration(ParsedDeclId<AbiDeclaration>),
    ConstantDeclaration(ParsedDeclId<ConstantDeclaration>),
    ConfigurableDeclaration(ParsedDeclId<ConfigurableDeclaration>),
    StorageDeclaration(ParsedDeclId<StorageDeclaration>),
    TypeAliasDeclaration(ParsedDeclId<TypeAliasDeclaration>),
    TraitTypeDeclaration(ParsedDeclId<TraitTypeDeclaration>),
    TraitFnDeclaration(ParsedDeclId<TraitFn>),
    ConstGenericDeclaration(ParsedDeclId<ConstGenericDeclaration>),
}

#[derive(Debug, Clone)]
pub struct EnumVariantDeclaration {
    pub enum_ref: ParsedDeclId<EnumDeclaration>,
    pub variant_name: Ident,
    pub variant_decl_span: Span,
}

impl Declaration {
    /// Checks if this `Declaration` is a test.
    pub(crate) fn is_test(&self, engines: &Engines) -> bool {
        if let Declaration::FunctionDeclaration(fn_decl) = self {
            let fn_decl = engines.pe().get_function(fn_decl);
            fn_decl.is_test()
        } else {
            false
        }
    }

    /// Friendly type name string used for error reporting,
    /// which consists of the type name of the declaration AST node.
    pub fn friendly_type_name(&self) -> &'static str {
        use Declaration::*;
        match self {
            VariableDeclaration(_) => "variable",
            ConstantDeclaration(_) => "constant",
            ConfigurableDeclaration(_) => "configurable",
            TraitTypeDeclaration(_) => "type",
            FunctionDeclaration(_) => "function",
            TraitDeclaration(_) => "trait",
            TraitFnDeclaration(_) => "trait fn",
            StructDeclaration(_) => "struct",
            EnumDeclaration(_) => "enum",
            EnumVariantDeclaration(_) => "enum variant",
            ImplSelfOrTrait(_) => "impl self/trait",
            AbiDeclaration(_) => "abi",
            StorageDeclaration(_) => "contract storage",
            TypeAliasDeclaration(_) => "type alias",
            ConstGenericDeclaration(_) => "const generic",
        }
    }

    pub fn span(&self, engines: &Engines) -> sway_types::Span {
        use Declaration::*;
        let pe = engines.pe();
        match self {
            VariableDeclaration(decl_id) => pe.get_variable(decl_id).span(),
            FunctionDeclaration(decl_id) => pe.get_function(decl_id).span(),
            TraitDeclaration(decl_id) => pe.get_trait(decl_id).span(),
            StructDeclaration(decl_id) => pe.get_struct(decl_id).span(),
            EnumDeclaration(decl_id) => pe.get_enum(decl_id).span(),
            EnumVariantDeclaration(decl) => decl.variant_decl_span.clone(),
            ImplSelfOrTrait(decl_id) => pe.get_impl_self_or_trait(decl_id).span(),
            AbiDeclaration(decl_id) => pe.get_abi(decl_id).span(),
            ConstantDeclaration(decl_id) => pe.get_constant(decl_id).span(),
            ConfigurableDeclaration(decl_id) => pe.get_configurable(decl_id).span(),
            StorageDeclaration(decl_id) => pe.get_storage(decl_id).span(),
            TypeAliasDeclaration(decl_id) => pe.get_type_alias(decl_id).span(),
            TraitTypeDeclaration(decl_id) => pe.get_trait_type(decl_id).span(),
            TraitFnDeclaration(decl_id) => pe.get_trait_fn(decl_id).span(),
            ConstGenericDeclaration(_) => {
                todo!("Will be implemented by https://github.com/FuelLabs/sway/issues/6860")
            }
        }
    }

    pub(crate) fn to_fn_ref(
        &self,
        handler: &Handler,
        engines: &Engines,
    ) -> Result<ParsedDeclId<FunctionDeclaration>, ErrorEmitted> {
        match self {
            Declaration::FunctionDeclaration(decl_id) => Ok(*decl_id),
            decl => Err(handler.emit_err(CompileError::DeclIsNotAFunction {
                actually: decl.friendly_type_name().to_string(),
                span: decl.span(engines),
            })),
        }
    }

    pub(crate) fn to_struct_decl(
        &self,
        handler: &Handler,
        engines: &Engines,
    ) -> Result<ParsedDeclId<StructDeclaration>, ErrorEmitted> {
        match self {
            Declaration::StructDeclaration(decl_id) => Ok(*decl_id),
            Declaration::TypeAliasDeclaration(decl_id) => {
                let alias = engines.pe().get_type_alias(decl_id);
                let struct_decl_id = engines.te().get(alias.ty.type_id()).expect_struct(
                    handler,
                    engines,
                    &self.span(engines),
                )?;

                let parsed_decl_id = engines.de().get_parsed_decl_id(&struct_decl_id);
                parsed_decl_id.ok_or_else(|| {
                    handler.emit_err(CompileError::InternalOwned(
                        "Cannot get parsed decl id from decl id".to_string(),
                        self.span(engines),
                    ))
                })
            }
            decl => Err(handler.emit_err(CompileError::DeclIsNotAStruct {
                actually: decl.friendly_type_name().to_string(),
                span: decl.span(engines),
            })),
        }
    }

    pub(crate) fn to_enum_decl(
        &self,
        handler: &Handler,
        engines: &Engines,
    ) -> Result<ParsedDeclId<EnumDeclaration>, ErrorEmitted> {
        match self {
            Declaration::EnumDeclaration(decl_id) => Ok(*decl_id),
            Declaration::TypeAliasDeclaration(decl_id) => {
                let alias = engines.pe().get_type_alias(decl_id);
                let enum_decl_id = engines.te().get(alias.ty.type_id()).expect_enum(
                    handler,
                    engines,
                    String::default(),
                    &self.span(engines),
                )?;

                let parsed_decl_id = engines.de().get_parsed_decl_id(&enum_decl_id);
                parsed_decl_id.ok_or_else(|| {
                    handler.emit_err(CompileError::InternalOwned(
                        "Cannot get parsed decl id from decl id".to_string(),
                        self.span(engines),
                    ))
                })
            }
            decl => Err(handler.emit_err(CompileError::DeclIsNotAnEnum {
                actually: decl.friendly_type_name().to_string(),
                span: decl.span(engines),
            })),
        }
    }

    #[allow(unused)]
    pub(crate) fn visibility(&self, decl_engine: &ParsedDeclEngine) -> Visibility {
        match self {
            Declaration::TraitDeclaration(decl_id) => decl_engine.get_trait(decl_id).visibility,
            Declaration::ConstantDeclaration(decl_id) => {
                decl_engine.get_constant(decl_id).visibility
            }
            Declaration::ConfigurableDeclaration(decl_id) => {
                decl_engine.get_configurable(decl_id).visibility
            }
            Declaration::StructDeclaration(decl_id) => decl_engine.get_struct(decl_id).visibility,
            Declaration::EnumDeclaration(decl_id) => decl_engine.get_enum(decl_id).visibility,
            Declaration::EnumVariantDeclaration(decl) => {
                decl_engine.get_enum(&decl.enum_ref).visibility
            }
            Declaration::FunctionDeclaration(decl_id) => {
                decl_engine.get_function(decl_id).visibility
            }
            Declaration::TypeAliasDeclaration(decl_id) => {
                decl_engine.get_type_alias(decl_id).visibility
            }
            Declaration::VariableDeclaration(_decl_id) => Visibility::Private,
            Declaration::ImplSelfOrTrait(_)
            | Declaration::StorageDeclaration(_)
            | Declaration::AbiDeclaration(_)
            | Declaration::TraitTypeDeclaration(_)
            | Declaration::TraitFnDeclaration(_) => Visibility::Public,
            Declaration::ConstGenericDeclaration(_) => {
                // const generics do not have visibility
                unreachable!()
            }
        }
    }
}

impl DisplayWithEngines for Declaration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: &Engines) -> std::fmt::Result {
        write!(
            f,
            "{} parsed declaration ({})",
            self.friendly_type_name(),
            match self {
                Declaration::VariableDeclaration(decl_id) => {
                    engines.pe().get(decl_id).name.as_str().into()
                }
                Declaration::FunctionDeclaration(decl_id) => {
                    engines.pe().get(decl_id).name.as_str().into()
                }
                Declaration::TraitDeclaration(decl_id) => {
                    engines.pe().get(decl_id).name.as_str().into()
                }
                Declaration::StructDeclaration(decl_id) => {
                    engines.pe().get(decl_id).name.as_str().into()
                }
                Declaration::EnumDeclaration(decl_id) => {
                    engines.pe().get(decl_id).name.as_str().into()
                }
                Declaration::ImplSelfOrTrait(decl_id) => {
                    engines
                        .pe()
                        .get(decl_id)
                        .trait_name
                        .as_vec_string()
                        .join("::")
                        .as_str()
                        .into()
                }
                Declaration::TypeAliasDeclaration(decl_id) =>
                    engines.pe().get(decl_id).name.as_str().into(),
                _ => String::new(),
            }
        )
    }
}

impl DebugWithEngines for Declaration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: &Engines) -> std::fmt::Result {
        DisplayWithEngines::fmt(&self, f, engines)
    }
}

impl EqWithEngines for Declaration {}
impl PartialEqWithEngines for Declaration {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        let decl_engine = ctx.engines().pe();
        match (self, other) {
            (Declaration::VariableDeclaration(lid), Declaration::VariableDeclaration(rid)) => {
                decl_engine.get(lid).eq(&decl_engine.get(rid), ctx)
            }
            (Declaration::FunctionDeclaration(lid), Declaration::FunctionDeclaration(rid)) => {
                decl_engine.get(lid).eq(&decl_engine.get(rid), ctx)
            }
            (Declaration::TraitDeclaration(lid), Declaration::TraitDeclaration(rid)) => {
                decl_engine.get(lid).eq(&decl_engine.get(rid), ctx)
            }
            (Declaration::StructDeclaration(lid), Declaration::StructDeclaration(rid)) => {
                decl_engine.get(lid).eq(&decl_engine.get(rid), ctx)
            }
            (Declaration::EnumDeclaration(lid), Declaration::EnumDeclaration(rid)) => {
                decl_engine.get(lid).eq(&decl_engine.get(rid), ctx)
            }
            (Declaration::ImplSelfOrTrait(lid), Declaration::ImplSelfOrTrait(rid)) => {
                decl_engine.get(lid).eq(&decl_engine.get(rid), ctx)
            }
            (Declaration::AbiDeclaration(lid), Declaration::AbiDeclaration(rid)) => {
                decl_engine.get(lid).eq(&decl_engine.get(rid), ctx)
            }
            (Declaration::ConstantDeclaration(lid), Declaration::ConstantDeclaration(rid)) => {
                decl_engine.get(lid).eq(&decl_engine.get(rid), ctx)
            }
            (Declaration::StorageDeclaration(lid), Declaration::StorageDeclaration(rid)) => {
                decl_engine.get(lid).eq(&decl_engine.get(rid), ctx)
            }
            (Declaration::TypeAliasDeclaration(lid), Declaration::TypeAliasDeclaration(rid)) => {
                decl_engine.get(lid).eq(&decl_engine.get(rid), ctx)
            }
            (Declaration::TraitTypeDeclaration(lid), Declaration::TraitTypeDeclaration(rid)) => {
                decl_engine.get(lid).eq(&decl_engine.get(rid), ctx)
            }
            _ => false,
        }
    }
}
