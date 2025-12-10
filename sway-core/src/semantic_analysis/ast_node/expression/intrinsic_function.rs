use ast_elements::type_argument::GenericTypeArgument;
use sway_ast::intrinsics::Intrinsic;
use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
};
use sway_types::Span;
use sway_types::{integer_bits::IntegerBits, Spanned};

use crate::{
    engine_threading::*,
    language::{
        parsed::{Expression, ExpressionKind},
        ty::{self, TyIntrinsicFunctionKind},
        Literal,
    },
    semantic_analysis::TypeCheckContext,
    type_system::*,
    BuildTarget,
};

impl ty::TyIntrinsicFunctionKind {
    pub(crate) fn type_check(
        handler: &Handler,
        ctx: TypeCheckContext,
        kind_binding: TypeBinding<Intrinsic>,
        arguments: &[Expression],
        span: Span,
    ) -> Result<(Self, TypeId), ErrorEmitted> {
        let TypeBinding {
            inner: kind,
            type_arguments,
            ..
        } = kind_binding;
        let type_arguments = type_arguments.as_slice();
        //ensure_intrinsic_supported(handler, &ctx, kind, span.clone())?;
        match kind {
            Intrinsic::SizeOfVal => {
                type_check_size_of_val(handler, ctx, kind, arguments, type_arguments, span)
            }
            Intrinsic::SizeOfType => {
                type_check_size_of_type(handler, ctx, kind, arguments, type_arguments, span)
            }
            Intrinsic::SizeOfStr => {
                type_check_size_of_type(handler, ctx, kind, arguments, type_arguments, span)
            }
            Intrinsic::IsReferenceType => {
                type_check_is_reference_type(handler, ctx, kind, arguments, type_arguments, span)
            }
            Intrinsic::IsStrArray => {
                type_check_is_reference_type(handler, ctx, kind, arguments, type_arguments, span)
            }
            Intrinsic::AssertIsStrArray => {
                type_check_assert_is_str_array(handler, ctx, kind, arguments, type_arguments, span)
            }
            Intrinsic::ToStrArray => type_check_to_str_array(handler, ctx, kind, arguments, span),
            Intrinsic::Eq | Intrinsic::Gt | Intrinsic::Lt => {
                type_check_cmp(handler, ctx, kind, arguments, span)
            }
            Intrinsic::Gtf => type_check_gtf(handler, ctx, kind, arguments, type_arguments, span),
            Intrinsic::AddrOf => type_check_addr_of(handler, ctx, kind, arguments, span),
            Intrinsic::StateClear => type_check_state_clear(handler, ctx, kind, arguments, span),
            Intrinsic::StateLoadWord => {
                type_check_state_load_word(handler, ctx, kind, arguments, span)
            }
            Intrinsic::StateStoreWord => {
                type_check_state_store_word(handler, ctx, kind, arguments, type_arguments, span)
            }
            Intrinsic::StateLoadQuad | Intrinsic::StateStoreQuad => {
                type_check_state_quad(handler, ctx, kind, arguments, type_arguments, span)
            }
            Intrinsic::Log => type_check_log(handler, ctx, kind, arguments, span),
            Intrinsic::Add | Intrinsic::Sub | Intrinsic::Mul | Intrinsic::Div | Intrinsic::Mod => {
                type_check_arith_binary_op(handler, ctx, kind, arguments, type_arguments, span)
            }
            Intrinsic::And | Intrinsic::Or | Intrinsic::Xor => {
                type_check_bitwise_binary_op(handler, ctx, kind, arguments, type_arguments, span)
            }
            Intrinsic::Lsh | Intrinsic::Rsh => {
                type_check_shift_binary_op(handler, ctx, kind, arguments, type_arguments, span)
            }
            Intrinsic::Revert => {
                type_check_revert(handler, ctx, kind, arguments, type_arguments, span)
            }
            Intrinsic::PtrAdd | Intrinsic::PtrSub => {
                type_check_ptr_ops(handler, ctx, kind, arguments, type_arguments, span)
            }
            Intrinsic::Smo => type_check_smo(handler, ctx, kind, arguments, type_arguments, span),
            Intrinsic::Not => type_check_not(handler, ctx, kind, arguments, type_arguments, span),
            Intrinsic::JmpMem => {
                type_check_jmp_mem(handler, ctx, kind, arguments, type_arguments, span)
            }
            Intrinsic::ContractCall => {
                type_check_contract_call(handler, ctx, kind, arguments, type_arguments, span)
            }
            Intrinsic::ContractRet => {
                type_check_contract_ret(handler, ctx, kind, arguments, type_arguments, span)
            }
            Intrinsic::EncodeBufferEmpty => {
                type_check_encode_buffer_empty(handler, ctx, kind, arguments, type_arguments, span)
            }
            Intrinsic::EncodeBufferAppend => {
                type_check_encode_append(handler, ctx, kind, arguments, type_arguments, span)
            }
            Intrinsic::EncodeBufferAsRawSlice => {
                type_check_encode_as_raw_slice(handler, ctx, kind, arguments, type_arguments, span)
            }
            Intrinsic::Slice => {
                type_check_slice(handler, ctx, kind, arguments, type_arguments, span)
            }
            Intrinsic::ElemAt => type_check_elem_at(arguments, handler, kind, span, ctx),
            Intrinsic::Transmute => {
                type_check_transmute(arguments, handler, kind, type_arguments, span, ctx)
            }
            Intrinsic::Dbg => {
                unreachable!("__dbg should not exist in the typed tree")
            }
        }
    }
}

fn ensure_intrinsic_supported(
    handler: &Handler,
    ctx: &TypeCheckContext,
    kind: Intrinsic,
    span: Span,
) -> Result<(), ErrorEmitted> {
    if ctx.build_target() == BuildTarget::Polkavm && is_fuel_intrinsic(kind) {
        return Err(handler.emit_err(CompileError::FuelIntrinsicNotSupported {
            intrinsic: kind.to_string(),
            target: ctx.build_target().to_string(),
            span,
        }));
    }
    Ok(())
}

fn is_fuel_intrinsic(kind: Intrinsic) -> bool {
    matches!(
        kind,
        Intrinsic::Gtf
            | Intrinsic::StateClear
            | Intrinsic::StateLoadWord
            | Intrinsic::StateStoreWord
            | Intrinsic::StateLoadQuad
            | Intrinsic::StateStoreQuad
            | Intrinsic::Log
            | Intrinsic::Revert
            | Intrinsic::JmpMem
            | Intrinsic::Smo
    )
}

fn type_check_transmute(
    arguments: &[Expression],
    handler: &Handler,
    kind: Intrinsic,
    type_arguments: &[GenericArgument],
    span: Span,
    mut ctx: TypeCheckContext,
) -> Result<(TyIntrinsicFunctionKind, TypeId), ErrorEmitted> {
    if arguments.len() != 1 {
        return Err(handler.emit_err(CompileError::IntrinsicIncorrectNumArgs {
            name: kind.to_string(),
            expected: 1,
            span,
        }));
    }

    let engines = ctx.engines();

    // Both type arguments needs to be explicitly defined
    if type_arguments.len() != 2 {
        return Err(handler.emit_err(CompileError::IntrinsicIncorrectNumTArgs {
            name: kind.to_string(),
            expected: 2,
            span,
        }));
    }

    let src_type = ctx
        .resolve_type(
            handler,
            type_arguments[0].type_id(),
            &type_arguments[0].span(),
            EnforceTypeArguments::Yes,
            None,
        )
        .unwrap_or_else(|err| engines.te().id_of_error_recovery(err));
    let return_type = ctx
        .resolve_type(
            handler,
            type_arguments[1].type_id(),
            &type_arguments[1].span(),
            EnforceTypeArguments::Yes,
            None,
        )
        .unwrap_or_else(|err| engines.te().id_of_error_recovery(err));

    // type check first argument
    let arg_type = engines.te().new_unknown();
    let first_argument_typed_expr = {
        let ctx = ctx
            .by_ref()
            .with_help_text("")
            .with_type_annotation(arg_type);
        ty::TyExpression::type_check(handler, ctx, &arguments[0])?
    };

    engines.te().unify(
        handler,
        engines,
        first_argument_typed_expr.return_type,
        src_type,
        &first_argument_typed_expr.span,
        "",
        || None,
    );

    let mut final_type_arguments = type_arguments.to_vec();
    *final_type_arguments[0].type_id_mut() = src_type;
    *final_type_arguments[1].type_id_mut() = return_type;
    Ok((
        TyIntrinsicFunctionKind {
            kind,
            arguments: vec![first_argument_typed_expr],
            type_arguments: final_type_arguments,
            span,
        },
        return_type,
    ))
}

