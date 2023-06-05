use std::collections::HashSet;

use sway_error::error::CompileError;
use sway_types::{Ident, Spanned};

use crate::{
    decl_engine::DeclEngineInsert,
    error::*,
    language::{
        parsed::*,
        ty::{self, TyImplItem, TyTraitItem},
    },
    semantic_analysis::{declaration::insert_supertraits_into_namespace, Mode, TypeCheckContext},
    CompileResult, ReplaceSelfType, TypeId, TypeInfo,
};

impl ty::TyAbiDecl {
    pub(crate) fn type_check(
        ctx: TypeCheckContext,
        abi_decl: AbiDeclaration,
    ) -> CompileResult<Self> {
        let mut warnings = vec![];
        let mut errors = vec![];

        let AbiDeclaration {
            name,
            interface_surface,
            supertraits,
            methods,
            span,
            attributes,
        } = abi_decl;

        // We don't want the user to waste resources by contract calling
        // themselves, and we don't want to do more work in the compiler,
        // so we don't support the case of calling a contract's own interface
        // from itself. This is by design.

        // A temporary namespace for checking within this scope.
        let type_engine = ctx.engines.te();
        let mut abi_namespace = ctx.namespace.clone();
        let self_type = type_engine.insert(ctx.engines(), TypeInfo::SelfType);
        let mut ctx = ctx
            .scoped(&mut abi_namespace)
            .with_mode(Mode::ImplAbiFn)
            .with_self_type(self_type);

        // Recursively make the interface surfaces and methods of the
        // supertraits available to this abi.
        check!(
            insert_supertraits_into_namespace(ctx.by_ref(), self_type, &supertraits),
            return err(warnings, errors),
            warnings,
            errors
        );

        // Type check the interface surface.
        let mut new_interface_surface = vec![];

        let mut ids: HashSet<Ident> = HashSet::default();

        for item in interface_surface.into_iter() {
            let decl_name = match item {
                TraitItem::TraitFn(method) => {
                    let method = check!(
                        ty::TyTraitFn::type_check(ctx.by_ref(), method),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    for param in &method.parameters {
                        if param.is_reference || param.is_mutable {
                            errors.push(CompileError::RefMutableNotAllowedInContractAbi {
                                param_name: param.name.clone(),
                                span: param.name.span(),
                            })
                        }
                    }
                    new_interface_surface.push(ty::TyTraitInterfaceItem::TraitFn(
                        ctx.engines.de().insert(method.clone()),
                    ));
                    method.name.clone()
                }
                TraitItem::Constant(const_decl) => {
                    let const_decl = check!(
                        ty::TyConstantDecl::type_check(ctx.by_ref(), const_decl.clone(),),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    let decl_ref = ctx.engines.de().insert(const_decl.clone());
                    new_interface_surface
                        .push(ty::TyTraitInterfaceItem::Constant(decl_ref.clone()));

                    let const_name = const_decl.call_path.suffix.clone();
                    check!(
                        ctx.namespace.insert_symbol(
                            const_name.clone(),
                            ty::TyDecl::ConstantDecl(ty::ConstantDecl {
                                name: const_name.clone(),
                                decl_id: *decl_ref.id(),
                                decl_span: const_decl.span.clone()
                            })
                        ),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );

                    const_name
                }
            };

            if !ids.insert(decl_name.clone()) {
                errors.push(CompileError::MultipleDefinitionsOfName {
                    name: decl_name.clone(),
                    span: decl_name.span(),
                })
            }
        }

        // Type check the items.
        let mut new_items = vec![];
        for method in methods.into_iter() {
            let method = check!(
                ty::TyFunctionDecl::type_check(ctx.by_ref(), method.clone(), true, false),
                ty::TyFunctionDecl::error(method.clone()),
                warnings,
                errors
            );
            for param in &method.parameters {
                if param.is_reference || param.is_mutable {
                    errors.push(CompileError::RefMutableNotAllowedInContractAbi {
                        param_name: param.name.clone(),
                        span: param.name.span(),
                    })
                }
            }
            new_items.push(TyTraitItem::Fn(ctx.engines.de().insert(method)));
        }

        // Compared to regular traits, we do not insert recursively methods of ABI supertraits
        // into the interface surface, we do not want supertrait methods to be available to
        // the ABI user, only the contract methods can use supertrait methods
        let abi_decl = ty::TyAbiDecl {
            interface_surface: new_interface_surface,
            supertraits,
            items: new_items,
            name,
            span,
            attributes,
        };
        ok(abi_decl, warnings, errors)
    }

    pub(crate) fn insert_interface_surface_and_items_into_namespace(
        &self,
        ctx: TypeCheckContext,
        type_id: TypeId,
    ) {
        let decl_engine = ctx.engines.de();
        let engines = ctx.engines();

        let ty::TyAbiDecl {
            interface_surface,
            items,
            ..
        } = self;

        let mut all_items = vec![];

        for item in interface_surface.iter() {
            match item {
                ty::TyTraitInterfaceItem::TraitFn(decl_ref) => {
                    let mut method = decl_engine.get_trait_fn(decl_ref);
                    method.replace_self_type(engines, type_id);
                    all_items.push(TyImplItem::Fn(
                        ctx.engines
                            .de()
                            .insert(method.to_dummy_func(Mode::ImplAbiFn))
                            .with_parent(ctx.engines.de(), (*decl_ref.id()).into()),
                    ));
                }
                ty::TyTraitInterfaceItem::Constant(decl_ref) => {
                    let const_decl = decl_engine.get_constant(decl_ref);
                    let const_name = const_decl.call_path.suffix.clone();
                    all_items.push(TyImplItem::Constant(decl_ref.clone()));
                    ctx.namespace.insert_symbol(
                        const_name.clone(),
                        ty::TyDecl::ConstantDecl(ty::ConstantDecl {
                            name: const_name,
                            decl_id: *decl_ref.id(),
                            decl_span: const_decl.span.clone(),
                        }),
                    );
                }
            }
        }
        for item in items.iter() {
            match item {
                ty::TyTraitItem::Fn(decl_ref) => {
                    let mut method = decl_engine.get_function(decl_ref);
                    method.replace_self_type(engines, type_id);
                    all_items.push(TyImplItem::Fn(
                        ctx.engines
                            .de()
                            .insert(method)
                            .with_parent(ctx.engines.de(), (*decl_ref.id()).into()),
                    ));
                }
                ty::TyTraitItem::Constant(decl_ref) => {
                    let mut const_decl = decl_engine.get_constant(decl_ref);
                    const_decl.replace_self_type(engines, type_id);
                    all_items.push(TyImplItem::Constant(ctx.engines.de().insert(const_decl)));
                }
            }
        }
    }
}
