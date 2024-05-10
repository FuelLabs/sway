use itertools::Itertools;
use sway_error::{
    error::{CompileError, StructFieldUsageContext},
    handler::{ErrorEmitted, Handler},
};
use sway_types::{BaseIdent, Ident, Span, Spanned};

use crate::{
    decl_engine::DeclEngineInsert,
    language::{
        parsed::*,
        ty::{self, StructAccessInfo, TyDecl, TyScrutinee, TyStructDecl, TyStructField},
        CallPath,
    },
    semantic_analysis::{
        type_check_context::EnforceTypeArguments, TypeCheckContext, TypeCheckFinalization,
        TypeCheckFinalizationContext,
    },
    type_system::*,
};

impl TyScrutinee {
    pub(crate) fn type_check(
        handler: &Handler,
        mut ctx: TypeCheckContext,
        scrutinee: Scrutinee,
    ) -> Result<Self, ErrorEmitted> {
        let type_engine = ctx.engines.te();
        let engines = ctx.engines();
        match scrutinee {
            Scrutinee::Or { elems, span } => {
                let type_id = type_engine.insert(engines, TypeInfo::Unknown, None);

                let mut typed_elems = Vec::with_capacity(elems.len());
                for scrutinee in elems {
                    typed_elems.push(ty::TyScrutinee::type_check(
                        handler,
                        ctx.by_ref(),
                        scrutinee,
                    )?);
                }
                let typed_scrutinee = ty::TyScrutinee {
                    variant: ty::TyScrutineeVariant::Or(typed_elems),
                    type_id,
                    span,
                };
                Ok(typed_scrutinee)
            }
            Scrutinee::CatchAll { span } => {
                let type_id = type_engine.insert(engines, TypeInfo::Unknown, None);
                let dummy_type_param = TypeParameter {
                    type_id,
                    initial_type_id: type_id,
                    name_ident: BaseIdent::new_with_override("_".into(), span.clone()),
                    trait_constraints: vec![],
                    trait_constraints_span: Span::dummy(),
                    is_from_parent: false,
                };
                let typed_scrutinee = ty::TyScrutinee {
                    variant: ty::TyScrutineeVariant::CatchAll,
                    type_id: type_engine.insert(
                        engines,
                        TypeInfo::Placeholder(dummy_type_param),
                        span.source_id(),
                    ),
                    span,
                };
                Ok(typed_scrutinee)
            }
            Scrutinee::Literal { value, span } => {
                let typed_scrutinee = ty::TyScrutinee {
                    variant: ty::TyScrutineeVariant::Literal(value.clone()),
                    type_id: type_engine.insert(engines, value.to_typeinfo(), span.source_id()),
                    span,
                };
                Ok(typed_scrutinee)
            }
            Scrutinee::Variable { name, span } => type_check_variable(handler, ctx, name, span),
            Scrutinee::StructScrutinee {
                struct_name,
                fields,
                span,
            } => type_check_struct(handler, ctx, struct_name.suffix, &fields, span),
            Scrutinee::EnumScrutinee {
                call_path,
                value,
                span,
            } => type_check_enum(handler, ctx, call_path, *value, span),
            Scrutinee::AmbiguousSingleIdent(ident) => {
                let maybe_enum = type_check_enum(
                    &Handler::default(),
                    ctx.by_ref(),
                    CallPath {
                        prefixes: vec![],
                        suffix: ident.clone(),
                        is_absolute: false,
                    },
                    Scrutinee::Tuple {
                        elems: vec![],
                        span: ident.span(),
                    },
                    ident.span(),
                );

                if maybe_enum.is_ok() {
                    maybe_enum
                } else {
                    type_check_variable(handler, ctx, ident.clone(), ident.span())
                }
            }
            Scrutinee::Tuple { elems, span } => type_check_tuple(handler, ctx, elems, span),
            Scrutinee::Error { err, .. } => Err(err),
        }
    }

