use sway_ast::intrinsics::Intrinsic;
use sway_error::error::{CompileError, Hint};
use sway_types::integer_bits::IntegerBits;
use sway_types::Span;

use crate::{
    error::{err, ok},
    language::{parsed::Expression, ty},
    semantic_analysis::TypeCheckContext,
    type_system::*,
    CompileResult,
};

#[derive(Clone)]
enum ExpectedType {
    NonGeneric(TypeInfo),
    Generic(u64),
}

impl ty::TyIntrinsicFunctionKind {
    pub(crate) fn type_check(
        mut ctx: TypeCheckContext,
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
            Intrinsic::Eq => type_check_eq(ctx, kind, arguments, type_arguments, span),
            Intrinsic::Gtf => {
                let expected_arg_types = [
                    ExpectedType::NonGeneric(TypeInfo::UnsignedInteger(IntegerBits::SixtyFour)),
                    ExpectedType::NonGeneric(TypeInfo::UnsignedInteger(IntegerBits::SixtyFour)),
                ]
                .to_vec();
                let expected_return_type = ExpectedType::Generic(0);
                type_check_intrinsic(
                    ctx,
                    kind,
                    arguments,
                    expected_arg_types,
                    expected_return_type,
                    type_arguments,
                    span,
                )
                //                type_check_gtf(ctx, kind, arguments, type_arguments, span)
            }
            Intrinsic::AddrOf => type_check_addr_of(ctx, kind, arguments, type_arguments, span),
            Intrinsic::StateLoadWord => {
                type_check_state_load_word(ctx, kind, arguments, type_arguments, span)
            }
            Intrinsic::StateStoreWord | Intrinsic::StateLoadQuad | Intrinsic::StateStoreQuad => {
                type_check_state_store_or_quad(ctx, kind, arguments, type_arguments, span)
            }
            Intrinsic::Log => type_check_log(ctx, kind, arguments, type_arguments, span),
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

fn type_check_intrinsic(
    mut ctx: TypeCheckContext,
    kind: sway_ast::Intrinsic,
    arguments: Vec<Expression>,
    expected_arg_types: Vec<ExpectedType>,
    expected_return_type: ExpectedType,
    type_arguments: Vec<TypeArgument>,
    span: Span,
) -> CompileResult<(ty::TyIntrinsicFunctionKind, TypeId)> {
    let mut warnings = vec![];
    let mut errors = vec![];

    if arguments.len() != expected_arg_types.len() {
        errors.push(CompileError::IntrinsicIncorrectNumArgs {
            name: kind.to_string(),
            expected: expected_arg_types.len() as u64,
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

    let mut args = Vec::new();
    for (ty, arg) in expected_arg_types.iter().zip(arguments.iter()) {
        // Type check the first argument which is the index
        let extracted_type = match ty {
            ExpectedType::NonGeneric(non_generic) => non_generic.clone(),
            ExpectedType::Generic(_) => TypeInfo::Unknown,
        };
        let mut ctx = ctx
            .by_ref()
            .with_type_annotation(insert_type(extracted_type));
        args.push(check!(
            ty::TyExpression::type_check(ctx.by_ref(), arg.clone()),
            return err(warnings, errors),
            warnings,
            errors
        ));
    }

    let targ = type_arguments[0].clone();
    let initial_type_info = check!(
        CompileResult::from(to_typeinfo(targ.type_id, &targ.span).map_err(CompileError::from)),
        TypeInfo::ErrorRecovery,
        warnings,
        errors
    );
    let initial_type_id = insert_type(initial_type_info);
    let type_id = check!(
        ctx.resolve_type_with_self(initial_type_id, &targ.span, EnforceTypeArguments::Yes, None),
        insert_type(TypeInfo::ErrorRecovery),
        warnings,
        errors,
    );

    ok(
        (
            ty::TyIntrinsicFunctionKind {
                kind,
                arguments: args,
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

fn type_check_size_of_val(
    ctx: TypeCheckContext,
    kind: sway_ast::Intrinsic,
    arguments: Vec<Expression>,
    _type_arguments: Vec<TypeArgument>,
    span: Span,
) -> CompileResult<(ty::TyIntrinsicFunctionKind, TypeId)> {
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
        .with_type_annotation(insert_type(TypeInfo::Unknown));
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
    let return_type = insert_type(TypeInfo::UnsignedInteger(IntegerBits::SixtyFour));
    ok((intrinsic_function, return_type), warnings, errors)
}

fn type_check_size_of_type(
    mut ctx: TypeCheckContext,
    kind: sway_ast::Intrinsic,
    arguments: Vec<Expression>,
    type_arguments: Vec<TypeArgument>,
    span: Span,
) -> CompileResult<(ty::TyIntrinsicFunctionKind, TypeId)> {
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
        CompileResult::from(to_typeinfo(targ.type_id, &targ.span).map_err(CompileError::from)),
        TypeInfo::ErrorRecovery,
        warnings,
        errors
    );
    let initial_type_id = insert_type(initial_type_info);
    let type_id = check!(
        ctx.resolve_type_with_self(initial_type_id, &targ.span, EnforceTypeArguments::Yes, None),
        insert_type(TypeInfo::ErrorRecovery),
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
    let return_type = insert_type(TypeInfo::UnsignedInteger(IntegerBits::SixtyFour));
    ok((intrinsic_function, return_type), warnings, errors)
}

fn type_check_is_reference_type(
    mut ctx: TypeCheckContext,
    kind: sway_ast::Intrinsic,
    _arguments: Vec<Expression>,
    type_arguments: Vec<TypeArgument>,
    span: Span,
) -> CompileResult<(ty::TyIntrinsicFunctionKind, TypeId)> {
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
        CompileResult::from(to_typeinfo(targ.type_id, &targ.span).map_err(CompileError::from)),
        TypeInfo::ErrorRecovery,
        warnings,
        errors
    );
    let initial_type_id = insert_type(initial_type_info);
    let type_id = check!(
        ctx.resolve_type_with_self(initial_type_id, &targ.span, EnforceTypeArguments::Yes, None),
        insert_type(TypeInfo::ErrorRecovery),
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
        (intrinsic_function, insert_type(TypeInfo::Boolean)),
        warnings,
        errors,
    )
}

fn type_check_get_storage_key(
    mut _ctx: TypeCheckContext,
    kind: sway_ast::Intrinsic,
    _arguments: Vec<Expression>,
    _type_arguments: Vec<TypeArgument>,
    span: Span,
) -> CompileResult<(ty::TyIntrinsicFunctionKind, TypeId)> {
    let mut warnings = vec![];
    let mut errors = vec![];

    ok(
        (
            ty::TyIntrinsicFunctionKind {
                kind,
                arguments: vec![],
                type_arguments: vec![],
                span,
            },
            insert_type(TypeInfo::B256),
        ),
        warnings,
        errors,
    )
}

fn type_check_eq(
    mut ctx: TypeCheckContext,
    kind: sway_ast::Intrinsic,
    arguments: Vec<Expression>,
    type_arguments: Vec<TypeArgument>,
    span: Span,
) -> CompileResult<(ty::TyIntrinsicFunctionKind, TypeId)> {
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
        .with_type_annotation(insert_type(TypeInfo::Unknown));

    let lhs = arguments[0].clone();
    let lhs = check!(
        ty::TyExpression::type_check(ctx.by_ref(), lhs),
        return err(warnings, errors),
        warnings,
        errors
    );

    // Check for supported argument types
    let arg_ty = check!(
        CompileResult::from(to_typeinfo(lhs.return_type, &lhs.span).map_err(CompileError::from)),
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
            insert_type(TypeInfo::Boolean),
        ),
        warnings,
        errors,
    )
}

fn type_check_gtf(
    mut ctx: TypeCheckContext,
    kind: sway_ast::Intrinsic,
    arguments: Vec<Expression>,
    type_arguments: Vec<TypeArgument>,
    span: Span,
) -> CompileResult<(ty::TyIntrinsicFunctionKind, TypeId)> {
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
        .with_type_annotation(insert_type(TypeInfo::Unknown));
    let index = check!(
        ty::TyExpression::type_check(ctx.by_ref(), arguments[0].clone()),
        return err(warnings, errors),
        warnings,
        errors
    );

    // Type check the second argument which is the tx field ID
    let mut ctx = ctx
        .by_ref()
        .with_type_annotation(insert_type(TypeInfo::Unknown));
    let tx_field_id = check!(
        ty::TyExpression::type_check(ctx.by_ref(), arguments[1].clone()),
        return err(warnings, errors),
        warnings,
        errors
    );

    // Make sure that the index argument is a `u64`
    let index_type_info = check!(
        CompileResult::from(
            to_typeinfo(index.return_type, &index.span).map_err(CompileError::from)
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
            to_typeinfo(tx_field_id.return_type, &tx_field_id.span).map_err(CompileError::from)
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
        CompileResult::from(to_typeinfo(targ.type_id, &targ.span).map_err(CompileError::from)),
        TypeInfo::ErrorRecovery,
        warnings,
        errors
    );
    let initial_type_id = insert_type(initial_type_info);
    let type_id = check!(
        ctx.resolve_type_with_self(initial_type_id, &targ.span, EnforceTypeArguments::Yes, None),
        insert_type(TypeInfo::ErrorRecovery),
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

fn type_check_addr_of(
    mut ctx: TypeCheckContext,
    kind: sway_ast::Intrinsic,
    arguments: Vec<Expression>,
    type_arguments: Vec<TypeArgument>,
    span: Span,
) -> CompileResult<(ty::TyIntrinsicFunctionKind, TypeId)> {
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
        .with_type_annotation(insert_type(TypeInfo::Unknown));
    let exp = check!(
        ty::TyExpression::type_check(ctx, arguments[0].clone()),
        return err(warnings, errors),
        warnings,
        errors
    );
    let copy_type_info = check!(
        CompileResult::from(to_typeinfo(exp.return_type, &span).map_err(CompileError::from)),
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
    let return_type = insert_type(TypeInfo::RawUntypedPtr);
    ok((intrinsic_function, return_type), warnings, errors)
}

fn type_check_state_load_word(
    mut ctx: TypeCheckContext,
    kind: sway_ast::Intrinsic,
    arguments: Vec<Expression>,
    type_arguments: Vec<TypeArgument>,
    span: Span,
) -> CompileResult<(ty::TyIntrinsicFunctionKind, TypeId)> {
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
        .with_type_annotation(insert_type(TypeInfo::Unknown));
    let exp = check!(
        ty::TyExpression::type_check(ctx, arguments[0].clone()),
        return err(warnings, errors),
        warnings,
        errors
    );
    let key_ty = check!(
        CompileResult::from(to_typeinfo(exp.return_type, &span).map_err(CompileError::from)),
        TypeInfo::ErrorRecovery,
        warnings,
        errors
    );
    if key_ty != TypeInfo::B256 {
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
    let return_type = insert_type(TypeInfo::UnsignedInteger(IntegerBits::SixtyFour));
    ok((intrinsic_function, return_type), warnings, errors)
}

fn type_check_state_store_or_quad(
    mut ctx: TypeCheckContext,
    kind: sway_ast::Intrinsic,
    arguments: Vec<Expression>,
    type_arguments: Vec<TypeArgument>,
    span: Span,
) -> CompileResult<(ty::TyIntrinsicFunctionKind, TypeId)> {
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
        .with_type_annotation(insert_type(TypeInfo::Unknown));
    let key_exp = check!(
        ty::TyExpression::type_check(ctx.by_ref(), arguments[0].clone()),
        return err(warnings, errors),
        warnings,
        errors
    );
    let key_ty = check!(
        CompileResult::from(to_typeinfo(key_exp.return_type, &span).map_err(CompileError::from)),
        TypeInfo::ErrorRecovery,
        warnings,
        errors
    );
    if key_ty != TypeInfo::B256 {
        errors.push(CompileError::IntrinsicUnsupportedArgType {
            name: kind.to_string(),
            span,
            hint: Hint::new("Argument type must be B256, a key into the state storage".to_string()),
        });
        return err(warnings, errors);
    }
    let mut ctx = ctx
        .with_help_text("")
        .with_type_annotation(insert_type(TypeInfo::Unknown));
    let val_exp = check!(
        ty::TyExpression::type_check(ctx.by_ref(), arguments[1].clone()),
        return err(warnings, errors),
        warnings,
        errors
    );
    let type_argument = type_arguments.get(0).map(|targ| {
        let mut ctx = ctx
            .with_help_text("")
            .with_type_annotation(insert_type(TypeInfo::Unknown));
        let initial_type_info = check!(
            CompileResult::from(to_typeinfo(targ.type_id, &targ.span).map_err(CompileError::from)),
            TypeInfo::ErrorRecovery,
            warnings,
            errors
        );
        let initial_type_id = insert_type(initial_type_info);
        let type_id = check!(
            ctx.resolve_type_with_self(
                initial_type_id,
                &targ.span,
                EnforceTypeArguments::Yes,
                None
            ),
            insert_type(TypeInfo::ErrorRecovery),
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
    let return_type = insert_type(TypeInfo::Tuple(vec![]));
    ok((intrinsic_function, return_type), warnings, errors)
}

fn type_check_log(
    mut ctx: TypeCheckContext,
    kind: sway_ast::Intrinsic,
    arguments: Vec<Expression>,
    type_arguments: Vec<TypeArgument>,
    span: Span,
) -> CompileResult<(ty::TyIntrinsicFunctionKind, TypeId)> {
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
        .with_type_annotation(insert_type(TypeInfo::Unknown));
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
    let return_type = insert_type(TypeInfo::Tuple(vec![]));
    ok((intrinsic_function, return_type), warnings, errors)
}

fn type_check_binary_op(
    mut ctx: TypeCheckContext,
    kind: sway_ast::Intrinsic,
    arguments: Vec<Expression>,
    type_arguments: Vec<TypeArgument>,
    span: Span,
) -> CompileResult<(ty::TyIntrinsicFunctionKind, TypeId)> {
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
        .with_type_annotation(insert_type(TypeInfo::Unknown));

    let lhs = arguments[0].clone();
    let lhs = check!(
        ty::TyExpression::type_check(ctx.by_ref(), lhs),
        return err(warnings, errors),
        warnings,
        errors
    );

    // Check for supported argument types
    let arg_ty = check!(
        CompileResult::from(to_typeinfo(lhs.return_type, &lhs.span).map_err(CompileError::from)),
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
            insert_type(arg_ty),
        ),
        warnings,
        errors,
    )
}

fn type_check_revert(
    mut ctx: TypeCheckContext,
    kind: sway_ast::Intrinsic,
    arguments: Vec<Expression>,
    type_arguments: Vec<TypeArgument>,
    span: Span,
) -> CompileResult<(ty::TyIntrinsicFunctionKind, TypeId)> {
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
        .with_type_annotation(insert_type(TypeInfo::Unknown));
    let revert_code = check!(
        ty::TyExpression::type_check(ctx.by_ref(), arguments[0].clone()),
        return err(warnings, errors),
        warnings,
        errors
    );

    // Make sure that the revert code is a `u64`
    if !matches!(
        to_typeinfo(revert_code.return_type, &revert_code.span).unwrap(),
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
            insert_type(TypeInfo::Unknown), // TODO: change this to the `Never` type when
                                            // available
        ),
        warnings,
        errors,
    )
}

fn type_check_ptr_ops(
    mut ctx: TypeCheckContext,
    kind: sway_ast::Intrinsic,
    arguments: Vec<Expression>,
    type_arguments: Vec<TypeArgument>,
    span: Span,
) -> CompileResult<(ty::TyIntrinsicFunctionKind, TypeId)> {
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
        CompileResult::from(to_typeinfo(targ.type_id, &targ.span).map_err(CompileError::from)),
        TypeInfo::ErrorRecovery,
        warnings,
        errors
    );
    let initial_type_id = insert_type(initial_type_info);
    let type_id = check!(
        ctx.resolve_type_with_self(initial_type_id, &targ.span, EnforceTypeArguments::No, None),
        insert_type(TypeInfo::ErrorRecovery),
        warnings,
        errors,
    );

    let mut ctx = ctx
        .by_ref()
        .with_type_annotation(insert_type(TypeInfo::Unknown));

    let lhs = arguments[0].clone();
    let lhs = check!(
        ty::TyExpression::type_check(ctx.by_ref(), lhs),
        return err(warnings, errors),
        warnings,
        errors
    );

    // Check for supported argument types
    let lhs_ty = check!(
        CompileResult::from(to_typeinfo(lhs.return_type, &lhs.span).map_err(CompileError::from)),
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
        .with_type_annotation(insert_type(TypeInfo::UnsignedInteger(
            IntegerBits::SixtyFour,
        )));
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
            insert_type(lhs_ty),
        ),
        warnings,
        errors,
    )
}
