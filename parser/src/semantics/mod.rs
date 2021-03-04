use crate::error::*;
use crate::parse_tree::*;
use crate::types::{IntegerBits, TypeInfo};
use crate::{AstNode, AstNodeContent, CodeBlock, ParseTree, ReturnStatement, TraitFn};
use either::Either;
use pest::Span;
use std::collections::HashMap;

const ERROR_RECOVERY_EXPR: TypedExpression = TypedExpression {
    expression: TypedExpressionVariant::Unit,
    return_type: TypeInfo::ErrorRecovery,
    is_constant: IsConstant::No,
};

const ERROR_RECOVERY_NODE_CONTENT: TypedAstNodeContent =
    TypedAstNodeContent::Expression(ERROR_RECOVERY_EXPR);

const ERROR_RECOVERY_DECLARATION: TypedDeclaration = TypedDeclaration::ErrorRecovery;

#[derive(Debug)]
pub(crate) struct TypedParseTree<'sc> {
    root_nodes: Vec<TypedAstNode<'sc>>,
}

#[derive(Clone, Debug)]
pub(crate) struct TypedAstNode<'sc> {
    content: TypedAstNodeContent<'sc>,
    span: Span<'sc>,
    scope: HashMap<VarName<'sc>, TypedDeclaration<'sc>>,
}

