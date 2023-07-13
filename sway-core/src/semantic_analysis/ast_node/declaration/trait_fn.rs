use sway_types::Spanned;

use crate::{
    error::*,
    language::{parsed, ty, Visibility},
    semantic_analysis::{Mode, TypeCheckContext},
    type_system::*,
};

impl ty::TyTraitFn {
    pub(crate) fn type_check(
        mut ctx: TypeCheckContext,
        trait_fn: parsed::TraitFn,
    ) -> CompileResult<ty::TyTraitFn> {
        let mut warnings = vec![];
        let mut errors = vec![];

        let parsed::TraitFn {
            name,
            span,
            purity,
            parameters,
            mut return_type,
            attributes,
        } = trait_fn;

        let type_engine = ctx.engines.te();
        let engines = ctx.engines();

        // Create a namespace for the trait function.
        let mut fn_namespace = ctx.namespace.clone();
        let mut ctx = ctx.by_ref().scoped(&mut fn_namespace).with_purity(purity);

        // TODO: when we add type parameters to trait fns, type check them here

        // Type check the parameters.
        let mut typed_parameters = vec![];
        for param in parameters.into_iter() {
            typed_parameters.push(check!(
                ty::TyFunctionParameter::type_check_interface_parameter(ctx.by_ref(), param),
                continue,
                warnings,
                errors
            ));
        }

        // Type check the return type.
        return_type.type_id = check!(
            ctx.resolve_type_with_self(
                return_type.type_id,
                &return_type.span,
                EnforceTypeArguments::Yes,
                None
            ),
            type_engine.insert(engines, TypeInfo::ErrorRecovery),
            warnings,
            errors,
        );

        let trait_fn = ty::TyTraitFn {
            name,
            span,
            parameters: typed_parameters,
            return_type,
            purity,
            attributes,
        };

        ok(trait_fn, warnings, errors)
    }

    /// This function is used in trait declarations to insert "placeholder"
    /// functions in the methods. This allows the methods to use functions
    /// declared in the interface surface.
    pub(crate) fn to_dummy_func(&self, mode: Mode) -> ty::TyFunctionDecl {
        ty::TyFunctionDecl {
            purity: self.purity,
            name: self.name.clone(),
            body: ty::TyCodeBlock { contents: vec![] },
            parameters: self.parameters.clone(),
            implementing_type: None,
            span: self.name.span(),
            attributes: self.attributes.clone(),
            return_type: self.return_type.clone(),
            visibility: Visibility::Public,
            type_parameters: vec![],
            is_contract_call: mode == Mode::ImplAbiFn,
            where_clause: vec![],
        }
    }
}
