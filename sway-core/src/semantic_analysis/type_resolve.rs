use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
};
use sway_types::{Ident, Span, Spanned};
use sway_utils::iter_prefixes;

use crate::{
    language::{
        ty::{self, TyTraitItem},
        CallPath, QualifiedCallPath,
    },
    monomorphization::type_decl_opt_to_type_id,
    namespace::{Module, ModulePath, ResolvedDeclaration, ResolvedTraitImplItem, Root},
    type_system::SubstTypes,
    EnforceTypeArguments, Engines, Namespace, SubstTypesContext, TypeId, TypeInfo,
};

/// Specifies if visibility checks should be performed as part of name resolution.
#[derive(Clone, Copy, PartialEq)]
pub enum VisibilityCheck {
    Yes,
    No,
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
    let type_engine = engines.te();
    let module_path = type_info_prefix.unwrap_or(mod_path);
    let type_id = match (*type_engine.get(type_id)).clone() {
        TypeInfo::Custom {
            qualified_call_path,
            type_arguments,
        } => {
            let type_decl_opt = resolve_qualified_call_path(
                handler,
                engines,
                namespace,
                module_path,
                &qualified_call_path,
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
        TypeInfo::Array(mut elem_ty, length) => {
            elem_ty.type_id = resolve_type(
                handler,
                engines,
                namespace,
                mod_path,
                elem_ty.type_id,
                span,
                enforce_type_arguments,
                None,
                self_type,
                subst_ctx,
                check_visibility,
            )
            .unwrap_or_else(|err| engines.te().id_of_error_recovery(err));

            engines.te().insert_array(engines, elem_ty, length)
        }
        TypeInfo::Slice(mut elem_ty) => {
            elem_ty.type_id = resolve_type(
                handler,
                engines,
                namespace,
                mod_path,
                elem_ty.type_id,
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
        TypeInfo::Tuple(mut type_arguments) => {
            for type_argument in type_arguments.iter_mut() {
                type_argument.type_id = resolve_type(
                    handler,
                    engines,
                    namespace,
                    mod_path,
                    type_argument.type_id,
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
            let trait_item_ref = namespace
                .current_package_root_module()
                .current_items()
                .implemented_traits
                .get_trait_item_for_type(handler, engines, &name, trait_type_id, None)?;

            if let ResolvedTraitImplItem::Typed(TyTraitItem::Type(type_ref)) = trait_item_ref {
                let type_decl = engines.de().get_type(type_ref.id());
                if let Some(ty) = &type_decl.ty {
                    ty.type_id
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
            referenced_type: mut ty,
            to_mutable_value,
        } => {
            ty.type_id = resolve_type(
                handler,
                engines,
                namespace,
                mod_path,
                ty.type_id,
                span,
                enforce_type_arguments,
                None,
                self_type,
                subst_ctx,
                check_visibility,
            )
            .unwrap_or_else(|err| engines.te().id_of_error_recovery(err));

            engines.te().insert_ref(engines, to_mutable_value, ty)
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
        let root_type_id = match &&*type_engine.get(qualified_path_root.ty.type_id) {
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
            _ => qualified_path_root.ty.type_id,
        };

        let as_trait_opt = match &&*type_engine.get(qualified_path_root.as_trait) {
            TypeInfo::Custom {
                qualified_call_path: call_path,
                ..
            } => Some(
                call_path
                    .clone()
                    .to_call_path(handler)?
                    .to_fullpath(engines, namespace),
            ),
            _ => None,
        };

        resolve_call_path_and_root_type_id(
            handler,
            engines,
            &namespace.current_package_root_module(),
            root_type_id,
            as_trait_opt,
            &qualified_call_path.call_path,
            self_type,
        )
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
    //    let full_path = call_path.to_fullpath(engines, namespace);
    
    //    let problem = call_path.suffix.as_str() == "MyStruct"
    //	&& full_path.prefixes.len() == 1
    //	&& full_path.prefixes[0].as_str() == "import_star_name_clash";
    //    if problem {
    //	dbg!(&mod_path);
    //	dbg!(&call_path);
    //	dbg!(&full_path);
    //	dbg!(&namespace.current_mod_path());
    //    }

    let full_path = call_path.to_fullpath_from_mod_path(engines, namespace, &mod_path.to_vec());

    let (decl, decl_mod_path) = resolve_symbol_and_mod_path(
        handler,
        engines,
        namespace.borrow_root(),
//	symbol_path,
        &full_path.prefixes,
//        &call_path.suffix,
        &full_path.suffix,
        self_type,
    )?;

    if check_visibility == VisibilityCheck::No {
        return Ok(decl);
    }

    // Private declarations are visibile within their own module, so no need to check for
    // visibility in that case
    if decl_mod_path == *namespace.current_mod_path() {
	return Ok(decl);
    }

    // check the visibility of the call path elements
    // we don't check the first prefix because direct children are always accessible
    for prefix in iter_prefixes(&call_path.prefixes).skip(1) {
        let module = namespace.require_module_from_absolute_path(handler, &prefix.to_vec())?;
        if module.visibility().is_private() {
            let prefix_last = prefix[prefix.len() - 1].clone();
            handler.emit_err(CompileError::ImportPrivateModule {
                span: prefix_last.span(),
                name: prefix_last,
            });
        }
    }

    // check the visibility of the symbol itself
    if !decl.visibility(engines).is_public() {
        handler.emit_err(CompileError::ImportPrivateSymbol {
            name: call_path.suffix.clone(),
            span: call_path.suffix.span(),
        });
    }

    Ok(decl)
}

// Resolve a path. The first identifier in the path is the package name, which may be the
// current package or an external one.
fn resolve_symbol_and_mod_path(
    handler: &Handler,
    engines: &Engines,
    root: &Root,
    mod_path: &ModulePath,
    symbol: &Ident,
    self_type: Option<TypeId>,
) -> Result<(ResolvedDeclaration, Vec<Ident>), ErrorEmitted> {
    assert!(!mod_path.is_empty());
    if mod_path[0] == *root.current_package_name() {
	resolve_symbol_and_mod_path_inner(handler, engines, root, mod_path, symbol, self_type)
    } else {
	match root.get_external_package(&mod_path[0].to_string()) {
	    Some(ext_root) => {
		// The path must be resolved in an external package.
		// The root module in that package may have a different name than the name we
		// use to refer to the package, so replace it.
		let mut new_mod_path = vec!(ext_root.current_package_name().clone());
		for id in mod_path.iter().skip(1) {
		    new_mod_path.push(id.clone());
		}
		resolve_symbol_and_mod_path_inner(handler, engines, &ext_root, &new_mod_path, symbol, self_type)
	    },
	    None => Err(handler.emit_err(crate::namespace::module_not_found(mod_path)))
	}
    }
}

fn resolve_symbol_and_mod_path_inner(
    handler: &Handler,
    engines: &Engines,
    root: &Root,
    mod_path: &ModulePath,
    symbol: &Ident,
    self_type: Option<TypeId>,
) -> Result<(ResolvedDeclaration, Vec<Ident>), ErrorEmitted> {
    assert!(!mod_path.is_empty());
    assert!(mod_path[0] == *root.current_package_name());

    //	let problem = symbol.as_str() == "MyStruct"
    //	    && mod_path.len() == 1
    //	    && mod_path[0].as_str() == "import_star_name_clash";
    
    
    //	if problem {
    //	    dbg!(&mod_path);
    //	    dbg!(&symbol);
    //	    // b::MyStruct is not resolved correctly
    //	}

    // This block tries to resolve associated types
    let mut current_module = root.current_package_root_module();
    let mut current_mod_path = vec![mod_path[0].clone()];
    let mut decl_opt = None;
    for ident in mod_path.iter().skip(1) {
        if let Some(decl) = decl_opt {
            decl_opt = Some(resolve_associated_type_or_item(
                handler,
                engines,
                current_module,
                ident,
                decl,
                None,
                self_type,
            )?);
        } else {
            match current_module.submodule(&[ident.clone()]) {
                Some(ns) => {
                    current_module = ns;
                    current_mod_path.push(ident.clone());
                }
                None => {
		    if ident.as_str() == "core" {
			dbg!("resolve_symbol_and_mod_path_inner");
			dbg!(&mod_path);
			dbg!(&symbol);
			dbg!(&current_mod_path);
		    }
                    decl_opt = Some(current_module.resolve_symbol(handler, engines, ident, root.current_package_name())?);
                }
            }
        }
    }
    if let Some(decl) = decl_opt {
        let decl = resolve_associated_type_or_item(
            handler,
            engines,
            current_module,
            symbol,
            decl,
            None,
            self_type,
        )?;
        return Ok((decl, current_mod_path));
    }

    root.require_module(handler, &mod_path.to_vec())
        .and_then(|module| {
            let decl = module.resolve_symbol(handler, engines, symbol, root.current_package_name())?;
            Ok((decl, mod_path.to_vec()))
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
                (*engines.te().get(type_decl.ty.clone().unwrap().type_id)).clone()
            }
            ty::TyDecl::GenericTypeForFunctionScope(decl) => {
                (*engines.te().get(decl.type_id)).clone()
            }
            _ =>{ 
//		dbg!("decl_to_type_info");
//		dbg!(&symbol);
                return Err(handler.emit_err(CompileError::SymbolNotFound {
                    name: symbol.clone(),
                    span: symbol.span(),
                }))
            }
        }),
    }
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
) -> Result<ResolvedDeclaration, ErrorEmitted> {
    let type_id = if engines.te().get(type_id).is_self_type() {
        if let Some(self_type) = self_type {
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
    let item_ref = module
        .current_items()
        .implemented_traits
        .get_trait_item_for_type(handler, engines, symbol, type_id, as_trait)?;
    match item_ref {
        ResolvedTraitImplItem::Parsed(_item) => todo!(),
        ResolvedTraitImplItem::Typed(item) => match item {
            TyTraitItem::Fn(fn_ref) => Ok(ResolvedDeclaration::Typed(fn_ref.into())),
            TyTraitItem::Constant(const_ref) => Ok(ResolvedDeclaration::Typed(const_ref.into())),
            TyTraitItem::Type(type_ref) => Ok(ResolvedDeclaration::Typed(type_ref.into())),
        },
    }
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
) -> Result<ResolvedDeclaration, ErrorEmitted> {
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
) -> Result<ResolvedDeclaration, ErrorEmitted> {
    // This block tries to resolve associated types
    let mut decl_opt = None;
    let mut type_id_opt = Some(root_type_id);
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
        } else if let Some(decl) = decl_opt {
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
        let decl = resolve_associated_item_from_type_id(
            handler,
            engines,
            module,
            &call_path.suffix,
            type_id,
            as_trait,
            self_type,
        )?;
        return Ok(decl);
    }
    if let Some(decl) = decl_opt {
        let decl = resolve_associated_type_or_item(
            handler,
            engines,
            module,
            &call_path.suffix,
            decl,
            as_trait,
            self_type,
        )?;
        Ok(decl)
    } else {
        Err(handler.emit_err(CompileError::Internal("Unexpected error", call_path.span())))
    }
}
