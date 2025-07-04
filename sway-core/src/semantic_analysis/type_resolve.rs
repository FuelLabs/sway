use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
};
use sway_types::{Ident, Span, Spanned};

use crate::{
    ast_elements::type_parameter::ConstGenericExpr, language::{
        ty::{self, TyTraitItem}, CallPath, CallPathType, QualifiedCallPath
    }, monomorphization::type_decl_opt_to_type_id, namespace::{Module, ModulePath, ResolvedDeclaration, ResolvedTraitImplItem}, type_system::SubstTypes, EnforceTypeArguments, Engines, Namespace, SubstTypesContext, TypeId, TypeInfo
};

use super::namespace::TraitMap;

/// Specifies if visibility checks should be performed as part of name resolution.
#[derive(Clone, Copy, PartialEq)]
pub enum VisibilityCheck {
    Yes,
    No,
}

fn resolve_const_generics_ambiguous(
    expr: &ConstGenericExpr,
    handler: &Handler,
    engines: &Engines,
    namespace: &Namespace,
    mod_path: &ModulePath,
    self_type: Option<TypeId>,
) -> Result<(), ErrorEmitted> {
    match expr {
        ConstGenericExpr::AmbiguousVariableExpression { ident } => {
            let _ = resolve_call_path(
                handler,
                engines,
                namespace,
                mod_path,
                &CallPath { prefixes: vec![], suffix: ident.clone(), callpath_type: CallPathType::Ambiguous },
                self_type,
                VisibilityCheck::No,
            )
            .map(|d| d.expect_typed())?;
        },
        _ => {}
    }
    Ok(())
}

