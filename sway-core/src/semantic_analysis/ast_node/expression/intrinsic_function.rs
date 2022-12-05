use sway_ast::intrinsics::Intrinsic;
use sway_error::error::{CompileError, Hint};
use sway_types::integer_bits::IntegerBits;
use sway_types::Span;

use crate::{
    engine_threading::*,
    error::{err, ok},
    language::{parsed::Expression, ty},
    semantic_analysis::TypeCheckContext,
    type_system::*,
    CompileResult,
};

impl ty::TyIntrinsicFunctionKind {
    pub(crate) fn type_check(
        ctx: TypeCheckContext,
        kind_binding: TypeBinding<Intrinsic>,
        arguments: Vec<Expression>,
        span: Span,
    ) -> CompileResult<(Self, TypeId)> {
        let TypeBinding {
            inner: kind,
            type_arguments,
            ..
        } = kind_binding;
        match kind {
            Intrinsic::SizeOfVal => {
                type_check_size_of_val(ctx, kind, arguments, type_arguments, span)
            }
            Intrinsic::SizeOfType => {
                type_check_size_of_type(ctx, kind, arguments, type_arguments, span)
            }
            Intrinsic::IsReferenceType => {
                type_check_is_reference_type(ctx, kind, arguments, type_arguments, span)
            }
            Intrinsic::GetStorageKey => {
                type_check_get_storage_key(ctx, kind, arguments, type_arguments, span)
            }
            Intrinsic::Eq => type_check_eq(ctx, kind, arguments, span),
            Intrinsic::Gtf => type_check_gtf(ctx, kind, arguments, type_arguments, span),
            Intrinsic::AddrOf => type_check_addr_of(ctx, kind, arguments, span),
            Intrinsic::StateLoadWord => type_check_state_load_word(ctx, kind, arguments, span),
            Intrinsic::StateStoreWord | Intrinsic::StateLoadQuad | Intrinsic::StateStoreQuad => {
                type_check_state_store_or_quad(ctx, kind, arguments, type_arguments, span)
            }
            Intrinsic::Log => type_check_log(ctx, kind, arguments, span),
            Intrinsic::Add | Intrinsic::Sub | Intrinsic::Mul | Intrinsic::Div => {
                type_check_binary_op(ctx, kind, arguments, type_arguments, span)
            }
            Intrinsic::Revert => type_check_revert(ctx, kind, arguments, type_arguments, span),
            Intrinsic::PtrAdd | Intrinsic::PtrSub => {
                type_check_ptr_ops(ctx, kind, arguments, type_arguments, span)
            }
        }
    }
}

/// Signature: `__size_of_val<T>(val: T) -> u64`
/// Description: Return the size of type `T` in bytes.
/// Constraints: None.
fn type_check_size_of_val(
    ctx: TypeCheckContext,
    kind: sway_ast::Intrinsic,
    arguments: Vec<Expression>,
    _type_arguments: Vec<TypeArgument>,
    span: Span,
) -> CompileResult<(ty::TyIntrinsicFunctionKind, TypeId)> {
    let type_engine = ctx.type_engine;

    let mut warnings = vec![];
    let mut errors = vec![];

    if arguments.len() != 1 {
        errors.push(CompileError::IntrinsicIncorrectNumArgs {
            name: kind.to_string(),
            expected: 1,
            span,
        });
        return err(warnings, errors);
    }
    let ctx = ctx
        .with_help_text("")
        .with_type_annotation(type_engine.insert_type(TypeInfo::Unknown));
    let exp = check!(
        ty::TyExpression::type_check(ctx, arguments[0].clone()),
        return err(warnings, errors),
        warnings,
        errors
    );
    let intrinsic_function = ty::TyIntrinsicFunctionKind {
        kind,
        arguments: vec![exp],
        type_arguments: vec![],
        span,
    };
    let return_type = type_engine.insert_type(TypeInfo::UnsignedInteger(IntegerBits::SixtyFour));
    ok((intrinsic_function, return_type), warnings, errors)
}