fn type_check_elem_at(
    arguments: &[Expression],
    handler: &Handler,
    kind: Intrinsic,
    span: Span,
    ctx: TypeCheckContext,
) -> Result<(TyIntrinsicFunctionKind, TypeId), ErrorEmitted> {
    if arguments.len() != 2 {
        return Err(handler.emit_err(CompileError::IntrinsicIncorrectNumArgs {
            name: kind.to_string(),
            expected: 2,
            span,
        }));
    }

    let type_engine = ctx.engines.te();
    let engines = ctx.engines();

    let mut ctx = ctx;

    // check first argument
    let first_argument_span = arguments[0].span.clone();
    let first_argument_type = type_engine.new_unknown();
    let first_argument_typed_expr = {
        let ctx = ctx
            .by_ref()
            .with_help_text("")
            .with_type_annotation(first_argument_type);
        ty::TyExpression::type_check(handler, ctx, &arguments[0])?
    };

    // first argument can be ref to array or ref to slice
    let elem_type = match &*type_engine.get(first_argument_type) {
        TypeInfo::Ref {
            referenced_type,
            to_mutable_value,
        } => match &*type_engine.get(referenced_type.type_id) {
            TypeInfo::Array(elem_ty, _) | TypeInfo::Slice(elem_ty) => {
                Some((*to_mutable_value, elem_ty.type_id))
            }
            _ => None,
        },
        _ => None,
    };
    let Some((to_mutable_value, elem_type_type_id)) = elem_type else {
        return Err(handler.emit_err(CompileError::IntrinsicUnsupportedArgType {
            name: kind.to_string(),
            span: first_argument_span,
            hint: "Only references to arrays or slices can be used as argument here".to_string(),
        }));
    };

    // index argument
    let index_typed_expr = {
        let ctx = ctx
            .by_ref()
            .with_help_text("")
            .with_type_annotation(type_engine.id_of_u64());
        ty::TyExpression::type_check(handler, ctx, &arguments[1])?
    };

    let return_type =
        type_engine.insert_ref_without_annotations(engines, to_mutable_value, elem_type_type_id);

    Ok((
        TyIntrinsicFunctionKind {
            kind,
            arguments: vec![first_argument_typed_expr, index_typed_expr],
            type_arguments: vec![],
            span,
        },
        return_type,
    ))
}

fn type_check_slice(
    handler: &Handler,
    mut ctx: TypeCheckContext,
    kind: sway_ast::Intrinsic,
    arguments: &[Expression],
    _type_arguments: &[GenericArgument],
    span: Span,
) -> Result<(ty::TyIntrinsicFunctionKind, TypeId), ErrorEmitted> {
    if arguments.len() != 3 {
        return Err(handler.emit_err(CompileError::IntrinsicIncorrectNumArgs {
            name: kind.to_string(),
            expected: 3,
            span,
        }));
    }

    let type_engine = ctx.engines.te();
    let engines = ctx.engines();

    // start index argument
    let start_ty_expr = {
        let ctx = ctx
            .by_ref()
            .with_help_text("")
            .with_type_annotation(type_engine.id_of_u64());
        ty::TyExpression::type_check(handler, ctx, &arguments[1])?
    };

    // end index argument
    let end_ty_expr = {
        let ctx = ctx
            .by_ref()
            .with_help_text("")
            .with_type_annotation(type_engine.id_of_u64());
        ty::TyExpression::type_check(handler, ctx, &arguments[2])?
    };

    // check first argument
    let first_argument_span = arguments[0].span.clone();
    let first_argument_type = type_engine.new_unknown();
    let first_argument_ty_expr = {
        let ctx = ctx
            .by_ref()
            .with_help_text("")
            .with_type_annotation(first_argument_type);
        ty::TyExpression::type_check(handler, ctx, &arguments[0])?
    };

    // statically check start and end, if possible
    let start_literal = start_ty_expr
        .expression
        .as_literal()
        .and_then(|x| x.cast_value_to_u64());

    let end_literal = end_ty_expr
        .expression
        .as_literal()
        .and_then(|x| x.cast_value_to_u64());

    if let (Some(start), Some(end)) = (start_literal, end_literal) {
        if start > end {
            return Err(
                handler.emit_err(CompileError::InvalidRangeEndGreaterThanStart {
                    start,
                    end,
                    span,
                }),
            );
        }
    }

    fn create_ref_to_slice(
        engines: &Engines,
        to_mutable_value: bool,
        elem_type_arg: GenericTypeArgument,
    ) -> TypeId {
        let type_engine = engines.te();
        let slice_type_id = type_engine.insert_slice(engines, elem_type_arg);
        type_engine.insert_ref_without_annotations(engines, to_mutable_value, slice_type_id)
    }

    // first argument can be ref to array or ref to slice
    let err = CompileError::IntrinsicUnsupportedArgType {
        name: kind.to_string(),
        span: first_argument_span,
        hint: "Only references to arrays or slices can be used as argument here".to_string(),
    };
    let r = match &*type_engine.get(first_argument_type) {
        TypeInfo::Ref {
            referenced_type,
            to_mutable_value,
        } => match &*type_engine.get(referenced_type.type_id) {
            TypeInfo::Array(elem_type_arg, array_len)
                if array_len.expr().as_literal_val().is_some() =>
            {
                // SAFETY: safe by the guard above
                let array_len = array_len
                    .expr()
                    .as_literal_val()
                    .expect("unexpected non literal array length")
                    as u64;

                if let Some(v) = start_literal {
                    if v > array_len {
                        return Err(handler.emit_err(CompileError::ArrayOutOfBounds {
                            index: v,
                            count: array_len,
                            span,
                        }));
                    }
                }

                if let Some(v) = end_literal {
                    if v > array_len {
                        return Err(handler.emit_err(CompileError::ArrayOutOfBounds {
                            index: v,
                            count: array_len,
                            span,
                        }));
                    }
                }

                Some((
                    TyIntrinsicFunctionKind {
                        kind,
                        arguments: vec![first_argument_ty_expr, start_ty_expr, end_ty_expr],
                        type_arguments: vec![],
                        span,
                    },
                    create_ref_to_slice(engines, *to_mutable_value, elem_type_arg.clone()),
                ))
            }
            TypeInfo::Slice(elem_type_arg) => Some((
                TyIntrinsicFunctionKind {
                    kind,
                    arguments: vec![first_argument_ty_expr, start_ty_expr, end_ty_expr],
                    type_arguments: vec![],
                    span,
                },
                create_ref_to_slice(engines, *to_mutable_value, elem_type_arg.clone()),
            )),
            _ => None,
        },
        _ => None,
    };

    match r {
        Some(r) => Ok(r),
        None => Err(handler.emit_err(err)),
    }
}

fn type_check_encode_as_raw_slice(
    handler: &Handler,
    mut ctx: TypeCheckContext,
    kind: sway_ast::Intrinsic,
    arguments: &[Expression],
    _type_arguments: &[GenericArgument],
    span: Span,
) -> Result<(ty::TyIntrinsicFunctionKind, TypeId), ErrorEmitted> {
    let type_engine = ctx.engines.te();

    let buffer_expr = {
        let ctx = ctx
            .by_ref()
            .with_help_text("")
            .with_type_annotation(type_engine.new_unknown());
        ty::TyExpression::type_check(handler, ctx, &arguments[0].clone())?
    };

    let kind = ty::TyIntrinsicFunctionKind {
        kind,
        arguments: vec![buffer_expr],
        type_arguments: vec![],
        span,
    };
    Ok((kind, type_engine.id_of_raw_slice()))
}

fn type_check_encode_buffer_empty(
    handler: &Handler,
    ctx: TypeCheckContext,
    kind: sway_ast::Intrinsic,
    arguments: &[Expression],
    _type_arguments: &[GenericArgument],
    span: Span,
) -> Result<(ty::TyIntrinsicFunctionKind, TypeId), ErrorEmitted> {
    if !arguments.is_empty() {
        return Err(handler.emit_err(CompileError::IntrinsicIncorrectNumArgs {
            name: kind.to_string(),
            expected: 0,
            span,
        }));
    }

    let kind = ty::TyIntrinsicFunctionKind {
        kind,
        arguments: vec![],
        type_arguments: vec![],
        span,
    };

    Ok((kind, get_encoding_buffer_type(ctx.engines())))
}