/// Resolve the type of the given [TypeId], replacing any instances of
/// [TypeInfo::Custom] with either a monomorphized struct, monomorphized
/// enum, or a reference to a type parameter.
#[allow(clippy::too_many_arguments)]
pub fn resolve_type(
    handler: &Handler,
    engines: &Engines,
    namespace: &Namespace,
    mod_path: &ModulePath,
    type_id: TypeId,
    span: &Span,
    enforce_type_arguments: EnforceTypeArguments,
    type_info_prefix: Option<&ModulePath>,
    self_type: Option<TypeId>,
    subst_ctx: &SubstTypesContext,
    check_visibility: VisibilityCheck,
) -> Result<TypeId, ErrorEmitted> {
    // if span.as_str().to_lowercase().contains("crazy") {
    //     eprintln!("{}: {}", span.as_str(), std::backtrace::Backtrace::force_capture());
    // }

    let type_engine = engines.te();
    let module_path = type_info_prefix.unwrap_or(mod_path);
    let type_id = match type_engine.get(type_id).as_ref() {
        TypeInfo::Custom {
            qualified_call_path,
            type_arguments,
        } => {
            let type_decl_opt = resolve_qualified_call_path(
                handler,
                engines,
                namespace,
                module_path,
                qualified_call_path,
                self_type,
                subst_ctx,
                check_visibility,
            )
            .ok();

            type_decl_opt_to_type_id(
                handler,
                engines,
                namespace,
                type_decl_opt,
                &qualified_call_path.call_path,
                span,
                enforce_type_arguments,
                mod_path,
                type_arguments.clone(),
                self_type,
                subst_ctx,
            )?
        }
        TypeInfo::Array(elem_ty, length) => {
            let mut elem_ty = elem_ty.clone();
            *elem_ty.type_id_mut() = resolve_type(
                handler,
                engines,
                namespace,
                mod_path,
                elem_ty.type_id(),
                span,
                enforce_type_arguments,
                None,
                self_type,
                subst_ctx,
                check_visibility,
            )
            .unwrap_or_else(|err| engines.te().id_of_error_recovery(err));
            resolve_const_generics_ambiguous(length.expr(), handler, engines, namespace, mod_path, self_type)?;

            engines.te().insert_array(engines, elem_ty, length.clone())
        }
        TypeInfo::Slice(elem_ty) => {
            let mut elem_ty = elem_ty.clone();
            *elem_ty.type_id_mut() = resolve_type(
                handler,
                engines,
                namespace,
                mod_path,
                elem_ty.type_id(),
                span,
                enforce_type_arguments,
                None,
                self_type,
                subst_ctx,
                check_visibility,
            )
            .unwrap_or_else(|err| engines.te().id_of_error_recovery(err));

            engines.te().insert_slice(engines, elem_ty)
        }
        TypeInfo::Tuple(type_arguments) => {
            let mut type_arguments = type_arguments.clone();
            for type_argument in type_arguments.iter_mut() {
                *type_argument.type_id_mut() = resolve_type(
                    handler,
                    engines,
                    namespace,
                    mod_path,
                    type_argument.type_id(),
                    span,
                    enforce_type_arguments,
                    None,
                    self_type,
                    subst_ctx,
                    check_visibility,
                )
                .unwrap_or_else(|err| engines.te().id_of_error_recovery(err));
            }

            engines.te().insert_tuple(engines, type_arguments)
        }
        TypeInfo::TraitType {
            name,
            trait_type_id,
        } => {
            let trait_item_ref = TraitMap::get_trait_item_for_type(
                namespace.current_package_root_module(),
                handler,
                engines,
                name,
                *trait_type_id,
                None,
            )?;

            if let ResolvedTraitImplItem::Typed(TyTraitItem::Type(type_ref)) = trait_item_ref {
                let type_decl = engines.de().get_type(type_ref.id());
                if let Some(ty) = &type_decl.ty {
                    ty.type_id()
                } else {
                    type_id
                }
            } else {
                return Err(handler.emit_err(CompileError::Internal(
                    "Expecting associated type",
                    trait_item_ref.span(engines),
                )));
            }
        }
        TypeInfo::Ref {
            referenced_type,
            to_mutable_value,
        } => {
            let mut ty = referenced_type.clone();
            *ty.type_id_mut() = resolve_type(
                handler,
                engines,
                namespace,
                mod_path,
                ty.type_id(),
                span,
                enforce_type_arguments,
                None,
                self_type,
                subst_ctx,
                check_visibility,
            )
            .unwrap_or_else(|err| engines.te().id_of_error_recovery(err));

            engines.te().insert_ref(engines, *to_mutable_value, ty)
        }
        TypeInfo::StringArray(length) => {
            resolve_const_generics_ambiguous(length.expr(), handler, engines, namespace, mod_path, self_type)?;
            type_id
        }
        _ => type_id,
    };

    let mut type_id = type_id;
    type_id.subst(subst_ctx);

    Ok(type_id)
}

#[allow(clippy::too_many_arguments)]
pub fn resolve_qualified_call_path(
    handler: &Handler,
    engines: &Engines,
    namespace: &Namespace,
    mod_path: &ModulePath,
    qualified_call_path: &QualifiedCallPath,
    self_type: Option<TypeId>,
    subst_ctx: &SubstTypesContext,
    check_visibility: VisibilityCheck,
) -> Result<ResolvedDeclaration, ErrorEmitted> {
    let type_engine = engines.te();
    if let Some(qualified_path_root) = qualified_call_path.clone().qualified_path_root {
        let root_type_id = match &&*type_engine.get(qualified_path_root.ty.type_id()) {
            TypeInfo::Custom {
                qualified_call_path,
                type_arguments,
                ..
            } => {
                let type_decl = resolve_call_path(
                    handler,
                    engines,
                    namespace,
                    mod_path,
                    &qualified_call_path.clone().to_call_path(handler)?,
                    self_type,
                    check_visibility,
                )?;
                type_decl_opt_to_type_id(
                    handler,
                    engines,
                    namespace,
                    Some(type_decl),
                    &qualified_call_path.call_path,
                    &qualified_path_root.ty.span(),
                    EnforceTypeArguments::No,
                    mod_path,
                    type_arguments.clone(),
                    self_type,
                    subst_ctx,
                )?
            }
            _ => qualified_path_root.ty.type_id(),
        };

        let as_trait_opt = match &&*type_engine.get(qualified_path_root.as_trait) {
            TypeInfo::Custom {
                qualified_call_path: call_path,
                ..
            } => Some(
                call_path
                    .clone()
                    .to_call_path(handler)?
                    .to_canonical_path(engines, namespace),
            ),
            _ => None,
        };

        resolve_call_path_and_root_type_id(
            handler,
            engines,
            namespace.current_package_root_module(),
            root_type_id,
            as_trait_opt,
            &qualified_call_path.call_path,
            self_type,
        )
        .map(|(d, _)| d)
    } else {
        resolve_call_path(
            handler,
            engines,
            namespace,
            mod_path,
            &qualified_call_path.call_path,
            self_type,
            check_visibility,
        )
    }
}

