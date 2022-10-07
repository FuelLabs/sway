impl TyStructDeclaration {
    pub(crate) fn type_check(
        ctx: TypeCheckContext,
        decl: StructDeclaration,
    ) -> CompileResult<Self> {
        let mut warnings = vec![];
        let mut errors = vec![];

        let StructDeclaration {
            name,
            fields,
            type_parameters,
            visibility,
            span,
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

        // type check the fields
        let mut new_fields = vec![];
        for field in fields.into_iter() {
            new_fields.push(check!(
                TyStructField::type_check(ctx.by_ref(), field),
                return err(warnings, errors),
                warnings,
                errors
            ));
        }

        // create the struct decl
        let decl = TyStructDeclaration {
            name,
            type_parameters: new_type_parameters,
            fields: new_fields,
            visibility,
            span,
        };

        ok(decl, warnings, errors)
    }

    pub(crate) fn expect_field(&self, field_to_access: &Ident) -> CompileResult<&TyStructField> {
        let warnings = vec![];
        let mut errors = vec![];
        match self
            .fields
            .iter()
            .find(|TyStructField { name, .. }| name.as_str() == field_to_access.as_str())
        {
            Some(field) => ok(field, warnings, errors),
            None => {
                errors.push(CompileError::FieldNotFound {
                    available_fields: self
                        .fields
                        .iter()
                        .map(|TyStructField { name, .. }| name.to_string())
                        .collect::<Vec<_>>()
                        .join("\n"),
                    field_name: field_to_access.clone(),
                    struct_name: self.name.clone(),
                });
                err(warnings, errors)
            }
        }
    }
}

impl TyStructField {
    pub(crate) fn type_check(mut ctx: TypeCheckContext, field: StructField) -> CompileResult<Self> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let initial_type_id = insert_type(field.type_info);
        let r#type = check!(
            ctx.resolve_type_with_self(
                initial_type_id,
                &field.type_span,
                EnforceTypeArguments::Yes,
                None
            ),
            insert_type(TypeInfo::ErrorRecovery),
            warnings,
            errors,
        );
        let field = TyStructField {
            name: field.name,
            type_id: r#type,
            initial_type_id,
            span: field.span,
            type_span: field.type_span,
        };
        ok(field, warnings, errors)
    }
}
