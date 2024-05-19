use std::{
    fmt,
    hash::{Hash, Hasher},
};

use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
};
use sway_types::{Ident, Named, Span, Spanned};

use crate::{
    decl_engine::*,
    engine_threading::*,
    language::{ty::*, Visibility},
    type_system::*,
    types::*,
};

#[derive(Clone, Debug)]
pub enum TyDecl {
    VariableDecl(Box<TyVariableDecl>),
    ConstantDecl(ConstantDecl),
    TraitTypeDecl(TraitTypeDecl),
    FunctionDecl(FunctionDecl),
    TraitDecl(TraitDecl),
    StructDecl(StructDecl),
    EnumDecl(EnumDecl),
    EnumVariantDecl(EnumVariantDecl),
    ImplTrait(ImplTrait),
    AbiDecl(AbiDecl),
    // If type parameters are defined for a function, they are put in the namespace just for
    // the body of that function.
    GenericTypeForFunctionScope(GenericTypeForFunctionScope),
    ErrorRecovery(Span, ErrorEmitted),
    StorageDecl(StorageDecl),
    TypeAliasDecl(TypeAliasDecl),
}

#[derive(Clone, Debug)]
pub struct ConstantDecl {
    pub decl_id: DeclId<TyConstantDecl>,
}

#[derive(Clone, Debug)]
pub struct TraitTypeDecl {
    pub decl_id: DeclId<TyTraitType>,
}

#[derive(Clone, Debug)]
pub struct FunctionDecl {
    pub decl_id: DeclId<TyFunctionDecl>,
}

#[derive(Clone, Debug)]
pub struct TraitDecl {
    pub decl_id: DeclId<TyTraitDecl>,
}

#[derive(Clone, Debug)]
pub struct StructDecl {
    pub decl_id: DeclId<TyStructDecl>,
}

#[derive(Clone, Debug)]
pub struct EnumDecl {
    pub decl_id: DeclId<TyEnumDecl>,
}

#[derive(Clone, Debug)]
pub struct EnumVariantDecl {
    pub enum_ref: DeclRefEnum,
    pub variant_name: Ident,
    pub variant_decl_span: Span,
}

#[derive(Clone, Debug)]
pub struct ImplTrait {
    pub decl_id: DeclId<TyImplTrait>,
}

#[derive(Clone, Debug)]
pub struct AbiDecl {
    pub decl_id: DeclId<TyAbiDecl>,
}

#[derive(Clone, Debug)]
pub struct GenericTypeForFunctionScope {
    pub name: Ident,
    pub type_id: TypeId,
}

#[derive(Clone, Debug)]
pub struct StorageDecl {
    pub decl_id: DeclId<TyStorageDecl>,
}

#[derive(Clone, Debug)]
pub struct TypeAliasDecl {
    pub decl_id: DeclId<TyTypeAliasDecl>,
}

