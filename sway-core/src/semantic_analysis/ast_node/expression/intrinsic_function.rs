use sway_ast::intrinsics::Intrinsic;
use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
};
use sway_types::integer_bits::IntegerBits;
use sway_types::Span;

use crate::{
    engine_threading::*,
    language::{
        parsed::{Expression, ExpressionKind},
        ty, Literal,
    },
    semantic_analysis::{type_check_context::EnforceTypeArguments, TypeCheckContext},
    type_system::*,
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
                type_check_encode_buffer_empty(ctx, kind, arguments, type_arguments, span)
            }
            Intrinsic::EncodeBufferAppend => {
                type_check_encode_append(handler, ctx, kind, arguments, type_arguments, span)
            }
            Intrinsic::EncodeBufferAsRawSlice => {
                type_check_encode_as_raw_slice(handler, ctx, kind, arguments, type_arguments, span)
            }
        }
    }
}

fn type_check_encode_as_raw_slice(
    handler: &Handler,
    mut ctx: TypeCheckContext,
    kind: sway_ast::Intrinsic,
    arguments: &[Expression],
    _type_arguments: &[TypeArgument],
    span: Span,
) -> Result<(ty::TyIntrinsicFunctionKind, TypeId), ErrorEmitted> {
    let type_engine = ctx.engines.te();
    let engines = ctx.engines();

    let buffer_expr = {
        let ctx = ctx
            .by_ref()
            .with_help_text("")
            .with_type_annotation(type_engine.insert(engines, TypeInfo::Unknown, None));
        ty::TyExpression::type_check(handler, ctx, &arguments[0].clone())?
    };

    let return_type = type_engine.insert(engines, TypeInfo::RawUntypedSlice, None);

    let kind = ty::TyIntrinsicFunctionKind {
        kind,
        arguments: vec![buffer_expr],
        type_arguments: vec![],
        span,
    };
    Ok((kind, return_type))
}

// TODO: Rename to `new_tuple` and move to `TypeInfo` once https://github.com/FuelLabs/sway/issues/5991 is implemented.
fn new_encoding_buffer_tuple(
    engines: &Engines,
    items: impl IntoIterator<Item = TypeInfo>,
    span: Span,
) -> TypeInfo {
    let te = engines.te();
    let items = items
        .into_iter()
        .map(|x| te.insert(engines, x, None))
        .map(|type_id| TypeArgument {
            initial_type_id: type_id,
            type_id,
            span: span.clone(),
            call_path_tree: None,
        })
        .collect();
    TypeInfo::Tuple(items)
}

fn type_check_encode_buffer_empty(
    ctx: TypeCheckContext,
    kind: sway_ast::Intrinsic,
    arguments: &[Expression],
    _type_arguments: &[TypeArgument],
    span: Span,
) -> Result<(ty::TyIntrinsicFunctionKind, TypeId), ErrorEmitted> {
    assert!(arguments.is_empty());

    let type_engine = ctx.engines.te();
    let engines = ctx.engines();

    let return_type = new_encoding_buffer_tuple(
        engines,
        [
            TypeInfo::RawUntypedPtr,
            TypeInfo::UnsignedInteger(IntegerBits::SixtyFour),
            TypeInfo::UnsignedInteger(IntegerBits::SixtyFour),
        ],
        span.clone(),
    );
    let return_type = type_engine.insert(engines, return_type, None);

    let kind = ty::TyIntrinsicFunctionKind {
        kind,
        arguments: vec![],
        type_arguments: vec![],
        span,
    };
    Ok((kind, return_type))
}

fn encode_buffer_type(engines: &Engines) -> TypeInfo {
    let raw_ptr = engines.te().insert(engines, TypeInfo::RawUntypedPtr, None);
    let uint64 = engines.te().insert(
        engines,
        TypeInfo::UnsignedInteger(IntegerBits::SixtyFour),
        None,
    );
    TypeInfo::Tuple(vec![
        TypeArgument {
            type_id: raw_ptr,
            initial_type_id: raw_ptr,
            span: Span::dummy(),
            call_path_tree: None,
        },
        TypeArgument {
            type_id: uint64,
            initial_type_id: uint64,
            span: Span::dummy(),
            call_path_tree: None,
        },
        TypeArgument {
            type_id: uint64,
            initial_type_id: uint64,
            span: Span::dummy(),
            call_path_tree: None,
        },
    ])
}

