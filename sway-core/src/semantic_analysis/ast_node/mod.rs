use super::ERROR_RECOVERY_DECLARATION;

use crate::{
    build_config::BuildConfig, control_flow_analysis::ControlFlowGraph, error::*, parse_tree::*,
    semantic_analysis::*, type_engine::*, AstNode, AstNodeContent, Ident, ReturnStatement,
};

use sway_types::span::{join_spans, Span};

use std::sync::Arc;

pub(crate) use crate::semantic_analysis::ast_node::declaration::ReassignmentLhs;

pub mod declaration;
pub(crate) use declaration::{
    OwnedTypedEnumVariant, OwnedTypedStructField, TypedReassignment, TypedTraitDeclaration,
    TypedVariableDeclaration, VariableMutability,
};
pub use declaration::{
    TypedAbiDeclaration, TypedConstantDeclaration, TypedDeclaration, TypedEnumDeclaration,
    TypedEnumVariant, TypedFunctionDeclaration, TypedFunctionParameter, TypedStructDeclaration,
    TypedStructField,
};

pub mod impl_trait;
pub(crate) use impl_trait::Mode;

mod code_block;
pub(crate) use code_block::TypedCodeBlock;

mod expression;
pub(crate) use expression::*;

mod return_statement;
pub(crate) use return_statement::TypedReturnStatement;

mod while_loop;
pub(crate) use while_loop::TypedWhileLoop;

/// whether or not something is constantly evaluatable (if the result is known at compile
/// time)
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub(crate) enum IsConstant {
    Yes,
    No,
}

#[derive(Clone, Debug)]
pub(crate) enum TypedAstNodeContent {
    ReturnStatement(TypedReturnStatement),
    Declaration(TypedDeclaration),
    Expression(TypedExpression),
    ImplicitReturnExpression(TypedExpression),
    WhileLoop(TypedWhileLoop),
    // a no-op node used for something that just issues a side effect, like an import statement.
    SideEffect,
}

#[derive(Clone)]
pub struct TypedAstNode {
    pub(crate) content: TypedAstNodeContent,
    pub(crate) span: Span,
}

impl std::fmt::Debug for TypedAstNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use TypedAstNodeContent::*;
        let text = match &self.content {
            ReturnStatement(TypedReturnStatement { ref expr }) => {
                format!("return {}", expr.pretty_print())
            }
            Declaration(ref typed_decl) => typed_decl.pretty_print(),
            Expression(exp) => exp.pretty_print(),
            ImplicitReturnExpression(exp) => format!("return {}", exp.pretty_print()),
            WhileLoop(w_loop) => w_loop.pretty_print(),
            SideEffect => "".into(),
        };
        f.write_str(&text)
    }
}