/// Signature: `__size_of<T>() -> u64`
/// Description: Return the size of type `T` in bytes.
/// Constraints: None.
fn type_check_size_of_type(
    mut ctx: TypeCheckContext,
    kind: sway_ast::Intrinsic,
    arguments: Vec<Expression>,
    type_arguments: Vec<TypeArgument>,
    span: Span,
) -> CompileResult<(ty::TyIntrinsicFunctionKind, TypeId)> {
    let type_engine = ctx.type_engine;

    let mut warnings = vec![];
    let mut errors = vec![];

    if !arguments.is_empty() {
        errors.push(CompileError::IntrinsicIncorrectNumArgs {
            name: kind.to_string(),
            expected: 0,
            span,
        });
        return err(warnings, errors);
    }
    if type_arguments.len() != 1 {
        errors.push(CompileError::IntrinsicIncorrectNumTArgs {
            name: kind.to_string(),
            expected: 1,
            span,
        });
        return err(warnings, errors);
    }
    let targ = type_arguments[0].clone();
    let initial_type_info = check!(
        CompileResult::from(
            type_engine
                .to_typeinfo(targ.type_id, &targ.span)
                .map_err(CompileError::from)
        ),
        TypeInfo::ErrorRecovery,
        warnings,
        errors
    );
    let initial_type_id = type_engine.insert_type(initial_type_info);
    let type_id = check!(
        ctx.resolve_type_with_self(initial_type_id, &targ.span, EnforceTypeArguments::Yes, None),
        type_engine.insert_type(TypeInfo::ErrorRecovery),
        warnings,
        errors,
    );
    let intrinsic_function = ty::TyIntrinsicFunctionKind {
        kind,
        arguments: vec![],
        type_arguments: vec![TypeArgument {
            type_id,
            initial_type_id,
            span: targ.span,
        }],
        span,
    };
    let return_type = type_engine.insert_type(TypeInfo::UnsignedInteger(IntegerBits::SixtyFour));
    ok((intrinsic_function, return_type), warnings, errors)
}

/// Signature: `__is_reference_type<T>() -> bool`
/// Description: Returns `true` if `T` is a _reference type_ and `false` otherwise.
/// Constraints: None.
fn type_check_is_reference_type(
    mut ctx: TypeCheckContext,
    kind: sway_ast::Intrinsic,
    _arguments: Vec<Expression>,
    type_arguments: Vec<TypeArgument>,
    span: Span,
) -> CompileResult<(ty::TyIntrinsicFunctionKind, TypeId)> {
    let type_engine = ctx.type_engine;

    let mut warnings = vec![];
    let mut errors = vec![];

    if type_arguments.len() != 1 {
        errors.push(CompileError::IntrinsicIncorrectNumTArgs {
            name: kind.to_string(),
            expected: 1,
            span,
        });
        return err(warnings, errors);
    }
    let targ = type_arguments[0].clone();
    let initial_type_info = check!(
        CompileResult::from(
            type_engine
                .to_typeinfo(targ.type_id, &targ.span)
                .map_err(CompileError::from)
        ),
        TypeInfo::ErrorRecovery,
        warnings,
        errors
    );
    let initial_type_id = type_engine.insert_type(initial_type_info);
    let type_id = check!(
        ctx.resolve_type_with_self(initial_type_id, &targ.span, EnforceTypeArguments::Yes, None),
        type_engine.insert_type(TypeInfo::ErrorRecovery),
        warnings,
        errors,
    );
    let intrinsic_function = ty::TyIntrinsicFunctionKind {
        kind,
        arguments: vec![],
        type_arguments: vec![TypeArgument {
            type_id,
            initial_type_id,
            span: targ.span,
        }],
        span,
    };
    ok(
        (
            intrinsic_function,
            type_engine.insert_type(TypeInfo::Boolean),
        ),
        warnings,
        errors,
    )
}

