use crate::{
    error::*,
    language::{parsed::*, ty},
    semantic_analysis::*,
    type_system::*,
};

impl ty::TyEnumDeclaration {
    pub fn type_check(ctx: TypeCheckContext, decl: EnumDeclaration) -> CompileResult<Self> {
        let mut errors = vec![];
        let mut warnings = vec![];

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
        let mut decl_namespace = ctx.namespace.clone();
        let mut ctx = ctx.scoped(&mut decl_namespace);

        // type check the type parameters
        // insert them into the namespace
        let mut new_type_parameters = vec![];
        for type_parameter in type_parameters.into_iter() {
            new_type_parameters.push(check!(
                TypeParameter::type_check(ctx.by_ref(), type_parameter),
                return err(warnings, errors),
                warnings,
                errors
            ));
        }

        // type check the variants
        let mut variants_buf = vec![];
        for variant in variants {
            variants_buf.push(check!(
                ty::TyEnumVariant::type_check(ctx.by_ref(), variant.clone()),
                continue,
                warnings,
                errors
            ));
        }

        // create the enum decl
        let decl = ty::TyEnumDeclaration {
            name,
            type_parameters: new_type_parameters,
            variants: variants_buf,
            span,
            attributes,
            visibility,
        };
        ok(decl, warnings, errors)
    }
}

impl ty::TyEnumVariant {
    pub(crate) fn type_check(
        mut ctx: TypeCheckContext,
        variant: EnumVariant,
    ) -> CompileResult<Self> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let initial_type_id = insert_type(variant.type_info);
        let enum_variant_type = check!(
            ctx.resolve_type_with_self(
                initial_type_id,
                &variant.span,
                EnforceTypeArguments::Yes,
                None
            ),
            insert_type(TypeInfo::ErrorRecovery),
            warnings,
            errors,
        );
        ok(
            ty::TyEnumVariant {
                name: variant.name.clone(),
                type_id: enum_variant_type,
                initial_type_id,
                type_span: variant.type_span.clone(),
                tag: variant.tag,
                span: variant.span,
                attributes: variant.attributes,
            },
            vec![],
            errors,
        )
    }
}
