use std::fmt;

use itertools::Itertools;
use sway_ast::intrinsics::Intrinsic;
use sway_types::Span;

use crate::{
    error::{err, ok},
    language::parsed::Expression,
    semantic_analysis::TypeCheckContext,
    type_system::*,
    types::DeterministicallyAborts,
    CompileError, CompileResult, Hint,
};

use super::TyExpression;

#[derive(Debug, Clone, PartialEq)]
pub struct TyIntrinsicFunctionKind {
    pub kind: Intrinsic,
    pub arguments: Vec<TyExpression>,
    pub type_arguments: Vec<TypeArgument>,
    pub span: Span,
}

impl CopyTypes for TyIntrinsicFunctionKind {
    fn copy_types(&mut self, type_mapping: &TypeMapping) {
        for arg in &mut self.arguments {
            arg.copy_types(type_mapping);
        }
        for targ in &mut self.type_arguments {
            targ.type_id.copy_types(type_mapping);
        }
    }
}

impl fmt::Display for TyIntrinsicFunctionKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let targs = self
            .type_arguments
            .iter()
            .map(|targ| look_up_type_id(targ.type_id))
            .join(", ");
        let args = self.arguments.iter().map(|e| format!("{}", e)).join(", ");

        write!(f, "{}::<{}>::({})", self.kind, targs, args)
    }
}

impl DeterministicallyAborts for TyIntrinsicFunctionKind {
    fn deterministically_aborts(&self) -> bool {
        matches!(self.kind, Intrinsic::Revert)
            || self.arguments.iter().any(|x| x.deterministically_aborts())
    }
}

impl CollectTypesMetadata for TyIntrinsicFunctionKind {
    fn collect_types_metadata(&self) -> CompileResult<Vec<TypeMetadata>> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let mut types_metadata = vec![];
        for type_arg in self.type_arguments.iter() {
            types_metadata.append(&mut check!(
                type_arg.type_id.collect_types_metadata(),
                return err(warnings, errors),
                warnings,
                errors
            ));
        }
        for arg in self.arguments.iter() {
            types_metadata.append(&mut check!(
                arg.collect_types_metadata(),
                return err(warnings, errors),
                warnings,
                errors
            ));
        }

        if matches!(self.kind, Intrinsic::Log) {
            types_metadata.push(TypeMetadata::LoggedType(self.arguments[0].return_type));
        }

        ok(types_metadata, warnings, errors)
    }
}

impl TyIntrinsicFunctionKind {
    pub(crate) fn type_check(
        mut ctx: TypeCheckContext,
        kind_binding: TypeBinding<Intrinsic>,
        arguments: Vec<Expression>,
        span: Span,
    ) -> CompileResult<(Self, TypeId)> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let TypeBinding {
            inner: kind,
            type_arguments,
            ..
        } = kind_binding;
        let (intrinsic_function, return_type) = match kind {
            Intrinsic::SizeOfVal => {
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
                    TyExpression::type_check(ctx, arguments[0].clone()),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                let intrinsic_function = TyIntrinsicFunctionKind {
                    kind,
                    arguments: vec![exp],
                    type_arguments: vec![],
                    span,
                };
                let return_type = insert_type(TypeInfo::UnsignedInteger(IntegerBits::SixtyFour));
                (intrinsic_function, return_type)
            }
            Intrinsic::SizeOfType => {
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
                        to_typeinfo(targ.type_id, &targ.span).map_err(CompileError::from)
                    ),
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
                let intrinsic_function = TyIntrinsicFunctionKind {
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
                (intrinsic_function, return_type)
            }
            Intrinsic::IsReferenceType => {
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
                        to_typeinfo(targ.type_id, &targ.span).map_err(CompileError::from)
                    ),
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
                let intrinsic_function = TyIntrinsicFunctionKind {
                    kind,
                    arguments: vec![],
                    type_arguments: vec![TypeArgument {
                        type_id,
                        initial_type_id,
                        span: targ.span,
                    }],
                    span,
                };
                (intrinsic_function, insert_type(TypeInfo::Boolean))
            }
            Intrinsic::GetStorageKey => (
                TyIntrinsicFunctionKind {
                    kind,
                    arguments: vec![],
                    type_arguments: vec![],
                    span,
                },
                insert_type(TypeInfo::B256),
            ),
            Intrinsic::Eq => {
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
                    TyExpression::type_check(ctx.by_ref(), lhs),
                    return err(warnings, errors),
                    warnings,
                    errors
                );

