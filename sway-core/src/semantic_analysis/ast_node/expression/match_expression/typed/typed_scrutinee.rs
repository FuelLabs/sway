use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
};
use sway_types::{BaseIdent, Ident, Span, Spanned};

use crate::{
    decl_engine::DeclEngineInsert,
    language::{
        parsed::*,
        ty::{self, TyDecl, TyScrutinee},
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
            } => type_check_struct(handler, ctx, struct_name.suffix, fields, span),
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
        .namespace
        .resolve_symbol(&Handler::default(), engines, &name, ctx.self_type())
        .ok()
    {
        // If this variable is a constant, then we turn it into a [TyScrutinee::Constant](ty::TyScrutinee::Constant).
        Some(ty::TyDecl::ConstantDecl(ty::ConstantDecl { decl_id, .. })) => {
            let constant_decl = decl_engine.get_constant(&decl_id);
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
                    return Err(handler.emit_err(CompileError::Unimplemented(
                        "constant values of this type are not supported yet",
                        span,
                    )));
                }
            };
            ty::TyScrutinee {
                type_id: value.return_type,
                variant: ty::TyScrutineeVariant::Constant(name, literal, constant_decl),
                span,
            }
        }
        // Variable isn't a constant, so so we turn it into a [ty::TyScrutinee::Variable].
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
    fields: Vec<StructScrutineeField>,
    span: Span,
) -> Result<ty::TyScrutinee, ErrorEmitted> {
    let engines = ctx.engines;
    let type_engine = engines.te();
    let decl_engine = engines.de();

    // find the struct definition from the name
    let unknown_decl =
        ctx.namespace
            .resolve_symbol(handler, engines, &struct_name, ctx.self_type())?;
    let struct_ref = unknown_decl.to_struct_ref(handler, ctx.engines())?;
    let mut struct_decl = decl_engine.get_struct(&struct_ref);

    // monomorphize the struct definition
    ctx.monomorphize(
        handler,
        &mut struct_decl,
        &mut [],
        EnforceTypeArguments::No,
        &struct_name.span(),
    )?;

    // type check the fields
    let mut typed_fields = vec![];
    let mut rest_pattern = None;
    for field in fields.into_iter() {
        match field {
            StructScrutineeField::Rest { .. } => rest_pattern = Some(field),
            StructScrutineeField::Field {
                field,
                scrutinee,
                span,
            } => {
                // ensure that the struct definition has this field
                let struct_field = struct_decl.expect_field(handler, &field)?;
                // type check the nested scrutinee
                let typed_scrutinee = match scrutinee {
                    None => None,
                    Some(scrutinee) => Some(ty::TyScrutinee::type_check(
                        handler,
                        ctx.by_ref(),
                        scrutinee,
                    )?),
                };
                typed_fields.push(ty::TyStructScrutineeField {
                    field,
                    scrutinee: typed_scrutinee,
                    span,
                    field_def_name: struct_field.name.clone(),
                });
            }
        }
    }

    // ensure that the pattern uses all fields of the struct unless the rest pattern is present
    if (struct_decl.fields.len() != typed_fields.len()) && rest_pattern.is_none() {
        let missing_fields = struct_decl
            .fields
            .iter()
            .filter(|f| !typed_fields.iter().any(|tf| f.name == tf.field))
            .map(|f| f.name.to_string())
            .collect::<Vec<_>>();

        return Err(
            handler.emit_err(CompileError::MatchStructPatternMissingFields {
                span,
                missing_fields,
            }),
        );
    }

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

    Ok(typed_scrutinee)
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
            let unknown_decl = ctx.namespace.resolve_call_path(
                handler,
                engines,
                &enum_callpath,
                ctx.self_type(),
            )?;
            let enum_ref = unknown_decl.to_enum_ref(handler, ctx.engines())?;
            (
                enum_callpath.span(),
                decl_engine.get_enum(&enum_ref),
                unknown_decl,
            )
        }
        None => {
            // we may have an imported variant
            let decl =
                ctx.namespace
                    .resolve_call_path(handler, engines, &call_path, ctx.self_type())?;
            if let TyDecl::EnumVariantDecl(ty::EnumVariantDecl { enum_ref, .. }) = decl.clone() {
                (
                    call_path.suffix.span(),
                    decl_engine.get_enum(enum_ref.id()),
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