impl EqWithEngines for TyDecl {}
impl PartialEqWithEngines for TyDecl {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        let decl_engine = ctx.engines().de();
        let type_engine = ctx.engines().te();
        match (self, other) {
            (TyDecl::VariableDecl(x), TyDecl::VariableDecl(y)) => x.eq(y, ctx),
            (
                TyDecl::ConstantDecl(ConstantDecl { decl_id: lid, .. }),
                TyDecl::ConstantDecl(ConstantDecl { decl_id: rid, .. }),
            ) => decl_engine.get(lid).eq(&decl_engine.get(rid), ctx),
            (
                TyDecl::FunctionDecl(FunctionDecl { decl_id: lid, .. }),
                TyDecl::FunctionDecl(FunctionDecl { decl_id: rid, .. }),
            ) => decl_engine.get(lid).eq(&decl_engine.get(rid), ctx),
            (
                TyDecl::TraitDecl(TraitDecl { decl_id: lid, .. }),
                TyDecl::TraitDecl(TraitDecl { decl_id: rid, .. }),
            ) => decl_engine.get(lid).eq(&decl_engine.get(rid), ctx),
            (
                TyDecl::StructDecl(StructDecl { decl_id: lid, .. }),
                TyDecl::StructDecl(StructDecl { decl_id: rid, .. }),
            ) => decl_engine.get(lid).eq(&decl_engine.get(rid), ctx),
            (
                TyDecl::EnumDecl(EnumDecl { decl_id: lid, .. }),
                TyDecl::EnumDecl(EnumDecl { decl_id: rid, .. }),
            ) => decl_engine.get(lid).eq(&decl_engine.get(rid), ctx),
            (
                TyDecl::EnumVariantDecl(EnumVariantDecl {
                    enum_ref: l_enum,
                    variant_name: ln,
                    ..
                }),
                TyDecl::EnumVariantDecl(EnumVariantDecl {
                    enum_ref: r_enum,
                    variant_name: rn,
                    ..
                }),
            ) => {
                ln == rn
                    && decl_engine
                        .get_enum(l_enum)
                        .eq(&decl_engine.get_enum(r_enum), ctx)
            }
            (
                TyDecl::ImplTrait(ImplTrait { decl_id: lid, .. }),
                TyDecl::ImplTrait(ImplTrait { decl_id: rid, .. }),
            ) => decl_engine.get(lid).eq(&decl_engine.get(rid), ctx),
            (
                TyDecl::AbiDecl(AbiDecl { decl_id: lid, .. }),
                TyDecl::AbiDecl(AbiDecl { decl_id: rid, .. }),
            ) => decl_engine.get(lid).eq(&decl_engine.get(rid), ctx),
            (
                TyDecl::StorageDecl(StorageDecl { decl_id: lid, .. }),
                TyDecl::StorageDecl(StorageDecl { decl_id: rid, .. }),
            ) => decl_engine.get(lid).eq(&decl_engine.get(rid), ctx),
            (
                TyDecl::TypeAliasDecl(TypeAliasDecl { decl_id: lid, .. }),
                TyDecl::TypeAliasDecl(TypeAliasDecl { decl_id: rid, .. }),
            ) => decl_engine.get(lid).eq(&decl_engine.get(rid), ctx),
            (
                TyDecl::GenericTypeForFunctionScope(GenericTypeForFunctionScope {
                    name: xn,
                    type_id: xti,
                }),
                TyDecl::GenericTypeForFunctionScope(GenericTypeForFunctionScope {
                    name: yn,
                    type_id: yti,
                }),
            ) => xn == yn && type_engine.get(*xti).eq(&type_engine.get(*yti), ctx),
            (TyDecl::ErrorRecovery(x, _), TyDecl::ErrorRecovery(y, _)) => x == y,
            _ => false,
        }
    }
}

impl HashWithEngines for TyDecl {
    fn hash<H: Hasher>(&self, state: &mut H, engines: &Engines) {
        let decl_engine = engines.de();
        let type_engine = engines.te();
        std::mem::discriminant(self).hash(state);
        match self {
            TyDecl::VariableDecl(decl) => {
                decl.hash(state, engines);
            }
            TyDecl::ConstantDecl(ConstantDecl { decl_id, .. }) => {
                decl_engine.get(decl_id).hash(state, engines);
            }
            TyDecl::TraitTypeDecl(TraitTypeDecl { decl_id, .. }) => {
                decl_engine.get(decl_id).hash(state, engines);
            }
            TyDecl::FunctionDecl(FunctionDecl { decl_id, .. }) => {
                decl_engine.get(decl_id).hash(state, engines);
            }
            TyDecl::TraitDecl(TraitDecl { decl_id, .. }) => {
                decl_engine.get(decl_id).hash(state, engines);
            }
            TyDecl::StructDecl(StructDecl { decl_id, .. }) => {
                decl_engine.get(decl_id).hash(state, engines);
            }
            TyDecl::EnumDecl(EnumDecl { decl_id, .. }) => {
                decl_engine.get(decl_id).hash(state, engines);
            }
            TyDecl::EnumVariantDecl(EnumVariantDecl {
                enum_ref,
                variant_name,
                ..
            }) => {
                enum_ref.hash(state, engines);
                variant_name.hash(state);
            }
            TyDecl::ImplTrait(ImplTrait { decl_id, .. }) => {
                decl_engine.get(decl_id).hash(state, engines);
            }
            TyDecl::AbiDecl(AbiDecl { decl_id, .. }) => {
                decl_engine.get(decl_id).hash(state, engines);
            }
            TyDecl::TypeAliasDecl(TypeAliasDecl { decl_id, .. }) => {
                decl_engine.get(decl_id).hash(state, engines);
            }
            TyDecl::StorageDecl(StorageDecl { decl_id, .. }) => {
                decl_engine.get(decl_id).hash(state, engines);
            }
            TyDecl::GenericTypeForFunctionScope(GenericTypeForFunctionScope { name, type_id }) => {
                name.hash(state);
                type_engine.get(*type_id).hash(state, engines);
            }
            TyDecl::ErrorRecovery(..) => {}
        }
    }
}

