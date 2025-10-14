use sway_types::Spanned;

use crate::{
    decl_engine::{parsed_id::ParsedDeclId, DeclId},
    language::{
        parsed::{self, Declaration, TraitFn},
        ty, CallPath, Visibility,
    },
    semantic_analysis::symbol_collection_context::SymbolCollectionContext,
    Engines,
};
use sway_error::handler::{ErrorEmitted, Handler};

use crate::{
    semantic_analysis::{AbiMode, TypeCheckContext},
    type_system::*,
};

impl ty::TyTraitFn {
    pub(crate) fn collect(
        handler: &Handler,
        engines: &Engines,
        ctx: &mut SymbolCollectionContext,
        decl_id: &ParsedDeclId<TraitFn>,
    ) -> Result<(), ErrorEmitted> {
        let trait_fn = engines.pe().get_trait_fn(decl_id);
        let decl = Declaration::TraitFnDeclaration(*decl_id);
        ctx.insert_parsed_symbol(handler, engines, trait_fn.name.clone(), decl.clone())?;
        let _ = ctx.scoped(engines, trait_fn.span.clone(), Some(decl), |_scoped_ctx| {
            Ok(())
        });
        Ok(())
    }

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

        // Create a namespace for the trait function.
        ctx.by_ref().scoped(handler, Some(span.clone()), |ctx| {
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
            *new_return_type.type_id_mut() = ctx
                .resolve_type(
                    handler,
                    return_type.type_id(),
                    &return_type.span(),
                    EnforceTypeArguments::Yes,
                    None,
                )
                .unwrap_or_else(|err| type_engine.id_of_error_recovery(err));

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
        implementing_for: Option<TypeId>,
    ) -> ty::TyFunctionDecl {
        ty::TyFunctionDecl {
            purity: self.purity,
            name: self.name.clone(),
            body: <_>::default(),
            parameters: self.parameters.clone(),
            implementing_type: match &abi_mode {
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
            implementing_for,
            span: self.name.span(),
            call_path: CallPath::from(self.name.clone()),
            attributes: self.attributes.clone(),
            return_type: self.return_type.clone(),
            visibility: Visibility::Public,
            type_parameters: vec![],
            is_contract_call: matches!(abi_mode, AbiMode::ImplAbiFn(..)),
            where_clause: vec![],
            is_trait_method_dummy: true,
            is_type_check_finalized: true,
            kind: ty::TyFunctionDeclKind::Default,
        }
    }
}