fn type_check_encode_append(
    handler: &Handler,
    mut ctx: TypeCheckContext,
    kind: sway_ast::Intrinsic,
    arguments: &[Expression],
    _type_arguments: &[TypeArgument],
    span: Span,
) -> Result<(ty::TyIntrinsicFunctionKind, TypeId), ErrorEmitted> {
    let type_engine = ctx.engines.te();
    let engines = ctx.engines();

    let buffer_type = type_engine.insert(engines, encode_buffer_type(engines), None);
    let buffer_expr = {
        let ctx = ctx
            .by_ref()
            .with_help_text("")
            .with_type_annotation(buffer_type);
        ty::TyExpression::type_check(handler, ctx, &arguments[0])?
    };

    let item_span = arguments[1].span.clone();
    let item_type = type_engine.insert(engines, TypeInfo::Unknown, None);
    let item_expr = {
        let ctx = ctx
            .by_ref()
            .with_help_text("")
            .with_type_annotation(item_type);
        ty::TyExpression::type_check(handler, ctx, &arguments[1])?
    };

    // only supported types
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
            return Err(handler.emit_err(CompileError::EncodingUnsupportedType { span: item_span }))
        }
    };

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
    _type_arguments: &[TypeArgument],
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

    let return_type = type_engine.insert(engines, TypeInfo::Unknown, None);

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
                expected: "numeric or b256".into(),
                received: engines.help_out(return_type).to_string(),
                help_text: "".into(),
                span,
                internal: "8".into(),
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
    _type_arguments: &[TypeArgument],
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
        .with_type_annotation(type_engine.insert(engines, TypeInfo::Unknown, None));
    let exp = ty::TyExpression::type_check(handler, ctx, &arguments[0])?;
    let intrinsic_function = ty::TyIntrinsicFunctionKind {
        kind,
        arguments: vec![exp],
        type_arguments: vec![],
        span: span.clone(),
    };
    let return_type = type_engine.insert(
        engines,
        TypeInfo::UnsignedInteger(IntegerBits::SixtyFour),
        span.source_id(),
    );
    Ok((intrinsic_function, return_type))
}

/// Signature: `__size_of<T>() -> u64`
/// Description: Return the size of type `T` in bytes.
/// Constraints: None.
fn type_check_size_of_type(
    handler: &Handler,
    mut ctx: TypeCheckContext,
    kind: sway_ast::Intrinsic,
    arguments: &[Expression],
    type_arguments: &[TypeArgument],
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
        .to_typeinfo(targ.type_id, &targ.span)
        .map_err(|e| handler.emit_err(e.into()))
        .unwrap_or_else(TypeInfo::ErrorRecovery);
    let initial_type_id = type_engine.insert(engines, initial_type_info, targ.span.source_id());
    let type_id = ctx
        .resolve_type(
            handler,
            initial_type_id,
            &targ.span,
            EnforceTypeArguments::Yes,
            None,
        )
        .unwrap_or_else(|err| type_engine.insert(engines, TypeInfo::ErrorRecovery(err), None));
    let intrinsic_function = ty::TyIntrinsicFunctionKind {
        kind,
        arguments: vec![],
        type_arguments: vec![TypeArgument {
            type_id,
            initial_type_id,
            span: targ.span,
            call_path_tree: targ.call_path_tree,
        }],
        span,
    };
    let return_type = type_engine.insert(
        engines,
        TypeInfo::UnsignedInteger(IntegerBits::SixtyFour),
        None,
    );
    Ok((intrinsic_function, return_type))
}

