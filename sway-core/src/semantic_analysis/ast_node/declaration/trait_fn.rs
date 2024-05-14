use sway_types::Spanned;

use crate::{
    decl_engine::DeclId,
    language::{parsed, ty, CallPath, Visibility},
    semantic_analysis::type_check_context::EnforceTypeArguments,
};
use sway_error::handler::{ErrorEmitted, Handler};

use crate::{
    semantic_analysis::{AbiMode, TypeCheckContext},
    type_system::*,
};

impl ty::TyTraitFn {
    pub(crate) fn type_check(
        handler: &Handler,
        mut ctx: TypeCheckContext,
        trait_fn: &parsed::TraitFn,
    ) -> Result<ty::TyTraitFn, ErrorEmitted> {
        let parsed::TraitFn {
            name,
            span,
            purity,
            parameters,
            return_type,
            attributes,
        } = trait_fn;

        let type_engine = ctx.engines.te();
        let engines = ctx.engines();

        // Create a namespace for the trait function.
        ctx.by_ref().with_purity(*purity).scoped(|mut ctx| {
            // TODO: when we add type parameters to trait fns, type check them here

            // Type check the parameters.
            let mut typed_parameters = vec![];
            for param in parameters.iter() {
                typed_parameters.push(
                    match ty::TyFunctionParameter::type_check_interface_parameter(
                        handler,
                        ctx.by_ref(),
                        param,
                    ) {
                        Ok(res) => res,
                        Err(_) => continue,
                    },
                );
            }

            // Type check the return type.
            let mut new_return_type = return_type.clone();
            new_return_type.type_id = ctx
                .resolve_type(
                    handler,
                    return_type.type_id,
                    &return_type.span,
                    EnforceTypeArguments::Yes,
                    None,
                )
                .unwrap_or_else(|err| {
                    type_engine.insert(engines, TypeInfo::ErrorRecovery(err), None)
                });

            let trait_fn = ty::TyTraitFn {
                name: name.clone(),
                span: span.clone(),
                parameters: typed_parameters,
                return_type: new_return_type,
                purity: *purity,
                attributes: attributes.clone(),
            };

            Ok(trait_fn)
        })
    }

    /// This function is used in trait declarations to insert "placeholder"
    /// functions in the methods. This allows the methods to use functions
    /// declared in the interface surface.
    pub(crate) fn to_dummy_func(
        &self,
        abi_mode: AbiMode,
        implementing_for_typeid: Option<TypeId>,
    ) -> ty::TyFunctionDecl {
        ty::TyFunctionDecl {
            purity: self.purity,
            name: self.name.clone(),
            body: ty::TyCodeBlock::default(),
            parameters: self.parameters.clone(),
            implementing_type: match abi_mode.clone() {
                AbiMode::ImplAbiFn(_abi_name, abi_decl_id) => {
                    // ABI and their super-ABI methods cannot have the same names,
                    // so in order to provide meaningful error messages if this condition
                    // is violated, we need to keep track of ABI names before we can
                    // provide type-checked `AbiDecl`s
                    Some(ty::TyDecl::AbiDecl(ty::AbiDecl {
                        decl_id: abi_decl_id.unwrap_or(DeclId::dummy()),
                    }))
                }
                AbiMode::NonAbi => None,
            },
            implementing_for_typeid,
            span: self.name.span(),
            call_path: CallPath::from(self.name.clone()),
            attributes: self.attributes.clone(),
            return_type: self.return_type.clone(),
            visibility: Visibility::Public,
            type_parameters: vec![],
            is_contract_call: matches!(abi_mode, AbiMode::ImplAbiFn(..)),
            where_clause: vec![],
            is_trait_method_dummy: true,
            kind: ty::TyFunctionDeclKind::Default,
        }
    }
}