    /// Returns true if the [ty::TyScrutinee] consists only of catch-all scrutinee variants, recursively.
    /// Catch-all variants are .., _, and variables. E.g.:
    ///
    /// ```ignore
    /// (_, x, Point { .. })
    /// ```
    ///
    /// An [ty::TyScrutineeVariant::Or] is considered to be catch-all if any of its alternatives
    /// is a catch-all [ty::TyScrutinee] according to the above definition. E.g.:
    ///
    /// ```ignore
    /// (1, x, Point { x: 3, y: 4 }) | (_, x, Point { .. })
    /// ```
    ///
    /// A catch-all [ty::TyScrutinee] matches all the values of its corresponding type.
    ///
    /// A scrutinee that matches all the values of its corresponding type but does not
    /// consists only of catch-all variants will not be considered a catch-all scrutinee.
    /// E.g., although it matches all values of `bool`, this scrutinee is not considered to
    /// be a catch-all scrutinee:
    ///
    /// ```ignore
    /// true | false
    /// ```
    pub(crate) fn is_catch_all(&self) -> bool {
        match &self.variant {
            ty::TyScrutineeVariant::CatchAll => true,
            ty::TyScrutineeVariant::Variable(_) => true,
            ty::TyScrutineeVariant::Literal(_) => false,
            ty::TyScrutineeVariant::Constant { .. } => false,
            ty::TyScrutineeVariant::StructScrutinee { fields, .. } => fields
                .iter()
                .filter_map(|x| x.scrutinee.as_ref())
                .all(|x| x.is_catch_all()),
            ty::TyScrutineeVariant::Or(elems) => elems.iter().any(|x| x.is_catch_all()),
            ty::TyScrutineeVariant::Tuple(elems) => elems.iter().all(|x| x.is_catch_all()),
            ty::TyScrutineeVariant::EnumScrutinee { .. } => false,
        }
    }
}

fn type_check_variable(
    handler: &Handler,
    ctx: TypeCheckContext,
    name: Ident,
    span: Span,
) -> Result<ty::TyScrutinee, ErrorEmitted> {
    let engines = ctx.engines;
    let type_engine = engines.te();
    let decl_engine = engines.de();

    let typed_scrutinee = match ctx
        .namespace()
        .resolve_symbol_typed(&Handler::default(), engines, &name, ctx.self_type())
        .ok()
    {
        // If this variable is a constant, then we turn it into a [TyScrutinee::Constant](ty::TyScrutinee::Constant).
        Some(ty::TyDecl::ConstantDecl(ty::ConstantDecl { decl_id, .. })) => {
            let constant_decl = (*decl_engine.get_constant(&decl_id)).clone();
            let value = match constant_decl.value {
                Some(ref value) => value,
                None => {
                    return Err(handler.emit_err(CompileError::Internal(
                        "constant value does not contain expression",
                        span,
                    )));
                }
            };
            let literal = match value.extract_literal_value() {
                Some(value) => value,
                None => {
                    return Err(handler.emit_err(CompileError::Unimplemented {
                        feature: "Supporting constant values of this type in patterns".to_string(),
                        help: vec![],
                        span,
                    }));
                }
            };
            ty::TyScrutinee {
                type_id: value.return_type,
                variant: ty::TyScrutineeVariant::Constant(name, literal, constant_decl),
                span,
            }
        }
        // Variable isn't a constant, so we turn it into a [ty::TyScrutinee::Variable].
        _ => ty::TyScrutinee {
            variant: ty::TyScrutineeVariant::Variable(name),
            type_id: type_engine.insert(ctx.engines(), TypeInfo::Unknown, None),
            span,
        },
    };

    Ok(typed_scrutinee)
}

