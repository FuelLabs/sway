use sway_error::error::CompileError;

use crate::{
    declaration_engine::de_get_trait_fn,
    error::*,
    language::{parsed::*, ty},
    semantic_analysis::{ast_node::type_check_interface_surface, TypeCheckContext},
    CompileResult,
};

impl ty::TyAbiDeclaration {
    pub(crate) fn type_check(
        mut ctx: TypeCheckContext,
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

        // type check the interface surface and methods
        // We don't want the user to waste resources by contract calling
        // themselves, and we don't want to do more work in the compiler,
        // so we don't support the case of calling a contract's own interface
        // from itself. This is by design.
        let interface_surface = check!(
            type_check_interface_surface(ctx.by_ref(), interface_surface),
            return err(warnings, errors),
            warnings,
            errors
        );
        for typed_fn_decl_id in &interface_surface {
            match de_get_trait_fn(typed_fn_decl_id.clone(), &span) {
                Ok(typed_fn) => {
                    for param in &typed_fn.parameters {
                        if param.is_reference && param.is_mutable {
                            errors.push(CompileError::RefMutableNotAllowedInContractAbi {
                                param_name: param.name.clone(),
                            })
                        }
                    }
                }
                Err(err) => errors.push(err),
            }
        }

        // type check these for errors but don't actually use them yet -- the real
        // ones will be type checked with proper symbols when the ABI is implemented
        let _methods = methods.iter().map(|method| {
            check!(
                ty::TyFunctionDeclaration::type_check(ctx.by_ref(), method.clone(), true),
                ty::TyFunctionDeclaration::error(method.clone()),
                warnings,
                errors
            )
        });
        for typed_fn in &methods {
            for param in &typed_fn.parameters {
                if param.is_reference && param.is_mutable {
                    errors.push(CompileError::RefMutableNotAllowedInContractAbi {
                        param_name: param.name.clone(),
                    })
                }
            }
        }

        let abi_decl = ty::TyAbiDeclaration {
            interface_surface,
            methods,
            name,
            span,
            attributes,
        };
        ok(abi_decl, warnings, errors)
    }
}