impl SubstTypes for TyDecl {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: &Engines) -> HasChanges {
        match self {
            TyDecl::VariableDecl(ref mut var_decl) => var_decl.subst(type_mapping, engines),
            TyDecl::FunctionDecl(FunctionDecl {
                ref mut decl_id, ..
            }) => decl_id.subst(type_mapping, engines),
            TyDecl::TraitDecl(TraitDecl {
                ref mut decl_id, ..
            }) => decl_id.subst(type_mapping, engines),
            TyDecl::StructDecl(StructDecl {
                ref mut decl_id, ..
            }) => decl_id.subst(type_mapping, engines),
            TyDecl::EnumDecl(EnumDecl {
                ref mut decl_id, ..
            }) => decl_id.subst(type_mapping, engines),
            TyDecl::EnumVariantDecl(EnumVariantDecl {
                ref mut enum_ref, ..
            }) => enum_ref.subst(type_mapping, engines),
            TyDecl::ImplTrait(ImplTrait {
                ref mut decl_id, ..
            }) => decl_id.subst(type_mapping, engines),
            TyDecl::TypeAliasDecl(TypeAliasDecl {
                ref mut decl_id, ..
            }) => decl_id.subst(type_mapping, engines),
            TyDecl::TraitTypeDecl(TraitTypeDecl {
                ref mut decl_id, ..
            }) => decl_id.subst(type_mapping, engines),
            // generics in an ABI is unsupported by design
            TyDecl::AbiDecl(_)
            | TyDecl::ConstantDecl(_)
            | TyDecl::StorageDecl(_)
            | TyDecl::GenericTypeForFunctionScope(_)
            | TyDecl::ErrorRecovery(..) => HasChanges::No,
        }
    }
}

impl SpannedWithEngines for TyDecl {
    fn span(&self, engines: &Engines) -> Span {
        match self {
            TyDecl::ConstantDecl(ConstantDecl { decl_id, .. }) => {
                let const_decl = engines.de().get(decl_id);
                const_decl.span.clone()
            }
            TyDecl::TraitTypeDecl(TraitTypeDecl { decl_id }) => {
                engines.de().get_type(decl_id).span.clone()
            }
            TyDecl::FunctionDecl(FunctionDecl { decl_id }) => {
                engines.de().get_function(decl_id).span.clone()
            }
            TyDecl::TraitDecl(TraitDecl { decl_id }) => {
                engines.de().get_trait(decl_id).span.clone()
            }
            TyDecl::StructDecl(StructDecl { decl_id }) => {
                engines.de().get_struct(decl_id).span.clone()
            }
            TyDecl::EnumDecl(EnumDecl { decl_id }) => engines.de().get_enum(decl_id).span.clone(),
            TyDecl::ImplTrait(ImplTrait { decl_id }) => {
                engines.de().get_impl_trait(decl_id).span.clone()
            }
            TyDecl::AbiDecl(AbiDecl { decl_id }) => engines.de().get_abi(decl_id).span.clone(),
            TyDecl::VariableDecl(decl) => decl.name.span(),
            TyDecl::StorageDecl(StorageDecl { decl_id }) => engines.de().get(decl_id).span.clone(),
            TyDecl::TypeAliasDecl(TypeAliasDecl { decl_id }) => {
                engines.de().get(decl_id).span.clone()
            }
            TyDecl::EnumVariantDecl(EnumVariantDecl {
                variant_decl_span, ..
            }) => variant_decl_span.clone(),
            TyDecl::GenericTypeForFunctionScope(GenericTypeForFunctionScope { name, .. }) => {
                name.span()
            }
            TyDecl::ErrorRecovery(span, _) => span.clone(),
        }
    }
}

