use crate::{
    language::{parsed::*, ty, CallPath},
    semantic_analysis::{type_check_context::EnforceTypeArguments, *},
    type_system::*,
};
use sway_error::handler::{ErrorEmitted, Handler};

impl ty::TyStructDecl {
    pub(crate) fn type_check(
        handler: &Handler,
        ctx: TypeCheckContext,
        decl: StructDeclaration,
    ) -> Result<Self, ErrorEmitted> {
        let StructDeclaration {
            name,
            fields,
            type_parameters,
            visibility,
            span,
            attributes,
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

            // type check the fields
            let mut new_fields = vec![];
            for field in fields.into_iter() {
                new_fields.push(ty::TyStructField::type_check(handler, ctx.by_ref(), field)?);
            }

            let mut path: CallPath = name.into();
            path = path.to_fullpath(ctx.engines(), ctx.namespace());

            // create the struct decl
            let decl = ty::TyStructDecl {
                call_path: path,
                type_parameters: new_type_parameters,
                fields: new_fields,
                visibility,
                span,
                attributes,
            };

            Ok(decl)
        })
    }
}

impl ty::TyStructField {
    pub(crate) fn type_check(
        handler: &Handler,
        mut ctx: TypeCheckContext,
        field: StructField,
    ) -> Result<Self, ErrorEmitted> {
        let type_engine = ctx.engines.te();

        let mut type_argument = field.type_argument;
        type_argument.type_id = ctx
            .resolve_type(
                handler,
                type_argument.type_id,
                &type_argument.span,
                EnforceTypeArguments::Yes,
                None,
            )
            .unwrap_or_else(|err| {
                type_engine.insert(ctx.engines(), TypeInfo::ErrorRecovery(err), None)
            });
        let field = ty::TyStructField {
            visibility: field.visibility,
            name: field.name,
            span: field.span,
            type_argument,
            attributes: field.attributes,
        };
        Ok(field)
    }
}

impl TypeCheckAnalysis for ty::TyStructDecl {
    fn type_check_analyze(
        &self,
        _handler: &Handler,
        _ctx: &mut TypeCheckAnalysisContext,
    ) -> Result<(), ErrorEmitted> {
        Ok(())
    }
}

impl TypeCheckFinalization for ty::TyStructDecl {
    fn type_check_finalize(
        &mut self,
        _handler: &Handler,
        _ctx: &mut TypeCheckFinalizationContext,
    ) -> Result<(), ErrorEmitted> {
        Ok(())
    }
}