/// Signature: `__get_storage_key() -> b256`
/// Description: Returns the storage key used by a given `struct` in storage when called from a
///              method on that `struct`.
/// Constraints: None.
fn type_check_get_storage_key(
    ctx: TypeCheckContext,
    kind: sway_ast::Intrinsic,
    _arguments: Vec<Expression>,
    _type_arguments: Vec<TypeArgument>,
    span: Span,
) -> CompileResult<(ty::TyIntrinsicFunctionKind, TypeId)> {
    let type_engine = ctx.type_engine;

    ok(
        (
            ty::TyIntrinsicFunctionKind {
                kind,
                arguments: vec![],
                type_arguments: vec![],
                span,
            },
            type_engine.insert_type(TypeInfo::B256),
        ),
        vec![],
        vec![],
    )
}

/// Signature: `__eq<T>(lhs: T, rhs: T) -> bool`
/// Description: Returns whether `lhs` and `rhs` are equal.
/// Constraints: `T` is `bool`, `u8`, `u16`, `u32`, `u64`, or `raw_ptr`.
fn type_check_eq(
    mut ctx: TypeCheckContext,
    kind: sway_ast::Intrinsic,
    arguments: Vec<Expression>,
    span: Span,
) -> CompileResult<(ty::TyIntrinsicFunctionKind, TypeId)> {
    let type_engine = ctx.type_engine;

    let mut warnings = vec![];
    let mut errors = vec![];
    if arguments.len() != 2 {
        errors.push(CompileError::IntrinsicIncorrectNumArgs {
            name: kind.to_string(),
            expected: 2,
            span,
        });
        return err(warnings, errors);
    }
    let mut ctx = ctx
        .by_ref()
        .with_type_annotation(type_engine.insert_type(TypeInfo::Unknown));

    let lhs = arguments[0].clone();
    let lhs = check!(
        ty::TyExpression::type_check(ctx.by_ref(), lhs),
        return err(warnings, errors),
        warnings,
        errors
    );

    // Check for supported argument types
    let arg_ty = check!(
        CompileResult::from(
            type_engine
                .to_typeinfo(lhs.return_type, &lhs.span)
                .map_err(CompileError::from)
        ),
        TypeInfo::ErrorRecovery,
        warnings,
        errors
    );
    let is_valid_arg_ty = matches!(
        arg_ty,
        TypeInfo::UnsignedInteger(_) | TypeInfo::Boolean | TypeInfo::RawUntypedPtr
    );
    if !is_valid_arg_ty {
        errors.push(CompileError::IntrinsicUnsupportedArgType {
            name: kind.to_string(),
            span: lhs.span,
            hint: Hint::empty(),
        });
        return err(warnings, errors);
    }

    let rhs = arguments[1].clone();
    let ctx = ctx
        .by_ref()
        .with_help_text("Incorrect argument type")
        .with_type_annotation(lhs.return_type);
    let rhs = check!(
        ty::TyExpression::type_check(ctx, rhs),
        return err(warnings, errors),
        warnings,
        errors
    );
    ok(
        (
            ty::TyIntrinsicFunctionKind {
                kind,
                arguments: vec![lhs, rhs],
                type_arguments: vec![],
                span,
            },
            type_engine.insert_type(TypeInfo::Boolean),
        ),
        warnings,
        errors,
    )
}