impl DisplayWithEngines for TyDecl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: &Engines) -> std::fmt::Result {
        let type_engine = engines.te();
        write!(
            f,
            "{} declaration ({})",
            self.friendly_type_name(),
            match self {
                TyDecl::VariableDecl(decl) => {
                    let TyVariableDecl {
                        mutability,
                        name,
                        type_ascription,
                        body,
                        ..
                    } = &**decl;
                    let mut builder = String::new();
                    match mutability {
                        VariableMutability::Mutable => builder.push_str("mut"),
                        VariableMutability::RefMutable => builder.push_str("ref mut"),
                        VariableMutability::Immutable => {}
                    }
                    builder.push_str(name.as_str());
                    builder.push_str(": ");
                    builder.push_str(
                        &engines
                            .help_out(&*type_engine.get(type_ascription.type_id))
                            .to_string(),
                    );
                    builder.push_str(" = ");
                    builder.push_str(&engines.help_out(body).to_string());
                    builder
                }
                TyDecl::FunctionDecl(FunctionDecl { decl_id }) => {
                    engines.de().get(decl_id).name.as_str().into()
                }
                TyDecl::TraitDecl(TraitDecl { decl_id }) => {
                    engines.de().get(decl_id).name.as_str().into()
                }
                TyDecl::StructDecl(StructDecl { decl_id }) => {
                    engines.de().get(decl_id).name().as_str().into()
                }
                TyDecl::EnumDecl(EnumDecl { decl_id }) => {
                    engines.de().get(decl_id).name().as_str().into()
                }
                TyDecl::ImplTrait(ImplTrait { decl_id }) => {
                    engines.de().get(decl_id).name().as_str().into()
                }
                TyDecl::TypeAliasDecl(TypeAliasDecl { decl_id }) =>
                    engines.de().get(decl_id).name().as_str().into(),
                _ => String::new(),
            }
        )
    }
}

impl DebugWithEngines for TyDecl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: &Engines) -> std::fmt::Result {
        DisplayWithEngines::fmt(&self, f, engines)
    }
}

impl CollectTypesMetadata for TyDecl {
    // this is only run on entry nodes, which must have all well-formed types
    fn collect_types_metadata(
        &self,
        handler: &Handler,
        ctx: &mut CollectTypesMetadataContext,
    ) -> Result<Vec<TypeMetadata>, ErrorEmitted> {
        let decl_engine = ctx.engines.de();
        let metadata = match self {
            TyDecl::VariableDecl(decl) => {
                let mut body = decl.body.collect_types_metadata(handler, ctx)?;
                body.append(
                    &mut decl
                        .type_ascription
                        .type_id
                        .collect_types_metadata(handler, ctx)?,
                );
                body
            }
            TyDecl::FunctionDecl(FunctionDecl { decl_id, .. }) => {
                let decl = decl_engine.get_function(decl_id);
                decl.collect_types_metadata(handler, ctx)?
            }
            TyDecl::ConstantDecl(ConstantDecl { decl_id, .. }) => {
                let const_decl = decl_engine.get_constant(decl_id);
                let TyConstantDecl { value, .. } = &*const_decl;
                if let Some(value) = value {
                    value.collect_types_metadata(handler, ctx)?
                } else {
                    return Ok(vec![]);
                }
            }
            TyDecl::ErrorRecovery(..)
            | TyDecl::StorageDecl(_)
            | TyDecl::TraitDecl(_)
            | TyDecl::StructDecl(_)
            | TyDecl::EnumDecl(_)
            | TyDecl::EnumVariantDecl(_)
            | TyDecl::ImplTrait(_)
            | TyDecl::AbiDecl(_)
            | TyDecl::TypeAliasDecl(_)
            | TyDecl::TraitTypeDecl(_)
            | TyDecl::GenericTypeForFunctionScope(_) => vec![],
        };
        Ok(metadata)
    }
}

