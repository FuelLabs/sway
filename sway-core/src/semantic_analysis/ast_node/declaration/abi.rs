use std::collections::{HashMap, HashSet};

use sway_error::error::CompileError;
use sway_types::{Ident, Named, Span, Spanned};

use crate::{
    ast_elements::type_parameter::GenericTypeParameter,
    decl_engine::{
        parsed_id::ParsedDeclId, DeclEngineGetParsedDeclId, DeclEngineInsert, DeclEngineInsertArc,
        DeclId,
    },
    language::ty::{TyAbiDecl, TyFunctionDecl},
    namespace::{IsExtendingExistingImpl, IsImplSelf},
    semantic_analysis::{
        symbol_collection_context::SymbolCollectionContext, TypeCheckAnalysis,
        TypeCheckAnalysisContext, TypeCheckFinalization, TypeCheckFinalizationContext,
    },
    Engines,
};
use sway_error::handler::{ErrorEmitted, Handler};

use crate::{
    language::{
        parsed::*,
        ty::{self, TyImplItem, TyTraitItem},
        CallPath,
    },
    semantic_analysis::declaration::SupertraitOf,
    semantic_analysis::{
        declaration::insert_supertraits_into_namespace, AbiMode, TypeCheckContext,
    },
    TypeId,
};

impl ty::TyAbiDecl {
    pub(crate) fn collect(
        handler: &Handler,
        engines: &Engines,
        ctx: &mut SymbolCollectionContext,
        decl_id: &ParsedDeclId<AbiDeclaration>,
    ) -> Result<(), ErrorEmitted> {
        let abi_decl = engines.pe().get_abi(decl_id);
        let decl = Declaration::AbiDeclaration(*decl_id);
        ctx.insert_parsed_symbol(handler, engines, abi_decl.name.clone(), decl.clone())?;

        let _ = ctx.scoped(engines, abi_decl.span.clone(), Some(decl), |scoped_ctx| {
            abi_decl.interface_surface.iter().for_each(|item| {
                let _ = TyTraitItem::collect(handler, engines, scoped_ctx, item);
            });

            abi_decl.methods.iter().for_each(|decl_id| {
                let _ = TyFunctionDecl::collect(handler, engines, scoped_ctx, decl_id);
            });
            Ok(())
        });
        Ok(())
    }

