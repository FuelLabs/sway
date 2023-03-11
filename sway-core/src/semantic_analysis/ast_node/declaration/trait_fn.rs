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
            purity,
            parameters,
            return_type,
            return_type_span,
            attributes,
        } = trait_fn;

        let type_engine = ctx.type_engine;
        let decl_engine = ctx.decl_engine;

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
        let return_type = check!(
            ctx.resolve_type(
                type_engine.insert(decl_engine, return_type),
                &return_type_span,
                EnforceTypeArguments::Yes,
                None
            ),
            type_engine.insert(decl_engine, TypeInfo::ErrorRecovery),
            warnings,
            errors,
        );

        let trait_fn = ty::TyTraitFn {
            name,
            parameters: typed_parameters,
            return_type,
            return_type_span,
            purity,
            attributes,
        };

        ok(trait_fn, warnings, errors)
    }

    /// This function is used in trait declarations to insert "placeholder"
    /// functions in the methods. This allows the methods to use functions
    /// declared in the interface surface.
    pub(crate) fn to_dummy_func(&self, mode: Mode) -> ty::TyFunctionDeclaration {
        ty::TyFunctionDeclaration {
            purity: self.purity,
            name: self.name.clone(),
            body: ty::TyCodeBlock { contents: vec![] },
            parameters: self.parameters.clone(),
            implementing_type: None,
            span: self.name.span(),
            attributes: self.attributes.clone(),
            return_type: TypeArgument {
                type_id: self.return_type,
                initial_type_id: self.return_type,
                span: self.return_type_span.clone(),
                call_path_tree: None,
            },
            visibility: Visibility::Public,
            type_parameters: vec![],
            is_contract_call: mode == Mode::ImplAbiFn,
            where_clause: vec![],
        }
    }
}