impl GetDeclIdent for TyDecl {
    fn get_decl_ident(&self, engines: &Engines) -> Option<Ident> {
        match self {
            TyDecl::ConstantDecl(ConstantDecl { decl_id }) => {
                Some(engines.de().get_constant(decl_id).name().clone())
            }
            TyDecl::TraitTypeDecl(TraitTypeDecl { decl_id }) => {
                Some(engines.de().get_type(decl_id).name().clone())
            }
            TyDecl::FunctionDecl(FunctionDecl { decl_id }) => {
                Some(engines.de().get(decl_id).name.clone())
            }
            TyDecl::TraitDecl(TraitDecl { decl_id }) => {
                Some(engines.de().get(decl_id).name.clone())
            }
            TyDecl::StructDecl(StructDecl { decl_id }) => {
                Some(engines.de().get(decl_id).name().clone())
            }
            TyDecl::EnumDecl(EnumDecl { decl_id }) => {
                Some(engines.de().get(decl_id).name().clone())
            }
            TyDecl::ImplTrait(ImplTrait { decl_id }) => {
                Some(engines.de().get(decl_id).name().clone())
            }
            TyDecl::AbiDecl(AbiDecl { decl_id }) => Some(engines.de().get(decl_id).name().clone()),
            TyDecl::VariableDecl(decl) => Some(decl.name.clone()),
            TyDecl::TypeAliasDecl(TypeAliasDecl { decl_id }) => {
                Some(engines.de().get(decl_id).name().clone())
            }
            TyDecl::GenericTypeForFunctionScope(GenericTypeForFunctionScope { name, .. }) => {
                Some(name.clone())
            }
            TyDecl::EnumVariantDecl(EnumVariantDecl { variant_name, .. }) => {
                Some(variant_name.clone())
            }
            TyDecl::ErrorRecovery(..) => None,
            TyDecl::StorageDecl(_) => None,
        }
    }
}

impl TyDecl {
    /// Retrieves the declaration as a `DeclRef<DeclId<TyEnumDecl>>`.
    ///
    /// Returns an error if `self` is not the [TyDecl][EnumDecl] variant.
    pub(crate) fn to_enum_ref(
        &self,
        handler: &Handler,
        engines: &Engines,
    ) -> Result<DeclRefEnum, ErrorEmitted> {
        match self {
            TyDecl::EnumDecl(EnumDecl { decl_id }) => {
                let enum_decl = engines.de().get_enum(decl_id);
                Ok(DeclRef::new(
                    enum_decl.name().clone(),
                    *decl_id,
                    enum_decl.span.clone(),
                ))
            }
            TyDecl::TypeAliasDecl(TypeAliasDecl { decl_id, .. }) => {
                let alias_decl = engines.de().get_type_alias(decl_id);
                let TyTypeAliasDecl { ty, span, .. } = &*alias_decl;
                engines
                    .te()
                    .get(ty.type_id)
                    .expect_enum(handler, engines, "", span)
            }
            // `Self` type parameter might resolve to an Enum
            TyDecl::GenericTypeForFunctionScope(GenericTypeForFunctionScope {
                type_id, ..
            }) => match &*engines.te().get(*type_id) {
                TypeInfo::Enum(r) => Ok(r.clone()),
                _ => Err(handler.emit_err(CompileError::DeclIsNotAnEnum {
                    actually: self.friendly_type_name().to_string(),
                    span: self.span(engines),
                })),
            },
            TyDecl::ErrorRecovery(_, err) => Err(*err),
            decl => Err(handler.emit_err(CompileError::DeclIsNotAnEnum {
                actually: decl.friendly_type_name().to_string(),
                span: decl.span(engines),
            })),
        }
    }

    /// Retrieves the declaration as a `DeclRef<DeclId<TyStructDecl>>`.
    ///
    /// Returns an error if `self` is not the [TyDecl][StructDecl] variant.
    pub(crate) fn to_struct_ref(
        &self,
        handler: &Handler,
        engines: &Engines,
    ) -> Result<DeclRefStruct, ErrorEmitted> {
        match self {
            TyDecl::StructDecl(StructDecl { decl_id }) => {
                let struct_decl = engines.de().get_struct(decl_id);
                Ok(DeclRef::new(
                    struct_decl.name().clone(),
                    *decl_id,
                    struct_decl.span.clone(),
                ))
            }
            TyDecl::TypeAliasDecl(TypeAliasDecl { decl_id, .. }) => {
                let alias_decl = engines.de().get_type_alias(decl_id);
                let TyTypeAliasDecl { ty, span, .. } = &*alias_decl;
                engines
                    .te()
                    .get(ty.type_id)
                    .expect_struct(handler, engines, span)
            }
            TyDecl::ErrorRecovery(_, err) => Err(*err),
            decl => Err(handler.emit_err(CompileError::DeclIsNotAStruct {
                actually: decl.friendly_type_name().to_string(),
                span: decl.span(engines),
            })),
        }
    }