fn type_check_struct(
    handler: &Handler,
    mut ctx: TypeCheckContext,
    struct_name: Ident,
    fields: &[StructScrutineeField],
    span: Span,
) -> Result<ty::TyScrutinee, ErrorEmitted> {
    let engines = ctx.engines;
    let type_engine = engines.te();
    let decl_engine = engines.de();

    // find the struct definition from the name
    let unknown_decl =
        ctx.namespace()
            .resolve_symbol_typed(handler, engines, &struct_name, ctx.self_type())?;
    let struct_ref = unknown_decl.to_struct_ref(handler, ctx.engines())?;
    let mut struct_decl = (*decl_engine.get_struct(&struct_ref)).clone();

    // monomorphize the struct definition
    ctx.monomorphize(
        handler,
        &mut struct_decl,
        &mut [],
        EnforceTypeArguments::No,
        &struct_name.span(),
    )?;

    let (struct_can_be_changed, is_public_struct_access) =
        StructAccessInfo::get_info(ctx.engines(), &struct_decl, ctx.namespace()).into();

    let has_rest_pattern = fields
        .iter()
        .any(|field| matches!(field, StructScrutineeField::Rest { .. }));

    // check for field existence and type check nested scrutinees; short-circuit if there are non-existing fields
    // TODO: Is short-circuiting really needed or was it more a convenience? In the first implementation
    //       we had a short-circuit on the first error non-existing field and didn't even collecting all errors.
    let mut typed_fields = vec![];
    handler.scope(|handler| {
        for field in fields.iter() {
            match field {
                StructScrutineeField::Field {
                    field,
                    scrutinee,
                    span,
                } => {
                    // ensure that the struct definition has this field
                    let struct_field = match expect_struct_field(
                        &struct_decl,
                        handler,
                        field,
                        has_rest_pattern,
                        is_public_struct_access,
                    ) {
                        Ok(struct_field) => struct_field,
                        Err(_) => continue,
                    };

                    // type check the nested scrutinee
                    let typed_scrutinee = match scrutinee {
                        None => None,
                        Some(scrutinee) => Some(ty::TyScrutinee::type_check(
                            handler,
                            ctx.by_ref(),
                            scrutinee.clone(),
                        )?),
                    };

                    typed_fields.push(ty::TyStructScrutineeField {
                        field: field.clone(),
                        scrutinee: typed_scrutinee,
                        span: span.clone(),
                        field_def_name: struct_field.name.clone(),
                    });
                }
                StructScrutineeField::Rest { .. } => {}
            }
        }

        Ok(())
    })?;

    handler.scope(|handler| {
        // report struct field privacy errors
        // This check is intentionally separated from checking the field existence and type-checking the scrutinees.
        // While we could check private field access immediately after finding the field and emit errors,
        // that would mean short-circuiting in case of privacy issues which we do not want to do.
        // The consequence is repeating the search for fields here, but the performance penalty is negligible.
        if is_public_struct_access {
            for field in fields {
                match field {
                    StructScrutineeField::Field {
                        field: ref field_name,
                        ..
                    } => {
                        let struct_field = struct_decl
                            .find_field(field_name)
                            .expect("The struct field with the given field name must exist.");

                        if struct_field.is_private() {
                            handler.emit_err(CompileError::StructFieldIsPrivate {
                                field_name: field_name.into(),
                                struct_name: struct_decl.call_path.suffix.clone(),
                                field_decl_span: struct_field.name.span(),
                                struct_can_be_changed,
                                usage_context: StructFieldUsageContext::PatternMatching {
                                    has_rest_pattern,
                                },
                            });
                        }
                    }
                    StructScrutineeField::Rest { .. } => {}
                }
            }
        }

        // ensure that the pattern uses all fields of the struct unless the rest pattern is present
        // Here we follow the approach Rust has, and show a dedicated error if only all public fields are
        // listed, but the mandatory `..` (because of the private fields) is missing because the struct
        // has private fields and is used outside of its decl module.
        // Also, in case of privacy issues and mixing public and private fields we list only the public
        // fields as missing.
        // The error message in both cases gives adequate explanation how to fix the reported issue.

        if !has_rest_pattern && (struct_decl.fields.len() != typed_fields.len()) {
            let all_public_fields_are_matched = struct_decl
                .fields
                .iter()
                .filter(|f| f.is_public())
                .all(|f| typed_fields.iter().any(|tf| f.name == tf.field));

            let only_public_fields_are_matched = typed_fields
                .iter()
                .map(|tf| {
                    struct_decl
                        .find_field(&tf.field)
                        .expect("The struct field with the given field name must exist.")
                })
                .all(|f| f.is_public());

            // In the case of public access where all public fields are listed along with some private fields,
            // we already have an error emitted for those private fields with the detailed, pattern matching related
            // explanation that proposes using ignore `..`.
            if !(is_public_struct_access
                && all_public_fields_are_matched
                && !only_public_fields_are_matched)
            {
                let missing_fields = |only_public: bool| {
                    struct_decl
                        .fields
                        .iter()
                        .filter(|f| !only_public || f.is_public())
                        .filter(|f| !typed_fields.iter().any(|tf| f.name == tf.field))
                        .map(|field| field.name.clone())
                        .collect_vec()
                };

                handler.emit_err(
                    match (
                        is_public_struct_access,
                        all_public_fields_are_matched,
                        only_public_fields_are_matched,
                    ) {
                        // Public access. Only all public fields are matched. All missing fields are private.
                        // -> Emit error for the mandatory ignore `..`.
                        (true, true, true) => {
                            CompileError::MatchStructPatternMustIgnorePrivateFields {
                                private_fields: missing_fields(false),
                                struct_name: struct_decl.call_path.suffix.clone(),
                                struct_decl_span: struct_decl.span(),
                                all_fields_are_private: struct_decl.has_only_private_fields(),
                                span: span.clone(),
                            }
                        }

                        // Public access. All public fields are matched. Some private fields are matched.
                        // -> Do not emit error here because it is already covered when reporting private field.
                        (true, true, false) => {
                            unreachable!("The above if condition eliminates this case.")
                        }

                        // Public access. Some or non of the public fields are matched. Some or none of the private fields are matched.
                        // -> Emit error listing only missing public fields. Recommendation for mandatory use of `..` is already given
                        //    when reporting the inaccessible private field.
                        //  or
                        // In struct decl module access. We do not distinguish between private and public fields here.
                        // -> Emit error listing all missing fields.
                        (true, false, _) | (false, _, _) => {
                            CompileError::MatchStructPatternMissingFields {
                                missing_fields: missing_fields(is_public_struct_access),
                                missing_fields_are_public: is_public_struct_access,
                                struct_name: struct_decl.call_path.suffix.clone(),
                                struct_decl_span: struct_decl.span(),
                                total_number_of_fields: struct_decl.fields.len(),
                                span: span.clone(),
                            }
                        }
                    },
                );
            }
        }

        Ok(())
    })?;

    let struct_ref = decl_engine.insert(struct_decl);
    let typed_scrutinee = ty::TyScrutinee {
        type_id: type_engine.insert(
            ctx.engines(),
            TypeInfo::Struct(struct_ref.clone()),
            struct_ref.span().source_id(),
        ),
        span,
        variant: ty::TyScrutineeVariant::StructScrutinee {
            struct_ref,
            fields: typed_fields,
            instantiation_call_path: CallPath {
                prefixes: vec![],
                suffix: struct_name,
                is_absolute: false,
            },
        },
    };

    return Ok(typed_scrutinee);

    fn expect_struct_field<'a>(
        struct_decl: &'a TyStructDecl,
        handler: &Handler,
        field_name: &Ident,
        has_rest_pattern: bool,
        is_public_struct_access: bool,
    ) -> Result<&'a TyStructField, ErrorEmitted> {
        match struct_decl.find_field(field_name) {
            Some(field) => Ok(field),
            None => Err(handler.emit_err(CompileError::StructFieldDoesNotExist {
                field_name: field_name.into(),
                available_fields: struct_decl.accessible_fields_names(is_public_struct_access),
                is_public_struct_access,
                struct_name: struct_decl.call_path.suffix.clone(),
                struct_decl_span: struct_decl.span(),
                struct_is_empty: struct_decl.is_empty(),
                usage_context: StructFieldUsageContext::PatternMatching { has_rest_pattern },
            })),
        }
    }
}