impl TypedAstNode {
    pub(crate) fn copy_types(&mut self, type_mapping: &[(TypeParameter, TypeId)]) {
        match self.content {
            TypedAstNodeContent::ReturnStatement(ref mut ret_stmt) => {
                ret_stmt.copy_types(type_mapping)
            }
            TypedAstNodeContent::ImplicitReturnExpression(ref mut exp) => {
                exp.copy_types(type_mapping)
            }
            TypedAstNodeContent::Declaration(ref mut decl) => decl.copy_types(type_mapping),
            TypedAstNodeContent::Expression(ref mut expr) => expr.copy_types(type_mapping),
            TypedAstNodeContent::WhileLoop(TypedWhileLoop {
                ref mut condition,
                ref mut body,
            }) => {
                condition.copy_types(type_mapping);
                body.copy_types(type_mapping);
            }
            TypedAstNodeContent::SideEffect => (),
        }
    }
    fn type_info(&self) -> TypeInfo {
        // return statement should be ()
        use TypedAstNodeContent::*;
        match &self.content {
            ReturnStatement(_) | Declaration(_) => TypeInfo::Tuple(Vec::new()),
            Expression(TypedExpression { return_type, .. }) => {
                crate::type_engine::look_up_type_id(*return_type)
            }
            ImplicitReturnExpression(TypedExpression { return_type, .. }) => {
                crate::type_engine::look_up_type_id(*return_type)
            }
            WhileLoop(_) | SideEffect => TypeInfo::Tuple(Vec::new()),
        }
    }
    pub(crate) fn type_check(
        arguments: TypeCheckArguments<'_, AstNode>,
    ) -> CompileResult<TypedAstNode> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let TypeCheckArguments {
            checkee: node,
            namespace,
            crate_namespace,
            return_type_annotation,
            help_text,
            self_type,
            build_config,
            dead_code_graph,
            opts,
            mode,
            ..
        } = arguments;
        let content = match node.content.clone() {
            AstNodeContent::UseStatement(a) => {
                let args = TypeCheckArguments {
                    checkee: a,
                    namespace,
                    crate_namespace,
                    return_type_annotation,
                    help_text,
                    self_type,
                    build_config,
                    dead_code_graph,
                    mode,
                    opts,
                };
                check!(
                    type_check_use_statement(args),
                    return err(warnings, errors),
                    warnings,
                    errors
                )
            }
            AstNodeContent::IncludeStatement(ref a) => {
                // Import the file, parse it, put it in the namespace under the module name (alias or
                // last part of the import by default)
                let _ = check!(
                    import_new_file(a, namespace, build_config, dead_code_graph),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                TypedAstNodeContent::SideEffect
            }
            AstNodeContent::Declaration(a) => {
                let args = TypeCheckArguments {
                    checkee: a,
                    namespace,
                    crate_namespace,
                    return_type_annotation,
                    help_text,
                    self_type,
                    build_config,
                    dead_code_graph,
                    mode,
                    opts,
                };
                let decl = check!(
                    TypedDeclaration::type_check(args, node.span.clone()),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                TypedAstNodeContent::Declaration(decl)
            }
            AstNodeContent::Expression(a) => {
                let args = TypeCheckArguments {
                    checkee: a.clone(),
                    namespace,
                    crate_namespace,
                    return_type_annotation: insert_type(TypeInfo::Unknown),
                    help_text: Default::default(),
                    self_type,
                    build_config,
                    dead_code_graph,
                    mode: Mode::NonAbi,
                    opts,
                };
                let inner = check!(
                    TypedExpression::type_check(args),
                    error_recovery_expr(a.span()),
                    warnings,
                    errors
                );
                TypedAstNodeContent::Expression(inner)
            }
            AstNodeContent::ReturnStatement(ReturnStatement { expr }) => {
                let args = TypeCheckArguments {
                    checkee: expr.clone(),
                    namespace,
                    crate_namespace,
                    return_type_annotation,
                    help_text: "Returned value must match up with the function return type \
                     annotation.",
                    self_type,
                    build_config,
                    dead_code_graph,
                    mode: Mode::NonAbi,
                    opts,
                };
                let expr = check!(
                    TypedExpression::type_check(args),
                    error_recovery_expr(expr.span()),
                    warnings,
                    errors
                );
                TypedAstNodeContent::ReturnStatement(TypedReturnStatement { expr })
            }
            AstNodeContent::ImplicitReturnExpression(expr) => {
                let args = TypeCheckArguments {
                    checkee: expr.clone(),
                    namespace,
                    crate_namespace,
                    return_type_annotation,
                    help_text: "Implicit return must match up with block's type.",
                    self_type,
                    build_config,
                    dead_code_graph,
                    mode: Mode::NonAbi,
                    opts,
                };
                let typed_expr = check!(
                    TypedExpression::type_check(args),
                    error_recovery_expr(expr.span()),
                    warnings,
                    errors
                );
                TypedAstNodeContent::ImplicitReturnExpression(typed_expr)
            }
            AstNodeContent::WhileLoop(WhileLoop { condition, body }) => {
                let args = TypeCheckArguments {
                    checkee: (condition, body),
                    namespace,
                    crate_namespace,
                    return_type_annotation,
                    help_text,
                    self_type,
                    build_config,
                    dead_code_graph,
                    mode,
                    opts,
                };
                check!(
                    type_check_while_loop(args),
                    return err(warnings, errors),
                    warnings,
                    errors
                )
            }
        };

        let node = TypedAstNode {
            content,
            span: node.span,
        };

        if let TypedAstNode {
            content: TypedAstNodeContent::Expression(TypedExpression { .. }),
            ..
        } = node
        {
            let warning = Warning::UnusedReturnValue {
                r#type: node.type_info(),
            };
            assert_or_warn!(
                node.type_info().is_unit() || node.type_info() == TypeInfo::ErrorRecovery,
                warnings,
                node.span.clone(),
                warning
            );
        }

        ok(node, warnings, errors)
    }
}

fn type_check_use_statement(
    arguments: TypeCheckArguments<'_, UseStatement>,
) -> CompileResult<TypedAstNodeContent> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let TypeCheckArguments {
        checkee: a,
        namespace,
        crate_namespace,
        ..
    } = arguments;
    let from_module = if a.is_absolute {
        Some(crate_namespace)
    } else {
        None
    };
    let res = match a.import_type {
        ImportType::Star => namespace.star_import(from_module, a.call_path),
        ImportType::SelfImport => namespace.self_import(from_module, a.call_path, a.alias),
        ImportType::Item(s) => namespace.item_import(from_module, a.call_path, &s, a.alias),
    };
    check!(res, (), warnings, errors);
    ok(TypedAstNodeContent::SideEffect, warnings, errors)
}

