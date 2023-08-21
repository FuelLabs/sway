use std::{
    fmt,
    hash::{Hash, Hasher},
};

use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
};
use sway_types::{Ident, Span, Spanned};

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
    TypeDecl(TypeDecl),
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
    pub name: Ident,
    pub decl_id: DeclId<TyConstantDecl>,
    pub decl_span: Span,
}

#[derive(Clone, Debug)]
pub struct TypeDecl {
    pub name: Ident,
    pub decl_id: DeclId<TyTraitType>,
    pub decl_span: Span,
}

#[derive(Clone, Debug)]
pub struct FunctionDecl {
    pub name: Ident,
    pub decl_id: DeclId<TyFunctionDecl>,
    pub subst_list: Template<SubstList>,
    pub decl_span: Span,
}

#[derive(Clone, Debug)]
pub struct TraitDecl {
    pub name: Ident,
    pub decl_id: DeclId<TyTraitDecl>,
    pub subst_list: Template<SubstList>,
    pub decl_span: Span,
}

#[derive(Clone, Debug)]
pub struct StructDecl {
    pub name: Ident,
    pub decl_id: DeclId<TyStructDecl>,
    pub subst_list: Template<SubstList>,
    pub decl_span: Span,
}

#[derive(Clone, Debug)]
pub struct EnumDecl {
    pub name: Ident,
    pub decl_id: DeclId<TyEnumDecl>,
    pub subst_list: Template<SubstList>,
    pub decl_span: Span,
}

#[derive(Clone, Debug)]
pub struct EnumVariantDecl {
    pub enum_ref: DeclRefEnum,
    pub variant_name: Ident,
    pub variant_decl_span: Span,
}

#[derive(Clone, Debug)]
pub struct ImplTrait {
    pub name: Ident,
    pub decl_id: DeclId<TyImplTrait>,
    pub subst_list: Template<SubstList>,
    pub decl_span: Span,
}

#[derive(Clone, Debug)]
pub struct AbiDecl {
    pub name: Ident,
    pub decl_id: DeclId<TyAbiDecl>,
    pub decl_span: Span,
}

#[derive(Clone, Debug)]
pub struct GenericTypeForFunctionScope {
    pub name: Ident,
    pub type_id: TypeId,
}

#[derive(Clone, Debug)]
pub struct StorageDecl {
    pub decl_id: DeclId<TyStorageDecl>,
    pub decl_span: Span,
}

#[derive(Clone, Debug)]
pub struct TypeAliasDecl {
    pub name: Ident,
    pub decl_id: DeclId<TyTypeAliasDecl>,
    pub decl_span: Span,
}