/// Signature: `__gtf<T>(index: u64, tx_field_id: u64) -> T`
/// Description: Returns transaction field with ID `tx_field_id` at index `index`, if applicable.
///              This is a wrapper around FuelVM's `gtf` instruction:
///              https://fuellabs.github.io/fuel-specs/master/vm/instruction_set#gtf-get-transaction-fields.
///              The resuting field is cast to `T`.
/// Constraints: None.
fn type_check_gtf(
    mut ctx: TypeCheckContext,
    kind: sway_ast::Intrinsic,
    arguments: Vec<Expression>,
    type_arguments: Vec<TypeArgument>,
    span: Span,
) -> CompileResult<(ty::TyIntrinsicFunctionKind, TypeId)> {
    let type_engine = ctx.type_engine;

    let mut warnings = vec![];
    let mut errors = vec![];

    if arguments.len() != 2 {
        errors.push(CompileError::IntrinsicIncorrectNumArgs {
            name: kind.to_string(),
            expected: 2,
            span,
        });
        return err(warnings, errors);
    }

    if type_arguments.len() != 1 {
        errors.push(CompileError::IntrinsicIncorrectNumTArgs {
            name: kind.to_string(),
            expected: 1,
            span,
        });
        return err(warnings, errors);
    }

    // Type check the first argument which is the index
    let mut ctx = ctx
        .by_ref()
        .with_type_annotation(type_engine.insert_type(TypeInfo::Unknown));
    let index = check!(
        ty::TyExpression::type_check(ctx.by_ref(), arguments[0].clone()),
        return err(warnings, errors),
        warnings,
        errors
    );

    // Type check the second argument which is the tx field ID
    let mut ctx = ctx
        .by_ref()
        .with_type_annotation(type_engine.insert_type(TypeInfo::Unknown));
    let tx_field_id = check!(
        ty::TyExpression::type_check(ctx.by_ref(), arguments[1].clone()),
        return err(warnings, errors),
        warnings,
        errors
    );

    // Make sure that the index argument is a `u64`
    let index_type_info = check!(
        CompileResult::from(
            type_engine
                .to_typeinfo(index.return_type, &index.span)
                .map_err(CompileError::from)
        ),
        TypeInfo::ErrorRecovery,
        warnings,
        errors
    );
    if !matches!(
        index_type_info,
        TypeInfo::UnsignedInteger(IntegerBits::SixtyFour)
    ) {
        errors.push(CompileError::IntrinsicUnsupportedArgType {
            name: kind.to_string(),
            span: index.span.clone(),
            hint: Hint::empty(),
        });
    }

    // Make sure that the tx field ID is a `u64`
    let tx_field_type_info = check!(
        CompileResult::from(
            type_engine
                .to_typeinfo(tx_field_id.return_type, &tx_field_id.span)
                .map_err(CompileError::from)
        ),
        TypeInfo::ErrorRecovery,
        warnings,
        errors
    );
    if !matches!(
        tx_field_type_info,
        TypeInfo::UnsignedInteger(IntegerBits::SixtyFour)
    ) {
        errors.push(CompileError::IntrinsicUnsupportedArgType {
            name: kind.to_string(),
            span: tx_field_id.span.clone(),
            hint: Hint::empty(),
        });
    }

    let targ = type_arguments[0].clone();
    let initial_type_info = check!(
        CompileResult::from(
            type_engine
                .to_typeinfo(targ.type_id, &targ.span)
                .map_err(CompileError::from)
        ),
        TypeInfo::ErrorRecovery,
        warnings,
        errors
    );
    let initial_type_id = type_engine.insert_type(initial_type_info);
    let type_id = check!(
        ctx.resolve_type_with_self(initial_type_id, &targ.span, EnforceTypeArguments::Yes, None),
        type_engine.insert_type(TypeInfo::ErrorRecovery),
        warnings,
        errors,
    );

    ok(
        (
            ty::TyIntrinsicFunctionKind {
                kind,
                arguments: vec![index, tx_field_id],
                type_arguments: vec![TypeArgument {
                    type_id,
                    initial_type_id,
                    span: targ.span,
                }],
                span,
            },
            type_id,
        ),
        warnings,
        errors,
    )
}