/// Returns the [TypeId] of the buffer type used in encoding: `(raw_ptr, u64, u64)`.
/// The buffer type is a shareable [TypeInfo::Tuple], so it will be inserted into
/// the [TypeEngine] only once, when this method is called for the first time.
fn get_encoding_buffer_type(engines: &Engines) -> TypeId {
    let type_engine = engines.te();
    type_engine.insert_tuple_without_annotations(
        engines,
        vec![
            type_engine.id_of_raw_ptr(),
            type_engine.id_of_u64(),
            type_engine.id_of_u64(),
        ],
    )
}

fn type_check_encode_append(
    handler: &Handler,
    mut ctx: TypeCheckContext,
    kind: sway_ast::Intrinsic,
    arguments: &[Expression],
    _type_arguments: &[GenericArgument],
    span: Span,
) -> Result<(ty::TyIntrinsicFunctionKind, TypeId), ErrorEmitted> {
    if arguments.len() != 2 {
        return Err(handler.emit_err(CompileError::IntrinsicIncorrectNumArgs {
            name: kind.to_string(),
            expected: 2,
            span,
        }));
    }

    let type_engine = ctx.engines.te();
    let engines = ctx.engines();

    let buffer_type = get_encoding_buffer_type(engines);
    let buffer_expr = {
        let ctx = ctx
            .by_ref()
            .with_help_text("")
            .with_type_annotation(buffer_type);
        ty::TyExpression::type_check(handler, ctx, &arguments[0])?
    };

    let item_span = arguments[1].span.clone();
    let item_type = type_engine.new_unknown();
    let item_expr = {
        let ctx = ctx
            .by_ref()
            .with_help_text("")
            .with_type_annotation(item_type);
        ty::TyExpression::type_check(handler, ctx, &arguments[1])?
    };

    // only supported types
    if item_type.is_concrete(engines, TreatNumericAs::Abstract) {
        match &*engines.te().get(item_type) {
            TypeInfo::Boolean
            | TypeInfo::UnsignedInteger(IntegerBits::Eight)
            | TypeInfo::UnsignedInteger(IntegerBits::Sixteen)
            | TypeInfo::UnsignedInteger(IntegerBits::ThirtyTwo)
            | TypeInfo::UnsignedInteger(IntegerBits::SixtyFour)
            | TypeInfo::UnsignedInteger(IntegerBits::V256)
            | TypeInfo::B256
            | TypeInfo::StringArray(_)
            | TypeInfo::StringSlice
            | TypeInfo::RawUntypedSlice => {}
            _ => {
                return Err(
                    handler.emit_err(CompileError::EncodingUnsupportedType { span: item_span })
                )
            }
        };
    }

    let kind = ty::TyIntrinsicFunctionKind {
        kind,
        arguments: vec![buffer_expr, item_expr],
        type_arguments: vec![],
        span,
    };
    Ok((kind, buffer_type))
}

/// Signature: `__not(val: u64) -> u64`
/// Description: Return the bitwise negation of the operator.
/// Constraints: None.
fn type_check_not(
    handler: &Handler,
    ctx: TypeCheckContext,
    kind: sway_ast::Intrinsic,
    arguments: &[Expression],
    _type_arguments: &[GenericArgument],
    span: Span,
) -> Result<(ty::TyIntrinsicFunctionKind, TypeId), ErrorEmitted> {
    let type_engine = ctx.engines.te();
    let engines = ctx.engines();

    if arguments.len() != 1 {
        return Err(handler.emit_err(CompileError::IntrinsicIncorrectNumArgs {
            name: kind.to_string(),
            expected: 1,
            span,
        }));
    }

    let return_type = type_engine.new_unknown();

    let mut ctx = ctx.with_help_text("").with_type_annotation(return_type);

    let operand = &arguments[0];
    let operand_expr = ty::TyExpression::type_check(handler, ctx.by_ref(), operand)?;

    let t_arc = engines.te().get(operand_expr.return_type);
    let t = &*t_arc;
    match t {
        TypeInfo::B256 | TypeInfo::UnsignedInteger(_) | TypeInfo::Numeric => Ok((
            ty::TyIntrinsicFunctionKind {
                kind,
                arguments: vec![operand_expr],
                type_arguments: vec![],
                span,
            },
            return_type,
        )),
        _ => Err(handler.emit_err(CompileError::TypeError(
            sway_error::type_error::TypeError::MismatchedType {
                expected: "unsigned integer or b256".into(),
                received: engines.help_out(return_type).to_string(),
                help_text: "Incorrect argument type".into(),
                span,
            },
        ))),
    }
}

/// Signature: `__size_of_val<T>(val: T) -> u64`
/// Description: Return the size of type `T` in bytes.
/// Constraints: None.
fn type_check_size_of_val(
    handler: &Handler,
    ctx: TypeCheckContext,
    kind: sway_ast::Intrinsic,
    arguments: &[Expression],
    _type_arguments: &[GenericArgument],
    span: Span,
) -> Result<(ty::TyIntrinsicFunctionKind, TypeId), ErrorEmitted> {
    let type_engine = ctx.engines.te();

    if arguments.len() != 1 {
        return Err(handler.emit_err(CompileError::IntrinsicIncorrectNumArgs {
            name: kind.to_string(),
            expected: 1,
            span,
        }));
    }
    let ctx = ctx
        .with_help_text("")
        .with_type_annotation(type_engine.new_unknown());
    let exp = ty::TyExpression::type_check(handler, ctx, &arguments[0])?;
    let intrinsic_function = ty::TyIntrinsicFunctionKind {
        kind,
        arguments: vec![exp],
        type_arguments: vec![],
        span: span.clone(),
    };
    Ok((intrinsic_function, type_engine.id_of_u64()))
}

/// Signature: `__size_of<T>() -> u64`
/// Description: Return the size of type `T` in bytes.
/// Constraints: None.
fn type_check_size_of_type(
    handler: &Handler,
    ctx: TypeCheckContext,
    kind: sway_ast::Intrinsic,
    arguments: &[Expression],
    type_arguments: &[GenericArgument],
    span: Span,
) -> Result<(ty::TyIntrinsicFunctionKind, TypeId), ErrorEmitted> {
    let type_engine = ctx.engines.te();
    let engines = ctx.engines();

    if !arguments.is_empty() {
        return Err(handler.emit_err(CompileError::IntrinsicIncorrectNumArgs {
            name: kind.to_string(),
            expected: 0,
            span,
        }));
    }
    if type_arguments.len() != 1 {
        return Err(handler.emit_err(CompileError::IntrinsicIncorrectNumTArgs {
            name: kind.to_string(),
            expected: 1,
            span,
        }));
    }
    let targ = type_arguments[0].clone();
    let initial_type_info = type_engine
        .to_typeinfo(targ.type_id(), &targ.span())
        .map_err(|e| handler.emit_err(e.into()))
        .unwrap_or_else(TypeInfo::ErrorRecovery);
    let initial_type_id = type_engine.insert(engines, initial_type_info, targ.span().source_id());
    let type_id = ctx
        .resolve_type(
            handler,
            initial_type_id,
            &targ.span(),
            EnforceTypeArguments::Yes,
            None,
        )
        .unwrap_or_else(|err| type_engine.id_of_error_recovery(err));
    let intrinsic_function = ty::TyIntrinsicFunctionKind {
        kind,
        arguments: vec![],
        type_arguments: vec![GenericArgument::Type(GenericTypeArgument {
            type_id,
            initial_type_id,
            span: targ.span(),
            call_path_tree: targ
                .as_type_argument()
                .unwrap()
                .call_path_tree
                .as_ref()
                .cloned(),
        })],
        span,
    };
    Ok((intrinsic_function, type_engine.id_of_u64()))
}