/// Signature: `__is_reference_type<T>() -> bool`
/// Description: Returns `true` if `T` is a _reference type_ and `false` otherwise.
/// Constraints: None.
fn type_check_is_reference_type(
    handler: &Handler,
    mut ctx: TypeCheckContext,
    kind: sway_ast::Intrinsic,
    _arguments: &[Expression],
    type_arguments: &[TypeArgument],
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
        .to_typeinfo(targ.type_id, &targ.span)
        .map_err(|e| handler.emit_err(e.into()))
        .unwrap_or_else(TypeInfo::ErrorRecovery);
    let initial_type_id = type_engine.insert(engines, initial_type_info, targ.span.source_id());
    let type_id = ctx
        .resolve_type(
            handler,
            initial_type_id,
            &targ.span,
            EnforceTypeArguments::Yes,
            None,
        )
        .unwrap_or_else(|err| type_engine.insert(engines, TypeInfo::ErrorRecovery(err), None));
    let intrinsic_function = ty::TyIntrinsicFunctionKind {
        kind,
        arguments: vec![],
        type_arguments: vec![TypeArgument {
            type_id,
            initial_type_id,
            span: targ.span,
            call_path_tree: targ.call_path_tree,
        }],
        span,
    };
    Ok((
        intrinsic_function,
        type_engine.insert(engines, TypeInfo::Boolean, None),
    ))
}