impl EqWithEngines for TyDecl {}
impl PartialEqWithEngines for TyDecl {
    fn eq(&self, other: &Self, engines: &Engines) -> bool {
        let decl_engine = engines.de();
        let type_engine = engines.te();
        match (self, other) {
            (TyDecl::VariableDecl(x), TyDecl::VariableDecl(y)) => x.eq(y, engines),
            (
                TyDecl::ConstantDecl(ConstantDecl {
                    name: ln,
                    decl_id: lid,
                    ..
                }),
                TyDecl::ConstantDecl(ConstantDecl {
                    name: rn,
                    decl_id: rid,
                    ..
                }),
            ) => ln == rn && decl_engine.get(lid).eq(&decl_engine.get(rid), engines),
            (
                TyDecl::FunctionDecl(FunctionDecl {
                    name: ln,
                    decl_id: lid,
                    ..
                }),
                TyDecl::FunctionDecl(FunctionDecl {
                    name: rn,
                    decl_id: rid,
                    ..
                }),
            ) => ln == rn && decl_engine.get(lid).eq(&decl_engine.get(rid), engines),
            (
                TyDecl::TraitDecl(TraitDecl {
                    name: ln,
                    decl_id: lid,
                    ..
                }),
                TyDecl::TraitDecl(TraitDecl {
                    name: rn,
                    decl_id: rid,
                    ..
                }),
            ) => ln == rn && decl_engine.get(lid).eq(&decl_engine.get(rid), engines),
            (
                TyDecl::StructDecl(StructDecl {
                    name: ln,
                    decl_id: lid,
                    ..
                }),
                TyDecl::StructDecl(StructDecl {
                    name: rn,
                    decl_id: rid,
                    ..
                }),
            ) => ln == rn && decl_engine.get(lid).eq(&decl_engine.get(rid), engines),
            (
                TyDecl::EnumDecl(EnumDecl {
                    name: ln,
                    decl_id: lid,
                    ..
                }),
                TyDecl::EnumDecl(EnumDecl {
                    name: rn,
                    decl_id: rid,
                    ..
                }),
            ) => ln == rn && decl_engine.get(lid).eq(&decl_engine.get(rid), engines),
            (
                TyDecl::ImplTrait(ImplTrait {
                    name: ln,
                    decl_id: lid,
                    ..
                }),
                TyDecl::ImplTrait(ImplTrait {
                    name: rn,
                    decl_id: rid,
                    ..
                }),
            ) => ln == rn && decl_engine.get(lid).eq(&decl_engine.get(rid), engines),
            (
                TyDecl::AbiDecl(AbiDecl {
                    name: ln,
                    decl_id: lid,
                    ..
                }),
                TyDecl::AbiDecl(AbiDecl {
                    name: rn,
                    decl_id: rid,
                    ..
                }),
            ) => ln == rn && decl_engine.get(lid).eq(&decl_engine.get(rid), engines),
            (
                TyDecl::StorageDecl(StorageDecl { decl_id: lid, .. }),
                TyDecl::StorageDecl(StorageDecl { decl_id: rid, .. }),
            ) => decl_engine.get(lid).eq(&decl_engine.get(rid), engines),
            (
                TyDecl::TypeAliasDecl(TypeAliasDecl { decl_id: lid, .. }),
                TyDecl::TypeAliasDecl(TypeAliasDecl { decl_id: rid, .. }),
            ) => decl_engine.get(lid).eq(&decl_engine.get(rid), engines),
            (
                TyDecl::GenericTypeForFunctionScope(GenericTypeForFunctionScope {
                    name: xn,
                    type_id: xti,
                }),
                TyDecl::GenericTypeForFunctionScope(GenericTypeForFunctionScope {
                    name: yn,
                    type_id: yti,
                }),
            ) => xn == yn && type_engine.get(*xti).eq(&type_engine.get(*yti), engines),
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
            TyDecl::TypeDecl(TypeDecl { decl_id, .. }) => {
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
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: &Engines) {
        match self {
            TyDecl::VariableDecl(ref mut var_decl) => var_decl.subst(type_mapping, engines),
            TyDecl::FunctionDecl(FunctionDecl {
                ref mut decl_id, ..
            }) => {
                decl_id.subst(type_mapping, engines);
            }
            TyDecl::TraitDecl(TraitDecl {
                ref mut decl_id, ..
            }) => {
                decl_id.subst(type_mapping, engines);
            }
            TyDecl::StructDecl(StructDecl {
                ref mut decl_id, ..
            }) => {
                decl_id.subst(type_mapping, engines);
            }
            TyDecl::EnumDecl(EnumDecl {
                ref mut decl_id, ..
            }) => {
                decl_id.subst(type_mapping, engines);
            }
            TyDecl::EnumVariantDecl(EnumVariantDecl {
                ref mut enum_ref, ..
            }) => {
                enum_ref.subst(type_mapping, engines);
            }
            TyDecl::ImplTrait(ImplTrait {
                ref mut decl_id, ..
            }) => {
                decl_id.subst(type_mapping, engines);
            }
            TyDecl::TypeAliasDecl(TypeAliasDecl {
                ref mut decl_id, ..
            }) => {
                decl_id.subst(type_mapping, engines);
            }
            TyDecl::TypeDecl(TypeDecl {
                ref mut decl_id, ..
            }) => {
                decl_id.subst(type_mapping, engines);
            }
            // generics in an ABI is unsupported by design
            TyDecl::AbiDecl(_)
            | TyDecl::ConstantDecl(_)
            | TyDecl::StorageDecl(_)
            | TyDecl::GenericTypeForFunctionScope(_)
            | TyDecl::ErrorRecovery(..) => (),
        }
    }
}

impl ReplaceSelfType for TyDecl {
    fn replace_self_type(&mut self, engines: &Engines, self_type: TypeId) {
        match self {
            TyDecl::VariableDecl(ref mut var_decl) => {
                var_decl.replace_self_type(engines, self_type)
            }
            TyDecl::FunctionDecl(FunctionDecl {
                ref mut decl_id, ..
            }) => decl_id.replace_self_type(engines, self_type),
            TyDecl::TraitDecl(TraitDecl {
                ref mut decl_id, ..
            }) => decl_id.replace_self_type(engines, self_type),
            TyDecl::StructDecl(StructDecl {
                ref mut decl_id, ..
            }) => decl_id.replace_self_type(engines, self_type),
            TyDecl::EnumDecl(EnumDecl {
                ref mut decl_id, ..
            }) => decl_id.replace_self_type(engines, self_type),
            TyDecl::EnumVariantDecl(EnumVariantDecl {
                ref mut enum_ref, ..
            }) => enum_ref.replace_self_type(engines, self_type),
            TyDecl::ImplTrait(ImplTrait {
                ref mut decl_id, ..
            }) => decl_id.replace_self_type(engines, self_type),
            TyDecl::TypeAliasDecl(TypeAliasDecl {
                ref mut decl_id, ..
            }) => decl_id.replace_self_type(engines, self_type),
            TyDecl::TypeDecl(TypeDecl {
                ref mut decl_id, ..
            }) => decl_id.replace_self_type(engines, self_type),
            // generics in an ABI is unsupported by design
            TyDecl::AbiDecl(_)
            | TyDecl::ConstantDecl(_)
            | TyDecl::StorageDecl(_)
            | TyDecl::GenericTypeForFunctionScope(_)
            | TyDecl::ErrorRecovery(..) => (),
        }
    }
}

impl TyDecl {
    pub fn get_fun_decl_ref(&self) -> Option<DeclRefFunction> {
        if let TyDecl::FunctionDecl(FunctionDecl {
            name,
            decl_id,
            subst_list: _,
            decl_span,
        }) = self
        {
            Some(DeclRef::new(name.clone(), *decl_id, decl_span.clone()))
        } else {
            None
        }
    }
}

impl Spanned for TyDecl {
    fn span(&self) -> Span {
        match self {
            TyDecl::VariableDecl(decl) => decl.name.span(),
            TyDecl::FunctionDecl(FunctionDecl { decl_span, .. })
            | TyDecl::TraitDecl(TraitDecl { decl_span, .. })
            | TyDecl::ImplTrait(ImplTrait { decl_span, .. })
            | TyDecl::ConstantDecl(ConstantDecl { decl_span, .. })
            | TyDecl::TypeDecl(TypeDecl { decl_span, .. })
            | TyDecl::StorageDecl(StorageDecl { decl_span, .. })
            | TyDecl::TypeAliasDecl(TypeAliasDecl { decl_span, .. })
            | TyDecl::AbiDecl(AbiDecl { decl_span, .. })
            | TyDecl::StructDecl(StructDecl { decl_span, .. })
            | TyDecl::EnumDecl(EnumDecl { decl_span, .. }) => decl_span.clone(),
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
                            .help_out(type_engine.get(type_ascription.type_id))
                            .to_string(),
                    );
                    builder.push_str(" = ");
                    builder.push_str(&engines.help_out(body).to_string());
                    builder
                }
                TyDecl::FunctionDecl(FunctionDecl { name, .. })
                | TyDecl::TraitDecl(TraitDecl { name, .. })
                | TyDecl::StructDecl(StructDecl { name, .. })
                | TyDecl::TypeAliasDecl(TypeAliasDecl { name, .. })
                | TyDecl::ImplTrait(ImplTrait { name, .. })
                | TyDecl::EnumDecl(EnumDecl { name, .. }) => name.as_str().into(),
                _ => String::new(),
            }
        )
    }
}