/// Signature: `__is_reference_type<T>() -> bool`
/// Description: Returns `true` if `T` is a _reference type_ and `false` otherwise.
/// Constraints: None.
fn type_check_is_reference_type(
    handler: &Handler,
    ctx: TypeCheckContext,
    kind: sway_ast::Intrinsic,
    _arguments: &[Expression],
    type_arguments: &[GenericArgument],
    span: Span,
) -> Result<(ty::TyIntrinsicFunctionKind, TypeId), ErrorEmitted> {
    let type_engine = ctx.engines.te();
    let engines = ctx.engines();

    if type_arguments.len() != 1 {
        return Err(handler.emit_err(CompileError::IntrinsicIncorrectNumTArgs {
            name: kind.to_string(),
            expected: 1,
            span,
        }));
    }
    let targ = type_arguments[0].clone();
    let initial_type_info = type_engine
        .to_typeinfo(targ.type_id(), &targ.span())
        .map_err(|e| handler.emit_err(e.into()))
        .unwrap_or_else(TypeInfo::ErrorRecovery);
    let initial_type_id = type_engine.insert(engines, initial_type_info, targ.span().source_id());
    let type_id = ctx
        .resolve_type(
            handler,
            initial_type_id,
            &targ.span(),
            EnforceTypeArguments::Yes,
            None,
        )
        .unwrap_or_else(|err| type_engine.id_of_error_recovery(err));
    let intrinsic_function = ty::TyIntrinsicFunctionKind {
        kind,
        arguments: vec![],
        type_arguments: vec![GenericArgument::Type(GenericTypeArgument {
            type_id,
            initial_type_id,
            span: targ.span(),
            call_path_tree: targ
                .as_type_argument()
                .unwrap()
                .call_path_tree
                .as_ref()
                .cloned(),
        })],
        span,
    };
    Ok((intrinsic_function, type_engine.id_of_bool()))
}

/// Signature: `__assert_is_str_array<T>()`
/// Description: Throws a compile error if `T` is not of type str.
/// Constraints: None.
fn type_check_assert_is_str_array(
    handler: &Handler,
    ctx: TypeCheckContext,
    kind: sway_ast::Intrinsic,
    _arguments: &[Expression],
    type_arguments: &[GenericArgument],
    span: Span,
) -> Result<(ty::TyIntrinsicFunctionKind, TypeId), ErrorEmitted> {
    let type_engine = ctx.engines.te();
    let engines = ctx.engines();

    if type_arguments.len() != 1 {
        return Err(handler.emit_err(CompileError::IntrinsicIncorrectNumTArgs {
            name: kind.to_string(),
            expected: 1,
            span,
        }));
    }
    let targ = type_arguments[0].clone();
    let initial_type_info = type_engine
        .to_typeinfo(targ.type_id(), &targ.span())
        .map_err(|e| handler.emit_err(e.into()))
        .unwrap_or_else(TypeInfo::ErrorRecovery);
    let initial_type_id = type_engine.insert(engines, initial_type_info, targ.span().source_id());
    let type_id = ctx
        .resolve_type(
            handler,
            initial_type_id,
            &targ.span(),
            EnforceTypeArguments::Yes,
            None,
        )
        .unwrap_or_else(|err| type_engine.id_of_error_recovery(err));
    let intrinsic_function = ty::TyIntrinsicFunctionKind {
        kind,
        arguments: vec![],
        type_arguments: vec![GenericArgument::Type(GenericTypeArgument {
            type_id,
            initial_type_id,
            span: targ.span(),
            call_path_tree: targ
                .as_type_argument()
                .unwrap()
                .call_path_tree
                .as_ref()
                .cloned(),
        })],
        span,
    };
    Ok((intrinsic_function, type_engine.id_of_unit()))
}

fn type_check_to_str_array(
    handler: &Handler,
    mut ctx: TypeCheckContext,
    kind: sway_ast::Intrinsic,
    arguments: &[Expression],
    span: Span,
) -> Result<(ty::TyIntrinsicFunctionKind, TypeId), ErrorEmitted> {
    let type_engine = ctx.engines.te();
    let engines = ctx.engines();

    if arguments.len() != 1 {
        return Err(handler.emit_err(CompileError::IntrinsicIncorrectNumArgs {
            name: kind.to_string(),
            expected: 1,
            span,
        }));
    }
    let arg = &arguments[0];

    match &arg.kind {
        ExpressionKind::Literal(Literal::String(s)) => {
            let span = arg.span.clone();

            let mut ctx = ctx.by_ref().with_type_annotation(type_engine.new_unknown());
            let new_type = ty::TyExpression::type_check(handler, ctx.by_ref(), arg)?;

            Ok((
                ty::TyIntrinsicFunctionKind {
                    kind,
                    arguments: vec![new_type],
                    type_arguments: vec![],
                    span,
                },
                type_engine.insert_string_array_without_annotations(engines, s.as_str().len()),
            ))
        }
        _ => Err(handler.emit_err(CompileError::ExpectedStringLiteral {
            span: arg.span.clone(),
        })),
    }
}

/// Signature: `__eq<T>(lhs: T, rhs: T) -> bool`
/// Description: Returns whether `lhs` and `rhs` are equal.
/// Constraints: `T` is `bool`, `u8`, `u16`, `u32`, `u64`, or `raw_ptr`.
///
/// Signature: `__gt<T>(lhs: T, rhs: T) -> bool`
/// Description: Returns whether `lhs` > `rhs`.
/// Constraints: `T` is `u8`, `u16`, `u32`, `u64`.
///
/// Signature: `__lt<T>(lhs: T, rhs: T) -> bool`
/// Description: Returns whether `lhs` < `rhs`.
/// Constraints: `T` is `u8`, `u16`, `u32`, `u64`.
fn type_check_cmp(
    handler: &Handler,
    mut ctx: TypeCheckContext,
    kind: sway_ast::Intrinsic,
    arguments: &[Expression],
    span: Span,
) -> Result<(ty::TyIntrinsicFunctionKind, TypeId), ErrorEmitted> {
    let type_engine = ctx.engines.te();

    if arguments.len() != 2 {
        return Err(handler.emit_err(CompileError::IntrinsicIncorrectNumArgs {
            name: kind.to_string(),
            expected: 2,
            span,
        }));
    }
    let mut ctx = ctx.by_ref().with_type_annotation(type_engine.new_unknown());

    let lhs = &arguments[0];
    let lhs = ty::TyExpression::type_check(handler, ctx.by_ref(), lhs)?;
    let rhs = &arguments[1];
    let rhs = ty::TyExpression::type_check(handler, ctx, rhs)?;

    // Check for supported argument types
    let arg_ty = type_engine
        .to_typeinfo(lhs.return_type, &lhs.span)
        .map_err(|e| handler.emit_err(e.into()))
        .unwrap_or_else(TypeInfo::ErrorRecovery);

    let is_eq_bool_ptr = matches!(&kind, Intrinsic::Eq)
        && matches!(arg_ty, TypeInfo::Boolean | TypeInfo::RawUntypedPtr);
    let is_valid_arg_ty = matches!(
        arg_ty,
        TypeInfo::UnsignedInteger(_) | TypeInfo::Numeric | TypeInfo::B256
    ) || is_eq_bool_ptr;

    if !is_valid_arg_ty {
        return Err(handler.emit_err(CompileError::IntrinsicUnsupportedArgType {
            name: kind.to_string(),
            span: lhs.span,
            hint: "".to_string(),
        }));
    }

    Ok((
        ty::TyIntrinsicFunctionKind {
            kind,
            arguments: vec![lhs, rhs],
            type_arguments: vec![],
            span,
        },
        type_engine.id_of_bool(),
    ))
}

/// Signature: `__gtf<T>(index: u64, tx_field_id: u64) -> T`
/// Description: Returns transaction field with ID `tx_field_id` at index `index`, if applicable.
///              This is a wrapper around FuelVM's `gtf` instruction:
///              https://fuellabs.github.io/fuel-specs/master/vm/instruction_set#gtf-get-transaction-fields.
///              The resulting field is cast to `T`.
/// Constraints: None.
fn type_check_gtf(
    handler: &Handler,
    mut ctx: TypeCheckContext,
    kind: sway_ast::Intrinsic,
    arguments: &[Expression],
    type_arguments: &[GenericArgument],
    span: Span,
) -> Result<(ty::TyIntrinsicFunctionKind, TypeId), ErrorEmitted> {
    let type_engine = ctx.engines.te();
    let engines = ctx.engines();

    if arguments.len() != 2 {
        return Err(handler.emit_err(CompileError::IntrinsicIncorrectNumArgs {
            name: kind.to_string(),
            expected: 2,
            span,
        }));
    }

    if type_arguments.len() != 1 {
        return Err(handler.emit_err(CompileError::IntrinsicIncorrectNumTArgs {
            name: kind.to_string(),
            expected: 1,
            span,
        }));
    }

    // Type check the first argument which is the index
    let mut ctx = ctx.by_ref().with_type_annotation(type_engine.id_of_u64());
    let index = ty::TyExpression::type_check(handler, ctx.by_ref(), &arguments[0])?;

    // Type check the second argument which is the tx field ID
    let mut ctx = ctx.by_ref().with_type_annotation(type_engine.id_of_u64());
    let tx_field_id = ty::TyExpression::type_check(handler, ctx.by_ref(), &arguments[1])?;

    let targ = type_arguments[0].clone();
    let initial_type_info = type_engine
        .to_typeinfo(targ.type_id(), &targ.span())
        .map_err(|e| handler.emit_err(e.into()))
        .unwrap_or_else(TypeInfo::ErrorRecovery);
    let initial_type_id = type_engine.insert(engines, initial_type_info, targ.span().source_id());
    let type_id = ctx
        .resolve_type(
            handler,
            initial_type_id,
            &targ.span(),
            EnforceTypeArguments::Yes,
            None,
        )
        .unwrap_or_else(|err| type_engine.id_of_error_recovery(err));

    Ok((
        ty::TyIntrinsicFunctionKind {
            kind,
            arguments: vec![index, tx_field_id],
            type_arguments: vec![GenericArgument::Type(GenericTypeArgument {
                type_id,
                initial_type_id,
                span: targ.span(),
                call_path_tree: targ
                    .as_type_argument()
                    .unwrap()
                    .call_path_tree
                    .as_ref()
                    .cloned(),
            })],
            span,
        },
        type_id,
    ))
}