fn type_check_while_loop(
    arguments: TypeCheckArguments<'_, (Expression, CodeBlock)>,
) -> CompileResult<TypedAstNodeContent> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let TypeCheckArguments {
        checkee: (condition, body),
        namespace,
        crate_namespace,
        self_type,
        build_config,
        dead_code_graph,
        opts,
        ..
    } = arguments;
    let condition_args = TypeCheckArguments {
        checkee: condition,
        namespace,
        crate_namespace,
        return_type_annotation: insert_type(TypeInfo::Boolean),
        help_text: "A while loop's loop condition must be a boolean expression.",
        self_type,
        build_config,
        dead_code_graph,
        mode: Mode::NonAbi,
        opts,
    };
    let typed_condition = check!(
        TypedExpression::type_check(condition_args),
        return err(warnings, errors),
        warnings,
        errors
    );
    let body_args = TypeCheckArguments {
        checkee: body.clone(),
        namespace,
        crate_namespace,
        return_type_annotation: insert_type(TypeInfo::Tuple(Vec::new())),
        help_text: "A while loop's loop body cannot implicitly return a value.Try \
         assigning it to a mutable variable declared outside of the loop \
         instead.",
        self_type,
        build_config,
        dead_code_graph,
        mode: Mode::NonAbi,
        opts,
    };
    let (typed_body, _block_implicit_return) = check!(
        TypedCodeBlock::type_check(body_args),
        (
            TypedCodeBlock {
                contents: vec![],
                whole_block_span: body.whole_block_span,
            },
            crate::type_engine::insert_type(TypeInfo::Tuple(Vec::new()))
        ),
        warnings,
        errors
    );
    let content = TypedAstNodeContent::WhileLoop(TypedWhileLoop {
        condition: typed_condition,
        body: typed_body,
    });
    ok(content, warnings, errors)
}

/// Imports a new file, populates the given [Namespace] with its content,
/// and appends the module's content to the control flow graph for later analysis.
fn import_new_file(
    statement: &IncludeStatement,
    namespace: NamespaceRef,
    build_config: &BuildConfig,
    dead_code_graph: &mut ControlFlowGraph,
) -> CompileResult<()> {
    let mut warnings = vec![];
    let mut errors = vec![];

    let mut canonical_path = (*build_config.dir_of_code).clone();
    canonical_path.push(statement.path_span.as_str());
    canonical_path.set_extension(crate::constants::DEFAULT_FILE_EXTENSION);

    let file_name = match canonical_path.strip_prefix(build_config.manifest_path.parent().unwrap())
    {
        Ok(file_name) => Arc::new(file_name.to_path_buf()),
        Err(_) => return err(warnings, errors),
    };

    let res = if canonical_path.exists() {
        std::fs::read_to_string(&*canonical_path)
    } else {
        errors.push(CompileError::FileNotFound {
            span: statement.path_span.clone(),
            file_path: canonical_path.to_string_lossy().to_string(),
        });
        return ok((), warnings, errors);
    };

    let file_as_string = match res {
        Ok(s) => Arc::from(s),
        Err(e) => {
            errors.push(CompileError::FileCouldNotBeRead {
                span: statement.path_span.clone(),
                file_path: canonical_path.to_string_lossy().to_string(),
                stringified_error: e.to_string(),
            });
            return ok((), warnings, errors);
        }
    };

    let mut dep_config = build_config.clone();
    let dep_path = {
        canonical_path.pop();
        canonical_path
    };
    dep_config.file_name = file_name;
    dep_config.dir_of_code = Arc::new(dep_path);
    let dep_namespace = create_new_scope(namespace);
    let crate::InnerDependencyCompileResult {
        name,
        namespace: module,
        ..
    } = check!(
        crate::compile_inner_dependency(file_as_string, dep_namespace, dep_config, dead_code_graph),
        return err(warnings, errors),
        warnings,
        errors
    );

    let name = match statement.alias {
        Some(ref alias) => alias,
        None => &name,
    };
    let name = name.as_str().to_string();
    namespace.insert_module(name, module);
    ok((), warnings, errors)
}

