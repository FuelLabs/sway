use crate::error::*;
use crate::parse_tree::*;
use crate::types::{IntegerBits, TypeInfo};
use crate::semantics::Namespace;
use crate::{AstNode, AstNodeContent, CodeBlock, ParseTree, ReturnStatement, TraitFn};
use either::Either;
use pest::Span;
use std::collections::HashMap;

mod code_block;
mod declaration;
mod expression;
mod return_statement;
mod while_loop;

use super::ERROR_RECOVERY_DECLARATION;
pub(crate) use code_block::TypedCodeBlock;
pub use declaration::{TypedDeclaration, TypedFunctionDeclaration};
pub(crate) use declaration::{TypedReassignment, TypedTraitDeclaration, TypedVariableDeclaration};
pub(crate) use expression::{TypedExpression, TypedExpressionVariant, ERROR_RECOVERY_EXPR};
use return_statement::TypedReturnStatement;
pub(crate) use while_loop::TypedWhileLoop;

pub(crate) const ERROR_RECOVERY_NODE_CONTENT: TypedAstNodeContent =
    TypedAstNodeContent::Expression(expression::ERROR_RECOVERY_EXPR);

/// whether or not something is constantly evaluatable (if the result is known at compile
/// time)
#[derive(Clone, Copy, Debug)]
pub(crate) enum IsConstant {
    Yes,
    No,
}