    /// Retrieves the declaration as a `DeclRef<DeclId<TyFunctionDecl>>`.
    ///
    /// Returns an error if `self` is not the [TyDecl][FunctionDecl] variant.
    pub(crate) fn to_fn_ref(
        &self,
        handler: &Handler,
        engines: &Engines,
    ) -> Result<DeclRefFunction, ErrorEmitted> {
        match self {
            TyDecl::FunctionDecl(FunctionDecl { decl_id }) => {
                let decl = engines.de().get(decl_id);
                Ok(DeclRef::new(decl.name.clone(), *decl_id, decl.span.clone()))
            }
            TyDecl::ErrorRecovery(_, err) => Err(*err),
            decl => Err(handler.emit_err(CompileError::DeclIsNotAFunction {
                actually: decl.friendly_type_name().to_string(),
                span: decl.span(engines),
            })),
        }
    }

    /// Retrieves the declaration as a variable declaration.
    ///
    /// Returns an error if `self` is not a [TyVariableDecl].
    pub(crate) fn expect_variable(
        &self,
        handler: &Handler,
        engines: &Engines,
    ) -> Result<&TyVariableDecl, ErrorEmitted> {
        match self {
            TyDecl::VariableDecl(decl) => Ok(decl),
            TyDecl::ErrorRecovery(_, err) => Err(*err),
            decl => Err(handler.emit_err(CompileError::DeclIsNotAVariable {
                actually: decl.friendly_type_name().to_string(),
                span: decl.span(engines),
            })),
        }
    }

    /// Retrieves the declaration as a `DeclRef<DeclId<TyAbiDecl>>`.
    ///
    /// Returns an error if `self` is not the [TyDecl][AbiDecl] variant.
    pub(crate) fn to_abi_ref(
        &self,
        handler: &Handler,
        engines: &Engines,
    ) -> Result<DeclRef<DeclId<TyAbiDecl>>, ErrorEmitted> {
        match self {
            TyDecl::AbiDecl(AbiDecl { decl_id }) => {
                let abi_decl = engines.de().get_abi(decl_id);
                Ok(DeclRef::new(
                    abi_decl.name().clone(),
                    *decl_id,
                    abi_decl.span.clone(),
                ))
            }
            TyDecl::ErrorRecovery(_, err) => Err(*err),
            decl => Err(handler.emit_err(CompileError::DeclIsNotAnAbi {
                actually: decl.friendly_type_name().to_string(),
                span: decl.span(engines),
            })),
        }
    }

    /// Retrieves the declaration as a `DeclRef<DeclId<TyConstantDecl>>`.
    ///
    /// Returns an error if `self` is not the [TyDecl][ConstantDecl] variant.
    pub(crate) fn to_const_ref(
        &self,
        handler: &Handler,
        engines: &Engines,
    ) -> Result<DeclRef<DeclId<TyConstantDecl>>, ErrorEmitted> {
        match self {
            TyDecl::ConstantDecl(ConstantDecl { decl_id }) => {
                let const_decl = engines.de().get_constant(decl_id);
                Ok(DeclRef::new(
                    const_decl.name().clone(),
                    *decl_id,
                    const_decl.span.clone(),
                ))
            }
            TyDecl::ErrorRecovery(_, err) => Err(*err),
            decl => Err(handler.emit_err(CompileError::DeclIsNotAConstant {
                actually: decl.friendly_type_name().to_string(),
                span: decl.span(engines),
            })),
        }
    }

    /// Friendly name string used for error reporting,
    /// which consists of the identifier for the declaration.
    pub fn friendly_name(&self, engines: &Engines) -> String {
        let decl_engine = engines.de();
        let type_engine = engines.te();
        match self {
            TyDecl::ImplTrait(ImplTrait { decl_id, .. }) => {
                let decl = decl_engine.get_impl_trait(decl_id);
                let implementing_for_type_id_arc = type_engine.get(decl.implementing_for.type_id);
                let implementing_for_type_id = &*implementing_for_type_id_arc;
                format!(
                    "{} for {:?}",
                    self.get_decl_ident(engines)
                        .map_or(String::from(""), |f| f.as_str().to_string()),
                    engines.help_out(implementing_for_type_id)
                )
            }
            _ => self
                .get_decl_ident(engines)
                .map_or(String::from(""), |f| f.as_str().to_string()),
        }
    }