impl DebugWithEngines for TyDecl {
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
                        format!(
                            "{:?}",
                            engines.help_out(type_engine.get(type_ascription.type_id))
                        )
                        .as_str(),
                    );
                    builder.push_str(" = ");
                    builder.push_str(format!("{:?}", engines.help_out(body)).as_str());
                    builder
                }
                TyDecl::FunctionDecl(FunctionDecl { name, .. })
                | TyDecl::TraitDecl(TraitDecl { name, .. })
                | TyDecl::StructDecl(StructDecl { name, .. })
                | TyDecl::EnumDecl(EnumDecl { name, .. }) => name.as_str().into(),
                _ => String::new(),
            }
        )
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
                let TyConstantDecl { value, .. } = decl_engine.get_constant(decl_id);
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
            | TyDecl::TypeDecl(_)
            | TyDecl::GenericTypeForFunctionScope(_) => vec![],
        };
        Ok(metadata)
    }
}

impl GetDeclIdent for TyDecl {
    fn get_decl_ident(&self) -> Option<Ident> {
        match self {
            TyDecl::VariableDecl(decl) => Some(decl.name.clone()),
            TyDecl::FunctionDecl(FunctionDecl { name, .. })
            | TyDecl::TraitDecl(TraitDecl { name, .. })
            | TyDecl::ConstantDecl(ConstantDecl { name, .. })
            | TyDecl::ImplTrait(ImplTrait { name, .. })
            | TyDecl::AbiDecl(AbiDecl { name, .. })
            | TyDecl::TypeAliasDecl(TypeAliasDecl { name, .. })
            | TyDecl::TypeDecl(TypeDecl { name, .. })
            | TyDecl::GenericTypeForFunctionScope(GenericTypeForFunctionScope { name, .. })
            | TyDecl::StructDecl(StructDecl { name, .. })
            | TyDecl::EnumDecl(EnumDecl { name, .. }) => Some(name.clone()),
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
            TyDecl::EnumDecl(EnumDecl {
                name,
                decl_id,
                subst_list: _,
                decl_span,
            }) => Ok(DeclRef::new(name.clone(), *decl_id, decl_span.clone())),
            TyDecl::TypeAliasDecl(TypeAliasDecl { decl_id, .. }) => {
                let TyTypeAliasDecl { ty, span, .. } = engines.de().get_type_alias(decl_id);
                engines
                    .te()
                    .get(ty.type_id)
                    .expect_enum(handler, engines, "", &span)
            }
            TyDecl::ErrorRecovery(_, err) => Err(*err),
            decl => Err(handler.emit_err(CompileError::DeclIsNotAnEnum {
                actually: decl.friendly_type_name().to_string(),
                span: decl.span(),
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
            TyDecl::StructDecl(StructDecl {
                name,
                decl_id,
                subst_list: _,
                decl_span,
            }) => Ok(DeclRef::new(name.clone(), *decl_id, decl_span.clone())),
            TyDecl::TypeAliasDecl(TypeAliasDecl { decl_id, .. }) => {
                let TyTypeAliasDecl { ty, span, .. } = engines.de().get_type_alias(decl_id);
                engines
                    .te()
                    .get(ty.type_id)
                    .expect_struct(handler, engines, &span)
            }
            TyDecl::ErrorRecovery(_, err) => Err(*err),
            decl => Err(handler.emit_err(CompileError::DeclIsNotAStruct {
                actually: decl.friendly_type_name().to_string(),
                span: decl.span(),
            })),
        }
    }

    /// Retrieves the declaration as a `DeclRef<DeclId<TyFunctionDecl>>`.
    ///
    /// Returns an error if `self` is not the [TyDecl][FunctionDecl] variant.
    pub(crate) fn to_fn_ref(
        &self,
        handler: &Handler,
    ) -> Result<DeclRef<DeclId<TyFunctionDecl>>, ErrorEmitted> {
        match self {
            TyDecl::FunctionDecl(FunctionDecl {
                name,
                decl_id,
                subst_list: _,
                decl_span,
            }) => Ok(DeclRef::new(name.clone(), *decl_id, decl_span.clone())),
            TyDecl::ErrorRecovery(_, err) => Err(*err),
            decl => Err(handler.emit_err(CompileError::DeclIsNotAFunction {
                actually: decl.friendly_type_name().to_string(),
                span: decl.span(),
            })),
        }
    }

    /// Retrieves the declaration as a variable declaration.
    ///
    /// Returns an error if `self` is not a [TyVariableDecl].
    pub(crate) fn expect_variable(
        &self,
        handler: &Handler,
    ) -> Result<&TyVariableDecl, ErrorEmitted> {
        match self {
            TyDecl::VariableDecl(decl) => Ok(decl),
            TyDecl::ErrorRecovery(_, err) => Err(*err),
            decl => Err(handler.emit_err(CompileError::DeclIsNotAVariable {
                actually: decl.friendly_type_name().to_string(),
                span: decl.span(),
            })),
        }
    }

    /// Retrieves the declaration as a `DeclRef<DeclId<TyAbiDecl>>`.
    ///
    /// Returns an error if `self` is not the [TyDecl][AbiDecl] variant.
    pub(crate) fn to_abi_ref(
        &self,
        handler: &Handler,
    ) -> Result<DeclRef<DeclId<TyAbiDecl>>, ErrorEmitted> {
        match self {
            TyDecl::AbiDecl(AbiDecl {
                name,
                decl_id,
                decl_span,
            }) => Ok(DeclRef::new(name.clone(), *decl_id, decl_span.clone())),
            TyDecl::ErrorRecovery(_, err) => Err(*err),
            decl => Err(handler.emit_err(CompileError::DeclIsNotAnAbi {
                actually: decl.friendly_type_name().to_string(),
                span: decl.span(),
            })),
        }
    }

    /// Retrieves the declaration as a `DeclRef<DeclId<TyConstantDecl>>`.
    ///
    /// Returns an error if `self` is not the [TyDecl][ConstantDecl] variant.
    pub(crate) fn to_const_ref(
        &self,
        handler: &Handler,
    ) -> Result<DeclRef<DeclId<TyConstantDecl>>, ErrorEmitted> {
        match self {
            TyDecl::ConstantDecl(ConstantDecl {
                name,
                decl_id,
                decl_span,
            }) => Ok(DeclRef::new(name.clone(), *decl_id, decl_span.clone())),
            TyDecl::ErrorRecovery(_, err) => Err(*err),
            decl => Err(handler.emit_err(CompileError::DeclIsNotAConstant {
                actually: decl.friendly_type_name().to_string(),
                span: decl.span(),
            })),
        }
    }

    /// friendly name string used for error reporting,
    /// which consists of the the identifier for the declaration.
    pub fn friendly_name(&self, engines: &Engines) -> String {
        let decl_engine = engines.de();
        let type_engine = engines.te();
        match self {
            TyDecl::ImplTrait(ImplTrait { decl_id, .. }) => {
                let decl = decl_engine.get_impl_trait(decl_id);
                let implementing_for_type_id = type_engine.get(decl.implementing_for.type_id);
                format!(
                    "{} for {:?}",
                    self.get_decl_ident()
                        .map_or(String::from(""), |f| f.as_str().to_string()),
                    engines.help_out(implementing_for_type_id)
                )
            }
            _ => self
                .get_decl_ident()
                .map_or(String::from(""), |f| f.as_str().to_string()),
        }
    }

    /// friendly type name string used for error reporting,
    /// which consists of the type name of the declaration AST node.
    pub fn friendly_type_name(&self) -> &'static str {
        use TyDecl::*;
        match self {
            VariableDecl(_) => "variable",
            ConstantDecl(_) => "constant",
            TypeDecl(_) => "type",
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

    /// name string used in `forc doc` file path generation that mirrors `cargo doc`.
    pub fn doc_name(&self) -> &'static str {
        use TyDecl::*;
        match self {
            StructDecl(_) => "struct",
            EnumDecl(_) => "enum",
            TraitDecl(_) => "trait",
            AbiDecl(_) => "abi",
            StorageDecl(_) => "contract_storage",
            ImplTrait(_) => "impl_trait",
            FunctionDecl(_) => "fn",
            ConstantDecl(_) => "constant",
            TypeAliasDecl(_) => "type alias",
            _ => unreachable!("these items are non-documentable"),
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
            TyDecl::StructDecl(StructDecl {
                name,
                decl_id,
                subst_list: _,
                decl_span,
            }) => type_engine.insert(
                engines,
                TypeInfo::Struct(DeclRef::new(name.clone(), *decl_id, decl_span.clone())),
            ),
            TyDecl::EnumDecl(EnumDecl {
                name,
                decl_id,
                subst_list: _,
                decl_span,
            }) => type_engine.insert(
                engines,
                TypeInfo::Enum(DeclRef::new(name.clone(), *decl_id, decl_span.clone())),
            ),
            TyDecl::StorageDecl(StorageDecl { decl_id, .. }) => {
                let storage_decl = decl_engine.get_storage(decl_id);
                type_engine.insert(
                    engines,
                    TypeInfo::Storage {
                        fields: storage_decl.fields_as_typed_struct_fields(),
                    },
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
                    span: decl.span(),
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
                let TyTraitDecl { visibility, .. } = decl_engine.get_trait(decl_id);
                visibility
            }
            TyDecl::ConstantDecl(ConstantDecl { decl_id, .. }) => {
                let TyConstantDecl { visibility, .. } = decl_engine.get_constant(decl_id);
                visibility
            }
            TyDecl::StructDecl(StructDecl { decl_id, .. }) => {
                let TyStructDecl { visibility, .. } = decl_engine.get_struct(decl_id);
                visibility
            }
            TyDecl::EnumDecl(EnumDecl { decl_id, .. }) => {
                let TyEnumDecl { visibility, .. } = decl_engine.get_enum(decl_id);
                visibility
            }
            TyDecl::EnumVariantDecl(EnumVariantDecl { enum_ref, .. }) => {
                let TyEnumDecl { visibility, .. } = decl_engine.get_enum(enum_ref.id());
                visibility
            }
            TyDecl::FunctionDecl(FunctionDecl { decl_id, .. }) => {
                let TyFunctionDecl { visibility, .. } = decl_engine.get_function(decl_id);
                visibility
            }
            TyDecl::TypeAliasDecl(TypeAliasDecl { decl_id, .. }) => {
                let TyTypeAliasDecl { visibility, .. } = decl_engine.get_type_alias(decl_id);
                visibility
            }
            TyDecl::GenericTypeForFunctionScope(_)
            | TyDecl::ImplTrait(_)
            | TyDecl::StorageDecl(_)
            | TyDecl::AbiDecl(_)
            | TyDecl::TypeDecl(_)
            | TyDecl::ErrorRecovery(_, _) => Visibility::Public,
            TyDecl::VariableDecl(decl) => decl.mutability.visibility(),
        }
    }
}

impl From<DeclRef<DeclId<TyTraitType>>> for TyDecl {
    fn from(decl_ref: DeclRef<DeclId<TyTraitType>>) -> Self {
        TyDecl::TypeDecl(TypeDecl {
            name: decl_ref.name().clone(),
            decl_id: *decl_ref.id(),
            decl_span: decl_ref.decl_span().clone(),
        })
    }
}

impl From<DeclRef<DeclId<TyConstantDecl>>> for TyDecl {
    fn from(decl_ref: DeclRef<DeclId<TyConstantDecl>>) -> Self {
        TyDecl::ConstantDecl(ConstantDecl {
            name: decl_ref.name().clone(),
            decl_id: *decl_ref.id(),
            decl_span: decl_ref.decl_span().clone(),
        })
    }
}

impl From<DeclRef<DeclId<TyEnumDecl>>> for TyDecl {
    fn from(decl_ref: DeclRef<DeclId<TyEnumDecl>>) -> Self {
        TyDecl::EnumDecl(EnumDecl {
            name: decl_ref.name().clone(),
            decl_id: *decl_ref.id(),
            subst_list: Template::new(decl_ref.subst_list().clone()),
            decl_span: decl_ref.decl_span().clone(),
        })
    }
}

impl From<DeclRef<DeclId<TyFunctionDecl>>> for TyDecl {
    fn from(decl_ref: DeclRef<DeclId<TyFunctionDecl>>) -> Self {
        TyDecl::FunctionDecl(FunctionDecl {
            name: decl_ref.name().clone(),
            decl_id: *decl_ref.id(),
            subst_list: Template::new(decl_ref.subst_list().clone()),
            decl_span: decl_ref.decl_span().clone(),
        })
    }
}

impl From<DeclRef<DeclId<TyTraitDecl>>> for TyDecl {
    fn from(decl_ref: DeclRef<DeclId<TyTraitDecl>>) -> Self {
        TyDecl::TraitDecl(TraitDecl {
            name: decl_ref.name().clone(),
            decl_id: *decl_ref.id(),
            subst_list: Template::new(decl_ref.subst_list().clone()),
            decl_span: decl_ref.decl_span().clone(),
        })
    }
}

impl From<DeclRef<DeclId<TyImplTrait>>> for TyDecl {
    fn from(decl_ref: DeclRef<DeclId<TyImplTrait>>) -> Self {
        TyDecl::ImplTrait(ImplTrait {
            name: decl_ref.name().clone(),
            decl_id: *decl_ref.id(),
            subst_list: Template::new(decl_ref.subst_list().clone()),
            decl_span: decl_ref.decl_span().clone(),
        })
    }
}

impl From<DeclRef<DeclId<TyStructDecl>>> for TyDecl {
    fn from(decl_ref: DeclRef<DeclId<TyStructDecl>>) -> Self {
        TyDecl::StructDecl(StructDecl {
            name: decl_ref.name().clone(),
            decl_id: *decl_ref.id(),
            subst_list: Template::new(decl_ref.subst_list().clone()),
            decl_span: decl_ref.decl_span().clone(),
        })
    }
}

impl From<DeclRef<DeclId<TyAbiDecl>>> for TyDecl {
    fn from(decl_ref: DeclRef<DeclId<TyAbiDecl>>) -> Self {
        TyDecl::AbiDecl(AbiDecl {
            name: decl_ref.name().clone(),
            decl_id: *decl_ref.id(),
            decl_span: decl_ref.decl_span().clone(),
        })
    }
}

impl From<DeclRef<DeclId<TyStorageDecl>>> for TyDecl {
    fn from(decl_ref: DeclRef<DeclId<TyStorageDecl>>) -> Self {
        TyDecl::StorageDecl(StorageDecl {
            decl_id: *decl_ref.id(),
            decl_span: decl_ref.decl_span().clone(),
        })
    }
}
impl From<DeclRef<DeclId<TyTypeAliasDecl>>> for TyDecl {
    fn from(decl_ref: DeclRef<DeclId<TyTypeAliasDecl>>) -> Self {
        TyDecl::TypeAliasDecl(TypeAliasDecl {
            name: decl_ref.name().clone(),
            decl_id: *decl_ref.id(),
            decl_span: decl_ref.decl_span().clone(),
        })
    }
}