/// Signature: `__addr_of<T>(val: T) -> raw_ptr`
/// Description: Returns the address in memory where `val` is stored.
/// Constraints: `T` is a reference type.
fn type_check_addr_of(
    ctx: TypeCheckContext,
    kind: sway_ast::Intrinsic,
    arguments: Vec<Expression>,
    span: Span,
) -> CompileResult<(ty::TyIntrinsicFunctionKind, TypeId)> {
    let type_engine = ctx.type_engine;

    let mut warnings = vec![];
    let mut errors = vec![];

    if arguments.len() != 1 {
        errors.push(CompileError::IntrinsicIncorrectNumArgs {
            name: kind.to_string(),
            expected: 1,
            span,
        });
        return err(warnings, errors);
    }
    let ctx = ctx
        .with_help_text("")
        .with_type_annotation(type_engine.insert_type(TypeInfo::Unknown));
    let exp = check!(
        ty::TyExpression::type_check(ctx, arguments[0].clone()),
        return err(warnings, errors),
        warnings,
        errors
    );
    let copy_type_info = check!(
        CompileResult::from(
            type_engine
                .to_typeinfo(exp.return_type, &span)
                .map_err(CompileError::from)
        ),
        TypeInfo::ErrorRecovery,
        warnings,
        errors
    );
    if copy_type_info.is_copy_type() {
        errors.push(CompileError::IntrinsicUnsupportedArgType {
            name: kind.to_string(),
            span,
            hint: Hint::new("Only a reference type can be used as argument here".to_string()),
        });
        return err(warnings, errors);
    }

    let intrinsic_function = ty::TyIntrinsicFunctionKind {
        kind,
        arguments: vec![exp],
        type_arguments: vec![],
        span,
    };
    let return_type = type_engine.insert_type(TypeInfo::RawUntypedPtr);
    ok((intrinsic_function, return_type), warnings, errors)
}

/// Signature: `__state_load_word(key: b256) -> u64`
/// Description: Reads and returns a single word from storage at key `key`.
/// Constraints: None.
fn type_check_state_load_word(
    ctx: TypeCheckContext,
    kind: sway_ast::Intrinsic,
    arguments: Vec<Expression>,
    span: Span,
) -> CompileResult<(ty::TyIntrinsicFunctionKind, TypeId)> {
    let type_engine = ctx.type_engine;
    let engines = ctx.engines();

    let mut warnings = vec![];
    let mut errors = vec![];
    if arguments.len() != 1 {
        errors.push(CompileError::IntrinsicIncorrectNumArgs {
            name: kind.to_string(),
            expected: 1,
            span,
        });
        return err(warnings, errors);
    }
    let ctx = ctx
        .with_help_text("")
        .with_type_annotation(type_engine.insert_type(TypeInfo::Unknown));
    let exp = check!(
        ty::TyExpression::type_check(ctx, arguments[0].clone()),
        return err(warnings, errors),
        warnings,
        errors
    );
    let key_ty = check!(
        CompileResult::from(
            type_engine
                .to_typeinfo(exp.return_type, &span)
                .map_err(CompileError::from)
        ),
        TypeInfo::ErrorRecovery,
        warnings,
        errors
    );
    if !key_ty.eq(&TypeInfo::B256, engines) {
        errors.push(CompileError::IntrinsicUnsupportedArgType {
            name: kind.to_string(),
            span,
            hint: Hint::new("Argument type must be B256, a key into the state storage".to_string()),
        });
        return err(warnings, errors);
    }
    let intrinsic_function = ty::TyIntrinsicFunctionKind {
        kind,
        arguments: vec![exp],
        type_arguments: vec![],
        span,
    };
    let return_type = type_engine.insert_type(TypeInfo::UnsignedInteger(IntegerBits::SixtyFour));
    ok((intrinsic_function, return_type), warnings, errors)
}

