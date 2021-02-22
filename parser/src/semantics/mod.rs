use crate::parse_tree::*;
use either::Either;
use crate::{AstNode, AstNodeContent, CodeBlock, ParseTree, ReturnStatement};
use std::collections::HashMap;
mod error;
use error::CompileError;
use pest::Span;
pub(crate) struct TypedParseTree<'sc> {
    root_nodes: Vec<TypedAstNode<'sc>>,
}

#[derive(Clone)]
pub(crate) struct TypedAstNode<'sc> {
    content: TypedAstNodeContent<'sc>,
    span: Span<'sc>,
}

#[derive(Clone)]
pub(crate) enum TypedAstNodeContent<'sc> {
    UseStatement(UseStatement<'sc>),
    CodeBlock(TypedCodeBlock<'sc>),
    ReturnStatement(ReturnStatement<'sc>),
    Declaration(Declaration<'sc>),
    Expression(TypedExpression<'sc>),
    TraitDeclaration(TraitDeclaration<'sc>),
}

/// whether or not something is constantly evaluatable (if the result is known at compile
/// time)
#[derive(Clone, Copy)]
enum IsConstant {
    Yes,
    No,
}

#[derive(Clone)]
pub(crate) enum TypedDeclaration<'sc> {
    VariableDeclaration(TypedVariableDeclaration<'sc>),
    FunctionDeclaration(FunctionDeclaration<'sc>),
    TraitDeclaration(TraitDeclaration<'sc>),
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

#[derive(Clone)]
pub(crate) struct TypedVariableDeclaration<'sc> {
    pub(crate) name: &'sc str,
    pub(crate) body: TypedExpression<'sc>, // will be codeblock variant
    pub(crate) is_mutable: bool,
}

#[derive(Clone)]
pub(crate) struct TypedExpression<'sc> {
    expression: TypedExpressionVariant<'sc>,
    return_type: TypeInfo<'sc>,
    /// whether or not this expression is constantly evaluatable (if the result is known at compile
    /// time)
    is_constant: IsConstant,
}
#[derive(Clone)]
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
#[derive( Clone)]
pub(crate) struct TypedStructExpressionField<'sc> {
    name: &'sc str,
    value: TypedExpression<'sc>,
}
#[derive( Clone)]
pub(crate) struct TypedMatchBranch<'sc> {
    condition: TypedMatchCondition<'sc>,
    result: Either<TypedCodeBlock<'sc>, TypedExpression<'sc>>,
}

#[derive( Clone)]
pub(crate) enum TypedMatchCondition<'sc> {
    CatchAll,
    Expression(TypedExpression<'sc>),
}
#[derive( Clone)]
pub(crate) struct TypedCodeBlock<'sc> {
    contents: Vec<TypedAstNode<'sc>>,
    scope: HashMap<&'sc str, TypedDeclaration<'sc>>,
}

impl<'sc> TypedExpression<'sc> {
    fn type_check(
        other: Expression<'sc>,
        namespace: HashMap<VarName<'sc>, TypedDeclaration<'sc>>,
        type_annotation: Option<&TypeInfo>
    ) -> Result<Self, CompileError<'sc>> {
        let typed_expression =  match other.clone() {
            Expression::Literal(lit) =>{ 
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
                    is_constant: IsConstant::Yes
                }
            },
            Expression::VariableExpression {
                name, name_span, unary_op
            } => match namespace.get(&name) {
                Some(TypedDeclaration::VariableDeclaration(TypedVariableDeclaration {
                    body:
                        TypedExpression {
                            return_type,
                            is_constant,
                            expression: TypedExpressionVariant::VariableExpression { unary_op, name, name_span }
                        },
                    ..
                })) => TypedExpression {
                            return_type: return_type.clone(),
                            is_constant: *is_constant,
                            expression: TypedExpressionVariant::VariableExpression { unary_op: unary_op.clone(), name: name.clone(), name_span: name_span.clone() }
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
            Expression::FunctionApplication { name, arguments } => {
                let function_declaration = namespace.get(&name);
                match function_declaration {
                    Some(TypedDeclaration::FunctionDeclaration(FunctionDeclaration {
                        parameters,
                        return_type,
                        ..
                    })) => {
                        // type check arguments in function application vs arguments in function
                        // declaration. Use parameter type annotations as annotations for the
                        // arguments
                        //
                        // namespace clone is necessary so interior mutations to namespace scope
                        // don't impact outer scope
                        let typed_call_arguments = arguments.into_iter().zip(parameters.iter()).map(|(arg, param)| TypedExpression::type_check(arg, namespace.clone(), Some(&param.r#type))).collect::<Result<Vec<_>, _>>()?;

                        TypedExpression {
                            return_type: return_type.clone(),
                            // now check the function call return type
                            // FEATURE this IsConstant can be true if the function itself is constant-able
                            // const functions would be an advanced feature and are not supported right
                            // now
                            is_constant: IsConstant::No,
                            expression: TypedExpressionVariant::FunctionApplication {
                                arguments: typed_call_arguments,
                                name: name.clone()
                            }
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

            _ => todo!(),
        };
        // if the return type cannot be cast into the annotation type then it is a type error
        if !typed_expression.return_type.is_convertable(&type_annotation) {
            return Err(todo!("type error"))
        }
        Ok(typed_expression)
    }
}

fn type_check_tree<'sc>(parsed: ParseTree<'sc>) -> TypedParseTree<'sc> {
    let typed_tree = parsed
        .root_nodes
        .into_iter()
        .map(|node| type_check_node(node))
        .collect::<Vec<_>>();
    todo!()
}

fn type_check_node<'sc>(node: AstNode<'sc>) -> Result<TypedAstNode<'sc>, CompileError> {
    let mut namespace = HashMap::default();
    Ok(TypedAstNode {
        content: match node.content {
            AstNodeContent::UseStatement(a) => {
                todo!("Insert things from use statement into namespace")
            }
            AstNodeContent::Declaration(a) => todo!("Insert into namespace"),
            AstNodeContent::TraitDeclaration(a) => TypedAstNodeContent::TraitDeclaration(a),
            AstNodeContent::Expression(a) => {
                TypedAstNodeContent::Expression(TypedExpression::type_check(a, namespace, None)?)
            }
            _ => todo!(),
        },
        span: node.span,
    })
}