/// Signature: `__addr_of<T>(val: T) -> raw_ptr`
/// Description: Returns the address in memory where `val` is stored.
fn type_check_addr_of(
    handler: &Handler,
    ctx: TypeCheckContext,
    kind: sway_ast::Intrinsic,
    arguments: &[Expression],
    span: Span,
) -> Result<(ty::TyIntrinsicFunctionKind, TypeId), ErrorEmitted> {
    let type_engine = ctx.engines.te();

    if arguments.len() != 1 {
        return Err(handler.emit_err(CompileError::IntrinsicIncorrectNumArgs {
            name: kind.to_string(),
            expected: 1,
            span,
        }));
    }
    let ctx = ctx
        .with_help_text("")
        .with_type_annotation(type_engine.new_unknown());
    let exp = ty::TyExpression::type_check(handler, ctx, &arguments[0])?;
    let intrinsic_function = ty::TyIntrinsicFunctionKind {
        kind,
        arguments: vec![exp],
        type_arguments: vec![],
        span,
    };
    Ok((intrinsic_function, type_engine.id_of_raw_ptr()))
}

/// Signature: `__state_load_clear(key: b256, slots: u64) -> bool`
/// Description: Clears `slots` number of slots (`b256` each) in storage starting at key `key`.
///              Returns a Boolean describing whether all the storage slots were previously set.
/// Constraints: None.
fn type_check_state_clear(
    handler: &Handler,
    ctx: TypeCheckContext,
    kind: sway_ast::Intrinsic,
    arguments: &[Expression],
    span: Span,
) -> Result<(ty::TyIntrinsicFunctionKind, TypeId), ErrorEmitted> {
    let type_engine = ctx.engines.te();

    if arguments.len() != 2 {
        return Err(handler.emit_err(CompileError::IntrinsicIncorrectNumArgs {
            name: kind.to_string(),
            expected: 1,
            span,
        }));
    }

    // `key` argument
    let mut ctx = ctx
        .with_help_text("")
        .with_type_annotation(type_engine.new_unknown());
    let key_exp = ty::TyExpression::type_check(handler, ctx.by_ref(), &arguments[0])?;
    let key_ty = type_engine
        .to_typeinfo(key_exp.return_type, &span)
        .map_err(|e| handler.emit_err(e.into()))
        .unwrap_or_else(TypeInfo::ErrorRecovery);
    if !key_ty.eq(
        &TypeInfo::B256,
        &PartialEqWithEnginesContext::new(ctx.engines()),
    ) {
        return Err(handler.emit_err(CompileError::IntrinsicUnsupportedArgType {
            name: kind.to_string(),
            span,
            hint: "Argument type must be B256, a key into the state storage".to_string(),
        }));
    }

    // `slots` argument
    let mut ctx = ctx.with_type_annotation(type_engine.id_of_u64());
    let number_of_slots_exp = ty::TyExpression::type_check(handler, ctx.by_ref(), &arguments[1])?;

    // Typed intrinsic
    let intrinsic_function = ty::TyIntrinsicFunctionKind {
        kind,
        arguments: vec![key_exp, number_of_slots_exp],
        type_arguments: vec![],
        span,
    };
    Ok((intrinsic_function, type_engine.id_of_bool()))
}

/// Signature: `__state_load_word(key: b256) -> u64`
/// Description: Reads and returns a single word from storage at key `key`.
/// Constraints: None.
fn type_check_state_load_word(
    handler: &Handler,
    ctx: TypeCheckContext,
    kind: sway_ast::Intrinsic,
    arguments: &[Expression],
    span: Span,
) -> Result<(ty::TyIntrinsicFunctionKind, TypeId), ErrorEmitted> {
    let type_engine = ctx.engines.te();
    let engines = ctx.engines();

    if arguments.len() != 1 {
        return Err(handler.emit_err(CompileError::IntrinsicIncorrectNumArgs {
            name: kind.to_string(),
            expected: 1,
            span,
        }));
    }
    let ctx = ctx
        .with_help_text("")
        .with_type_annotation(type_engine.new_unknown());
    let exp = ty::TyExpression::type_check(handler, ctx, &arguments[0])?;
    let key_ty = type_engine
        .to_typeinfo(exp.return_type, &span)
        .map_err(|e| handler.emit_err(e.into()))
        .unwrap_or_else(TypeInfo::ErrorRecovery);
    if !key_ty.eq(&TypeInfo::B256, &PartialEqWithEnginesContext::new(engines)) {
        return Err(handler.emit_err(CompileError::IntrinsicUnsupportedArgType {
            name: kind.to_string(),
            span,
            hint: "Argument type must be B256, a key into the state storage".to_string(),
        }));
    }
    let intrinsic_function = ty::TyIntrinsicFunctionKind {
        kind,
        arguments: vec![exp],
        type_arguments: vec![],
        span,
    };
    Ok((intrinsic_function, type_engine.id_of_u64()))
}

/// Signature: `__state_store_word(key: b256, val: u64) -> bool`
/// Description: Stores a single word `val` into storage at key `key`. Returns a Boolean describing
///              whether the store slot was previously set.
/// Constraints: None.
fn type_check_state_store_word(
    handler: &Handler,
    ctx: TypeCheckContext,
    kind: sway_ast::Intrinsic,
    arguments: &[Expression],
    type_arguments: &[GenericArgument],
    span: Span,
) -> Result<(ty::TyIntrinsicFunctionKind, TypeId), ErrorEmitted> {
    let type_engine = ctx.engines.te();
    let engines = ctx.engines();

    if arguments.len() != 2 {
        return Err(handler.emit_err(CompileError::IntrinsicIncorrectNumArgs {
            name: kind.to_string(),
            expected: 2,
            span,
        }));
    }
    if type_arguments.len() > 1 {
        return Err(handler.emit_err(CompileError::IntrinsicIncorrectNumTArgs {
            name: kind.to_string(),
            expected: 1,
            span,
        }));
    }
    let mut ctx = ctx
        .with_help_text("")
        .with_type_annotation(type_engine.new_unknown());
    let key_exp = ty::TyExpression::type_check(handler, ctx.by_ref(), &arguments[0])?;
    let key_ty = type_engine
        .to_typeinfo(key_exp.return_type, &span)
        .map_err(|e| handler.emit_err(e.into()))
        .unwrap_or_else(TypeInfo::ErrorRecovery);
    if !key_ty.eq(
        &TypeInfo::B256,
        &PartialEqWithEnginesContext::new(ctx.engines()),
    ) {
        return Err(handler.emit_err(CompileError::IntrinsicUnsupportedArgType {
            name: kind.to_string(),
            span,
            hint: "Argument type must be B256, a key into the state storage".to_string(),
        }));
    }
    let mut ctx = ctx.with_type_annotation(type_engine.new_unknown());
    let val_exp = ty::TyExpression::type_check(handler, ctx.by_ref(), &arguments[1])?;
    let ctx = ctx.with_type_annotation(type_engine.id_of_u64());
    let type_argument = type_arguments.first().map(|targ| {
        let ctx = ctx.with_type_annotation(type_engine.new_unknown());
        let initial_type_info = type_engine
            .to_typeinfo(targ.type_id(), &targ.span())
            .map_err(|e| handler.emit_err(e.into()))
            .unwrap_or_else(TypeInfo::ErrorRecovery);
        let initial_type_id =
            type_engine.insert(engines, initial_type_info, targ.span().source_id());
        let type_id = ctx
            .resolve_type(
                handler,
                initial_type_id,
                &targ.span(),
                EnforceTypeArguments::Yes,
                None,
            )
            .unwrap_or_else(|err| type_engine.id_of_error_recovery(err));
        GenericArgument::Type(GenericTypeArgument {
            type_id,
            initial_type_id,
            span: span.clone(),
            call_path_tree: targ
                .as_type_argument()
                .unwrap()
                .call_path_tree
                .as_ref()
                .cloned(),
        })
    });
    let intrinsic_function = ty::TyIntrinsicFunctionKind {
        kind,
        arguments: vec![key_exp, val_exp],
        type_arguments: type_argument.map_or(vec![], |ta| vec![ta]),
        span,
    };
    Ok((intrinsic_function, type_engine.id_of_bool()))
}