impl<'sc> TypedAstNode<'sc> {
    fn type_info(&self) -> TypeInfo<'sc> {
        // return statement should be ()
        use TypedAstNodeContent::*;
        match &self.content {
            UseStatement(_) | ReturnStatement(_) | Declaration(_) | TraitDeclaration(_) => {
                TypeInfo::Unit
            }
            Expression(TypedExpression { return_type, .. }) => return_type.clone(),
            ImplicitReturnExpression(TypedExpression { return_type, .. }) => return_type.clone(),
        }
    }
}
#[derive(Clone, Debug)]
pub(crate) enum TypedAstNodeContent<'sc> {
    UseStatement(UseStatement<'sc>),
    //    CodeBlock(TypedCodeBlock<'sc>),
    ReturnStatement(TypedReturnStatement<'sc>),
    Declaration(TypedDeclaration<'sc>),
    Expression(TypedExpression<'sc>),
    TraitDeclaration(TraitDeclaration<'sc>),
    ImplicitReturnExpression(TypedExpression<'sc>),
}

/// whether or not something is constantly evaluatable (if the result is known at compile
/// time)
#[derive(Clone, Copy, Debug)]
enum IsConstant {
    Yes,
    No,
}

#[derive(Clone, Debug)]
pub(crate) enum TypedDeclaration<'sc> {
    VariableDeclaration(TypedVariableDeclaration<'sc>),
    FunctionDeclaration(TypedFunctionDeclaration<'sc>),
    TraitDeclaration(TypedTraitDeclaration<'sc>),
    StructDeclaration(StructDeclaration<'sc>),
    EnumDeclaration(EnumDeclaration<'sc>),
    ErrorRecovery,
}

impl<'sc> TypedDeclaration<'sc> {
    /// friendly name string used for error reporting.
    pub(crate) fn friendly_name(&self) -> &'static str {
        use TypedDeclaration::*;
        match self {
            VariableDeclaration(_) => "variable",
            FunctionDeclaration(_) => "function",
            TraitDeclaration(_) => "trait",
            StructDeclaration(_) => "struct",
            EnumDeclaration(_) => "enum",
            ErrorRecovery => "invalid declaration",
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct TypedReturnStatement<'sc> {
    expr: TypedExpression<'sc>,
}
#[derive(Clone, Debug)]
pub(crate) struct TypedVariableDeclaration<'sc> {
    pub(crate) name: VarName<'sc>,
    pub(crate) body: TypedExpression<'sc>, // will be codeblock variant
    pub(crate) is_mutable: bool,
}

// TODO: type check generic type args and their usage
#[derive(Clone, Debug)]
pub(crate) struct TypedFunctionDeclaration<'sc> {
    pub(crate) name: &'sc str,
    pub(crate) body: TypedCodeBlock<'sc>,
    pub(crate) parameters: Vec<FunctionParameter<'sc>>,
    pub(crate) span: pest::Span<'sc>,
    pub(crate) return_type: TypeInfo<'sc>,
    pub(crate) type_parameters: Vec<TypeParameter<'sc>>,
}

#[derive(Clone, Debug)]
pub(crate) struct TypedTraitDeclaration<'sc> {
    name: VarName<'sc>,
    interface_surface: Vec<TraitFn<'sc>>, // TODO typed TraitFn which checks geneerics
    methods: Vec<TypedFunctionDeclaration<'sc>>,
}

#[derive(Clone, Debug)]
pub(crate) struct TypedExpression<'sc> {
    expression: TypedExpressionVariant<'sc>,
    return_type: TypeInfo<'sc>,
    /// whether or not this expression is constantly evaluatable (if the result is known at compile
    /// time)
    is_constant: IsConstant,
}
#[derive(Clone, Debug)]
pub(crate) enum TypedExpressionVariant<'sc> {
    Literal(Literal<'sc>),
    FunctionApplication {
        name: VarName<'sc>,
        arguments: Vec<TypedExpression<'sc>>,
    },
    VariableExpression {
        unary_op: Option<UnaryOp>,
        name: VarName<'sc>,
    },
    Unit,
    Array {
        contents: Vec<TypedExpression<'sc>>,
    },
    MatchExpression {
        primary_expression: Box<TypedExpression<'sc>>,
        branches: Vec<TypedMatchBranch<'sc>>,
    },
    StructExpression {
        struct_name: &'sc str,
        fields: Vec<TypedStructExpressionField<'sc>>,
    },
    CodeBlock(TypedCodeBlock<'sc>),
    // a flag that this value will later be provided as a parameter, but is currently unknown
    FunctionParameter,
    IfExp {
        condition: Box<TypedExpression<'sc>>,
        then: Box<TypedExpression<'sc>>,
        r#else: Option<Box<TypedExpression<'sc>>>,
    },
}
#[derive(Clone, Debug)]
pub(crate) struct TypedStructExpressionField<'sc> {
    name: &'sc str,
    value: TypedExpression<'sc>,
}
#[derive(Clone, Debug)]
pub(crate) struct TypedMatchBranch<'sc> {
    condition: TypedMatchCondition<'sc>,
    result: Either<TypedCodeBlock<'sc>, TypedExpression<'sc>>,
}

#[derive(Clone, Debug)]
pub(crate) enum TypedMatchCondition<'sc> {
    CatchAll,
    Expression(TypedExpression<'sc>),
}
#[derive(Clone, Debug)]
pub(crate) struct TypedCodeBlock<'sc> {
    contents: Vec<TypedAstNode<'sc>>,
}

impl<'sc> TypedExpression<'sc> {
    fn type_check(
        other: Expression<'sc>,
        namespace: HashMap<VarName<'sc>, TypedDeclaration<'sc>>,
        type_annotation: Option<TypeInfo<'sc>>,
        help_text: impl Into<String> + Clone,
    ) -> CompileResult<'sc, Self> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let expr_span = other.span();
        let typed_expression = match other {
            Expression::Literal { value: lit, .. } => {
                let return_type = match lit {
                    Literal::String(_) => TypeInfo::String,
                    Literal::U8(_) => TypeInfo::UnsignedInteger(IntegerBits::Eight),
                    Literal::U16(_) => TypeInfo::UnsignedInteger(IntegerBits::Sixteen),
                    Literal::U32(_) => TypeInfo::UnsignedInteger(IntegerBits::ThirtyTwo),
                    Literal::U64(_) => TypeInfo::UnsignedInteger(IntegerBits::SixtyFour),
                    Literal::U128(_) => TypeInfo::UnsignedInteger(IntegerBits::OneTwentyEight),
                    Literal::Boolean(_) => TypeInfo::Boolean,
                    Literal::Byte(_) => TypeInfo::Byte,
                    Literal::Byte32(_) => TypeInfo::Byte32,
                };
                TypedExpression {
                    expression: TypedExpressionVariant::Literal(lit),
                    return_type,
                    is_constant: IsConstant::Yes,
                }
            }
            Expression::VariableExpression { name, unary_op, .. } => match namespace.get(&name) {
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
            },
            Expression::FunctionApplication {
                name, arguments, ..
            } => {
                let function_declaration = namespace.get(&name);
                match function_declaration {
                    Some(TypedDeclaration::FunctionDeclaration(TypedFunctionDeclaration {
                        parameters,
                        return_type,
                        ..
                    })) => {
                        // type check arguments in function application vs arguments in function
                        // declaration. Use parameter type annotations as annotations for the
                        // arguments
                        //
                        let mut typed_call_arguments = Vec::new();
                        for (arg, param) in arguments.into_iter().zip(parameters.iter()) {
                            let res = TypedExpression::type_check(
                                arg,
                                namespace.clone(),
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
                    Some(a) => {
                        errors.push(CompileError::NotAFunction {
                            name: name.span.as_str(),
                            span: name.span,
                            what_it_is: a.friendly_name(),
                        });
                        ERROR_RECOVERY_EXPR.clone()
                    }
                    None => {
                        errors.push(CompileError::UnknownFunction {
                            name: name.span.as_str(),
                            span: name.span,
                        });
                        ERROR_RECOVERY_EXPR.clone()
                    }
                }
            }
            Expression::MatchExpression {
                primary_expression,
                branches,
                span,
                ..
            } => {
                let typed_primary_expression = type_check!(
                    TypedExpression,
                    *primary_expression,
                    namespace.clone(),
                    None,
                    "",
                    ERROR_RECOVERY_EXPR.clone(),
                    warnings,
                    errors
                );
                let first_branch_result = type_check!(
                    TypedExpression,
                    branches[0].result.clone(),
                    namespace.clone(),
                    type_annotation.clone(),
                    help_text.clone(),
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
                                TypedExpression,
                                result,
                                namespace.clone(),
                                Some(first_branch_result[0].return_type.clone()),
                                "All branches of a match expression must be of the same type.",
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
            }
            Expression::CodeBlock { contents, .. } => {
                let (typed_block, block_return_type) = type_check!(
                    TypedCodeBlock,
                    contents.clone(),
                    namespace.clone(),
                    type_annotation.clone(),
                    help_text.clone(),
                    (TypedCodeBlock { contents: vec![] }, TypeInfo::Unit),
                    warnings,
                    errors
                );
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
                    TypedExpression,
                    *condition,
                    namespace.clone(),
                    Some(TypeInfo::Boolean),
                    "The condition of an if expression must be a boolean expression.",
                    ERROR_RECOVERY_EXPR.clone(),
                    warnings,
                    errors
                ));
                let then = Box::new(type_check!(
                    TypedExpression,
                    *then,
                    namespace.clone(),
                    None,
                    "",
                    ERROR_RECOVERY_EXPR.clone(),
                    warnings,
                    errors
                ));
                let r#else = if let Some(expr) = r#else {
                    Some(Box::new(type_check!(
                        TypedExpression,
                        *expr,
                        namespace,
                        Some(then.return_type.clone()),
                        "",
                        ERROR_RECOVERY_EXPR.clone(),
                        warnings,
                        errors
                    )))
                } else {
                    None
                };

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
            a => {
                println!("Unimplemented: {:?}", a);
                errors.push(CompileError::Unimplemented(
                    "Unimplemented expression",
                    a.span(),
                ));

                ERROR_RECOVERY_EXPR
            }
        };
        // if the return type cannot be cast into the annotation type then it is a type error
        if let Some(type_annotation) = type_annotation {
            let convertability = typed_expression.return_type.clone().is_convertable(
                type_annotation.clone(),
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
        }
        ok(typed_expression, warnings, errors)
    }
}

impl<'sc> TypedCodeBlock<'sc> {
    fn type_check(
        other: CodeBlock<'sc>,
        namespace: HashMap<VarName<'sc>, TypedDeclaration<'sc>>,
        // this is for the return or implicit return
        type_annotation: Option<TypeInfo<'sc>>,
        help_text: impl Into<String> + Clone,
    ) -> CompileResult<'sc, (Self, TypeInfo<'sc>)> {
        // TODO implicit returns from blocks
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let mut evaluated_contents = Vec::new();
        let mut local_namespace = namespace.clone();
        let last_node = other
            .contents
            .last()
            .expect("empty code block? TODO check if this is handled earlier")
            .clone();
        for node in &other.contents[0..other.contents.len() - 1] {
            match type_check_node(node.clone(), &mut local_namespace, None, "") {
                CompileResult::Ok {
                    value,
                    warnings: mut l_w,
                    errors: mut l_e,
                } => {
                    warnings.append(&mut l_w);
                    errors.append(&mut l_e);
                    evaluated_contents.push(value);
                }
                CompileResult::Err {
                    errors: mut l_e,
                    warnings: mut l_w,
                } => {
                    warnings.append(&mut l_w);
                    errors.append(&mut l_e);
                }
            };
        }
        // now, handle the final line with the type annotation.
        let res = match type_check_node(
            last_node.clone(),
            &mut local_namespace,
            type_annotation.clone(),
            help_text.clone(),
        ) {
            CompileResult::Ok {
                value,
                errors: mut l_e,
                warnings: mut l_w,
            } => {
                warnings.append(&mut l_w);
                errors.append(&mut l_e);
                value
            }
            CompileResult::Err {
                errors: mut l_e,
                warnings: mut l_w,
            } => {
                warnings.append(&mut l_w);
                errors.append(&mut l_e);
                TypedAstNode {
                    content: ERROR_RECOVERY_NODE_CONTENT.clone(),
                    span: last_node.span,
                    scope: namespace.clone(),
                }
            }
        };
        evaluated_contents.push(res.clone());
        if let Some(type_annotation) = type_annotation {
            let convertability = res.type_info().is_convertable(
                type_annotation.clone(),
                res.span.clone(),
                help_text,
            );
            match convertability {
                Ok(warning) => {
                    if let Some(warning) = warning {
                        warnings.push(CompileWarning {
                            warning_content: warning,
                            span: res.span.clone(),
                        });
                    }
                }
                Err(err) => {
                    errors.push(err.into());
                }
            }
        }

        ok(
            (
                TypedCodeBlock {
                    contents: evaluated_contents,
                },
                res.type_info(),
            ),
            warnings,
            errors,
        )
    }
}

pub(crate) enum TreeType {
    Predicate,
    Script,
    Contract,
}

pub(crate) fn type_check_tree<'sc>(
    parsed: ParseTree<'sc>,
    tree_type: TreeType,
) -> CompileResult<'sc, TypedParseTree> {
    let typed_tree = parsed
        .root_nodes
        .into_iter()
        // this is the initialization of the global namespace
        // when we have actual default imports and stuff
        // this will be a clone of the initialized namespace
        // for now it is empty, i.e. `HashMap::default()`
        //
        // Top level functions are expected to return the Unit type, hence the annotation here
        .map(|node| type_check_node(node, &mut HashMap::default(), None, ""))
        .collect::<Vec<CompileResult<_>>>();

    let mut typed_tree_nodes = Vec::new();
    let mut warnings = Vec::new();
    let mut errors = Vec::new();
    for res in typed_tree {
        match res {
            CompileResult::Ok {
                value: node,
                warnings: mut l_w,
                errors: mut l_e,
            } => {
                errors.append(&mut l_e);
                warnings.append(&mut l_w);
                typed_tree_nodes.push(node);
            }
            CompileResult::Err {
                errors: mut l_e,
                warnings: mut l_w,
            } => {
                warnings.append(&mut l_w);
                errors.append(&mut l_e);
            }
        }
    }
    // perform validation based on the tree type
    match tree_type {
        TreeType::Predicate => {
            // a predicate must have a main function and that function must return a boolean
            let main_func_vec = typed_tree_nodes
                .iter()
                .filter_map(|TypedAstNode { content, .. }| match content {
                    TypedAstNodeContent::Declaration(TypedDeclaration::FunctionDeclaration(
                        TypedFunctionDeclaration {
                            name,
                            return_type,
                            span,
                            ..
                        },
                    )) => {
                        if name == &"main" {
                            Some((return_type, span))
                        } else {
                            None
                        }
                    }
                    _ => None,
                })
                .collect::<Vec<_>>();

            if main_func_vec.len() > 1 {
                errors.push(CompileError::MultiplePredicateMainFunctions(
                    main_func_vec.last().unwrap().1.clone(),
                ));
            } else if main_func_vec.is_empty() {
                errors.push(CompileError::NoPredicateMainFunction(parsed.span));
                return err(warnings, errors);
            }
            let main_func = main_func_vec[0];
            match main_func {
                (TypeInfo::Boolean, _span) => (),
                (_, span) => {
                    errors.push(CompileError::PredicateMainDoesNotReturnBool(span.clone()))
                }
            }
        },
        TreeType::Script =>  {
            // a script must have exactly one main function
            let main_func_vec = typed_tree_nodes
                .iter()
                .filter_map(|TypedAstNode { content, .. }| match content {
                    TypedAstNodeContent::Declaration(TypedDeclaration::FunctionDeclaration(
                        TypedFunctionDeclaration {
                            name,
                            return_type,
                            span,
                            ..
                        },
                    )) => {
                        if name == &"main" {
                            Some(span)
                        } else {
                            None
                        }
                    }
                    _ => None,
                })
                .collect::<Vec<_>>();

            if main_func_vec.len() > 1 {
                errors.push(CompileError::MultipleScriptMainFunctions(
                    main_func_vec.into_iter().last().unwrap().clone()
                ));
            } else if main_func_vec.is_empty() {
                errors.push(CompileError::NoScriptMainFunction(parsed.span));
                return err(warnings, errors);
            }

        }
        _ => (),
    }
    ok(
        TypedParseTree {
            root_nodes: typed_tree_nodes,
        },
        warnings,
        errors,
    )
}

fn type_check_node<'sc>(
    node: AstNode<'sc>,
    namespace: &mut HashMap<VarName<'sc>, TypedDeclaration<'sc>>,
    return_type_annotation: Option<TypeInfo<'sc>>,
    help_text: impl Into<String>,
) -> CompileResult<'sc, TypedAstNode<'sc>> {
    let mut warnings = Vec::new();
    let mut errors = Vec::new();
    let node = TypedAstNode {
        content: match node.content.clone() {
            AstNodeContent::UseStatement(a) => {
                errors.push(CompileError::Unimplemented(
                    "Use statements are unimplemented.",
                    node.span.clone(),
                ));
                ERROR_RECOVERY_NODE_CONTENT.clone()
            }
            AstNodeContent::Declaration(a) => TypedAstNodeContent::Declaration(match a {
                Declaration::VariableDeclaration(VariableDeclaration {
                    name,
                    type_ascription,
                    body,
                    is_mutable,
                }) => {
                    let body = type_check!(TypedExpression, body, namespace.clone(), type_ascription.clone(), 
                    format!("Variable declaration's type annotation (type {}) does not match up with the assigned expression's type.", type_ascription.map(|x| x.friendly_type_str()).unwrap_or("none".into())), ERROR_RECOVERY_EXPR.clone(), warnings, errors);
                    let body = TypedDeclaration::VariableDeclaration(TypedVariableDeclaration {
                        name: name.clone(),
                        body,
                        is_mutable,
                    });
                    namespace.insert(name, body.clone());
                    body
                }
                Declaration::EnumDeclaration(e) => {
                    let span = e.span.clone();
                    let primary_name = e.name;
                    let decl = TypedDeclaration::EnumDeclaration(e);
                    namespace.insert(
                        VarName {
                            primary_name,
                            sub_names: Vec::new(),
                            span,
                        },
                        decl.clone(),
                    );
                    decl
                }
                Declaration::FunctionDeclaration(FunctionDeclaration {
                    name,
                    body,
                    parameters,
                    span,
                    return_type,
                    type_parameters,
                    ..
                }) => {
                    // insert parameters into namespace
                    let mut namespace = namespace.clone();
                    parameters.clone().into_iter().for_each(
                        |FunctionParameter { name, r#type }| {
                            namespace.insert(
                                name.clone(),
                                TypedDeclaration::VariableDeclaration(TypedVariableDeclaration {
                                    name: name.clone(),
                                    body: TypedExpression {
                                        expression: TypedExpressionVariant::FunctionParameter,
                                        return_type: r#type,
                                        is_constant: IsConstant::No,
                                    },
                                    is_mutable: false, // TODO allow mutable function params?
                                }),
                            );
                        },
                    );
                    let (body, _implicit_block_return) = type_check!(TypedCodeBlock,
                        body,
                        namespace.clone(),
                        Some(return_type.clone()),
                        "Function body's return type does not match up with its return type annotation.",
                        (TypedCodeBlock { contents: vec![] }, TypeInfo::Unit), warnings ,errors
                    );
                    let decl = TypedDeclaration::FunctionDeclaration(TypedFunctionDeclaration {
                        name,
                        body,
                        parameters,
                        span: span.clone(),
                        return_type,
                        type_parameters,
                    });
                    namespace.insert(
                        VarName {
                            primary_name: name,
                            sub_names: Vec::new(),
                            span,
                        },
                        decl.clone(),
                    );
                    decl
                }
                Declaration::TraitDeclaration(TraitDeclaration {
                    name,
                    interface_surface,
                    methods,
                    type_parameters: _type_parameters
                }) => {
                    let mut methods_buf = Vec::new();
                    for FunctionDeclaration {
                        body,
                        name,
                        parameters,
                        span,
                        return_type,
                        type_parameters,
                        ..
                    } in methods
                    {
                        // TODO check code block implicit return
                        let (body, _code_block_implicit_return) = 
                                        type_check!(
                                            TypedCodeBlock,
                                           body,
                                            namespace.clone(),
                                            Some(return_type.clone()),
                                            "Trait method body's return type does not match up with its return type annotation.",
                                            continue, 
                                            warnings, errors
                                        );
                        methods_buf.push(TypedFunctionDeclaration {
                            name,
                            body,
                            parameters,
                            span,
                            return_type,
                            type_parameters,
                        });
                    }
                    let trait_decl = TypedDeclaration::TraitDeclaration(TypedTraitDeclaration {
                        name: name.clone(),
                        interface_surface,
                        methods: methods_buf,
                    });
                    namespace.insert(name, trait_decl.clone());
                    trait_decl
                }
                a => {
                    println!("Unimplemented: {:?}", a);
                    errors.push(CompileError::Unimplemented(
                        "Unimplemented declaration variant",
                        node.span.clone(),
                    ));

                    ERROR_RECOVERY_DECLARATION
                }
            }),
            AstNodeContent::TraitDeclaration(a) => TypedAstNodeContent::TraitDeclaration(a),
            AstNodeContent::Expression(a) => {
                let inner = type_check!(
                    TypedExpression,
                    a,
                    namespace.clone(),
                    None,
                    "",
                    ERROR_RECOVERY_EXPR.clone(),
                    warnings,
                    errors
                );
                TypedAstNodeContent::Expression(inner)
            }
            AstNodeContent::ReturnStatement(ReturnStatement { expr }) => {
                if return_type_annotation.is_none() {
                    errors.push(CompileError::Internal(
                        "Parsed a return type without an annotation. All returns should be typed. ",
                        node.span.clone(),
                    ));
                    ERROR_RECOVERY_NODE_CONTENT.clone()
                } else {
                    TypedAstNodeContent::ReturnStatement (TypedReturnStatement {
                        expr: type_check!(TypedExpression, expr, namespace.clone(), return_type_annotation, 
                        "Returned value must match up with the function return type annotation.",
                        ERROR_RECOVERY_EXPR.clone(), warnings, errors)
                    })
                }
            }
            AstNodeContent::ImplicitReturnExpression(expr) => {
                let typed_expr = type_check!(
                    TypedExpression,
                    expr,
                    namespace.clone(),
                    return_type_annotation,
                    format!(
                        "Implicit return must match up with block's type. {}",
                        help_text.into()
                    ),
                    ERROR_RECOVERY_EXPR.clone(),
                    warnings,
                    errors
                );
                TypedAstNodeContent::ImplicitReturnExpression(typed_expr)
            }
            a => {
                println!("Unimplemented: {:?}", a);
                errors.push(CompileError::Unimplemented(
                    "Unimplemented AST Node",
                    node.span.clone(),
                ));

                ERROR_RECOVERY_NODE_CONTENT
            }
        },
        span: node.span.clone(),
        scope: namespace.clone(),
    };

    ok(node, warnings, errors)
}