/// Signature: `__assert_is_str_array<T>()`
/// Description: Throws a compile error if `T` is not of type str.
/// Constraints: None.
fn type_check_assert_is_str_array(
    handler: &Handler,
    mut ctx: TypeCheckContext,
    kind: sway_ast::Intrinsic,
    _arguments: &[Expression],
    type_arguments: &[TypeArgument],
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
        .to_typeinfo(targ.type_id, &targ.span)
        .map_err(|e| handler.emit_err(e.into()))
        .unwrap_or_else(TypeInfo::ErrorRecovery);
    let initial_type_id = type_engine.insert(engines, initial_type_info, targ.span.source_id());
    let type_id = ctx
        .resolve_type(
            handler,
            initial_type_id,
            &targ.span,
            EnforceTypeArguments::Yes,
            None,
        )
        .unwrap_or_else(|err| type_engine.insert(engines, TypeInfo::ErrorRecovery(err), None));
    let intrinsic_function = ty::TyIntrinsicFunctionKind {
        kind,
        arguments: vec![],
        type_arguments: vec![TypeArgument {
            type_id,
            initial_type_id,
            span: targ.span,
            call_path_tree: targ.call_path_tree,
        }],
        span,
    };
    Ok((
        intrinsic_function,
        type_engine.insert(engines, TypeInfo::Tuple(vec![]), None),
    ))
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
            let literal_length = s.as_str().len();
            let l = Length::new(literal_length, s.clone());
            let t = TypeInfo::StringArray(l);

            let span = arg.span.clone();

            let mut ctx = ctx.by_ref().with_type_annotation(type_engine.insert(
                engines,
                TypeInfo::Unknown,
                None,
            ));
            let new_type = ty::TyExpression::type_check(handler, ctx.by_ref(), arg)?;

            Ok((
                ty::TyIntrinsicFunctionKind {
                    kind,
                    arguments: vec![new_type],
                    type_arguments: vec![],
                    span,
                },
                type_engine.insert(engines, t, None),
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
    let engines = ctx.engines();

    if arguments.len() != 2 {
        return Err(handler.emit_err(CompileError::IntrinsicIncorrectNumArgs {
            name: kind.to_string(),
            expected: 2,
            span,
        }));
    }
    let mut ctx =
        ctx.by_ref()
            .with_type_annotation(type_engine.insert(engines, TypeInfo::Unknown, None));

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
        type_engine.insert(engines, TypeInfo::Boolean, None),
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
    type_arguments: &[TypeArgument],
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
    let mut ctx = ctx.by_ref().with_type_annotation(type_engine.insert(
        engines,
        TypeInfo::UnsignedInteger(IntegerBits::SixtyFour),
        None,
    ));
    let index = ty::TyExpression::type_check(handler, ctx.by_ref(), &arguments[0])?;

    // Type check the second argument which is the tx field ID
    let mut ctx = ctx.by_ref().with_type_annotation(type_engine.insert(
        engines,
        TypeInfo::UnsignedInteger(IntegerBits::SixtyFour),
        None,
    ));
    let tx_field_id = ty::TyExpression::type_check(handler, ctx.by_ref(), &arguments[1])?;

    let targ = type_arguments[0].clone();
    let initial_type_info = type_engine
        .to_typeinfo(targ.type_id, &targ.span)
        .map_err(|e| handler.emit_err(e.into()))
        .unwrap_or_else(TypeInfo::ErrorRecovery);
    let initial_type_id = type_engine.insert(engines, initial_type_info, targ.span.source_id());
    let type_id = ctx
        .resolve_type(
            handler,
            initial_type_id,
            &targ.span,
            EnforceTypeArguments::Yes,
            None,
        )
        .unwrap_or_else(|err| type_engine.insert(engines, TypeInfo::ErrorRecovery(err), None));

    Ok((
        ty::TyIntrinsicFunctionKind {
            kind,
            arguments: vec![index, tx_field_id],
            type_arguments: vec![TypeArgument {
                type_id,
                initial_type_id,
                span: targ.span,
                call_path_tree: targ.call_path_tree,
            }],
            span,
        },
        type_id,
    ))
}

/// Signature: `__addr_of<T>(val: T) -> raw_ptr`
/// Description: Returns the address in memory where `val` is stored.
/// Constraints: `T` is a reference type.
fn type_check_addr_of(
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
        .with_type_annotation(type_engine.insert(engines, TypeInfo::Unknown, None));
    let exp = ty::TyExpression::type_check(handler, ctx, &arguments[0])?;
    let copy_type_info = type_engine
        .to_typeinfo(exp.return_type, &span)
        .map_err(|e| handler.emit_err(e.into()))
        .unwrap_or_else(TypeInfo::ErrorRecovery);
    if copy_type_info.is_copy_type() {
        return Err(handler.emit_err(CompileError::IntrinsicUnsupportedArgType {
            name: kind.to_string(),
            span,
            hint: "Only a reference type can be used as argument here".to_string(),
        }));
    }

    let intrinsic_function = ty::TyIntrinsicFunctionKind {
        kind,
        arguments: vec![exp],
        type_arguments: vec![],
        span,
    };
    let return_type = type_engine.insert(engines, TypeInfo::RawUntypedPtr, None);
    Ok((intrinsic_function, return_type))
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
    let engines = ctx.engines();

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
        .with_type_annotation(type_engine.insert(engines, TypeInfo::Unknown, None));
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
    let mut ctx = ctx.with_type_annotation(type_engine.insert(
        engines,
        TypeInfo::UnsignedInteger(IntegerBits::SixtyFour),
        None,
    ));
    let number_of_slots_exp = ty::TyExpression::type_check(handler, ctx.by_ref(), &arguments[1])?;

    // Typed intrinsic
    let intrinsic_function = ty::TyIntrinsicFunctionKind {
        kind,
        arguments: vec![key_exp, number_of_slots_exp],
        type_arguments: vec![],
        span,
    };
    let return_type = type_engine.insert(engines, TypeInfo::Boolean, None);
    Ok((intrinsic_function, return_type))
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
        .with_type_annotation(type_engine.insert(engines, TypeInfo::Unknown, None));
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
    let return_type = type_engine.insert(
        engines,
        TypeInfo::UnsignedInteger(IntegerBits::SixtyFour),
        None,
    );
    Ok((intrinsic_function, return_type))
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
    type_arguments: &[TypeArgument],
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
        .with_type_annotation(type_engine.insert(engines, TypeInfo::Unknown, None));
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
    let mut ctx = ctx.with_type_annotation(type_engine.insert(engines, TypeInfo::Unknown, None));
    let val_exp = ty::TyExpression::type_check(handler, ctx.by_ref(), &arguments[1])?;
    let ctx = ctx.with_type_annotation(type_engine.insert(
        engines,
        TypeInfo::UnsignedInteger(IntegerBits::SixtyFour),
        None,
    ));
    let type_argument = type_arguments.first().map(|targ| {
        let mut ctx =
            ctx.with_type_annotation(type_engine.insert(engines, TypeInfo::Unknown, None));
        let initial_type_info = type_engine
            .to_typeinfo(targ.type_id, &targ.span)
            .map_err(|e| handler.emit_err(e.into()))
            .unwrap_or_else(TypeInfo::ErrorRecovery);
        let initial_type_id = type_engine.insert(engines, initial_type_info, targ.span.source_id());
        let type_id = ctx
            .resolve_type(
                handler,
                initial_type_id,
                &targ.span,
                EnforceTypeArguments::Yes,
                None,
            )
            .unwrap_or_else(|err| type_engine.insert(engines, TypeInfo::ErrorRecovery(err), None));
        TypeArgument {
            type_id,
            initial_type_id,
            span: span.clone(),
            call_path_tree: targ.call_path_tree.clone(),
        }
    });
    let intrinsic_function = ty::TyIntrinsicFunctionKind {
        kind,
        arguments: vec![key_exp, val_exp],
        type_arguments: type_argument.map_or(vec![], |ta| vec![ta]),
        span,
    };
    let return_type = type_engine.insert(engines, TypeInfo::Boolean, None);
    Ok((intrinsic_function, return_type))
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
    type_arguments: &[TypeArgument],
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
        .with_type_annotation(type_engine.insert(engines, TypeInfo::Unknown, None));
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
    let mut ctx = ctx.with_type_annotation(type_engine.insert(engines, TypeInfo::Unknown, None));
    let val_exp = ty::TyExpression::type_check(handler, ctx.by_ref(), &arguments[1])?;
    let mut ctx = ctx.with_type_annotation(type_engine.insert(
        engines,
        TypeInfo::UnsignedInteger(IntegerBits::SixtyFour),
        None,
    ));
    let number_of_slots_exp = ty::TyExpression::type_check(handler, ctx.by_ref(), &arguments[2])?;
    let type_argument = type_arguments.first().map(|targ| {
        let mut ctx =
            ctx.with_type_annotation(type_engine.insert(engines, TypeInfo::Unknown, None));
        let initial_type_info = type_engine
            .to_typeinfo(targ.type_id, &targ.span)
            .map_err(|e| handler.emit_err(e.into()))
            .unwrap_or_else(TypeInfo::ErrorRecovery);
        let initial_type_id = type_engine.insert(engines, initial_type_info, targ.span.source_id());
        let type_id = ctx
            .resolve_type(
                handler,
                initial_type_id,
                &targ.span,
                EnforceTypeArguments::Yes,
                None,
            )
            .unwrap_or_else(|err| type_engine.insert(engines, TypeInfo::ErrorRecovery(err), None));
        TypeArgument {
            type_id,
            initial_type_id,
            span: span.clone(),
            call_path_tree: targ.call_path_tree.clone(),
        }
    });
    let intrinsic_function = ty::TyIntrinsicFunctionKind {
        kind,
        arguments: vec![key_exp, val_exp, number_of_slots_exp],
        type_arguments: type_argument.map_or(vec![], |ta| vec![ta]),
        span,
    };
    let return_type = type_engine.insert(engines, TypeInfo::Boolean, None);
    Ok((intrinsic_function, return_type))
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
    let engines = ctx.engines();

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
        .with_type_annotation(type_engine.insert(engines, TypeInfo::Unknown, None));
    let exp = ty::TyExpression::type_check(handler, ctx, &arguments[0])?;
    let intrinsic_function = ty::TyIntrinsicFunctionKind {
        kind,
        arguments: vec![exp],
        type_arguments: vec![],
        span,
    };
    let return_type = type_engine.insert(engines, TypeInfo::Tuple(vec![]), None);
    Ok((intrinsic_function, return_type))
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
    type_arguments: &[TypeArgument],
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

    let return_type = type_engine.insert(engines, TypeInfo::Numeric, None);
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
    type_arguments: &[TypeArgument],
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

    let return_type = type_engine.insert(engines, TypeInfo::Unknown, None);
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
                expected: "numeric or b256".into(),
                received: engines.help_out(return_type).to_string(),
                help_text: "".into(),
                span,
                internal: "7".into(),
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
    type_arguments: &[TypeArgument],
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

    let return_type = engines.te().insert(engines, TypeInfo::Unknown, None);
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
            .with_type_annotation(engines.te().insert(engines, TypeInfo::Numeric, None)),
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
                expected: "numeric or b256".into(),
                received: engines.help_out(return_type).to_string(),
                help_text: "Incorrect argument type".into(),
                span: lhs.span,
                internal: "6".into(),
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
    type_arguments: &[TypeArgument],
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

    if !type_arguments.is_empty() {
        return Err(handler.emit_err(CompileError::IntrinsicIncorrectNumTArgs {
            name: kind.to_string(),
            expected: 0,
            span,
        }));
    }

    // Type check the argument which is the revert code
    let mut ctx = ctx.by_ref().with_type_annotation(type_engine.insert(
        engines,
        TypeInfo::UnsignedInteger(IntegerBits::SixtyFour),
        None,
    ));
    let revert_code = ty::TyExpression::type_check(handler, ctx.by_ref(), &arguments[0])?;

    Ok((
        ty::TyIntrinsicFunctionKind {
            kind,
            arguments: vec![revert_code],
            type_arguments: vec![],
            span,
        },
        type_engine.insert(engines, TypeInfo::Never, None),
    ))
}

/// Signature: `__jmp_mem() -> !`
/// Description: Jumps to `MEM[$hp]`.
fn type_check_jmp_mem(
    handler: &Handler,
    ctx: TypeCheckContext,
    kind: sway_ast::Intrinsic,
    arguments: &[Expression],
    type_arguments: &[TypeArgument],
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
        type_engine.insert(engines, TypeInfo::Never, None),
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
    type_arguments: &[TypeArgument],
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
        .to_typeinfo(targ.type_id, &targ.span)
        .map_err(|e| handler.emit_err(e.into()))
        .unwrap_or_else(TypeInfo::ErrorRecovery);
    let initial_type_id = type_engine.insert(engines, initial_type_info, targ.span.source_id());
    let type_id = ctx
        .resolve_type(
            handler,
            initial_type_id,
            &targ.span,
            EnforceTypeArguments::No,
            None,
        )
        .unwrap_or_else(|err| type_engine.insert(engines, TypeInfo::ErrorRecovery(err), None));

    let mut ctx =
        ctx.by_ref()
            .with_type_annotation(type_engine.insert(engines, TypeInfo::Unknown, None));

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
        .with_type_annotation(type_engine.insert(
            engines,
            TypeInfo::UnsignedInteger(IntegerBits::SixtyFour),
            None,
        ));
    let rhs = ty::TyExpression::type_check(handler, ctx, rhs)?;

    Ok((
        ty::TyIntrinsicFunctionKind {
            kind,
            arguments: vec![lhs.clone(), rhs],
            type_arguments: vec![TypeArgument {
                type_id,
                initial_type_id,
                span: targ.span,
                call_path_tree: targ.call_path_tree,
            }],
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
    type_arguments: &[TypeArgument],
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
        let mut ctx = ctx
            .by_ref()
            .with_help_text("")
            .with_type_annotation(type_engine.insert(engines, TypeInfo::Unknown, None));
        let initial_type_info = type_engine
            .to_typeinfo(targ.type_id, &targ.span)
            .map_err(|e| handler.emit_err(e.into()))
            .unwrap_or_else(TypeInfo::ErrorRecovery);
        let initial_type_id = type_engine.insert(engines, initial_type_info, targ.span.source_id());
        let type_id = ctx
            .resolve_type(
                handler,
                initial_type_id,
                &targ.span,
                EnforceTypeArguments::Yes,
                None,
            )
            .unwrap_or_else(|err| type_engine.insert(engines, TypeInfo::ErrorRecovery(err), None));
        TypeArgument {
            type_id,
            initial_type_id,
            span: span.clone(),
            call_path_tree: targ.call_path_tree.clone(),
        }
    });

    // Type check the first argument which is the recipient address, so it has to be a `b256`.
    let mut ctx =
        ctx.by_ref()
            .with_type_annotation(type_engine.insert(engines, TypeInfo::B256, None));
    let recipient = ty::TyExpression::type_check(handler, ctx.by_ref(), &arguments[0])?;

    // Type check the second argument which is the data, which can be anything. If a type
    // argument is provided, make sure that it matches the type of the data.
    let mut ctx = ctx.by_ref().with_type_annotation(
        type_argument
            .clone()
            .map_or(type_engine.insert(engines, TypeInfo::Unknown, None), |ta| {
                ta.type_id
            }),
    );
    let data = ty::TyExpression::type_check(handler, ctx.by_ref(), &arguments[1])?;

    // Type check the third argument which is the output index, so it has to be a `u64`.
    let mut ctx = ctx.by_ref().with_type_annotation(type_engine.insert(
        engines,
        TypeInfo::UnsignedInteger(IntegerBits::SixtyFour),
        None,
    ));

    // Type check the fourth argument which is the amount of coins to send, so it has to be a `u64`.
    let mut ctx = ctx.by_ref().with_type_annotation(type_engine.insert(
        engines,
        TypeInfo::UnsignedInteger(IntegerBits::SixtyFour),
        None,
    ));
    let coins = ty::TyExpression::type_check(handler, ctx.by_ref(), &arguments[2])?;

    Ok((
        ty::TyIntrinsicFunctionKind {
            kind,
            arguments: vec![recipient, data, coins],
            type_arguments: type_argument.map_or(vec![], |ta| vec![ta]),
            span,
        },
        type_engine.insert(engines, TypeInfo::Tuple(vec![]), None),
    ))
}

/// Signature: `__contract_call<T>()`
/// Description: Calls another contract
/// Constraints: None.
fn type_check_contract_ret(
    handler: &Handler,
    mut ctx: TypeCheckContext,
    _kind: sway_ast::Intrinsic,
    arguments: &[Expression],
    _type_arguments: &[TypeArgument],
    _span: Span,
) -> Result<(ty::TyIntrinsicFunctionKind, TypeId), ErrorEmitted> {
    let type_engine = ctx.engines.te();
    let engines = ctx.engines();

    let arguments: Vec<ty::TyExpression> = arguments
        .iter()
        .map(|x| {
            let ctx = ctx
                .by_ref()
                .with_help_text("")
                .with_type_annotation(type_engine.insert(engines, TypeInfo::Unknown, None));
            ty::TyExpression::type_check(handler, ctx, x)
        })
        .collect::<Result<Vec<_>, _>>()?;

    let t = ctx
        .engines
        .te()
        .insert(ctx.engines, TypeInfo::Tuple(vec![]), None);

    Ok((
        ty::TyIntrinsicFunctionKind {
            kind: Intrinsic::ContractRet,
            arguments,
            type_arguments: vec![],
            span: Span::dummy(),
        },
        t,
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
    type_arguments: &[TypeArgument],
    span: Span,
) -> Result<(ty::TyIntrinsicFunctionKind, TypeId), ErrorEmitted> {
    let type_engine = ctx.engines.te();
    let engines = ctx.engines();

    if !type_arguments.is_empty() {
        return Err(handler.emit_err(CompileError::TypeArgumentsNotAllowed { span }));
    }

    let return_type_id = ctx
        .engines
        .te()
        .insert(ctx.engines, TypeInfo::Tuple(vec![]), None);

    // Arguments
    let arguments: Vec<ty::TyExpression> = arguments
        .iter()
        .map(|x| {
            let ctx = ctx
                .by_ref()
                .with_help_text("")
                .with_type_annotation(type_engine.insert(engines, TypeInfo::Unknown, None));
            ty::TyExpression::type_check(handler, ctx, x)
        })
        .collect::<Result<Vec<_>, _>>()?;

    let intrinsic_function = ty::TyIntrinsicFunctionKind {
        kind,
        arguments,
        type_arguments: vec![],
        span,
    };

    Ok((intrinsic_function, return_type_id))
}