/// Signature: `__state_load_quad(key: b256, ptr: raw_ptr, slots: u64)`
/// Description: Reads `slots` number of slots (`b256` each) from storage starting at key `key` and
///              stores them in memory starting at address `ptr`. Returns a Boolean describing
///              whether all the storage slots were previously set.
/// Constraints: None.
///
/// Signature: `__state_store_quad(key: b256, ptr: raw_ptr, slots: u64) -> bool`
/// Description: Stores `slots` number of slots (`b256` each) starting at address `ptr` in memory
///              into storage starting at key `key`. Returns a Boolean describing
///              whether the first storage slot was previously set.
/// Constraints: None.
fn type_check_state_quad(
    handler: &Handler,
    ctx: TypeCheckContext,
    kind: sway_ast::Intrinsic,
    arguments: &[Expression],
    type_arguments: &[GenericArgument],
    span: Span,
) -> Result<(ty::TyIntrinsicFunctionKind, TypeId), ErrorEmitted> {
    let type_engine = ctx.engines.te();
    let engines = ctx.engines();

    if arguments.len() != 3 {
        return Err(handler.emit_err(CompileError::IntrinsicIncorrectNumArgs {
            name: kind.to_string(),
            expected: 3,
            span,
        }));
    }
    if type_arguments.len() > 1 {
        return Err(handler.emit_err(CompileError::IntrinsicIncorrectNumTArgs {
            name: kind.to_string(),
            expected: 1,
            span,
        }));
    }
    let mut ctx = ctx
        .with_help_text("")
        .with_type_annotation(type_engine.new_unknown());
    let key_exp = ty::TyExpression::type_check(handler, ctx.by_ref(), &arguments[0])?;
    let key_ty = type_engine
        .to_typeinfo(key_exp.return_type, &span)
        .map_err(|e| handler.emit_err(e.into()))
        .unwrap_or_else(TypeInfo::ErrorRecovery);
    if !key_ty.eq(
        &TypeInfo::B256,
        &PartialEqWithEnginesContext::new(ctx.engines()),
    ) {
        return Err(handler.emit_err(CompileError::IntrinsicUnsupportedArgType {
            name: kind.to_string(),
            span,
            hint: "Argument type must be B256, a key into the state storage".to_string(),
        }));
    }
    let mut ctx = ctx.with_type_annotation(type_engine.new_unknown());
    let val_exp = ty::TyExpression::type_check(handler, ctx.by_ref(), &arguments[1])?;
    let mut ctx = ctx.with_type_annotation(type_engine.id_of_u64());
    let number_of_slots_exp = ty::TyExpression::type_check(handler, ctx.by_ref(), &arguments[2])?;
    let type_argument = type_arguments.first().map(|targ| {
        let ctx = ctx.with_type_annotation(type_engine.new_unknown());
        let initial_type_info = type_engine
            .to_typeinfo(targ.type_id(), &targ.span())
            .map_err(|e| handler.emit_err(e.into()))
            .unwrap_or_else(TypeInfo::ErrorRecovery);
        let initial_type_id =
            type_engine.insert(engines, initial_type_info, targ.span().source_id());
        let type_id = ctx
            .resolve_type(
                handler,
                initial_type_id,
                &targ.span(),
                EnforceTypeArguments::Yes,
                None,
            )
            .unwrap_or_else(|err| type_engine.id_of_error_recovery(err));
        GenericArgument::Type(GenericTypeArgument {
            type_id,
            initial_type_id,
            span: span.clone(),
            call_path_tree: targ
                .as_type_argument()
                .unwrap()
                .call_path_tree
                .as_ref()
                .cloned(),
        })
    });
    let intrinsic_function = ty::TyIntrinsicFunctionKind {
        kind,
        arguments: vec![key_exp, val_exp, number_of_slots_exp],
        type_arguments: type_argument.map_or(vec![], |ta| vec![ta]),
        span,
    };
    Ok((intrinsic_function, type_engine.id_of_bool()))
}

/// Signature: `__log<T>(val: T)`
/// Description: Logs value `val`.
/// Constraints: None.
fn type_check_log(
    handler: &Handler,
    mut ctx: TypeCheckContext,
    kind: sway_ast::Intrinsic,
    arguments: &[Expression],
    span: Span,
) -> Result<(ty::TyIntrinsicFunctionKind, TypeId), ErrorEmitted> {
    let type_engine = ctx.engines.te();

    if arguments.len() != 1 {
        return Err(handler.emit_err(CompileError::IntrinsicIncorrectNumArgs {
            name: kind.to_string(),
            expected: 1,
            span,
        }));
    }
    let ctx = ctx
        .by_ref()
        .with_help_text("")
        .with_type_annotation(type_engine.new_unknown());
    let exp = ty::TyExpression::type_check(handler, ctx, &arguments[0])?;
    let intrinsic_function = ty::TyIntrinsicFunctionKind {
        kind,
        arguments: vec![exp],
        type_arguments: vec![],
        span,
    };
    Ok((intrinsic_function, type_engine.id_of_unit()))
}

/// Signature: `__add<T>(lhs: T, rhs: T) -> T`
/// Description: Adds `lhs` and `rhs` and returns the result.
/// Constraints: `T` is an integer type, i.e. `u8`, `u16`, `u32`, `u64`.
///
/// Signature: `__sub<T>(lhs: T, rhs: T) -> T`
/// Description: Subtracts `lhs` and `rhs` and returns the result.
/// Constraints: `T` is an integer type, i.e. `u8`, `u16`, `u32`, `u64`.
///
/// Signature: `__mul<T>(lhs: T, rhs: T) -> T`
/// Description: Multiplies `lhs` and `rhs` and returns the result.
/// Constraints: `T` is an integer type, i.e. `u8`, `u16`, `u32`, `u64`.
///
/// Signature: `__div<T>(lhs: T, rhs: T) -> T`
/// Description: Divides `lhs` and `rhs` and returns the result.
/// Constraints: `T` is an integer type, i.e. `u8`, `u16`, `u32`, `u64`.
///
/// Signature: `__and<T>(lhs: T, rhs: T) -> T`
/// Description: Bitwise And of `lhs` and `rhs` and returns the result.
/// Constraints: `T` is an integer type, i.e. `u8`, `u16`, `u32`, `u64`.
///
/// Signature: `__or<T>(lhs: T, rhs: T) -> T`
/// Description: Bitwise Or `lhs` and `rhs` and returns the result.
/// Constraints: `T` is an integer type, i.e. `u8`, `u16`, `u32`, `u64`.
///
/// Signature: `__xor<T>(lhs: T, rhs: T) -> T`
/// Description: Bitwise Xor `lhs` and `rhs` and returns the result.
/// Constraints: `T` is an integer type, i.e. `u8`, `u16`, `u32`, `u64`.
fn type_check_arith_binary_op(
    handler: &Handler,
    mut ctx: TypeCheckContext,
    kind: sway_ast::Intrinsic,
    arguments: &[Expression],
    type_arguments: &[GenericArgument],
    span: Span,
) -> Result<(ty::TyIntrinsicFunctionKind, TypeId), ErrorEmitted> {
    let type_engine = ctx.engines.te();

    if arguments.len() != 2 {
        return Err(handler.emit_err(CompileError::IntrinsicIncorrectNumArgs {
            name: kind.to_string(),
            expected: 2,
            span,
        }));
    }
    if !type_arguments.is_empty() {
        return Err(handler.emit_err(CompileError::IntrinsicIncorrectNumTArgs {
            name: kind.to_string(),
            expected: 0,
            span,
        }));
    }

    let return_type = type_engine.new_numeric();
    let mut ctx = ctx
        .by_ref()
        .with_type_annotation(return_type)
        .with_help_text("Incorrect argument type");

    let lhs = &arguments[0];
    let lhs = ty::TyExpression::type_check(handler, ctx.by_ref(), lhs)?;
    let rhs = &arguments[1];
    let rhs = ty::TyExpression::type_check(handler, ctx, rhs)?;

    Ok((
        ty::TyIntrinsicFunctionKind {
            kind,
            arguments: vec![lhs, rhs],
            type_arguments: vec![],
            span,
        },
        return_type,
    ))
}