impl TypeCheckFinalization for TyScrutinee {
    fn type_check_finalize(
        &mut self,
        _handler: &Handler,
        _ctx: &mut TypeCheckFinalizationContext,
    ) -> Result<(), ErrorEmitted> {
        Ok(())
    }
}

fn type_check_enum(
    handler: &Handler,
    mut ctx: TypeCheckContext,
    call_path: CallPath<Ident>,
    value: Scrutinee,
    span: Span,
) -> Result<ty::TyScrutinee, ErrorEmitted> {
    let type_engine = ctx.engines.te();
    let decl_engine = ctx.engines.de();
    let engines = ctx.engines();

    let mut prefixes = call_path.prefixes.clone();
    let (callsite_span, mut enum_decl, call_path_decl) = match prefixes.pop() {
        Some(enum_name) => {
            let enum_callpath = CallPath {
                suffix: enum_name,
                prefixes,
                is_absolute: call_path.is_absolute,
            };
            // find the enum definition from the name
            let unknown_decl = ctx.namespace().resolve_call_path_typed(
                handler,
                engines,
                &enum_callpath,
                ctx.self_type(),
            )?;
            let enum_ref = unknown_decl.to_enum_ref(handler, ctx.engines())?;
            (
                enum_callpath.span(),
                (*decl_engine.get_enum(&enum_ref)).clone(),
                unknown_decl,
            )
        }
        None => {
            // we may have an imported variant
            let decl = ctx.namespace().resolve_call_path_typed(
                handler,
                engines,
                &call_path,
                ctx.self_type(),
            )?;
            if let TyDecl::EnumVariantDecl(ty::EnumVariantDecl { enum_ref, .. }) = decl.clone() {
                (
                    call_path.suffix.span(),
                    (*decl_engine.get_enum(enum_ref.id())).clone(),
                    decl,
                )
            } else {
                return Err(handler.emit_err(CompileError::EnumNotFound {
                    name: call_path.suffix.clone(),
                    span: call_path.suffix.span(),
                }));
            }
        }
    };
    let variant_name = call_path.suffix.clone();

    // monomorphize the enum definition
    ctx.monomorphize(
        handler,
        &mut enum_decl,
        &mut [],
        EnforceTypeArguments::No,
        &callsite_span,
    )?;

    // check to see if the variant exists and grab it if it does
    let variant = enum_decl
        .expect_variant_from_name(handler, &variant_name)
        .cloned()?;

    // type check the nested scrutinee
    let typed_value = ty::TyScrutinee::type_check(handler, ctx, value)?;

    let enum_ref = decl_engine.insert(enum_decl);
    let typed_scrutinee = ty::TyScrutinee {
        variant: ty::TyScrutineeVariant::EnumScrutinee {
            enum_ref: enum_ref.clone(),
            variant: Box::new(variant),
            call_path_decl,
            value: Box::new(typed_value),
            instantiation_call_path: call_path,
        },
        type_id: type_engine.insert(
            engines,
            TypeInfo::Enum(enum_ref.clone()),
            enum_ref.span().source_id(),
        ),
        span,
    };

    Ok(typed_scrutinee)
}

fn type_check_tuple(
    handler: &Handler,
    mut ctx: TypeCheckContext,
    elems: Vec<Scrutinee>,
    span: Span,
) -> Result<ty::TyScrutinee, ErrorEmitted> {
    let type_engine = ctx.engines.te();
    let engines = ctx.engines();

    let mut typed_elems = vec![];
    for elem in elems.into_iter() {
        typed_elems.push(
            match ty::TyScrutinee::type_check(handler, ctx.by_ref(), elem) {
                Ok(res) => res,
                Err(_) => continue,
            },
        );
    }
    let type_id = type_engine.insert(
        engines,
        TypeInfo::Tuple(
            typed_elems
                .iter()
                .map(|x| TypeArgument {
                    type_id: x.type_id,
                    initial_type_id: x.type_id,
                    span: span.clone(),
                    call_path_tree: None,
                })
                .collect(),
        ),
        span.source_id(),
    );
    let typed_scrutinee = ty::TyScrutinee {
        variant: ty::TyScrutineeVariant::Tuple(typed_elems),
        type_id,
        span,
    };

    Ok(typed_scrutinee)
}