/// Resolve a symbol that is potentially prefixed with some path, e.g. `foo::bar::symbol`.
///
/// This will concatenate the `mod_path` with the `call_path`'s prefixes and
/// then calling `resolve_symbol` with the resulting path and call_path's suffix.
///
/// The `mod_path` is significant here as we assume the resolution is done within the
/// context of the module pointed to by `mod_path` and will only check the call path prefixes
/// and the symbol's own visibility.
pub fn resolve_call_path(
    handler: &Handler,
    engines: &Engines,
    namespace: &Namespace,
    mod_path: &ModulePath,
    call_path: &CallPath,
    self_type: Option<TypeId>,
    check_visibility: VisibilityCheck,
) -> Result<ResolvedDeclaration, ErrorEmitted> {
    let full_path = call_path.to_fullpath_from_mod_path(engines, namespace, &mod_path.to_vec());

    let (decl, is_self_type, decl_mod_path) = resolve_symbol_and_mod_path(
        handler,
        engines,
        namespace,
        &full_path.prefixes,
        &full_path.suffix,
        self_type,
    )?;

    if check_visibility == VisibilityCheck::No {
        return Ok(decl);
    }

    // Check that the modules in full_path are visible from the current module.
    let _ = namespace.check_module_visibility(handler, &full_path.prefixes);

    // If the full path is different from the declaration path, then we are accessing a reexport,
    // which is by definition public.
    if decl_mod_path != full_path.prefixes {
        return Ok(decl);
    }

    // All declarations in the current module are visible, regardless of their visibility modifier.
    if decl_mod_path == *namespace.current_mod_path() {
        return Ok(decl);
    }

    // Otherwise, check the visibility modifier
    if !decl.visibility(engines).is_public() && is_self_type == IsSelfType::No {
        handler.emit_err(CompileError::ImportPrivateSymbol {
            name: call_path.suffix.clone(),
            span: call_path.suffix.span(),
        });
    }

    Ok(decl)
}

// Resolve a path. The first identifier in the path is the package name, which may be the
// current package or an external one.
pub(super) fn resolve_symbol_and_mod_path(
    handler: &Handler,
    engines: &Engines,
    namespace: &Namespace,
    mod_path: &ModulePath,
    symbol: &Ident,
    self_type: Option<TypeId>,
) -> Result<(ResolvedDeclaration, IsSelfType, Vec<Ident>), ErrorEmitted> {
    assert!(!mod_path.is_empty());
    if mod_path[0] == *namespace.current_package_name() {
        resolve_symbol_and_mod_path_inner(
            handler,
            engines,
            namespace.current_package_root_module(),
            mod_path,
            symbol,
            self_type,
        )
    } else {
        match namespace.get_external_package(mod_path[0].as_str()) {
            Some(ext_package) => {
                // The path must be resolved in an external package.
                // The root module in that package may have a different name than the name we
                // use to refer to the package, so replace it.
                let mut new_mod_path = vec![ext_package.name().clone()];
                for id in mod_path.iter().skip(1) {
                    new_mod_path.push(id.clone());
                }
                resolve_symbol_and_mod_path_inner(
                    handler,
                    engines,
                    ext_package.root_module(),
                    &new_mod_path,
                    symbol,
                    self_type,
                )
            }
            None => Err(handler.emit_err(crate::namespace::module_not_found(
                mod_path,
                mod_path[0] == *namespace.current_package_name(),
            ))),
        }
    }
}