/// Signature: `__state_load_quad(key: b256, ptr: raw_ptr)`
/// Description: Reads a `b256` from storage at key `key` and stores it in memory at address
///              `raw_ptr`
/// Constraints: None.
///
/// Signature: `__state_store_word(key: b256, val: u64)`
/// Description: Stores a single word `val` into storage at key `key`.
/// Constraints: None.
///
/// Signature: `__state_store_quad(key: b256, ptr: raw_ptr)`
/// Description: Stores a `b256` from address `ptr` in memory into storage at key `key`.
/// Constraints: None.
fn type_check_state_store_or_quad(
    ctx: TypeCheckContext,
    kind: sway_ast::Intrinsic,
    arguments: Vec<Expression>,
    type_arguments: Vec<TypeArgument>,
    span: Span,
) -> CompileResult<(ty::TyIntrinsicFunctionKind, TypeId)> {
    let type_engine = ctx.type_engine;

    let mut warnings = vec![];
    let mut errors = vec![];
    if arguments.len() != 2 {
        errors.push(CompileError::IntrinsicIncorrectNumArgs {
            name: kind.to_string(),
            expected: 2,
            span,
        });
        return err(warnings, errors);
    }
    if type_arguments.len() > 1 {
        errors.push(CompileError::IntrinsicIncorrectNumTArgs {
            name: kind.to_string(),
            expected: 1,
            span,
        });
        return err(warnings, errors);
    }
    let mut ctx = ctx
        .with_help_text("")
        .with_type_annotation(type_engine.insert_type(TypeInfo::Unknown));
    let key_exp = check!(
        ty::TyExpression::type_check(ctx.by_ref(), arguments[0].clone()),
        return err(warnings, errors),
        warnings,
        errors
    );
    let key_ty = check!(
        CompileResult::from(
            type_engine
                .to_typeinfo(key_exp.return_type, &span)
                .map_err(CompileError::from)
        ),
        TypeInfo::ErrorRecovery,
        warnings,
        errors
    );
    if !key_ty.eq(&TypeInfo::B256, ctx.engines()) {
        errors.push(CompileError::IntrinsicUnsupportedArgType {
            name: kind.to_string(),
            span,
            hint: Hint::new("Argument type must be B256, a key into the state storage".to_string()),
        });
        return err(warnings, errors);
    }
    let mut ctx = ctx
        .with_help_text("")
        .with_type_annotation(type_engine.insert_type(TypeInfo::Unknown));
    let val_exp = check!(
        ty::TyExpression::type_check(ctx.by_ref(), arguments[1].clone()),
        return err(warnings, errors),
        warnings,
        errors
    );
    let type_argument = type_arguments.get(0).map(|targ| {
        let mut ctx = ctx
            .with_help_text("")
            .with_type_annotation(type_engine.insert_type(TypeInfo::Unknown));
        let initial_type_info = check!(
            CompileResult::from(
                type_engine
                    .to_typeinfo(targ.type_id, &targ.span)
                    .map_err(CompileError::from)
            ),
            TypeInfo::ErrorRecovery,
            warnings,
            errors
        );
        let initial_type_id = type_engine.insert_type(initial_type_info);
        let type_id = check!(
            ctx.resolve_type_with_self(
                initial_type_id,
                &targ.span,
                EnforceTypeArguments::Yes,
                None
            ),
            type_engine.insert_type(TypeInfo::ErrorRecovery),
            warnings,
            errors,
        );
        TypeArgument {
            type_id,
            initial_type_id,
            span: span.clone(),
        }
    });
    let intrinsic_function = ty::TyIntrinsicFunctionKind {
        kind,
        arguments: vec![key_exp, val_exp],
        type_arguments: type_argument.map_or(vec![], |ta| vec![ta]),
        span,
    };
    let return_type = type_engine.insert_type(TypeInfo::Tuple(vec![]));
    ok((intrinsic_function, return_type), warnings, errors)
}

