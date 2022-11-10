use sway_error::error::CompileError;

use crate::{
    declaration_engine::{de_insert_function, de_insert_trait_fn},
    error::*,
    language::{parsed::*, ty},
    semantic_analysis::{Mode, TypeCheckContext},
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
            methods,
            span,
            attributes,
        } = abi_decl;

        // We don't want the user to waste resources by contract calling
        // themselves, and we don't want to do more work in the compiler,
        // so we don't support the case of calling a contract's own interface
        // from itself. This is by design.

        // A temporary namespace for checking within this scope.
        let mut abi_namespace = ctx.namespace.clone();
        let mut ctx = ctx.scoped(&mut abi_namespace).with_mode(Mode::ImplAbiFn);

        // Type check the interface surface.
        let mut new_interface_surface = vec![];
        for method in interface_surface.into_iter() {
            let method = check!(
                ty::TyTraitFn::type_check(ctx.by_ref(), method),
                return err(warnings, errors),
                warnings,
                errors
            );
            for param in &method.parameters {
                if param.is_reference && param.is_mutable {
                    errors.push(CompileError::RefMutableNotAllowedInContractAbi {
                        param_name: param.name.clone(),
                    })
                }
            }
            new_interface_surface.push(de_insert_trait_fn(method));
        }

        // Type check the methods.
        let mut new_methods = vec![];
        for method in methods.into_iter() {
            let method = check!(
                ty::TyFunctionDeclaration::type_check(ctx.by_ref(), method.clone(), true),
                ty::TyFunctionDeclaration::error(method.clone()),
                warnings,
                errors
            );
            for param in &method.parameters {
                if param.is_reference && param.is_mutable {
                    errors.push(CompileError::RefMutableNotAllowedInContractAbi {
                        param_name: param.name.clone(),
                    })
                }
            }
            new_methods.push(de_insert_function(method));
        }

        let abi_decl = ty::TyAbiDeclaration {
            interface_surface: new_interface_surface,
            methods: new_methods,
            name,
            span,
            attributes,
        };
        ok(abi_decl, warnings, errors)
    }
}