fn resolve_symbol_and_mod_path_inner(
    handler: &Handler,
    engines: &Engines,
    root_module: &Module,
    mod_path: &ModulePath,
    symbol: &Ident,
    self_type: Option<TypeId>,
) -> Result<(ResolvedDeclaration, IsSelfType, Vec<Ident>), ErrorEmitted> {
    assert!(!mod_path.is_empty());
    assert!(root_module.mod_path().len() == 1);
    assert!(mod_path[0] == root_module.mod_path()[0]);

    // This block tries to resolve associated types
    let mut current_module = root_module;
    let mut current_mod_path = vec![mod_path[0].clone()];
    let mut decl_opt = None;
    let mut is_self_type = IsSelfType::No;
    for ident in mod_path.iter().skip(1) {
        if let Some(decl) = decl_opt {
            let (decl, ret_is_self_type) = resolve_associated_type_or_item(
                handler,
                engines,
                current_module,
                ident,
                decl,
                None,
                self_type,
            )?;
            decl_opt = Some(decl);
            if ret_is_self_type == IsSelfType::Yes {
                is_self_type = IsSelfType::Yes;
            }
        } else {
            match current_module.submodule(&[ident.clone()]) {
                Some(ns) => {
                    current_module = ns;
                    current_mod_path.push(ident.clone());
                }
                None => {
                    if ident.as_str() == "Self" {
                        is_self_type = IsSelfType::Yes;
                    }
                    let (decl, _) = current_module.resolve_symbol(handler, engines, ident)?;
                    decl_opt = Some(decl);
                }
            }
        }
    }
    if let Some(decl) = decl_opt {
        let (decl, ret_is_self_type) = resolve_associated_type_or_item(
            handler,
            engines,
            current_module,
            symbol,
            decl,
            None,
            self_type,
        )?;
        if ret_is_self_type == IsSelfType::Yes {
            is_self_type = IsSelfType::Yes;
        }
        return Ok((decl, is_self_type, current_mod_path));
    }

    root_module
        .lookup_submodule(handler, &mod_path[1..])
        .and_then(|module| {
            let (decl, decl_path) = module.resolve_symbol(handler, engines, symbol)?;
            Ok((decl, is_self_type, decl_path))
        })
}