#[derive(Clone, Debug)]
pub(crate) enum TypedAstNodeContent<'sc> {
    UseStatement,
    //    CodeBlock(TypedCodeBlock<'sc>),
    ReturnStatement(TypedReturnStatement<'sc>),
    Declaration(TypedDeclaration<'sc>),
    Expression(TypedExpression<'sc>),
    TraitDeclaration(TraitDeclaration<'sc>),
    ImplicitReturnExpression(TypedExpression<'sc>),
    WhileLoop(TypedWhileLoop<'sc>),
    // a no-op node used for something that just issues a side effect, like an import statement.
    SideEffect,
}

#[derive(Clone, Debug)]
pub(crate) struct TypedAstNode<'sc> {
    pub(crate) content: TypedAstNodeContent<'sc>,
    pub(crate) span: Span<'sc>,
}

impl<'sc> TypedAstNode<'sc> {
    fn type_info(&self) -> TypeInfo<'sc> {
        // return statement should be ()
        use TypedAstNodeContent::*;
        match &self.content {
            UseStatement | ReturnStatement(_) | Declaration(_) | TraitDeclaration(_) => {
                TypeInfo::Unit
            }
            Expression(TypedExpression { return_type, .. }) => return_type.clone(),
            ImplicitReturnExpression(TypedExpression { return_type, .. }) => return_type.clone(),
            WhileLoop(_) | SideEffect => TypeInfo::Unit,
        }
    }
}

impl<'sc> TypedAstNode<'sc> {
    pub(crate) fn type_check<'manifest>(
        node: AstNode<'sc>,
        namespace: &mut Namespace<'sc>,
        return_type_annotation: Option<TypeInfo<'sc>>,
        help_text: impl Into<String>,
    ) -> CompileResult<'sc, TypedAstNode<'sc>> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let node = TypedAstNode {
            content: match node.content.clone() {
                AstNodeContent::UseStatement(a) => {
                    match a.import_type {
                        ImportType::Star => namespace.star_import(a.call_path),
                        ImportType::Item(s) => namespace.item_import(a.call_path, &s, None/*TODO support aliasing in grammar*/)
                    };
                    TypedAstNodeContent::SideEffect
                }
                AstNodeContent::Declaration(a) => TypedAstNodeContent::Declaration(match a {
                    Declaration::VariableDeclaration(VariableDeclaration {
                        name,
                        type_ascription,
                        body,
                        is_mutable,
                    }) => {
                        let body = type_check!(
                        TypedExpression::type_check(
                        body,
                        &namespace,
                        type_ascription.clone(), 
                        format!("Variable declaration's type annotation (type {}) does not match up with the assigned expression's type.",
                            type_ascription.map(|x| x.friendly_type_str()).unwrap_or("none".into()))),
                        ERROR_RECOVERY_EXPR.clone(),
                        warnings,
                        errors);
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
                        namespace.insert(Ident { primary_name, span }, decl.clone());
                        decl
                    }
                    Declaration::FunctionDeclaration(fn_decl) => {
                        let decl = type_check!(
                            TypedFunctionDeclaration::type_check(
                                fn_decl,
                                &namespace,
                                None,
                                ""
                            ),
                            return err(warnings, errors),
                            warnings,
                            errors
                        );

                        namespace.insert(
                            decl.name.clone(),
                            TypedDeclaration::FunctionDeclaration(decl.clone()),
                        );
                        TypedDeclaration::FunctionDeclaration(decl)
                    }
                    Declaration::TraitDeclaration(TraitDeclaration {
                        name,
                        interface_surface,
                        methods,
                        type_parameters,
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
                            let mut namespace = namespace.clone();
                            parameters.clone().into_iter().for_each(
                                |FunctionParameter { name, r#type, .. }| {
                                    namespace.insert(
                                        name.clone(),
                                        TypedDeclaration::VariableDeclaration(
                                            TypedVariableDeclaration {
                                                name: name.clone(),
                                                body: TypedExpression {
                                                    expression:
                                                        TypedExpressionVariant::FunctionParameter,
                                                    return_type: r#type,
                                                    is_constant: IsConstant::No,
                                                },
                                                is_mutable: false, // TODO allow mutable function params?
                                            },
                                        ),
                                    );
                                },
                            );
                            // check the generic types in the arguments, make sure they are in the type
                            // scope
                            let mut generic_params_buf_for_error_message = Vec::new();
                            for param in parameters.iter() {
                                if let TypeInfo::Generic { name } = param.r#type {
                                    generic_params_buf_for_error_message.push(name);
                                }
                            }
                            let comma_separated_generic_params =
                                generic_params_buf_for_error_message.join(", ");
                            for FunctionParameter {
                                ref r#type, name, ..
                            } in parameters.iter()
                            {
                                let span = name.span.clone();
                                if let TypeInfo::Generic { name, .. } = r#type {
                                    let args_span = parameters.iter().fold(
                                        parameters[0].name.span.clone(),
                                        |acc,
                                         FunctionParameter {
                                             name: Ident { span, .. },
                                             ..
                                         }| {
                                            crate::utils::join_spans(acc, span.clone())
                                        },
                                    );
                                    if type_parameters.iter().find(|x| x.name == *name).is_none() {
                                        errors.push(CompileError::TypeParameterNotInTypeScope {
                                            name,
                                            span: span.clone(),
                                            comma_separated_generic_params:
                                                comma_separated_generic_params.clone(),
                                            fn_name: name,
                                            args: args_span.as_str(),
                                            return_type: return_type.friendly_type_str(),
                                        });
                                    }
                                }
                            }
                            // TODO check code block implicit return
                            let (body, _code_block_implicit_return) = 
                                        type_check!(
                                            TypedCodeBlock::type_check(
                                            body,
                                            &namespace,
                                            Some(return_type.clone()),
                                            "Trait method body's return type does not match up with its return type annotation."),
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
                        let trait_decl =
                            TypedDeclaration::TraitDeclaration(TypedTraitDeclaration {
                                name: name.clone(),
                                interface_surface,
                                methods: methods_buf,
                                type_parameters,
                            });
                        namespace.insert(name, trait_decl.clone());
                        trait_decl
                    }
                    Declaration::Reassignment(Reassignment { lhs, rhs, span }) => {
                        // check that the reassigned name exists
                        let thing_to_reassign = match namespace.get_symbol(&lhs) {
                            Some(TypedDeclaration::VariableDeclaration(
                                TypedVariableDeclaration {
                                    body, is_mutable, ..
                                },
                            )) => {
                                // allow the type checking to continue unhindered even though this is
                                // an error
                                // basically pretending that this isn't an error by not
                                // early-returning, for the sake of better error reporting
                                if !is_mutable {
                                    errors.push(CompileError::AssignmentToNonMutable(
                                        lhs.primary_name,
                                        span,
                                    ));
                                }

                                body
                            }
                            Some(o) => {
                                errors.push(CompileError::ReassignmentToNonVariable {
                                    name: lhs.primary_name,
                                    kind: o.friendly_name(),
                                    span,
                                });
                                return err(warnings, errors);
                            }
                            None => {
                                errors.push(CompileError::UnknownVariable {
                                    var_name: lhs.primary_name,
                                    span: lhs.span,
                                });
                                return err(warnings, errors);
                            }
                        };
                        // type check the reassignment
                        let rhs = type_check!(
                            TypedExpression::type_check(
                                rhs,
                                &namespace,
                                Some(thing_to_reassign.return_type.clone()),
                                "You can only reassign a value of the same type to a variable."
                            ),
                            ERROR_RECOVERY_EXPR.clone(),
                            warnings,
                            errors
                        );

                        TypedDeclaration::Reassignment(TypedReassignment { lhs, rhs })
                    }
                    Declaration::ImplTrait(ImplTrait {
                        trait_name,
                        type_arguments,
                        functions,
                        type_implementing_for,
                        type_arguments_span,
                        block_span,
                    }) => match namespace.get_symbol(&trait_name) {
                        Some(TypedDeclaration::TraitDeclaration(tr)) => {
                            if type_arguments.len() != tr.type_parameters.len() {
                                errors.push(CompileError::IncorrectNumberOfTypeArguments {
                                    given: type_arguments.len(),
                                    expected: tr.type_parameters.len(),
                                    span: type_arguments_span,
                                })
                            }

                            // type check all components of the impl trait functions
                            let mut functions_buf: Vec<TypedFunctionDeclaration> = vec![];
                            // this list keeps track of the remaining functions in the
                            // interface surface that still need to be implemented for the
                            // trait to be fully implemented
                            let mut function_checklist: Vec<&Ident> = tr
                                .interface_surface
                                .iter()
                                .map(|TraitFn { name, .. }| name)
                                .collect();
                            for mut fn_decl in functions.into_iter() {
                                let mut type_arguments = type_arguments.clone();
                                // add generic params from impl trait into function type params
                                fn_decl.type_parameters.append(&mut type_arguments);
                                // ensure this fn decl's parameters and signature lines up with the one
                                // in the trait
                                if let Some(mut l_e) = tr.interface_surface.iter().find_map(|TraitFn { name, parameters, return_type }| {
                                if fn_decl.name == *name {
                                    let mut errors = vec![];
                                    if let Some(mut maybe_err) = parameters.iter().zip(fn_decl.parameters.iter()).find_map(|(fn_decl_param, trait_param)| {
                                    let mut errors = vec![];
                                    if let TypeInfo::Generic { .. /* TODO use trait constraints as part of the type here to implement trait constraint solver */ } = fn_decl_param.r#type {
                                        if let TypeInfo::Generic { .. } = trait_param.r#type {
                                            // nothing -- this is ok, no error
                                        } else 
                                        {
                                            errors.push(CompileError::MismatchedTypeInTrait {
                                                span: fn_decl_param.type_span.clone(),
                                                given: fn_decl_param.r#type.friendly_type_str(),
                                                expected: trait_param.r#type.friendly_type_str()
                                            });
                                        }
                                    } else {
                                        if fn_decl_param.r#type == trait_param.r#type  { /* nothing -- this is ok */ } else {
                                            errors.push(CompileError::MismatchedTypeInTrait {span: fn_decl_param.type_span.clone(),
                                            given: fn_decl_param.r#type.friendly_type_str(),
                                            expected: trait_param.r#type.friendly_type_str()});
                                        }
                                    }
                                    if errors.is_empty() { None } else { Some(errors) }
                                }) {
                                    errors.append(&mut maybe_err);
                                }
                                if fn_decl.return_type == *return_type {
                                    // nothing -- this is fine, no error
                                }
                                else {
                                    errors.push(CompileError::MismatchedTypeInTrait {
                                        span: fn_decl.return_type_span.clone(),
                                        expected: return_type.friendly_type_str(),
                                        given: fn_decl.return_type.friendly_type_str() 
                                    });
                                }
                                if errors.is_empty() { None } else { Some(errors) }
                            } else {
                                None 
                            } 
                        })
                        {
                            errors.append(&mut l_e);
                            continue;
                        }
                                // remove this function from the "checklist"
                                let ix_of_thing_to_remove = match function_checklist
                                    .iter()
                                    .position(|name| **name == fn_decl.name)
                                {
                                    Some(ix) => ix,
                                    None => {
                                        errors.push(
                                            CompileError::FunctionNotAPartOfInterfaceSurface {
                                                name: fn_decl.name.primary_name.clone(),
                                                trait_name: trait_name.primary_name.clone(),
                                                span: fn_decl.name.span.clone(),
                                            },
                                        );
                                        return err(warnings, errors);
                                    }
                                };
                                function_checklist.remove(ix_of_thing_to_remove);

                                // replace SelfType with type of implementor
                                // i.e. fn add(self, other: u64) -> Self becomes fn
                                // add(self: u64, other: u64) -> u64
                                if let Some(ix) = fn_decl.parameters.iter().position(
                                    |FunctionParameter { name, r#type, .. }| {
                                        r#type == &TypeInfo::SelfType
                                    },
                                ) {
                                    fn_decl.parameters[ix].r#type = type_implementing_for.clone();
                                }
                                if fn_decl.return_type == TypeInfo::SelfType {
                                    fn_decl.return_type = type_implementing_for.clone();
                                }

                                functions_buf.push(type_check!(
                                    TypedFunctionDeclaration::type_check(
                                        fn_decl,
                                        &namespace,
                                        None,
                                        ""
                                    ),
                                    continue,
                                    warnings,
                                    errors
                                ));
                            }

                            // check that the implementation checklist is complete
                            if !function_checklist.is_empty() {
                                errors.push(CompileError::MissingInterfaceSurfaceMethods {
                                    span: block_span,
                                    missing_functions: function_checklist
                                        .into_iter()
                                        .map(|Ident { primary_name, .. }| primary_name.to_string())
                                        .collect::<Vec<_>>()
                                        .join("\n"),
                                });
                            }

                            namespace.insert_trait_implementation(trait_name, type_implementing_for, functions_buf);
                            TypedDeclaration::SideEffect
                        }
                        Some(_) => {
                            errors.push(CompileError::NotATrait {
                                span: trait_name.span,
                                name: trait_name.primary_name,
                            });
                            ERROR_RECOVERY_DECLARATION.clone()
                        }
                        None => {
                            errors.push(CompileError::UnknownTrait {
                                name: trait_name.primary_name,
                                span: trait_name.span,
                            });
                            ERROR_RECOVERY_DECLARATION.clone()
                        }
                    },

                    Declaration::ImplSelf(ImplSelf {
                        type_arguments,
                        functions,
                        type_implementing_for,
                        type_arguments_span,
                        block_span,
                    }) => {
                        let mut functions_buf: Vec<TypedFunctionDeclaration> = vec![];
                        for mut fn_decl in functions.into_iter() {
                            let mut type_arguments = type_arguments.clone();
                            // add generic params from impl trait into function type params
                            fn_decl.type_parameters.append(&mut type_arguments);
                            // ensure this fn decl's parameters and signature lines up with the one
                            // in the trait

                            // replace SelfType with type of implementor
                            // i.e. fn add(self, other: u64) -> Self becomes fn
                            // add(self: u64, other: u64) -> u64
                            if let Some(ix) = fn_decl.parameters.iter().position(
                                |FunctionParameter { name, r#type, .. }| {
                                    r#type == &TypeInfo::SelfType
                                },
                            ) {
                                fn_decl.parameters[ix].r#type = type_implementing_for.clone();
                            }
                            if fn_decl.return_type == TypeInfo::SelfType {
                                fn_decl.return_type = type_implementing_for.clone();
                            }

                            functions_buf.push(type_check!(
                                TypedFunctionDeclaration::type_check(
                                    fn_decl,
                                    &namespace,
                                    None,
                                    ""
                                ),
                                continue,
                                warnings,
                                errors
                            ));
                        }

                        namespace.insert_trait_implementation(Ident { primary_name: "r#Self", span: block_span.clone() }, type_implementing_for, functions_buf);
                        TypedDeclaration::SideEffect
                    }
                    Declaration::StructDeclaration(decl) => {
                        // insert struct into namespace
                        namespace.insert(
                            decl.name.clone(),
                            TypedDeclaration::StructDeclaration(decl.clone()),
                        );

                        TypedDeclaration::StructDeclaration(decl)
                    }
                    a => {
                        dbg!("Unimplemented ast node (declaration): ", &a);
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
                        TypedExpression::type_check(
                            a,
                            &namespace,
                            None,
                            ""
                        ),
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
                        expr: type_check!(TypedExpression::type_check(
                                  expr,
                                  &namespace,
                                  return_type_annotation, 
                                  "Returned value must match up with the function return type annotation."),
                                  ERROR_RECOVERY_EXPR.clone(),
                                  warnings,
                                  errors)
                    })
                    }
                }
                AstNodeContent::ImplicitReturnExpression(expr) => {
                    let typed_expr = type_check!(
                        TypedExpression::type_check(
                            expr,
                            &namespace,
                            return_type_annotation,
                            format!(
                                "Implicit return must match up with block's type. {}",
                                help_text.into()
                            )
                        ),
                        ERROR_RECOVERY_EXPR.clone(),
                        warnings,
                        errors
                    );
                    TypedAstNodeContent::ImplicitReturnExpression(typed_expr)
                }
                AstNodeContent::WhileLoop(WhileLoop { condition, body }) => {
                    let typed_condition = type_check!(
                        TypedExpression::type_check(
                            condition,
                            &namespace,
                            Some(TypeInfo::Boolean),
                            "A while loop's loop condition must be a boolean expression."
                        ),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    let (typed_body, _block_implicit_return) = type_check!(
                    TypedCodeBlock::type_check(
                    body,
                    &namespace,
                    Some(TypeInfo::Unit),
                    "A while loop's loop body cannot implicitly return a value. Try assigning it to a mutable variable declared outside of the loop instead."),
                    (TypedCodeBlock { contents: vec![] }, TypeInfo::Unit),
                    warnings,
                    errors
                );
                    TypedAstNodeContent::WhileLoop(TypedWhileLoop {
                        condition: typed_condition,
                        body: typed_body,
                    })
                }
                a => {
                    dbg!("Unimplemented ast node content: ", &a);
                    errors.push(CompileError::Unimplemented(
                        "Unimplemented AST Node",
                        node.span.clone(),
                    ));

                    ERROR_RECOVERY_NODE_CONTENT
                }
            },
            span: node.span.clone(),
        };
        match node {
            TypedAstNode {
                content: TypedAstNodeContent::Expression(TypedExpression { .. }),
                ..
            } => {
                let warning = Warning::UnusedReturnValue {
                    r#type: node.type_info(),
                };
                assert_or_warn!(
                    node.type_info() == TypeInfo::Unit,
                    warnings,
                    node.span.clone(),
                    warning
                );
            }
            _ => (),
        }

        ok(node, warnings, errors)
    }
}