fn type_check_bitwise_binary_op(
    handler: &Handler,
    mut ctx: TypeCheckContext,
    kind: sway_ast::Intrinsic,
    arguments: &[Expression],
    type_arguments: &[GenericArgument],
    span: Span,
) -> Result<(ty::TyIntrinsicFunctionKind, TypeId), ErrorEmitted> {
    let type_engine = ctx.engines.te();
    let engines = ctx.engines();

    if arguments.len() != 2 {
        return Err(handler.emit_err(CompileError::IntrinsicIncorrectNumArgs {
            name: kind.to_string(),
            expected: 2,
            span,
        }));
    }
    if !type_arguments.is_empty() {
        return Err(handler.emit_err(CompileError::IntrinsicIncorrectNumTArgs {
            name: kind.to_string(),
            expected: 0,
            span,
        }));
    }

    let return_type = type_engine.new_unknown();
    let mut ctx = ctx
        .by_ref()
        .with_type_annotation(return_type)
        .with_help_text("Incorrect argument type");

    let lhs = &arguments[0];
    let lhs = ty::TyExpression::type_check(handler, ctx.by_ref(), lhs)?;
    let rhs = &arguments[1];
    let rhs = ty::TyExpression::type_check(handler, ctx, rhs)?;

    let t_arc = engines.te().get(lhs.return_type);
    let t = &*t_arc;
    match t {
        TypeInfo::B256 | TypeInfo::UnsignedInteger(_) | TypeInfo::Numeric => Ok((
            ty::TyIntrinsicFunctionKind {
                kind,
                arguments: vec![lhs, rhs],
                type_arguments: vec![],
                span,
            },
            return_type,
        )),
        _ => Err(handler.emit_err(CompileError::TypeError(
            sway_error::type_error::TypeError::MismatchedType {
                expected: "unsigned integer or b256".into(),
                received: engines.help_out(return_type).to_string(),
                help_text: "Incorrect argument type".into(),
                span,
            },
        ))),
    }
}

/// Signature: `__lsh<T, U>(lhs: T, rhs: U) -> T`
/// Description: Logical left shifts the `lhs` by the `rhs` and returns the result.
/// Constraints: `T` and `U` are an integer type, i.e. `u8`, `u16`, `u32`, `u64`.
///
/// Signature: `__rsh<T, U>(lhs: T, rhs: U) -> T`
/// Description: Logical right shifts the `lhs` by the `rhs` and returns the result.
/// Constraints: `T` and `U` are an integer type, i.e. `u8`, `u16`, `u32`, `u64`.
fn type_check_shift_binary_op(
    handler: &Handler,
    mut ctx: TypeCheckContext,
    kind: sway_ast::Intrinsic,
    arguments: &[Expression],
    type_arguments: &[GenericArgument],
    span: Span,
) -> Result<(ty::TyIntrinsicFunctionKind, TypeId), ErrorEmitted> {
    let engines = ctx.engines();

    if arguments.len() != 2 {
        return Err(handler.emit_err(CompileError::IntrinsicIncorrectNumArgs {
            name: kind.to_string(),
            expected: 2,
            span,
        }));
    }
    if !type_arguments.is_empty() {
        return Err(handler.emit_err(CompileError::IntrinsicIncorrectNumTArgs {
            name: kind.to_string(),
            expected: 0,
            span,
        }));
    }

    let return_type = engines.te().new_unknown();
    let lhs = &arguments[0];
    let lhs = ty::TyExpression::type_check(
        handler,
        ctx.by_ref()
            .with_help_text("Incorrect argument type")
            .with_type_annotation(return_type),
        lhs,
    )?;

    let rhs = &arguments[1];
    let rhs = ty::TyExpression::type_check(
        handler,
        ctx.by_ref()
            .with_help_text("Incorrect argument type")
            .with_type_annotation(engines.te().new_numeric()),
        rhs,
    )?;

    let t_arc = engines.te().get(lhs.return_type);
    let t = &*t_arc;
    match t {
        TypeInfo::B256 | TypeInfo::UnsignedInteger(_) | TypeInfo::Numeric => Ok((
            ty::TyIntrinsicFunctionKind {
                kind,
                arguments: vec![lhs, rhs],
                type_arguments: vec![],
                span,
            },
            return_type,
        )),
        _ => Err(handler.emit_err(CompileError::TypeError(
            sway_error::type_error::TypeError::MismatchedType {
                expected: "unsigned integer or b256".into(),
                received: engines.help_out(return_type).to_string(),
                help_text: "Incorrect argument type".into(),
                span: lhs.span,
            },
        ))),
    }
}

/// Signature: `__revert(code: u64)`
/// Description: Reverts with error code `code`.
/// Constraints: None.
fn type_check_revert(
    handler: &Handler,
    mut ctx: TypeCheckContext,
    kind: sway_ast::Intrinsic,
    arguments: &[Expression],
    type_arguments: &[GenericArgument],
    span: Span,
) -> Result<(ty::TyIntrinsicFunctionKind, TypeId), ErrorEmitted> {
    let type_engine = ctx.engines.te();

    if arguments.len() != 1 {
        return Err(handler.emit_err(CompileError::IntrinsicIncorrectNumArgs {
            name: kind.to_string(),
            expected: 1,
            span,
        }));
    }

    if !type_arguments.is_empty() {
        return Err(handler.emit_err(CompileError::IntrinsicIncorrectNumTArgs {
            name: kind.to_string(),
            expected: 0,
            span,
        }));
    }

    // Type check the argument which is the revert code
    let mut ctx = ctx.by_ref().with_type_annotation(type_engine.id_of_u64());
    let revert_code = ty::TyExpression::type_check(handler, ctx.by_ref(), &arguments[0])?;

    Ok((
        ty::TyIntrinsicFunctionKind {
            kind,
            arguments: vec![revert_code],
            type_arguments: vec![],
            span,
        },
        type_engine.id_of_never(),
    ))
}

/// Signature: `__jmp_mem() -> !`
/// Description: Jumps to `MEM[$hp]`.
fn type_check_jmp_mem(
    handler: &Handler,
    ctx: TypeCheckContext,
    kind: sway_ast::Intrinsic,
    arguments: &[Expression],
    type_arguments: &[GenericArgument],
    span: Span,
) -> Result<(ty::TyIntrinsicFunctionKind, TypeId), ErrorEmitted> {
    let type_engine = ctx.engines.te();

    if !arguments.is_empty() {
        return Err(handler.emit_err(CompileError::IntrinsicIncorrectNumArgs {
            name: kind.to_string(),
            expected: 0,
            span,
        }));
    }

    if !type_arguments.is_empty() {
        return Err(handler.emit_err(CompileError::IntrinsicIncorrectNumTArgs {
            name: kind.to_string(),
            expected: 0,
            span,
        }));
    }

    Ok((
        ty::TyIntrinsicFunctionKind {
            kind,
            arguments: vec![],
            type_arguments: vec![],
            span,
        },
        type_engine.id_of_never(),
    ))
}