    /// Friendly type name string used for various reporting,
    /// which consists of the type name of the declaration AST node.
    ///
    /// Note that all friendly type names are lowercase.
    /// This is also the case for acronyms like ABI.
    /// For contexts in which acronyms need to be uppercase, like
    /// e.g., error reporting, use `friendly_type_name_with_acronym`
    /// instead.
    pub fn friendly_type_name(&self) -> &'static str {
        use TyDecl::*;
        match self {
            VariableDecl(_) => "variable",
            ConstantDecl(_) => "constant",
            TraitTypeDecl(_) => "type",
            FunctionDecl(_) => "function",
            TraitDecl(_) => "trait",
            StructDecl(_) => "struct",
            EnumDecl(_) => "enum",
            EnumVariantDecl(_) => "enum variant",
            ImplTrait(_) => "impl trait",
            AbiDecl(_) => "abi",
            GenericTypeForFunctionScope(_) => "generic type parameter",
            ErrorRecovery(_, _) => "error",
            StorageDecl(_) => "contract storage",
            TypeAliasDecl(_) => "type alias",
        }
    }

    pub fn friendly_type_name_with_acronym(&self) -> &'static str {
        match self.friendly_type_name() {
            "abi" => "ABI",
            friendly_name => friendly_name,
        }
    }

    pub(crate) fn return_type(
        &self,
        handler: &Handler,
        engines: &Engines,
    ) -> Result<TypeId, ErrorEmitted> {
        let type_engine = engines.te();
        let decl_engine = engines.de();
        let type_id = match self {
            TyDecl::VariableDecl(decl) => decl.body.return_type,
            TyDecl::FunctionDecl(FunctionDecl { decl_id, .. }) => {
                let decl = decl_engine.get_function(decl_id);
                decl.return_type.type_id
            }
            TyDecl::StructDecl(StructDecl { decl_id }) => {
                let decl = decl_engine.get_struct(decl_id);
                type_engine.insert(
                    engines,
                    TypeInfo::Struct(DeclRef::new(
                        decl.name().clone(),
                        *decl_id,
                        decl.span.clone(),
                    )),
                    decl.name().span().source_id(),
                )
            }
            TyDecl::EnumDecl(EnumDecl { decl_id }) => {
                let decl = decl_engine.get_enum(decl_id);
                type_engine.insert(
                    engines,
                    TypeInfo::Enum(DeclRef::new(
                        decl.name().clone(),
                        *decl_id,
                        decl.span.clone(),
                    )),
                    decl.name().span().source_id(),
                )
            }
            TyDecl::StorageDecl(StorageDecl { decl_id, .. }) => {
                let storage_decl = decl_engine.get_storage(decl_id);
                type_engine.insert(
                    engines,
                    TypeInfo::Storage {
                        fields: storage_decl.fields_as_typed_struct_fields(),
                    },
                    storage_decl.span().source_id(),
                )
            }
            TyDecl::TypeAliasDecl(TypeAliasDecl { decl_id, .. }) => {
                let decl = decl_engine.get_type_alias(decl_id);
                decl.create_type_id(engines)
            }
            TyDecl::GenericTypeForFunctionScope(GenericTypeForFunctionScope {
                type_id, ..
            }) => *type_id,
            decl => {
                return Err(handler.emit_err(CompileError::NotAType {
                    span: decl.span(engines),
                    name: engines.help_out(decl).to_string(),
                    actually_is: decl.friendly_type_name(),
                }));
            }
        };
        Ok(type_id)
    }

    pub(crate) fn visibility(&self, decl_engine: &DeclEngine) -> Visibility {
        match self {
            TyDecl::TraitDecl(TraitDecl { decl_id, .. }) => {
                decl_engine.get_trait(decl_id).visibility
            }
            TyDecl::ConstantDecl(ConstantDecl { decl_id, .. }) => {
                decl_engine.get_constant(decl_id).visibility
            }
            TyDecl::StructDecl(StructDecl { decl_id, .. }) => {
                decl_engine.get_struct(decl_id).visibility
            }
            TyDecl::EnumDecl(EnumDecl { decl_id, .. }) => decl_engine.get_enum(decl_id).visibility,
            TyDecl::EnumVariantDecl(EnumVariantDecl { enum_ref, .. }) => {
                decl_engine.get_enum(enum_ref.id()).visibility
            }
            TyDecl::FunctionDecl(FunctionDecl { decl_id, .. }) => {
                decl_engine.get_function(decl_id).visibility
            }
            TyDecl::TypeAliasDecl(TypeAliasDecl { decl_id, .. }) => {
                decl_engine.get_type_alias(decl_id).visibility
            }
            TyDecl::GenericTypeForFunctionScope(_)
            | TyDecl::ImplTrait(_)
            | TyDecl::StorageDecl(_)
            | TyDecl::AbiDecl(_)
            | TyDecl::TraitTypeDecl(_)
            | TyDecl::ErrorRecovery(_, _) => Visibility::Public,
            TyDecl::VariableDecl(decl) => decl.mutability.visibility(),
        }
    }
}

