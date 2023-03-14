use crate::{
    error::*,
    language::{parsed::*, ty, CallPath},
    semantic_analysis::*,
    type_system::*,
};

impl ty::TyStructDeclaration {
    pub(crate) fn type_check(
        ctx: TypeCheckContext,
        decl: StructDeclaration,
    ) -> CompileResult<(Self, TypeSubstList)> {
        let mut warnings = vec![];
        let mut errors = vec![];

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

        // Type check the type parameters. This will also insert them into the
        // current namespace.
        let (new_type_parameters, type_subst_list) = check!(
            TypeParameter::type_check_type_params(ctx.by_ref(), type_parameters, true),
            return err(warnings, errors),
            warnings,
            errors
        );

        // type check the fields
        let mut new_fields = vec![];
        for field in fields.into_iter() {
            new_fields.push(check!(
                ty::TyStructField::type_check(ctx.by_ref(), field),
                return err(warnings, errors),
                warnings,
                errors
            ));
        }

        let mut path: CallPath = name.into();
        path = path.to_fullpath(ctx.namespace);

        // create the struct decl
        let decl = ty::TyStructDeclaration {
            call_path: path,
            type_parameters: new_type_parameters,
            fields: new_fields,
            visibility,
            span,
            attributes,
        };

        ok((decl, type_subst_list), warnings, errors)
    }
}

impl ty::TyStructField {
    pub(crate) fn type_check(mut ctx: TypeCheckContext, field: StructField) -> CompileResult<Self> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let type_engine = ctx.type_engine;
        let decl_engine = ctx.decl_engine;

        let mut type_argument = field.type_argument;
        type_argument.type_id = check!(
            ctx.resolve_type(
                type_argument.type_id,
                &type_argument.span,
                EnforceTypeArguments::Yes,
                None
            ),
            type_engine.insert(decl_engine, TypeInfo::ErrorRecovery),
            warnings,
            errors,
        );
        let field = ty::TyStructField {
            name: field.name,
            span: field.span,
            type_argument,
            attributes: field.attributes,
        };
        ok(field, warnings, errors)
    }
}