/// Signature: `__ptr_add(ptr: raw_ptr, offset: u64)`
/// Description: Adds `offset` to the raw value of pointer `ptr`.
/// Constraints: None.
///
/// Signature: `__ptr_sub(ptr: raw_ptr, offset: u64)`
/// Description: Subtracts `offset` to the raw value of pointer `ptr`.
/// Constraints: None.
fn type_check_ptr_ops(
    handler: &Handler,
    mut ctx: TypeCheckContext,
    kind: sway_ast::Intrinsic,
    arguments: &[Expression],
    type_arguments: &[GenericArgument],
    span: Span,
) -> Result<(ty::TyIntrinsicFunctionKind, TypeId), ErrorEmitted> {
    let type_engine = ctx.engines.te();
    let engines = ctx.engines();

    if arguments.len() != 2 {
        return Err(handler.emit_err(CompileError::IntrinsicIncorrectNumArgs {
            name: kind.to_string(),
            expected: 2,
            span,
        }));
    }
    if type_arguments.len() != 1 {
        return Err(handler.emit_err(CompileError::IntrinsicIncorrectNumTArgs {
            name: kind.to_string(),
            expected: 1,
            span,
        }));
    }
    let targ = type_arguments[0].clone();
    let initial_type_info = type_engine
        .to_typeinfo(targ.type_id(), &targ.span())
        .map_err(|e| handler.emit_err(e.into()))
        .unwrap_or_else(TypeInfo::ErrorRecovery);
    let initial_type_id = type_engine.insert(engines, initial_type_info, targ.span().source_id());
    let type_id = ctx
        .resolve_type(
            handler,
            initial_type_id,
            &targ.span(),
            EnforceTypeArguments::No,
            None,
        )
        .unwrap_or_else(|err| type_engine.id_of_error_recovery(err));

    let mut ctx = ctx.by_ref().with_type_annotation(type_engine.new_unknown());

    let lhs = &arguments[0];
    let lhs = ty::TyExpression::type_check(handler, ctx.by_ref(), lhs)?;

    // Check for supported argument types
    let lhs_ty = type_engine
        .to_typeinfo(lhs.return_type, &lhs.span)
        .map_err(|e| handler.emit_err(e.into()))
        .unwrap_or_else(TypeInfo::ErrorRecovery);
    if !matches!(lhs_ty, TypeInfo::RawUntypedPtr) {
        return Err(handler.emit_err(CompileError::IntrinsicUnsupportedArgType {
            name: kind.to_string(),
            span: lhs.span,
            hint: "".to_string(),
        }));
    }

    let rhs = &arguments[1];
    let ctx = ctx
        .by_ref()
        .with_help_text("Incorrect argument type")
        .with_type_annotation(type_engine.id_of_u64());
    let rhs = ty::TyExpression::type_check(handler, ctx, rhs)?;

    Ok((
        ty::TyIntrinsicFunctionKind {
            kind,
            arguments: vec![lhs.clone(), rhs],
            type_arguments: vec![GenericArgument::Type(GenericTypeArgument {
                type_id,
                initial_type_id,
                span: targ.span(),
                call_path_tree: targ
                    .as_type_argument()
                    .unwrap()
                    .call_path_tree
                    .as_ref()
                    .cloned(),
            })],
            span,
        },
        type_engine.insert(engines, lhs_ty, lhs.span.source_id()),
    ))
}

/// Signature: `__smo<T>(recipient: b256, data: T, coins: u64)`
/// Description: Sends a message `data` of arbitrary type `T` and `coins` amount of the base asset
/// to address `recipient`.
/// Constraints: None.
fn type_check_smo(
    handler: &Handler,
    mut ctx: TypeCheckContext,
    kind: sway_ast::Intrinsic,
    arguments: &[Expression],
    type_arguments: &[GenericArgument],
    span: Span,
) -> Result<(ty::TyIntrinsicFunctionKind, TypeId), ErrorEmitted> {
    let type_engine = ctx.engines.te();
    let engines = ctx.engines();

    if arguments.len() != 3 {
        return Err(handler.emit_err(CompileError::IntrinsicIncorrectNumArgs {
            name: kind.to_string(),
            expected: 3,
            span,
        }));
    }

    if type_arguments.len() > 1 {
        return Err(handler.emit_err(CompileError::IntrinsicIncorrectNumTArgs {
            name: kind.to_string(),
            expected: 1,
            span,
        }));
    }

    // Type check the type argument
    let type_argument = type_arguments.first().map(|targ| {
        let ctx = ctx
            .by_ref()
            .with_help_text("")
            .with_type_annotation(type_engine.new_unknown());
        let initial_type_info = type_engine
            .to_typeinfo(targ.type_id(), &targ.span())
            .map_err(|e| handler.emit_err(e.into()))
            .unwrap_or_else(TypeInfo::ErrorRecovery);
        let initial_type_id =
            type_engine.insert(engines, initial_type_info, targ.span().source_id());
        let type_id = ctx
            .resolve_type(
                handler,
                initial_type_id,
                &targ.span(),
                EnforceTypeArguments::Yes,
                None,
            )
            .unwrap_or_else(|err| type_engine.id_of_error_recovery(err));
        GenericArgument::Type(GenericTypeArgument {
            type_id,
            initial_type_id,
            span: span.clone(),
            call_path_tree: targ
                .as_type_argument()
                .unwrap()
                .call_path_tree
                .as_ref()
                .cloned(),
        })
    });

    // Type check the first argument which is the recipient address, so it has to be a `b256`.
    let mut ctx = ctx.by_ref().with_type_annotation(type_engine.id_of_b256());
    let recipient = ty::TyExpression::type_check(handler, ctx.by_ref(), &arguments[0])?;

    // Type check the second argument which is the data, which can be anything. If a type
    // argument is provided, make sure that it matches the type of the data.
    let mut ctx = ctx.by_ref().with_type_annotation(
        type_argument
            .clone()
            .map_or(type_engine.new_unknown(), |ta| ta.type_id()),
    );
    let data = ty::TyExpression::type_check(handler, ctx.by_ref(), &arguments[1])?;

    // Type check the third argument which is the amount of coins to send, so it has to be a `u64`.
    let mut ctx = ctx.by_ref().with_type_annotation(type_engine.id_of_u64());
    let coins = ty::TyExpression::type_check(handler, ctx.by_ref(), &arguments[2])?;

    Ok((
        ty::TyIntrinsicFunctionKind {
            kind,
            arguments: vec![recipient, data, coins],
            type_arguments: type_argument.map_or(vec![], |ta| vec![ta]),
            span,
        },
        type_engine.id_of_unit(),
    ))
}

/// Signature: `__contract_ret(ptr: raw_ptr, len: u64) -> !`
/// Description: Returns from contract. The returned data is located at the memory location `ptr` and has
/// the length of `len` bytes.
/// Constraints: None.
fn type_check_contract_ret(
    handler: &Handler,
    mut ctx: TypeCheckContext,
    kind: sway_ast::Intrinsic,
    arguments: &[Expression],
    type_arguments: &[GenericArgument],
    span: Span,
) -> Result<(ty::TyIntrinsicFunctionKind, TypeId), ErrorEmitted> {
    let type_engine = ctx.engines.te();

    if arguments.len() != 2 {
        return Err(handler.emit_err(CompileError::IntrinsicIncorrectNumArgs {
            name: kind.to_string(),
            expected: 2,
            span,
        }));
    }

    if !type_arguments.is_empty() {
        return Err(handler.emit_err(CompileError::IntrinsicIncorrectNumTArgs {
            name: kind.to_string(),
            expected: 0,
            span,
        }));
    }

    let arguments: Vec<ty::TyExpression> = arguments
        .iter()
        .map(|x| {
            let ctx = ctx
                .by_ref()
                .with_help_text("")
                .with_type_annotation(type_engine.new_unknown());
            ty::TyExpression::type_check(handler, ctx, x)
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok((
        ty::TyIntrinsicFunctionKind {
            kind: Intrinsic::ContractRet,
            arguments,
            type_arguments: vec![],
            span,
        },
        ctx.engines.te().id_of_never(),
    ))
}

/// Signature: `__contract_call()`
/// Description: Calls another contract
/// Constraints: None.
fn type_check_contract_call(
    handler: &Handler,
    mut ctx: TypeCheckContext,
    kind: sway_ast::Intrinsic,
    arguments: &[Expression],
    type_arguments: &[GenericArgument],
    span: Span,
) -> Result<(ty::TyIntrinsicFunctionKind, TypeId), ErrorEmitted> {
    let type_engine = ctx.engines.te();

    if !type_arguments.is_empty() {
        return Err(handler.emit_err(CompileError::TypeArgumentsNotAllowed { span }));
    }

    // Arguments
    let arguments: Vec<ty::TyExpression> = arguments
        .iter()
        .map(|x| {
            let ctx = ctx
                .by_ref()
                .with_help_text("")
                .with_type_annotation(type_engine.new_unknown());
            ty::TyExpression::type_check(handler, ctx, x)
        })
        .collect::<Result<Vec<_>, _>>()?;

    let intrinsic_function = ty::TyIntrinsicFunctionKind {
        kind,
        arguments,
        type_arguments: vec![],
        span,
    };

    Ok((intrinsic_function, type_engine.id_of_unit()))
}