fn decl_to_type_info(
    handler: &Handler,
    engines: &Engines,
    symbol: &Ident,
    decl: ResolvedDeclaration,
) -> Result<TypeInfo, ErrorEmitted> {
    match decl {
        ResolvedDeclaration::Parsed(_decl) => todo!(),
        ResolvedDeclaration::Typed(decl) => Ok(match decl.clone() {
            ty::TyDecl::StructDecl(struct_ty_decl) => TypeInfo::Struct(struct_ty_decl.decl_id),
            ty::TyDecl::EnumDecl(enum_ty_decl) => TypeInfo::Enum(enum_ty_decl.decl_id),
            ty::TyDecl::TraitTypeDecl(type_decl) => {
                let type_decl = engines.de().get_type(&type_decl.decl_id);
                if type_decl.ty.is_none() {
                    return Err(handler.emit_err(CompileError::Internal(
                        "Trait type declaration has no type",
                        symbol.span(),
                    )));
                }
                (*engines.te().get(type_decl.ty.clone().unwrap().type_id())).clone()
            }
            ty::TyDecl::GenericTypeForFunctionScope(decl) => {
                (*engines.te().get(decl.type_id)).clone()
            }
            _ => {
                return Err(handler.emit_err(CompileError::SymbolNotFound {
                    name: symbol.clone(),
                    span: symbol.span(),
                }))
            }
        }),
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum IsSelfType {
    Yes,
    No,
}

#[allow(clippy::too_many_arguments)]
fn resolve_associated_item_from_type_id(
    handler: &Handler,
    engines: &Engines,
    module: &Module,
    symbol: &Ident,
    type_id: TypeId,
    as_trait: Option<CallPath>,
    self_type: Option<TypeId>,
) -> Result<(ResolvedDeclaration, IsSelfType), ErrorEmitted> {
    let mut is_self_type = IsSelfType::No;
    let type_id = if engines.te().get(type_id).is_self_type() {
        if let Some(self_type) = self_type {
            is_self_type = IsSelfType::Yes;
            self_type
        } else {
            return Err(handler.emit_err(CompileError::Internal(
                "Self type not provided.",
                symbol.span(),
            )));
        }
    } else {
        type_id
    };
    let item_ref =
        TraitMap::get_trait_item_for_type(module, handler, engines, symbol, type_id, as_trait)?;
    let resolved = match item_ref {
        ResolvedTraitImplItem::Parsed(_item) => todo!(),
        ResolvedTraitImplItem::Typed(item) => match item {
            TyTraitItem::Fn(fn_ref) => ResolvedDeclaration::Typed(fn_ref.into()),
            TyTraitItem::Constant(const_ref) => ResolvedDeclaration::Typed(const_ref.into()),
            TyTraitItem::Type(type_ref) => ResolvedDeclaration::Typed(type_ref.into()),
        },
    };
    Ok((resolved, is_self_type))
}

#[allow(clippy::too_many_arguments)]
fn resolve_associated_type_or_item(
    handler: &Handler,
    engines: &Engines,
    module: &Module,
    symbol: &Ident,
    decl: ResolvedDeclaration,
    as_trait: Option<CallPath>,
    self_type: Option<TypeId>,
) -> Result<(ResolvedDeclaration, IsSelfType), ErrorEmitted> {
    let type_info = decl_to_type_info(handler, engines, symbol, decl)?;
    let type_id = engines
        .te()
        .insert(engines, type_info, symbol.span().source_id());

    resolve_associated_item_from_type_id(
        handler, engines, module, symbol, type_id, as_trait, self_type,
    )
}

#[allow(clippy::too_many_arguments)]
fn resolve_call_path_and_root_type_id(
    handler: &Handler,
    engines: &Engines,
    module: &Module,
    root_type_id: TypeId,
    mut as_trait: Option<CallPath>,
    call_path: &CallPath,
    self_type: Option<TypeId>,
) -> Result<(ResolvedDeclaration, IsSelfType), ErrorEmitted> {
    // This block tries to resolve associated types
    let mut decl_opt = None;
    let mut type_id_opt = Some(root_type_id);
    let mut is_self_type = IsSelfType::No;
    for ident in call_path.prefixes.iter() {
        if let Some(type_id) = type_id_opt {
            type_id_opt = None;
            decl_opt = Some(resolve_associated_item_from_type_id(
                handler,
                engines,
                module,
                ident,
                type_id,
                as_trait.clone(),
                self_type,
            )?);
            as_trait = None;
        } else if let Some((decl, ret_is_self_type)) = decl_opt {
            if ret_is_self_type == IsSelfType::Yes {
                is_self_type = IsSelfType::Yes;
            }
            decl_opt = Some(resolve_associated_type_or_item(
                handler,
                engines,
                module,
                ident,
                decl,
                as_trait.clone(),
                self_type,
            )?);
            as_trait = None;
        }
    }
    if let Some(type_id) = type_id_opt {
        let (decl, ret_is_self_type) = resolve_associated_item_from_type_id(
            handler,
            engines,
            module,
            &call_path.suffix,
            type_id,
            as_trait,
            self_type,
        )?;
        if ret_is_self_type == IsSelfType::Yes {
            is_self_type = IsSelfType::Yes;
        }
        return Ok((decl, is_self_type));
    }
    if let Some((decl, ret_is_self_type)) = decl_opt {
        if ret_is_self_type == IsSelfType::Yes {
            is_self_type = IsSelfType::Yes;
        }
        let (decl, ret_is_self_type) = resolve_associated_type_or_item(
            handler,
            engines,
            module,
            &call_path.suffix,
            decl,
            as_trait,
            self_type,
        )?;
        if ret_is_self_type == IsSelfType::Yes {
            is_self_type = IsSelfType::Yes;
        }
        Ok((decl, is_self_type))
    } else {
        Err(handler.emit_err(CompileError::Internal("Unexpected error", call_path.span())))
    }
}
