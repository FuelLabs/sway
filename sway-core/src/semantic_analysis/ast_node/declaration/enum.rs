use crate::{
    language::{parsed::*, ty, CallPath},
    semantic_analysis::{type_check_context::EnforceTypeArguments, *},
    type_system::*,
};
use sway_error::handler::{ErrorEmitted, Handler};

impl ty::TyEnumDecl {
    pub fn type_check(
        handler: &Handler,
        ctx: TypeCheckContext,
        decl: EnumDeclaration,
    ) -> Result<Self, ErrorEmitted> {
        let EnumDeclaration {
            name,
            type_parameters,
            variants,
            span,
            attributes,
            visibility,
            ..
        } = decl;

        // create a namespace for the decl, used to create a scope for generics
        ctx.scoped(|mut ctx| {
            // Type check the type parameters.
            let new_type_parameters = TypeParameter::type_check_type_params(
                handler,
                ctx.by_ref(),
                type_parameters,
                None,
            )?;

            // type check the variants
            let mut variants_buf = vec![];
            for variant in variants {
                variants_buf.push(
                    match ty::TyEnumVariant::type_check(handler, ctx.by_ref(), variant.clone()) {
                        Ok(res) => res,
                        Err(_) => continue,
                    },
                );
            }

            let mut call_path: CallPath = name.into();
            call_path = call_path.to_fullpath(ctx.engines(), ctx.namespace());

            // create the enum decl
            let decl = ty::TyEnumDecl {
                call_path,
                type_parameters: new_type_parameters,
                variants: variants_buf,
                span,
                attributes,
                visibility,
            };
            Ok(decl)
        })
    }
}

impl ty::TyEnumVariant {
    pub(crate) fn type_check(
        handler: &Handler,
        mut ctx: TypeCheckContext,
        variant: EnumVariant,
    ) -> Result<Self, ErrorEmitted> {
        let type_engine = ctx.engines.te();
        let engines = ctx.engines();
        let mut type_argument = variant.type_argument;
        type_argument.type_id = ctx
            .resolve_type(
                handler,
                type_argument.type_id,
                &type_argument.span,
                EnforceTypeArguments::Yes,
                None,
            )
            .unwrap_or_else(|err| type_engine.insert(engines, TypeInfo::ErrorRecovery(err), None));
        Ok(ty::TyEnumVariant {
            name: variant.name.clone(),
            type_argument,
            tag: variant.tag,
            span: variant.span,
            attributes: variant.attributes,
        })
    }
}

impl TypeCheckAnalysis for ty::TyEnumDecl {
    fn type_check_analyze(
        &self,
        _handler: &Handler,
        _ctx: &mut TypeCheckAnalysisContext,
    ) -> Result<(), ErrorEmitted> {
        Ok(())
    }
}

impl TypeCheckFinalization for ty::TyEnumDecl {
    fn type_check_finalize(
        &mut self,
        _handler: &Handler,
        _ctx: &mut TypeCheckFinalizationContext,
    ) -> Result<(), ErrorEmitted> {
        Ok(())
    }
}
