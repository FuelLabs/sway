use super::*;
use crate::semantics::ast_node::*;
use crate::types::{IntegerBits, ResolvedType};
use either::Either;

#[derive(Clone, Debug)]
pub(crate) struct TypedExpression<'sc> {
    pub(crate) expression: TypedExpressionVariant<'sc>,
    pub(crate) return_type: ResolvedType<'sc>,
    /// whether or not this expression is constantly evaluatable (if the result is known at compile
    /// time)
    pub(crate) is_constant: IsConstant,
}

pub(crate) const ERROR_RECOVERY_EXPR: TypedExpression = TypedExpression {
    expression: TypedExpressionVariant::Unit,
    return_type: ResolvedType::ErrorRecovery,
    is_constant: IsConstant::No,
};

impl<'sc> TypedExpression<'sc> {
    pub(crate) fn type_check(
        other: Expression<'sc>,
        namespace: &Namespace<'sc>,
        type_annotation: Option<ResolvedType<'sc>>,
        help_text: impl Into<String> + Clone,
    ) -> CompileResult<'sc, Self> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let expr_span = other.span();
        let mut typed_expression = match other {
            Expression::Literal { value: lit, .. } => {
                let return_type = match lit {
                    Literal::String(_) => ResolvedType::String,
                    Literal::U8(_) => ResolvedType::UnsignedInteger(IntegerBits::Eight),
                    Literal::U16(_) => ResolvedType::UnsignedInteger(IntegerBits::Sixteen),
                    Literal::U32(_) => ResolvedType::UnsignedInteger(IntegerBits::ThirtyTwo),
                    Literal::U64(_) => ResolvedType::UnsignedInteger(IntegerBits::SixtyFour),
                    Literal::U128(_) => ResolvedType::UnsignedInteger(IntegerBits::OneTwentyEight),
                    Literal::Boolean(_) => ResolvedType::Boolean,
                    Literal::Byte(_) => ResolvedType::Byte,
                    Literal::Byte32(_) => ResolvedType::Byte32,
                };
                TypedExpression {
                    expression: TypedExpressionVariant::Literal(lit),
                    return_type,
                    is_constant: IsConstant::Yes,
                }
            }
            Expression::VariableExpression { name, unary_op, .. } => {
                match namespace.get_symbol(&name) {
                    Some(TypedDeclaration::VariableDeclaration(TypedVariableDeclaration {
                        body,
                        ..
                    })) => TypedExpression {
                        return_type: body.return_type.clone(),
                        is_constant: body.is_constant,
                        expression: TypedExpressionVariant::VariableExpression {
                            unary_op: unary_op.clone(),
                            name: name.clone(),
                        },
                    },
                    Some(a) => {
                        errors.push(CompileError::NotAVariable {
                            name: name.span.as_str(),
                            span: name.span,
                            what_it_is: a.friendly_name(),
                        });
                        ERROR_RECOVERY_EXPR.clone()
                    }
                    None => {
                        errors.push(CompileError::UnknownVariable {
                            var_name: name.span.as_str().trim(),
                            span: name.span,
                        });
                        ERROR_RECOVERY_EXPR.clone()
                    }
                }
            }
            Expression::FunctionApplication {
                name, arguments, ..
            } => {
                let function_declaration = type_check!(
                    namespace.get_call_path(&name),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                match function_declaration {
                    TypedDeclaration::FunctionDeclaration(TypedFunctionDeclaration {
                        parameters,
                        return_type,
                        ..
                    }) => {
                        // type check arguments in function application vs arguments in function
                        // declaration. Use parameter type annotations as annotations for the
                        // arguments
                        //
                        let mut typed_call_arguments = Vec::new();
                        for (arg, param) in arguments.into_iter().zip(parameters.iter()) {
                            let res = TypedExpression::type_check(
                                arg,
                                &namespace,
                                Some(param.r#type.clone()),
                                    "The argument that has been provided to this function's type does not match the declared type of the parameter in the function declaration."
                            );
                            let arg = match res {
                                CompileResult::Ok {
                                    value,
                                    warnings: mut l_w,
                                    errors: mut l_e,
                                } => {
                                    warnings.append(&mut l_w);
                                    errors.append(&mut l_e);
                                    value
                                }
                                CompileResult::Err {
                                    warnings: mut l_w,
                                    errors: mut l_e,
                                } => {
                                    warnings.append(&mut l_w);
                                    errors.append(&mut l_e);
                                    ERROR_RECOVERY_EXPR.clone()
                                }
                            };
                            typed_call_arguments.push(arg);
                        }

                        TypedExpression {
                            return_type: return_type.clone(),
                            // now check the function call return type
                            // FEATURE this IsConstant can be true if the function itself is constant-able
                            // const functions would be an advanced feature and are not supported right
                            // now
                            is_constant: IsConstant::No,
                            expression: TypedExpressionVariant::FunctionApplication {
                                arguments: typed_call_arguments,
                                name: name.clone(),
                            },
                        }
                    }
                    a => {
                        errors.push(CompileError::NotAFunction {
                            name: name.span().as_str(),
                            span: name.span(),
                            what_it_is: a.friendly_name(),
                        });
                        ERROR_RECOVERY_EXPR.clone()
                    }
                }
            }
            Expression::MatchExpression { .. } => {
                todo!("Match expressions");
                /*
                let typed_primary_expression = type_check!(
                    TypedExpression::type_check(*primary_expression, &namespace, None, ""),
                    ERROR_RECOVERY_EXPR.clone(),
                    warnings,
                    errors
                );
                let first_branch_result = type_check!(
                    TypedExpression::type_check(
                        branches[0].result.clone(),
                        &namespace,
                        type_annotation.clone(),
                        help_text.clone()
                    ),
                    ERROR_RECOVERY_EXPR.clone(),
                    warnings,
                    errors
                );

                let first_branch_result = vec![first_branch_result];
                // use type of first branch for annotation on the rest of the branches
                // we checked the first branch separately just to get its return type for inferencing the rest
                let mut rest_of_branches = branches
                    .into_iter()
                    .skip(1)
                    .map(
                        |MatchBranch {
                             condition, result, ..
                         }| {
                            type_check!(
                                TypedExpression::type_check(
                                    result,
                                    &namespace,
                                    Some(first_branch_result[0].return_type.clone()),
                                    "All branches of a match expression must be of the same type.",
                                ),
                                ERROR_RECOVERY_EXPR.clone(),
                                warnings,
                                errors
                            )
                        },
                    )
                    .collect::<Vec<_>>();

                let mut all_branches = first_branch_result;
                all_branches.append(&mut rest_of_branches);

                errors.push(CompileError::Unimplemented(
                    "Match expressions and pattern matching",
                    span,
                ));
                ERROR_RECOVERY_EXPR.clone()
                */
            }
            Expression::CodeBlock { contents, span, .. } => {
                let (typed_block, block_return_type) = type_check!(
                    TypedCodeBlock::type_check(
                        contents.clone(),
                        &namespace,
                        type_annotation.clone(),
                        help_text.clone()
                    ),
                    (TypedCodeBlock { contents: vec![] }, Some(ResolvedType::Unit)),
                    warnings,
                    errors
                );
                let block_return_type = match block_return_type {
                    Some(ty) => ty,
                    None => {
                        match type_annotation {
                            Some(ref ty) if ty != &ResolvedType::Unit =>{
                                errors.push(CompileError::ExpectedImplicitReturnFromBlockWithType { span: span.clone(), ty: ty.friendly_type_str() });
                                ResolvedType::ErrorRecovery
                            }
                            _ => ResolvedType::Unit
                        }
                    }
                };
                TypedExpression {
                    expression: TypedExpressionVariant::CodeBlock(TypedCodeBlock {
                        contents: typed_block.contents,
                    }),
                    return_type: block_return_type,
                    is_constant: IsConstant::No, // TODO if all elements of block are constant then this is constant
                }
            }
            // TODO if _condition_ is constant, evaluate it and compile this to a regular
            // expression with only one branch
            Expression::IfExp {
                condition,
                then,
                r#else,
                span,
            } => {
                let condition = Box::new(type_check!(
                    TypedExpression::type_check(
                        *condition,
                        &namespace,
                        Some(ResolvedType::Boolean),
                        "The condition of an if expression must be a boolean expression.",
                    ),
                    ERROR_RECOVERY_EXPR.clone(),
                    warnings,
                    errors
                ));
                let then = Box::new(type_check!(
                    TypedExpression::type_check(*then, &namespace, type_annotation.clone(), ""),
                    ERROR_RECOVERY_EXPR.clone(),
                    warnings,
                    errors
                ));
                let r#else = if let Some(expr) = r#else {
                    Some(Box::new(type_check!(
                        TypedExpression::type_check(
                            *expr,
                            namespace,
                            Some(then.return_type.clone()),
                            ""
                        ),
                        ERROR_RECOVERY_EXPR.clone(),
                        warnings,
                        errors
                    )))
                } else {
                    None
                };

                // if there is a type annotation, then the else branch must exist
                if let Some(ref annotation) = type_annotation {
                    if r#else.is_none() {
                        errors.push(CompileError::NoElseBranch {
                            span: span.clone(),
                            r#type: annotation.friendly_type_str(),
                        });
                    }
                }

                TypedExpression {
                    expression: TypedExpressionVariant::IfExp {
                        condition,
                        then: then.clone(),
                        r#else,
                    },
                    is_constant: IsConstant::No, // TODO
                    return_type: then.return_type,
                }
            }
            Expression::AsmExpression { asm, .. } => {
                let return_type = if asm.returns.is_some() {
                    ResolvedType::UnsignedInteger(IntegerBits::SixtyFour)
                } else {
                    ResolvedType::Unit
                };
                TypedExpression {
                    expression: TypedExpressionVariant::AsmExpression { asm },
                    return_type,
                    is_constant: IsConstant::No,
                }
            }
            Expression::StructExpression {
                span,
                struct_name,
                fields,
            } => {
                // TODO in here replace generic types with provided types
                // find the struct definition in the namespace
                let definition: &TypedStructDeclaration = match namespace.get_symbol(&struct_name) {
                    Some(TypedDeclaration::StructDeclaration(st)) => st,
                    Some(_) => {
                        errors.push(CompileError::DeclaredNonStructAsStruct {
                            name: struct_name.primary_name,
                            span: span.clone(),
                        });
                        return err(warnings, errors);
                    }
                    None => {
                        errors.push(CompileError::StructNotFound {
                            name: struct_name.primary_name,
                            span: span.clone(),
                        });
                        return err(warnings, errors);
                    }
                };
                let mut typed_fields_buf = vec![];

                // match up the names with their type annotations from the declaration
                for def_field in definition.fields.iter() {
                    let expr_field = match fields.iter().find(|x| x.name == def_field.name) {
                        Some(val) => val.clone(),
                        None => {
                            errors.push(CompileError::StructMissingField {
                                field_name: def_field.name,
                                struct_name: definition.name.primary_name,
                                span: span.clone(),
                            });
                            typed_fields_buf.push(TypedStructExpressionField {
                                name: def_field.name,
                                value: TypedExpression {
                                    expression: TypedExpressionVariant::Unit,
                                    return_type: ResolvedType::ErrorRecovery,
                                    is_constant: IsConstant::No,
                                },
                            });
                            continue;
                        }
                    };

                    let typed_field = type_check!(
                        TypedExpression::type_check(
                            expr_field.value,
                            &namespace,
                            Some(def_field.r#type.clone()),
                            "Struct field's type must match up with the type specified in its declaration.",
                        ),
                        continue,
                        warnings,
                        errors
                    );

                    typed_fields_buf.push(TypedStructExpressionField {
                        value: typed_field,
                        name: expr_field.name,
                    });
                }

                // check that there are no extra fields
                for field in fields {
                    if definition
                        .fields
                        .iter()
                        .find(|x| x.name == field.name)
                        .is_none()
                    {
                        errors.push(CompileError::StructDoesntHaveThisField {
                            field_name: field.name,
                            struct_name: definition.name.primary_name,
                            span: field.span,
                        });
                    }
                }
                TypedExpression {
                    expression: TypedExpressionVariant::StructExpression {
                        struct_name: definition.name.clone(),
                        fields: typed_fields_buf,
                    },
                    return_type: ResolvedType::Struct {
                        name: definition.name.clone(),
                    },
                    is_constant: IsConstant::No,
                }
            }
            Expression::SubfieldExpression {
                unary_op,
                name_parts,
                span,
            } => {
                // this must be >= 2, or else the parser would not have matched it. asserting that
                // invariant here, since it is an assumption that is acted upon later.
                assert!(name_parts.len() >= 2);
                let return_type = type_check!(
                    namespace.find_subfield(&name_parts),
                    return err(warnings, errors),
                    warnings,
                    errors
                );

                TypedExpression {
                    return_type,
                    expression: TypedExpressionVariant::SubfieldExpression {
                        unary_op,
                        name: name_parts,
                        span,
                    },
                    is_constant: IsConstant::No,
                }
            }
            Expression::MethodApplication {
                subfield_exp,
                method_name,
                arguments,
                span,
            } => {
                let (method, parent_type) = if subfield_exp.is_empty() {
                    // if subfield exp is empty, then we are calling a method using either ::
                    // syntax or an operator
                    let ns = type_check!(
                        namespace.find_module(&method_name.prefixes),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    // a method is defined by the type of the parent, and in this case the parent
                    // is the first argument
                    let parent_expr = match TypedExpression::type_check(
                        arguments[0].clone(),
                        namespace,
                        None,
                        "",
                    ) {
                        // throw away warnings and errors since this will be checked again later
                        CompileResult::Ok { value, .. } => value,
                        CompileResult::Err {
                            warnings: mut l_w,
                            errors: mut l_e,
                        } => {
                            warnings.append(&mut l_w);
                            errors.append(&mut l_e);
                            return err(warnings, errors);
                        }
                    };
                    (
                        match ns.find_method_for_type(
                            &parent_expr.return_type.clone(),
                            method_name.suffix.clone(),
                        ) {
                            Some(o) => o,
                            None => {
                                errors.push(CompileError::MethodNotFound {
                                    span,
                                    method_name: method_name.suffix.clone().primary_name,
                                    type_name: parent_expr.return_type.friendly_type_str(),
                                });
                                return err(warnings, errors);
                            }
                        },
                        parent_expr.return_type,
                    )
                } else {
                    let parent_type = type_check!(
                        namespace.find_subfield(&subfield_exp.clone()),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    (
                        match namespace
                            .find_method_for_type(&parent_type, method_name.suffix.clone())
                        {
                            Some(o) => o,
                            None => todo!("Method not found error"),
                        },
                        parent_type,
                    )
                };

                // zip parameters to arguments to perform type checking
                let zipped = method.parameters.iter().zip(arguments.iter());

                let mut typed_arg_buf = vec![];
                for (TypedFunctionParameter { r#type, .. }, arg) in zipped {
                    let un_self_type = if r#type == &ResolvedType::SelfType {
                        parent_type.clone()
                    } else {
                        r#type.clone()
                    };
                    typed_arg_buf.push(type_check!(
                        TypedExpression::type_check(
                            arg.clone(),
                            &namespace,
                            Some(un_self_type),
                            "Function argument must be of the same type declared in the function declaration."),
                        continue, 
                        warnings,
                        errors
                    ));
                }

                TypedExpression {
                    expression: TypedExpressionVariant::FunctionApplication {
                        // TODO the prefix should be a type info maybe? and then the first arg can
                        // be self?
                        name: method_name.into(), // TODO todo!("put the actual fully-typed function bodies in these applications"),
                        arguments: typed_arg_buf,
                    },
                    return_type: method.return_type,
                    is_constant: IsConstant::No,
                }
            }
            Expression::Unit { span: _span } => TypedExpression {
                expression: TypedExpressionVariant::Unit,
                return_type: ResolvedType::Unit,
                is_constant: IsConstant::Yes,
            },
            Expression::DelineatedPath {
                call_path,
                span,
                instantiator,
                type_arguments,
            } => {
                // The first step is to determine if the call path refers to a module or an enum.
                // We could rely on the capitalization convention, where modules are lowercase
                // and enums are uppercase, but this is not robust in the long term.
                // Instead, we try to resolve both paths.
                // If only one exists, then we use that one. Otherwise, if both exist, it is
                // an ambiguous reference error.
                let module_result = namespace.find_module(&call_path.prefixes).ok().cloned();
                /*
                let enum_result_result = {
                    // an enum could be combined with a module path
                    // e.g.
                    // ```
                    // module1::MyEnum::Variant1
                    // ```
                    //
                    // so, in this case, the suffix is Variant1 and the prefixes are module1 and
                    // MyEnum. When looking for an enum, we just want the _last_ prefix entry in the
                    // namespace of the first 0..len-1 entries' module
                    namespace.find_enum(&all_path.prefixes[0])
                };
                */
                let enum_module_combined_result = {
                    // also, check if this is an enum _in_ another module.
                    let (module_path, enum_name) =
                        call_path.prefixes.split_at(call_path.prefixes.len() - 1);
                    let enum_name = enum_name[0].clone();
                    let namespace = namespace.find_module(module_path);
                    let namespace = namespace.ok();
                    namespace.map(|ns| ns.find_enum(&enum_name)).flatten()
                };

                let type_arguments = type_arguments
                    .iter()
                    .map(|x| namespace.resolve_type(x))
                    .collect();
                // now we can see if this thing is a symbol (typed declaration) or reference to an
                // enum instantiation
                let this_thing: Either<TypedDeclaration, TypedExpression> =
                    match (module_result, enum_module_combined_result) {
                        (Some(_module), Some(_enum_res)) => todo!("Ambiguous reference error"),
                        (Some(module), None) => {
                            match module.get_symbol(&call_path.suffix).cloned() {
                                Some(decl) => Either::Left(decl),
                                None => todo!("symbol not found in module error"),
                            }
                        }
                        (None, Some(enum_decl)) => Either::Right(type_check!(
                            instantiate_enum(
                                enum_decl,
                                call_path.suffix,
                                instantiator,
                                type_arguments,
                                namespace
                            ),
                            return err(warnings, errors),
                            warnings,
                            errors
                        )),
                        (None, None) => todo!("symbol not found error"),
                    };

                match this_thing {
                    Either::Left(_) => {
                        errors.push(CompileError::Unimplemented("Unable to refer to declarations in other modules directly. Try importing it instead.", span));
                        return err(warnings, errors);
                    }
                    Either::Right(expr) => expr,
                }
            }
            a => {
                println!("Unimplemented semantics for expression: {:?}", a);
                errors.push(CompileError::Unimplemented(
                    "Unimplemented expression",
                    a.span(),
                ));

                ERROR_RECOVERY_EXPR
            }
        };
        // if the return type cannot be cast into the annotation type then it is a type error
        if let Some(type_annotation) = type_annotation {
            let convertability = typed_expression.return_type.is_convertable(
                &type_annotation,
                expr_span.clone(),
                help_text,
            );
            match convertability {
                Ok(warning) => {
                    if let Some(warning) = warning {
                        warnings.push(CompileWarning {
                            warning_content: warning,
                            span: expr_span,
                        });
                    }
                }
                Err(err) => {
                    errors.push(err.into());
                }
            }
            // The annotation will result in a cast, so set the return type accordingly.
            typed_expression.return_type = type_annotation
        }

        ok(typed_expression, warnings, errors)
    }
    pub(crate) fn pretty_print(&self) -> String {
        format!(
            "{} ({})",
            self.expression.pretty_print(),
            self.return_type.friendly_type_str()
        )
    }
}
