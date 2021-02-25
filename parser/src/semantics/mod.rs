use crate::error::CompileWarning;
use crate::parse_tree::*;
use crate::types::{IntegerBits, TypeInfo};
use crate::{AstNode, AstNodeContent, CodeBlock, ParseTree, ReturnStatement, TraitFn};
use either::Either;
use std::collections::HashMap;
pub(crate) mod error;
use error::CompileError;
use pest::Span;
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

#[derive(Clone, Debug)]
pub(crate) enum TypedAstNodeContent<'sc> {
    UseStatement(UseStatement<'sc>),
    //    CodeBlock(TypedCodeBlock<'sc>),
    ReturnStatement(TypedReturnStatement<'sc>),
    Declaration(TypedDeclaration<'sc>),
    Expression(TypedExpression<'sc>),
    TraitDeclaration(TraitDeclaration<'sc>),
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
        name_span: Span<'sc>,
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
    return_type: TypeInfo<'sc>,
}

impl<'sc> TypedExpression<'sc> {
    fn type_check(
        other: Expression<'sc>,
        namespace: HashMap<VarName<'sc>, TypedDeclaration<'sc>>,
        type_annotation: Option<TypeInfo<'sc>>,
    ) -> Result<(Self, Vec<CompileWarning<'sc>>), CompileError<'sc>> {
        let mut warnings = Vec::new();
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
            Expression::VariableExpression {
                name,
                name_span,
                unary_op,
                ..
            } => match namespace.get(&name) {
                Some(TypedDeclaration::VariableDeclaration(TypedVariableDeclaration {
                    body:
                        TypedExpression {
                            return_type,
                            is_constant,
                            expression:
                                TypedExpressionVariant::VariableExpression {
                                    unary_op,
                                    name,
                                    name_span,
                                },
                        },
                    ..
                })) => TypedExpression {
                    return_type: return_type.clone(),
                    is_constant: *is_constant,
                    expression: TypedExpressionVariant::VariableExpression {
                        unary_op: unary_op.clone(),
                        name: name.clone(),
                        name_span: name_span.clone(),
                    },
                },
                Some(a) => {
                    return Err(CompileError::NotAVariable {
                        name: name_span.as_str(),
                        span: name_span,
                        what_it_is: a.friendly_name(),
                    })
                }
                None => {
                    return Err(CompileError::UnknownVariable {
                        var_name: name_span.as_str(),
                        span: name_span,
                    })
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
                        let typed_call_arguments = arguments
                            .into_iter()
                            .zip(parameters.iter())
                            .map(|(arg, param)| {
                                TypedExpression::type_check(
                                    arg,
                                    namespace.clone(),
                                    Some(param.r#type.clone()),
                                )
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        let (typed_call_arguments, mut l_warnings): (
                            _,
                            Vec<Vec<CompileWarning<'_>>>,
                        ) = typed_call_arguments.into_iter().unzip();
                        let mut warn_buf = Vec::new();
                        for mut l_warning in l_warnings {
                            warn_buf.append(&mut l_warning);
                        }

                        warnings.append(&mut warn_buf);

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
                        return Err(CompileError::NotAFunction {
                            name: name.span.as_str(),
                            span: name.span,
                            what_it_is: a.friendly_name(),
                        })
                    }
                    None => {
                        return Err(CompileError::UnknownFunction {
                            name: name.span.as_str(),
                            span: name.span,
                        })
                    }
                }
            }
            Expression::MatchExpression {
                primary_expression,
                branches,
                ..
            } => {
                let (typed_primary_expression, mut l_warnings) =
                    TypedExpression::type_check(*primary_expression, namespace.clone(), None)?;
                warnings.append(&mut l_warnings);

                // TODO handle pattern matching on LHS
                let (first_branch_result, mut l_warnings) = TypedExpression::type_check(
                    branches[0].result.clone(),
                    namespace.clone(),
                    type_annotation.clone(),
                )?;
                warnings.append(&mut l_warnings);
                let first_branch_result = vec![first_branch_result];
                // use type of first branch for annotation on the rest of the branches
                let rest_of_branches = branches
                    .into_iter()
                    .skip(1)
                    .map(
                        |MatchBranch {
                             condition, result, ..
                         }| {
                            TypedExpression::type_check(
                                result,
                                namespace.clone(),
                                Some(first_branch_result[0].return_type.clone()),
                            )
                        },
                    )
                    .collect::<Result<Vec<_>, _>>()?;

                let (mut rest_of_branches, mut l_warnings): (_, Vec<Vec<CompileWarning<'_>>>) =
                    rest_of_branches.into_iter().unzip();
                let mut warn_buf = Vec::new();
                for mut l_warning in l_warnings {
                    warn_buf.append(&mut l_warning);
                }

                warnings.append(&mut warn_buf);
                let mut all_branches = first_branch_result;
                all_branches.append(&mut rest_of_branches);

                todo!()
            }
            _ => todo!(),
        };
        // if the return type cannot be cast into the annotation type then it is a type error
        if let Some(type_annotation) = type_annotation {
            let convertability = typed_expression
                .return_type
                .clone()
                .is_convertable(type_annotation.clone(), expr_span.clone());
            match convertability {
                Ok(warning) => {
                    if let Some(warning) = warning {
                        warnings.push(CompileWarning {
                            warning_content: warning,
                            span: expr_span,
                        });
                    }
                }
                Err(err) => Err(err)?,
            }
        }
        Ok((typed_expression, warnings))
    }
}

impl<'sc> TypedCodeBlock<'sc> {
    fn type_check(
        other: CodeBlock<'sc>,
        namespace: HashMap<VarName<'sc>, TypedDeclaration<'sc>>,
        // this is for the return or implicit return
        type_annotation: Option<TypeInfo<'sc>>,
    ) -> Result<(Self, Vec<CompileWarning<'sc>>), CompileError<'sc>> {
        // TODO implicit returns from blocks
        let mut warnings = Vec::new();
        let mut evaluated_contents = Vec::new();
        let mut local_namespace = namespace.clone();
        let last_node = other
            .contents
            .last()
            .expect("empty code block? TODO check if this is handled earlier")
            .clone();
        for node in &other.contents[0..other.contents.len() - 1] {
            dbg!(&node);
            let (res, mut l_warnings) = type_check_node(node.clone(), &mut local_namespace, None)?;
            warnings.append(&mut l_warnings);
            evaluated_contents.push(res);
        }
        // now, handle the final line with the type annotation.
        let (res, mut l_warnings) =
            type_check_node(last_node, &mut local_namespace, type_annotation)?;
        warnings.append(&mut l_warnings);
        evaluated_contents.push(res);

        Ok((
            TypedCodeBlock {
                contents: evaluated_contents,
                return_type: TypeInfo::Unit, // TODO return type of code block
            },
            warnings,
        ))
    }
}

pub(crate) fn type_check_tree<'sc>(
    parsed: ParseTree<'sc>,
) -> Result<(TypedParseTree<'sc>, Vec<CompileWarning<'sc>>), CompileError<'sc>> {
    let typed_tree = parsed
        .root_nodes
        .into_iter()
        // this is the initialization of the global namespace
        // when we have actual default imports and stuff
        // this will be a clone of the initialized namespace
        // for now it is empty, i.e. `HashMap::default()`
        //
        // Top level functions are expected to return the Unit type, hence the annotation here
        .map(|node| type_check_node(node, &mut HashMap::default(), None))
        .collect::<Result<Vec<_>, _>>()?;
    let mut typed_tree_nodes = Vec::new();
    let mut warnings = Vec::new();
    for (node, mut l_warnings) in typed_tree {
        warnings.append(&mut l_warnings);
        typed_tree_nodes.push(node);
    }
    Ok((
        TypedParseTree {
            root_nodes: typed_tree_nodes,
        },
        warnings,
    ))
}

fn type_check_node<'sc>(
    node: AstNode<'sc>,
    namespace: &mut HashMap<VarName<'sc>, TypedDeclaration<'sc>>,
    return_type_annotation: Option<TypeInfo<'sc>>,
) -> Result<(TypedAstNode<'sc>, Vec<CompileWarning<'sc>>), CompileError<'sc>> {
    let mut warnings = Vec::new();
    let mut namespace = HashMap::default();
    Ok((
        TypedAstNode {
            content: match node.content {
                AstNodeContent::UseStatement(a) => {
                    todo!("Insert things from use statement into namespace")
                }
                AstNodeContent::Declaration(a) => TypedAstNodeContent::Declaration(match a {
                    Declaration::VariableDeclaration(VariableDeclaration {
                        name,
                        type_ascription,
                        body,
                        is_mutable,
                    }) => {
                        let (body, mut l_warnings) =
                            TypedExpression::type_check(body, namespace.clone(), type_ascription)?;
                        warnings.append(&mut l_warnings);
                        let body =
                            TypedDeclaration::VariableDeclaration(TypedVariableDeclaration {
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
                    }) => {
                        let (body, mut l_warnings) = TypedCodeBlock::type_check(
                            body,
                            namespace.clone(),
                            Some(return_type.clone()),
                        )?;
                        warnings.append(&mut l_warnings);
                        let decl =
                            TypedDeclaration::FunctionDeclaration(TypedFunctionDeclaration {
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
                                span: span,
                            },
                            decl.clone(),
                        );
                        decl
                    }
                    Declaration::TraitDeclaration(TraitDeclaration {
                        name,
                        interface_surface,
                        methods,
                    }) => {
                        let methods = methods
                            .into_iter()
                            .map(|x| {
                                Ok(TypedFunctionDeclaration {
                                    name: x.name,
                                    body: {
                                        let (block, mut l_warnings) = TypedCodeBlock::type_check(
                                            x.body,
                                            namespace.clone(),
                                            Some(x.return_type.clone()),
                                        )?;
                                        warnings.append(&mut l_warnings);
                                        block
                                    },
                                    parameters: x.parameters,
                                    span: x.span,
                                    return_type: x.return_type,
                                    type_parameters: x.type_parameters,
                                })
                            })
                            .collect::<Result<Vec<_>, CompileError>>()?;
                        let trait_decl =
                            TypedDeclaration::TraitDeclaration(TypedTraitDeclaration {
                                name: name.clone(),
                                interface_surface,
                                methods,
                            });
                        namespace.insert(name, trait_decl.clone());
                        trait_decl
                    }
                    a => todo!("{:?}", a),
                }),
                AstNodeContent::TraitDeclaration(a) => TypedAstNodeContent::TraitDeclaration(a),
                AstNodeContent::Expression(a) => {
                    let (inner, mut l_warnings) =
                        TypedExpression::type_check(a, namespace.clone(), None)?;
                    warnings.append(&mut l_warnings);
                    TypedAstNodeContent::Expression(inner)
                }
                AstNodeContent::ReturnStatement(ReturnStatement { expr }) => {
                    if return_type_annotation.is_none() {
                        return Err(CompileError::Internal("Parsed a return type without an annotation. All returns should be typed. ", node.span));
                    }
                    let (res, mut l_warnings) = TypedExpression::type_check(
                        expr,
                        namespace.clone(),
                        return_type_annotation,
                    )?;
                    warnings.append(&mut l_warnings);

                    TypedAstNodeContent::ReturnStatement(TypedReturnStatement { expr: res })
                }
                a => todo!("{:?}", a),
            },
            span: node.span,
            scope: namespace.clone(),
        },
        warnings,
    ))
}