impl From<DeclRef<DeclId<TyTraitType>>> for TyDecl {
    fn from(decl_ref: DeclRef<DeclId<TyTraitType>>) -> Self {
        TyDecl::TraitTypeDecl(TraitTypeDecl {
            decl_id: *decl_ref.id(),
        })
    }
}

impl From<DeclRef<DeclId<TyConstantDecl>>> for TyDecl {
    fn from(decl_ref: DeclRef<DeclId<TyConstantDecl>>) -> Self {
        TyDecl::ConstantDecl(ConstantDecl {
            decl_id: *decl_ref.id(),
        })
    }
}

impl From<DeclRef<DeclId<TyEnumDecl>>> for TyDecl {
    fn from(decl_ref: DeclRef<DeclId<TyEnumDecl>>) -> Self {
        TyDecl::EnumDecl(EnumDecl {
            decl_id: *decl_ref.id(),
        })
    }
}

impl From<DeclRef<DeclId<TyFunctionDecl>>> for TyDecl {
    fn from(decl_ref: DeclRef<DeclId<TyFunctionDecl>>) -> Self {
        TyDecl::FunctionDecl(FunctionDecl {
            decl_id: *decl_ref.id(),
        })
    }
}

impl From<DeclRef<DeclId<TyTraitDecl>>> for TyDecl {
    fn from(decl_ref: DeclRef<DeclId<TyTraitDecl>>) -> Self {
        TyDecl::TraitDecl(TraitDecl {
            decl_id: *decl_ref.id(),
        })
    }
}

impl From<DeclRef<DeclId<TyImplTrait>>> for TyDecl {
    fn from(decl_ref: DeclRef<DeclId<TyImplTrait>>) -> Self {
        TyDecl::ImplTrait(ImplTrait {
            decl_id: *decl_ref.id(),
        })
    }
}

impl From<DeclRef<DeclId<TyStructDecl>>> for TyDecl {
    fn from(decl_ref: DeclRef<DeclId<TyStructDecl>>) -> Self {
        TyDecl::StructDecl(StructDecl {
            decl_id: *decl_ref.id(),
        })
    }
}

impl From<DeclRef<DeclId<TyAbiDecl>>> for TyDecl {
    fn from(decl_ref: DeclRef<DeclId<TyAbiDecl>>) -> Self {
        TyDecl::AbiDecl(AbiDecl {
            decl_id: *decl_ref.id(),
        })
    }
}

impl From<DeclRef<DeclId<TyStorageDecl>>> for TyDecl {
    fn from(decl_ref: DeclRef<DeclId<TyStorageDecl>>) -> Self {
        TyDecl::StorageDecl(StorageDecl {
            decl_id: *decl_ref.id(),
        })
    }
}
impl From<DeclRef<DeclId<TyTypeAliasDecl>>> for TyDecl {
    fn from(decl_ref: DeclRef<DeclId<TyTypeAliasDecl>>) -> Self {
        TyDecl::TypeAliasDecl(TypeAliasDecl {
            decl_id: *decl_ref.id(),
        })
    }
}