fn reassignment(
    arguments: TypeCheckArguments<'_, (Box<Expression>, Expression)>,
    span: Span,
) -> CompileResult<TypedDeclaration> {
    let TypeCheckArguments {
        checkee: (lhs, rhs),
        namespace,
        crate_namespace,
        self_type,
        build_config,
        dead_code_graph,
        opts,
        ..
    } = arguments;
    let mut errors = vec![];
    let mut warnings = vec![];
    // ensure that the lhs is a variable expression or struct field access
    match *lhs {
        Expression::VariableExpression { name, span } => {
            // check that the reassigned name exists
            let thing_to_reassign = match namespace.clone().get_symbol(&name).value {
                Some(TypedDeclaration::VariableDeclaration(TypedVariableDeclaration {
                    body,
                    is_mutable,
                    name,
                    ..
                })) => {
                    if !is_mutable.is_mutable() {
                        errors.push(CompileError::AssignmentToNonMutable(
                            name.as_str().to_string(),
                            span.clone(),
                        ));
                    }

                    body
                }
                Some(o) => {
                    errors.push(CompileError::ReassignmentToNonVariable {
                        name: name.clone(),
                        kind: o.friendly_name(),
                        span,
                    });
                    return err(warnings, errors);
                }
                None => {
                    errors.push(CompileError::UnknownVariable {
                        var_name: name.as_str().to_string(),
                        span: name.span().clone(),
                    });
                    return err(warnings, errors);
                }
            };
            // the RHS is a ref type to the LHS
            let rhs_type_id = insert_type(TypeInfo::Ref(thing_to_reassign.return_type));
            // type check the reassignment
            let rhs = check!(
                TypedExpression::type_check(TypeCheckArguments {
                    checkee: rhs,
                    namespace,
                    crate_namespace,
                    return_type_annotation: rhs_type_id,
                    help_text: "You can only reassign a value of the same type to a variable.",
                    self_type,
                    build_config,
                    dead_code_graph,
                    mode: Mode::NonAbi,
                    opts
                }),
                error_recovery_expr(span),
                warnings,
                errors
            );

            ok(
                TypedDeclaration::Reassignment(TypedReassignment {
                    lhs: vec![ReassignmentLhs {
                        name,
                        r#type: thing_to_reassign.return_type,
                    }],
                    rhs,
                }),
                warnings,
                errors,
            )
        }
        Expression::SubfieldExpression {
            prefix,
            field_to_access,
            span,
        } => {
            let mut expr = *prefix;
            let mut names_vec = vec![];
            let final_return_type = loop {
                let type_checked = check!(
                    TypedExpression::type_check(TypeCheckArguments {
                        checkee: expr.clone(),
                        namespace,
                        crate_namespace,
                        return_type_annotation: insert_type(TypeInfo::Unknown),
                        help_text: Default::default(),
                        self_type,
                        build_config,
                        dead_code_graph,
                        mode: Mode::NonAbi,
                        opts
                    }),
                    error_recovery_expr(expr.span()),
                    warnings,
                    errors
                );

                match expr {
                    Expression::VariableExpression { name, .. } => {
                        names_vec.push(ReassignmentLhs {
                            name,
                            r#type: type_checked.return_type,
                        });
                        break type_checked.return_type;
                    }
                    Expression::SubfieldExpression {
                        field_to_access,
                        prefix,
                        ..
                    } => {
                        names_vec.push(ReassignmentLhs {
                            name: field_to_access,
                            r#type: type_checked.return_type,
                        });
                        expr = *prefix;
                    }
                    _ => {
                        errors.push(CompileError::InvalidExpressionOnLhs { span });
                        return err(warnings, errors);
                    }
                }
            };

            let mut names_vec = names_vec.into_iter().rev().collect::<Vec<_>>();
            names_vec.push(ReassignmentLhs {
                name: field_to_access,
                r#type: final_return_type,
            });

            let (ty_of_field, _ty_of_parent) = check!(
                namespace.find_subfield_type(
                    names_vec
                        .iter()
                        .map(|ReassignmentLhs { name, .. }| name.clone())
                        .collect::<Vec<_>>()
                        .as_slice()
                ),
                return err(warnings, errors),
                warnings,
                errors
            );
            // type check the reassignment
            let rhs = check!(
                TypedExpression::type_check(TypeCheckArguments {
                    checkee: rhs,
                    namespace,
                    crate_namespace,
                    return_type_annotation: ty_of_field,
                    help_text: Default::default(),
                    self_type,
                    build_config,
                    dead_code_graph,
                    mode: Mode::NonAbi,
                    opts,
                }),
                error_recovery_expr(span),
                warnings,
                errors
            );

            ok(
                TypedDeclaration::Reassignment(TypedReassignment {
                    lhs: names_vec,
                    rhs,
                }),
                warnings,
                errors,
            )
        }
        _ => {
            errors.push(CompileError::InvalidExpressionOnLhs { span });
            err(warnings, errors)
        }
    }
}
