use sway_error::handler::{ErrorEmitted, Handler};

use crate::{
    language::{parsed::*, ty, CallPath},
    semantic_analysis::*,
    type_system::*,
};

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
        let mut decl_namespace = ctx.namespace.clone();
        let mut ctx = ctx.scoped(&mut decl_namespace);

        // Type check the type parameters.
        let new_type_parameters =
            TypeParameter::type_check_type_params(handler, ctx.by_ref(), type_parameters, None)?;

        // Insert them into the current namespace.
        for p in &new_type_parameters {
            p.insert_into_namespace(handler, ctx.by_ref())?;
        }

        // type check the fields
        let mut new_fields = vec![];
        for field in fields.into_iter() {
            new_fields.push(ty::TyStructField::type_check(handler, ctx.by_ref(), field)?);
        }

        let mut path: CallPath = name.into();
        path = path.to_fullpath(ctx.namespace);

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
            .unwrap_or_else(|err| type_engine.insert(ctx.engines(), TypeInfo::ErrorRecovery(err)));
        let field = ty::TyStructField {
            name: field.name,
            span: field.span,
            type_argument,
            attributes: field.attributes,
        };
        Ok(field)
    }
}
