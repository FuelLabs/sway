use sway_error::error::CompileError;
use sway_types::Spanned;

use crate::{
    decl_engine::DeclEngineIndex,
    error::*,
    language::{
        parsed::*,
        ty::{self, TyTraitItem},
    },
    semantic_analysis::{declaration::insert_supertraits_into_namespace, Mode, TypeCheckContext},
    CompileResult,
};

impl ty::TyAbiDeclaration {
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
        let type_engine = ctx.type_engine;
        let decl_engine = ctx.decl_engine;
        let contract_type = type_engine.insert(decl_engine, crate::TypeInfo::Contract);
        let mut abi_namespace = ctx.namespace.clone();
        let mut ctx = ctx.scoped(&mut abi_namespace).with_mode(Mode::ImplAbiFn);

        // Recursively make the interface surfaces and methods of the
        // supertraits available to this abi.
        check!(
            insert_supertraits_into_namespace(ctx.by_ref(), contract_type, &supertraits),
            return err(warnings, errors),
            warnings,
            errors
        );

        // Type check the interface surface.
        let mut new_interface_surface = vec![];
        for item in interface_surface.into_iter() {
            match item {
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
                        ctx.decl_engine.insert(method),
                    ));
                }
            }
        }

        // Type check the methods.
        let mut new_items = vec![];
        for method in methods.into_iter() {
            let method = check!(
                ty::TyFunctionDeclaration::type_check(ctx.by_ref(), method.clone(), true, false),
                ty::TyFunctionDeclaration::error(method.clone()),
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
            new_items.push(TyTraitItem::Fn(ctx.decl_engine.insert(method)));
        }

        // Compared to regular traits, we do not insert recursively methods of ABI supertraits
        // into the interface surface, we do not want supertrait methods to be available to
        // the ABI user, only the contract methods can use supertrait methods
        let abi_decl = ty::TyAbiDeclaration {
            interface_surface: new_interface_surface,
            supertraits,
            items: new_items,
            name,
            span,
            attributes,
        };
        ok(abi_decl, warnings, errors)
    }
}