    pub(crate) fn type_check(
        handler: &Handler,
        ctx: TypeCheckContext,
        abi_decl: AbiDeclaration,
    ) -> Result<Self, ErrorEmitted> {
        let engines = ctx.engines();
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

        // The span of the `abi_decl` `name` points to the file (use site) in which
        // the ABI is getting declared, so we can use it as the `use_site_span`.
        let self_type_param = GenericTypeParameter::new_self_type(ctx.engines, name.span());
        let self_type_id = self_type_param.type_id;

        let mod_path = ctx.namespace().current_mod_path().clone();

        // A temporary namespace for checking within this scope.
        ctx.with_abi_mode(AbiMode::ImplAbiFn(name.clone(), None))
            .with_self_type(Some(self_type_id))
            .scoped(handler, Some(span.clone()), |ctx| {
                // Insert the "self" type param into the namespace.
                self_type_param.insert_self_type_into_namespace(handler, ctx.by_ref());

                // Recursively make the interface surfaces and methods of the
                // supertraits available to this abi.
                insert_supertraits_into_namespace(
                    handler,
                    ctx.by_ref(),
                    self_type_id,
                    &supertraits,
                    &SupertraitOf::Abi(span.clone()),
                )?;

                // Type check the interface surface.
                let mut new_interface_surface = vec![];

                let mut ids: HashSet<Ident> = HashSet::default();
                let error_on_shadowing_superabi_method =
                    |method_name: &Ident, ctx: &mut TypeCheckContext| {
                        if let Ok(superabi_impl_method_ref) = ctx.find_method_for_type(
                            &Handler::default(),
                            self_type_id,
                            &mod_path,
                            &method_name.clone(),
                            ctx.type_annotation(),
                            &[],
                            None,
                        ) {
                            let superabi_impl_method =
                                ctx.engines.de().get_function(&superabi_impl_method_ref);
                            if let Some(ty::TyDecl::AbiDecl(abi_decl)) =
                                &superabi_impl_method.implementing_type
                            {
                                let abi_decl = engines.de().get_abi(&abi_decl.decl_id);
                                handler.emit_err(CompileError::AbiShadowsSuperAbiMethod {
                                    span: method_name.span(),
                                    superabi: abi_decl.name().clone(),
                                });
                            }
                        }
                    };

                for item in interface_surface.into_iter() {
                    let decl_name = match item {
                        TraitItem::TraitFn(decl_id) => {
                            let method = engines.pe().get_trait_fn(&decl_id);
                            // check that a super-trait does not define a method
                            // with the same name as the current interface method
                            error_on_shadowing_superabi_method(&method.name, ctx);
                            let method = ty::TyTraitFn::type_check(handler, ctx.by_ref(), &method)?;
                            for param in &method.parameters {
                                if param.is_reference || param.is_mutable {
                                    handler.emit_err(
                                        CompileError::RefMutableNotAllowedInContractAbi {
                                            param_name: param.name.clone(),
                                            span: param.name.span(),
                                        },
                                    );
                                }
                            }
                            new_interface_surface.push(ty::TyTraitInterfaceItem::TraitFn(
                                ctx.engines.de().insert(method.clone(), Some(&decl_id)),
                            ));

                            method.name.clone()
                        }
                        TraitItem::Constant(decl_id) => {
                            let const_decl = engines.pe().get_constant(&decl_id).as_ref().clone();
                            let const_decl =
                                ty::TyConstantDecl::type_check(handler, ctx.by_ref(), const_decl)?;
                            let decl_ref =
                                ctx.engines.de().insert(const_decl.clone(), Some(&decl_id));
                            new_interface_surface
                                .push(ty::TyTraitInterfaceItem::Constant(decl_ref.clone()));
                            const_decl.call_path.suffix.clone()
                        }
                        TraitItem::Type(decl_id) => {
                            let type_decl = engines.pe().get_trait_type(&decl_id).as_ref().clone();
                            handler.emit_err(CompileError::AssociatedTypeNotSupportedInAbi {
                                span: type_decl.span.clone(),
                            });
                            let type_decl =
                                ty::TyTraitType::type_check(handler, ctx.by_ref(), type_decl)?;
                            let decl_ref =
                                ctx.engines().de().insert(type_decl.clone(), Some(&decl_id));
                            new_interface_surface
                                .push(ty::TyTraitInterfaceItem::Type(decl_ref.clone()));
                            type_decl.name
                        }
                        TraitItem::Error(_, _) => {
                            continue;
                        }
                    };

                    if !ids.insert(decl_name.clone()) {
                        handler.emit_err(CompileError::MultipleDefinitionsOfName {
                            name: decl_name.clone(),
                            span: decl_name.span(),
                        });
                    }
                }

                // Type check the items.
                let mut new_items = vec![];
                for method_id in methods.into_iter() {
                    let method = engines.pe().get_function(&method_id);
                    let method = ty::TyFunctionDecl::type_check(
                        handler,
                        ctx.by_ref(),
                        &method,
                        false,
                        false,
                        Some(self_type_param.type_id),
                    )
                    .unwrap_or_else(|_| ty::TyFunctionDecl::error(&method));
                    error_on_shadowing_superabi_method(&method.name, ctx);
                    for param in method.parameters.iter() {
                        if param.is_reference || param.is_mutable {
                            handler.emit_err(CompileError::RefMutableNotAllowedInContractAbi {
                                param_name: param.name.clone(),
                                span: param.name.span(),
                            });
                        }
                    }
                    if !ids.insert(method.name.clone()) {
                        handler.emit_err(CompileError::MultipleDefinitionsOfName {
                            name: method.name.clone(),
                            span: method.name.span(),
                        });
                    }
                    new_items.push(TyTraitItem::Fn(
                        ctx.engines.de().insert(method, Some(&method_id)),
                    ));
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
                Ok(abi_decl)
            })
    }

    pub(crate) fn insert_interface_surface_and_items_into_namespace(
        &self,
        handler: &Handler,
        self_decl_id: DeclId<ty::TyAbiDecl>,
        mut ctx: TypeCheckContext,
        type_id: TypeId,
        subabi_span: Option<Span>,
    ) -> Result<(), ErrorEmitted> {
        let decl_engine = ctx.engines.de();
        let engines = ctx.engines();

        let ty::TyAbiDecl {
            interface_surface,
            items,
            ..
        } = self;

        let mut all_items = vec![];

        let (look_for_conflicting_abi_methods, subabi_span) = if let Some(subabi) = subabi_span {
            (true, subabi)
        } else {
            (false, Span::dummy())
        };

        let mod_path = ctx.namespace().current_mod_path().clone();
        let mut const_symbols = HashMap::<Ident, ty::TyDecl>::new();

        handler.scope(|handler| {
            for item in interface_surface.iter() {
                match item {
                    ty::TyTraitInterfaceItem::TraitFn(decl_ref) => {
                        let method = decl_engine.get_trait_fn(decl_ref);
                        if look_for_conflicting_abi_methods {
                            // looking for conflicting ABI methods for triangle-like ABI hierarchies
                            if let Ok(superabi_method_ref) = ctx.find_method_for_type(
                                &Handler::default(),
                                type_id,
                                &mod_path,
                                &method.name.clone(),
                                ctx.type_annotation(),
                                &[],
                                None,
                            ) {
                                let superabi_method =
                                    ctx.engines.de().get_function(&superabi_method_ref);
                                if let Some(ty::TyDecl::AbiDecl(abi_decl)) =
                                    superabi_method.implementing_type.clone()
                                {
                                    // rule out the diamond superABI hierarchy:
                                    // it's not an error if the "conflicting" methods
                                    // actually come from the same super-ABI
                                    //            Top
                                    //      /              \
                                    //   Left            Right
                                    //      \              /
                                    //           Bottom
                                    // if we are accumulating methods from Left and Right
                                    // to place it into Bottom we will encounter
                                    // the same method from Top in both Left and Right
                                    if self_decl_id != abi_decl.decl_id {
                                        let abi_decl = engines.de().get_abi(&abi_decl.decl_id);
                                        handler.emit_err(
                                            CompileError::ConflictingSuperAbiMethods {
                                                span: subabi_span.clone(),
                                                method_name: method.name.to_string(),
                                                superabi1: abi_decl.name().to_string(),
                                                superabi2: self.name.to_string(),
                                            },
                                        );
                                    }
                                }
                            }
                        }
                        all_items.push(TyImplItem::Fn(
                            decl_engine
                                .insert(
                                    method.to_dummy_func(
                                        AbiMode::ImplAbiFn(self.name.clone(), Some(self_decl_id)),
                                        Some(type_id),
                                    ),
                                    None,
                                )
                                .with_parent(ctx.engines.de(), (*decl_ref.id()).into()),
                        ));
                    }
                    ty::TyTraitInterfaceItem::Constant(decl_ref) => {
                        let const_decl = decl_engine.get_constant(decl_ref);
                        let const_name = const_decl.call_path.suffix.clone();
                        all_items.push(TyImplItem::Constant(decl_ref.clone()));
                        const_symbols.insert(
                            const_name,
                            ty::TyDecl::ConstantDecl(ty::ConstantDecl {
                                decl_id: *decl_ref.id(),
                            }),
                        );
                    }
                    ty::TyTraitInterfaceItem::Type(decl_ref) => {
                        all_items.push(TyImplItem::Type(decl_ref.clone()));
                    }
                }
            }
            for item in items.iter() {
                match item {
                    ty::TyTraitItem::Fn(decl_ref) => {
                        let method = decl_engine.get_function(decl_ref);
                        // check if we inherit the same impl method from different branches
                        // XXX this piece of code can be abstracted out into a closure
                        // and reused for interface methods if the issue of mutable ctx is solved
                        if let Ok(superabi_impl_method_ref) = ctx.find_method_for_type(
                            &Handler::default(),
                            type_id,
                            &mod_path,
                            &method.name.clone(),
                            ctx.type_annotation(),
                            &[],
                            None,
                        ) {
                            let superabi_impl_method =
                                ctx.engines.de().get_function(&superabi_impl_method_ref);
                            if let Some(ty::TyDecl::AbiDecl(abi_decl)) =
                                superabi_impl_method.implementing_type.clone()
                            {
                                // allow the diamond superABI hierarchy
                                if self_decl_id != abi_decl.decl_id {
                                    let abi_decl = engines.de().get_abi(&abi_decl.decl_id);
                                    handler.emit_err(CompileError::ConflictingSuperAbiMethods {
                                        span: subabi_span.clone(),
                                        method_name: method.name.to_string(),
                                        superabi1: abi_decl.name().to_string(),
                                        superabi2: self.name.to_string(),
                                    });
                                }
                            }
                        }
                        all_items.push(TyImplItem::Fn(
                            decl_engine
                                .insert_arc(
                                    method,
                                    decl_engine.get_parsed_decl_id(decl_ref.id()).as_ref(),
                                )
                                .with_parent(ctx.engines.de(), (*decl_ref.id()).into()),
                        ));
                    }
                    ty::TyTraitItem::Constant(decl_ref) => {
                        let const_decl = decl_engine.get_constant(decl_ref);
                        let const_name = const_decl.name().clone();
                        let const_has_value = const_decl.value.is_some();
                        let decl_id = decl_engine.insert_arc(
                            const_decl,
                            decl_engine.get_parsed_decl_id(decl_ref.id()).as_ref(),
                        );
                        all_items.push(TyImplItem::Constant(decl_id.clone()));

                        // If this non-interface item has a value, then we want to overwrite the
                        // the previously inserted constant symbol from the interface surface.
                        if const_has_value {
                            const_symbols.insert(
                                const_name,
                                ty::TyDecl::ConstantDecl(ty::ConstantDecl {
                                    decl_id: *decl_id.id(),
                                }),
                            );
                        }
                    }
                    ty::TyTraitItem::Type(decl_ref) => {
                        let type_decl = decl_engine.get_type(decl_ref);
                        all_items.push(TyImplItem::Type(decl_engine.insert_arc(
                            type_decl,
                            decl_engine.get_parsed_decl_id(decl_ref.id()).as_ref(),
                        )));
                    }
                }
            }

            // Insert the constants into the namespace.
            for (name, decl) in const_symbols.into_iter() {
                let _ = ctx.insert_symbol(handler, name, decl);
            }

            // Insert the methods of the ABI into the namespace.
            // Specifically do not check for conflicting definitions because
            // this is just a temporary namespace for type checking and
            // these are not actual impl blocks.
            // We check that a contract method cannot call a contract method
            // from the same ABI later, during method application typechecking.
            let _ = ctx.insert_trait_implementation(
                &Handler::default(),
                CallPath::ident_to_fullpath(self.name.clone(), ctx.namespace()),
                vec![],
                type_id,
                vec![],
                &all_items,
                &self.span,
                Some(self.span()),
                IsImplSelf::No,
                IsExtendingExistingImpl::No,
            );
            Ok(())
        })
    }
}

impl TypeCheckAnalysis for TyAbiDecl {
    fn type_check_analyze(
        &self,
        handler: &Handler,
        ctx: &mut TypeCheckAnalysisContext,
    ) -> Result<(), ErrorEmitted> {
        handler.scope(|handler| {
            for item in self.items.iter() {
                let _ = item.type_check_analyze(handler, ctx);
            }
            Ok(())
        })
    }
}

impl TypeCheckFinalization for TyAbiDecl {
    fn type_check_finalize(
        &mut self,
        handler: &Handler,
        ctx: &mut TypeCheckFinalizationContext,
    ) -> Result<(), ErrorEmitted> {
        handler.scope(|handler| {
            for item in self.items.iter_mut() {
                let _ = item.type_check_finalize(handler, ctx);
            }
            Ok(())
        })
    }
}