                // Check for supported argument types
                let arg_ty = check!(
                    CompileResult::from(
                        to_typeinfo(lhs.return_type, &lhs.span).map_err(CompileError::from)
                    ),
                    TypeInfo::ErrorRecovery,
                    warnings,
                    errors
                );
                let is_valid_arg_ty = matches!(arg_ty, TypeInfo::UnsignedInteger(_))
                    || matches!(arg_ty, TypeInfo::Boolean);
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
                    TyExpression::type_check(ctx, rhs),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                (
                    TyIntrinsicFunctionKind {
                        kind,
                        arguments: vec![lhs, rhs],
                        type_arguments: vec![],
                        span,
                    },
                    insert_type(TypeInfo::Boolean),
                )
            }
            Intrinsic::Gtf => {
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
                    TyExpression::type_check(ctx.by_ref(), arguments[0].clone()),
                    return err(warnings, errors),
                    warnings,
                    errors
                );

                // Type check the second argument which is the tx field ID
                let mut ctx = ctx
                    .by_ref()
                    .with_type_annotation(insert_type(TypeInfo::Unknown));
                let tx_field_id = check!(
                    TyExpression::type_check(ctx.by_ref(), arguments[1].clone()),
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
                        to_typeinfo(tx_field_id.return_type, &tx_field_id.span)
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
                        to_typeinfo(targ.type_id, &targ.span).map_err(CompileError::from)
                    ),
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

                (
                    TyIntrinsicFunctionKind {
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
                )
            }
            Intrinsic::AddrOf => {
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
                    TyExpression::type_check(ctx, arguments[0].clone()),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                let copy_type_info = check!(
                    CompileResult::from(
                        to_typeinfo(exp.return_type, &span).map_err(CompileError::from)
                    ),
                    TypeInfo::ErrorRecovery,
                    warnings,
                    errors
                );
                if copy_type_info.is_copy_type() {
                    errors.push(CompileError::IntrinsicUnsupportedArgType {
                        name: kind.to_string(),
                        span,
                        hint: Hint::new(
                            "Only a reference type can be used as argument here".to_string(),
                        ),
                    });
                    return err(warnings, errors);
                }

                let intrinsic_function = TyIntrinsicFunctionKind {
                    kind,
                    arguments: vec![exp],
                    type_arguments: vec![],
                    span,
                };
                let return_type = insert_type(TypeInfo::UnsignedInteger(IntegerBits::SixtyFour));
                (intrinsic_function, return_type)
            }
            Intrinsic::StateLoadWord => {
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
                    TyExpression::type_check(ctx, arguments[0].clone()),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                let key_ty = check!(
                    CompileResult::from(
                        to_typeinfo(exp.return_type, &span).map_err(CompileError::from)
                    ),
                    TypeInfo::ErrorRecovery,
                    warnings,
                    errors
                );
                if key_ty != TypeInfo::B256 {
                    errors.push(CompileError::IntrinsicUnsupportedArgType {
                        name: kind.to_string(),
                        span,
                        hint: Hint::new(
                            "Argument type must be B256, a key into the state storage".to_string(),
                        ),
                    });
                    return err(warnings, errors);
                }
                let intrinsic_function = TyIntrinsicFunctionKind {
                    kind,
                    arguments: vec![exp],
                    type_arguments: vec![],
                    span,
                };
                let return_type = insert_type(TypeInfo::UnsignedInteger(IntegerBits::SixtyFour));
                (intrinsic_function, return_type)
            }
            Intrinsic::StateStoreWord | Intrinsic::StateLoadQuad | Intrinsic::StateStoreQuad => {
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
                    TyExpression::type_check(ctx.by_ref(), arguments[0].clone()),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                let key_ty = check!(
                    CompileResult::from(
                        to_typeinfo(key_exp.return_type, &span).map_err(CompileError::from)
                    ),
                    TypeInfo::ErrorRecovery,
                    warnings,
                    errors
                );
                if key_ty != TypeInfo::B256 {
                    errors.push(CompileError::IntrinsicUnsupportedArgType {
                        name: kind.to_string(),
                        span,
                        hint: Hint::new(
                            "Argument type must be B256, a key into the state storage".to_string(),
                        ),
                    });
                    return err(warnings, errors);
                }
                let mut ctx = ctx
                    .with_help_text("")
                    .with_type_annotation(insert_type(TypeInfo::Unknown));
                let val_exp = check!(
                    TyExpression::type_check(ctx.by_ref(), arguments[1].clone()),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                let type_argument = type_arguments.get(0).map(|targ| {
                    let mut ctx = ctx
                        .with_help_text("")
                        .with_type_annotation(insert_type(TypeInfo::Unknown));
                    let initial_type_info = check!(
                        CompileResult::from(
                            to_typeinfo(targ.type_id, &targ.span).map_err(CompileError::from)
                        ),
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
                let intrinsic_function = TyIntrinsicFunctionKind {
                    kind,
                    arguments: vec![key_exp, val_exp],
                    type_arguments: type_argument.map_or(vec![], |ta| vec![ta]),
                    span,
                };
                let return_type = insert_type(TypeInfo::Tuple(vec![]));
                (intrinsic_function, return_type)
            }
            Intrinsic::Log => {
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
                    TyExpression::type_check(ctx, arguments[0].clone()),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                let intrinsic_function = TyIntrinsicFunctionKind {
                    kind,
                    arguments: vec![exp],
                    type_arguments: vec![],
                    span,
                };
                let return_type = insert_type(TypeInfo::Tuple(vec![]));
                (intrinsic_function, return_type)
            }
            Intrinsic::Add | Intrinsic::Sub | Intrinsic::Mul | Intrinsic::Div => {
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
                    TyExpression::type_check(ctx.by_ref(), lhs),
                    return err(warnings, errors),
                    warnings,
                    errors
                );

                // Check for supported argument types
                let arg_ty = check!(
                    CompileResult::from(
                        to_typeinfo(lhs.return_type, &lhs.span).map_err(CompileError::from)
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
                    TyExpression::type_check(ctx, rhs),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                (
                    TyIntrinsicFunctionKind {
                        kind,
                        arguments: vec![lhs, rhs],
                        type_arguments: vec![],
                        span,
                    },
                    insert_type(arg_ty),
                )
            }
            Intrinsic::Revert => {
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
                    TyExpression::type_check(ctx.by_ref(), arguments[0].clone()),
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

                (
                    TyIntrinsicFunctionKind {
                        kind,
                        arguments: vec![revert_code],
                        type_arguments: vec![],
                        span,
                    },
                    insert_type(TypeInfo::Unknown), // TODO: change this to the `Never` type when
                                                    // available
                )
            }
        };
        ok((intrinsic_function, return_type), warnings, errors)
    }
}