/// Signature: `__log<T>(val: T)`
/// Description: Logs value `val`.
/// Constraints: None.
fn type_check_log(
    mut ctx: TypeCheckContext,
    kind: sway_ast::Intrinsic,
    arguments: Vec<Expression>,
    span: Span,
) -> CompileResult<(ty::TyIntrinsicFunctionKind, TypeId)> {
    let type_engine = ctx.type_engine;

    let mut warnings = vec![];
    let mut errors = vec![];

    if arguments.len() != 1 {
        errors.push(CompileError::IntrinsicIncorrectNumArgs {
            name: kind.to_string(),
            expected: 1,
            span,
        });
        return err(warnings, errors);
    }
    let ctx = ctx
        .by_ref()
        .with_help_text("")
        .with_type_annotation(type_engine.insert_type(TypeInfo::Unknown));
    let exp = check!(
        ty::TyExpression::type_check(ctx, arguments[0].clone()),
        return err(warnings, errors),
        warnings,
        errors
    );
    let intrinsic_function = ty::TyIntrinsicFunctionKind {
        kind,
        arguments: vec![exp],
        type_arguments: vec![],
        span,
    };
    let return_type = type_engine.insert_type(TypeInfo::Tuple(vec![]));
    ok((intrinsic_function, return_type), warnings, errors)
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
fn type_check_binary_op(
    mut ctx: TypeCheckContext,
    kind: sway_ast::Intrinsic,
    arguments: Vec<Expression>,
    type_arguments: Vec<TypeArgument>,
    span: Span,
) -> CompileResult<(ty::TyIntrinsicFunctionKind, TypeId)> {
    let type_engine = ctx.type_engine;

    let mut warnings = vec![];
    let mut errors = vec![];

    if arguments.len() != 2 {
        errors.push(CompileError::IntrinsicIncorrectNumArgs {
            name: kind.to_string(),
            expected: 2,
            span,
        });
        return err(warnings, errors);
    }
    if !type_arguments.is_empty() {
        errors.push(CompileError::IntrinsicIncorrectNumTArgs {
            name: kind.to_string(),
            expected: 0,
            span,
        });
        return err(warnings, errors);
    }

    let mut ctx = ctx
        .by_ref()
        .with_type_annotation(type_engine.insert_type(TypeInfo::Unknown));

    let lhs = arguments[0].clone();
    let lhs = check!(
        ty::TyExpression::type_check(ctx.by_ref(), lhs),
        return err(warnings, errors),
        warnings,
        errors
    );

    // Check for supported argument types
    let arg_ty = check!(
        CompileResult::from(
            type_engine
                .to_typeinfo(lhs.return_type, &lhs.span)
                .map_err(CompileError::from)
        ),
        TypeInfo::ErrorRecovery,
        warnings,
        errors
    );
    let is_valid_arg_ty = matches!(arg_ty, TypeInfo::UnsignedInteger(_));
    if !is_valid_arg_ty {
        errors.push(CompileError::IntrinsicUnsupportedArgType {
            name: kind.to_string(),
            span: lhs.span,
            hint: Hint::empty(),
        });
        return err(warnings, errors);
    }

    let rhs = arguments[1].clone();
    let ctx = ctx
        .by_ref()
        .with_help_text("Incorrect argument type")
        .with_type_annotation(lhs.return_type);
    let rhs = check!(
        ty::TyExpression::type_check(ctx, rhs),
        return err(warnings, errors),
        warnings,
        errors
    );
    ok(
        (
            ty::TyIntrinsicFunctionKind {
                kind,
                arguments: vec![lhs, rhs],
                type_arguments: vec![],
                span,
            },
            type_engine.insert_type(arg_ty),
        ),
        warnings,
        errors,
    )
}

/// Signature: `__revert(code: u64)`
/// Description: Reverts with error code `code`.
/// Constraints: None.
fn type_check_revert(
    mut ctx: TypeCheckContext,
    kind: sway_ast::Intrinsic,
    arguments: Vec<Expression>,
    type_arguments: Vec<TypeArgument>,
    span: Span,
) -> CompileResult<(ty::TyIntrinsicFunctionKind, TypeId)> {
    let type_engine = ctx.type_engine;

    let mut warnings = vec![];
    let mut errors = vec![];

    if arguments.len() != 1 {
        errors.push(CompileError::IntrinsicIncorrectNumArgs {
            name: kind.to_string(),
            expected: 1,
            span,
        });
        return err(warnings, errors);
    }

    if !type_arguments.is_empty() {
        errors.push(CompileError::IntrinsicIncorrectNumTArgs {
            name: kind.to_string(),
            expected: 0,
            span,
        });
        return err(warnings, errors);
    }

    // Type check the argument which is the revert code
    let mut ctx = ctx
        .by_ref()
        .with_type_annotation(type_engine.insert_type(TypeInfo::Unknown));
    let revert_code = check!(
        ty::TyExpression::type_check(ctx.by_ref(), arguments[0].clone()),
        return err(warnings, errors),
        warnings,
        errors
    );

    // Make sure that the revert code is a `u64`
    if !matches!(
        type_engine
            .to_typeinfo(revert_code.return_type, &revert_code.span)
            .unwrap(),
        TypeInfo::UnsignedInteger(IntegerBits::SixtyFour)
    ) {
        errors.push(CompileError::IntrinsicUnsupportedArgType {
            name: kind.to_string(),
            span: revert_code.span.clone(),
            hint: Hint::empty(),
        });
    }

    ok(
        (
            ty::TyIntrinsicFunctionKind {
                kind,
                arguments: vec![revert_code],
                type_arguments: vec![],
                span,
            },
            type_engine.insert_type(TypeInfo::Unknown), // TODO: change this to the `Never` type when
                                                        // available
        ),
        warnings,
        errors,
    )
}

/// Signature: `__ptr_add(ptr: raw_ptr, offset: u64)`
/// Description: Adds `offset` to the raw value of pointer `ptr`.
/// Constraints: None.
///
/// Signature: `__ptr_sub(ptr: raw_ptr, offset: u64)`
/// Description: Subtracts `offset` to the raw value of pointer `ptr`.
/// Constraints: None.
fn type_check_ptr_ops(
    mut ctx: TypeCheckContext,
    kind: sway_ast::Intrinsic,
    arguments: Vec<Expression>,
    type_arguments: Vec<TypeArgument>,
    span: Span,
) -> CompileResult<(ty::TyIntrinsicFunctionKind, TypeId)> {
    let type_engine = ctx.type_engine;

    let mut warnings = vec![];
    let mut errors = vec![];

    if arguments.len() != 2 {
        errors.push(CompileError::IntrinsicIncorrectNumArgs {
            name: kind.to_string(),
            expected: 2,
            span,
        });
        return err(warnings, errors);
    }
    if type_arguments.len() != 1 {
        errors.push(CompileError::IntrinsicIncorrectNumTArgs {
            name: kind.to_string(),
            expected: 1,
            span,
        });
        return err(warnings, errors);
    }
    let targ = type_arguments[0].clone();
    let initial_type_info = check!(
        CompileResult::from(
            type_engine
                .to_typeinfo(targ.type_id, &targ.span)
                .map_err(CompileError::from)
        ),
        TypeInfo::ErrorRecovery,
        warnings,
        errors
    );
    let initial_type_id = type_engine.insert_type(initial_type_info);
    let type_id = check!(
        ctx.resolve_type_with_self(initial_type_id, &targ.span, EnforceTypeArguments::No, None),
        type_engine.insert_type(TypeInfo::ErrorRecovery),
        warnings,
        errors,
    );

    let mut ctx = ctx
        .by_ref()
        .with_type_annotation(type_engine.insert_type(TypeInfo::Unknown));

    let lhs = arguments[0].clone();
    let lhs = check!(
        ty::TyExpression::type_check(ctx.by_ref(), lhs),
        return err(warnings, errors),
        warnings,
        errors
    );

    // Check for supported argument types
    let lhs_ty = check!(
        CompileResult::from(
            type_engine
                .to_typeinfo(lhs.return_type, &lhs.span)
                .map_err(CompileError::from)
        ),
        TypeInfo::ErrorRecovery,
        warnings,
        errors
    );
    if !matches!(lhs_ty, TypeInfo::RawUntypedPtr) {
        errors.push(CompileError::IntrinsicUnsupportedArgType {
            name: kind.to_string(),
            span: lhs.span,
            hint: Hint::empty(),
        });
        return err(warnings, errors);
    }

    let rhs = arguments[1].clone();
    let ctx = ctx
        .by_ref()
        .with_help_text("Incorrect argument type")
        .with_type_annotation(
            type_engine.insert_type(TypeInfo::UnsignedInteger(IntegerBits::SixtyFour)),
        );
    let rhs = check!(
        ty::TyExpression::type_check(ctx, rhs),
        return err(warnings, errors),
        warnings,
        errors
    );

    ok(
        (
            ty::TyIntrinsicFunctionKind {
                kind,
                arguments: vec![lhs, rhs],
                type_arguments: vec![TypeArgument {
                    type_id,
                    initial_type_id,
                    span: targ.span,
                }],
                span,
            },
            type_engine.insert_type(lhs_ty),
        ),
        warnings,
        errors,
    )
}
