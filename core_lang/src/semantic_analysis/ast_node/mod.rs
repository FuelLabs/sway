use crate::parse_tree::*;
use crate::semantic_analysis::Namespace;
use crate::types::{MaybeResolvedType, PartiallyResolvedType, ResolvedType, TypeInfo};
use crate::{error::*, types::IntegerBits};
use crate::{AstNode, AstNodeContent, Ident, ReturnStatement};
use declaration::TypedTraitFn;
use pest::Span;

mod code_block;
mod declaration;
mod expression;
mod impl_trait;
mod return_statement;
mod while_loop;

use super::ERROR_RECOVERY_DECLARATION;
pub(crate) use code_block::TypedCodeBlock;
pub use declaration::{
    TypedDeclaration, TypedEnumDeclaration, TypedEnumVariant, TypedFunctionDeclaration,
    TypedFunctionParameter, TypedStructDeclaration, TypedStructField,
};
pub(crate) use declaration::{TypedReassignment, TypedTraitDeclaration, TypedVariableDeclaration};
pub(crate) use expression::*;
use impl_trait::implementation_of_trait;
pub(crate) use return_statement::TypedReturnStatement;
pub(crate) use while_loop::TypedWhileLoop;

/// whether or not something is constantly evaluatable (if the result is known at compile
/// time)
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub(crate) enum IsConstant {
    Yes,
    No,
}

#[derive(Clone, Debug)]
pub(crate) enum TypedAstNodeContent<'sc> {
    ReturnStatement(TypedReturnStatement<'sc>),
    Declaration(TypedDeclaration<'sc>),
    Expression(TypedExpression<'sc>),
    ImplicitReturnExpression(TypedExpression<'sc>),
    WhileLoop(TypedWhileLoop<'sc>),
    // a no-op node used for something that just issues a side effect, like an import statement.
    SideEffect,
}

#[derive(Clone)]
pub struct TypedAstNode<'sc> {
    pub(crate) content: TypedAstNodeContent<'sc>,
    pub(crate) span: Span<'sc>,
}

impl<'sc> std::fmt::Debug for TypedAstNode<'sc> {
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
impl<'sc> TypedAstNode<'sc> {
    fn type_info(&self) -> MaybeResolvedType<'sc> {
        // return statement should be ()
        use TypedAstNodeContent::*;
        match &self.content {
            ReturnStatement(_) | Declaration(_) => MaybeResolvedType::Resolved(ResolvedType::Unit),
            Expression(TypedExpression { return_type, .. }) => return_type.clone(),
            ImplicitReturnExpression(TypedExpression { return_type, .. }) => return_type.clone(),
            WhileLoop(_) | SideEffect => MaybeResolvedType::Resolved(ResolvedType::Unit),
        }
    }
    pub(crate) fn type_check(
        node: AstNode<'sc>,
        namespace: &mut Namespace<'sc>,
        return_type_annotation: Option<MaybeResolvedType<'sc>>,
        help_text: impl Into<String>,
        self_type: &MaybeResolvedType<'sc>,
    ) -> CompileResult<'sc, TypedAstNode<'sc>> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        let node = TypedAstNode {
            content: match node.content.clone() {
                AstNodeContent::UseStatement(a) => {
                    match a.import_type {
                        ImportType::Star => namespace.star_import(a.call_path),

                        ImportType::Item(s) => namespace.item_import(a.call_path, &s, None),
                    };
                    TypedAstNodeContent::SideEffect
                }
                AstNodeContent::Declaration(a) => {
                    TypedAstNodeContent::Declaration(match a {
                        Declaration::VariableDeclaration(VariableDeclaration {
                            name,
                            type_ascription,
                            body,
                            is_mutable,
                        }) => {
                            let type_ascription = type_ascription.map(|type_ascription| {
                                namespace.resolve_type(&type_ascription, self_type)
                            });
                            let body = type_check!(
                                TypedExpression::type_check(
                                    body,
                                    &namespace,
                                    type_ascription.clone(),
                                    format!(
                                        "Variable declaration's type annotation (type {}) does \
                                         not match up with the assigned expression's type.",
                                        type_ascription
                                            .map(|x| x.friendly_type_str())
                                            .unwrap_or("none".into())
                                    ),
                                    self_type
                                ),
                                error_recovery_expr(name.span.clone()),
                                warnings,
                                errors
                            );

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
                            let primary_name = e.name.primary_name;
                            let decl = TypedDeclaration::EnumDeclaration(
                                e.to_typed_decl(namespace, self_type),
                            );

                            namespace.insert(Ident { primary_name, span }, decl.clone());
                            decl
                        }
                        Declaration::FunctionDeclaration(fn_decl) => {
                            let decl = type_check!(
                                TypedFunctionDeclaration::type_check(
                                    fn_decl, &namespace, None, "", self_type
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
                            visibility,
                        }) => {
                            let mut methods_buf = Vec::new();
                            let interface_surface = interface_surface
                        .into_iter()
                        .map(|TraitFn {
                             name,
                              parameters,
                              return_type,
                              return_type_span
                             }| TypedTraitFn {
                            name,
                            return_type_span,
                            parameters: parameters
                                .into_iter()
                                .map(|FunctionParameter { name, r#type, type_span }|
                                    TypedFunctionParameter {
                                         name,
                                         r#type: namespace.resolve_type(&r#type, 
                                            &MaybeResolvedType::Partial(PartiallyResolvedType::SelfType)),
                                             type_span }
                                ).collect(),
                            return_type: namespace.resolve_type(&return_type,
                                &MaybeResolvedType::Partial(PartiallyResolvedType::SelfType)
                            )
                        }).collect::<Vec<_>>();
                            for FunctionDeclaration {
                                body,
                                name: fn_name,
                                parameters,
                                span,
                                return_type,
                                type_parameters,
                                return_type_span,
                                ..
                            } in methods
                            {
                                let mut namespace = namespace.clone();
                                parameters.clone().into_iter().for_each(
                                    |FunctionParameter { name, r#type, .. }| {
                                        let r#type = namespace.resolve_type(
                                            &r#type,
                                            &MaybeResolvedType::Partial(
                                                PartiallyResolvedType::SelfType,
                                            ),
                                        );
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
                                                    span: name.span.clone(),
                                                },
                                                // TODO allow mutable function params?
                                                is_mutable: false,
                                            },
                                        ),
                                    );
                                    },
                                );
                                // check the generic types in the arguments, make sure they are in
                                // the type scope
                                let mut generic_params_buf_for_error_message = Vec::new();
                                for param in parameters.iter() {
                                    if let TypeInfo::Custom { ref name } = param.r#type {
                                        generic_params_buf_for_error_message
                                            .push(name.primary_name);
                                    }
                                }
                                let comma_separated_generic_params =
                                    generic_params_buf_for_error_message.join(", ");
                                for FunctionParameter {
                                    ref r#type, name, ..
                                } in parameters.iter()
                                {
                                    let span = name.span.clone();
                                    if let TypeInfo::Custom { name, .. } = r#type {
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
                                        if type_parameters
                                            .iter()
                                            .find(|x| x.name == name.primary_name)
                                            .is_none()
                                        {
                                            errors.push(
                                                CompileError::TypeParameterNotInTypeScope {
                                                    name: name.primary_name,
                                                    span: span.clone(),
                                                    comma_separated_generic_params:
                                                        comma_separated_generic_params.clone(),
                                                    fn_name: fn_name.primary_name,
                                                    args: args_span.as_str(),
                                                },
                                            );
                                        }
                                    }
                                }
                                let parameters = parameters
                                    .into_iter()
                                    .map(
                                        |FunctionParameter {
                                             name,
                                             r#type,
                                             type_span,
                                         }| {
                                            TypedFunctionParameter {
                                                name,
                                                r#type: namespace.resolve_type(
                                                    &r#type,
                                                    &MaybeResolvedType::Partial(
                                                        PartiallyResolvedType::SelfType,
                                                    ),
                                                ),
                                                type_span,
                                            }
                                        },
                                    )
                                    .collect::<Vec<_>>();
                                // TODO check code block implicit return
                                let return_type = namespace.resolve_type(&return_type, self_type);
                                let (body, _code_block_implicit_return) = type_check!(
                                    TypedCodeBlock::type_check(
                                        body,
                                        &namespace,
                                        Some(return_type.clone()),
                                        "Trait method body's return type does not match up with \
                                         its return type annotation.",
                                        self_type
                                    ),
                                    continue,
                                    warnings,
                                    errors
                                );

                                methods_buf.push(TypedFunctionDeclaration {
                                    name: fn_name,
                                    body,
                                    parameters,
                                    span,
                                    return_type,
                                    type_parameters,
                                    // For now, any method declared is automatically public.
                                    // We can tweak that later if we want.
                                    visibility: Visibility::Public,
                                    return_type_span,
                                });
                            }
                            let trait_decl =
                                TypedDeclaration::TraitDeclaration(TypedTraitDeclaration {
                                    name: name.clone(),
                                    interface_surface,
                                    methods: methods_buf,
                                    type_parameters,
                                    visibility,
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
                                    // allow the type checking to continue unhindered even though
                                    // this is an error
                                    // basically pretending that this isn't an error by not
                                    // early-returning, for the sake of better error reporting
                                    if !is_mutable {
                                        errors.push(CompileError::AssignmentToNonMutable(
                                            lhs.primary_name,
                                            span.clone(),
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
                                    "You can only reassign a value of the same type to a variable.",
                                    self_type
                                ),
                                error_recovery_expr(span),
                                warnings,
                                errors
                            );

                            TypedDeclaration::Reassignment(TypedReassignment { lhs, rhs })
                        }
                        Declaration::ImplTrait(impl_trait) => type_check!(
                            implementation_of_trait(impl_trait, namespace),
                            return err(warnings, errors),
                            warnings,
                            errors
                        ),

                        Declaration::ImplSelf(ImplSelf {
                            type_arguments,
                            functions,
                            type_implementing_for,
                            block_span,
                            ..
                        }) => {
                            let type_implementing_for_resolved =
                                namespace.resolve_type_without_self(&type_implementing_for);
                            // check, if this is a custom type, if it is in scope or a generic.
                            let mut functions_buf: Vec<TypedFunctionDeclaration> = vec![];
                            for mut fn_decl in functions.into_iter() {
                                let mut type_arguments = type_arguments.clone();
                                // add generic params from impl trait into function type params
                                fn_decl.type_parameters.append(&mut type_arguments);
                                // ensure this fn decl's parameters and signature lines up with the
                                // one in the trait

                                // replace SelfType with type of implementor
                                // i.e. fn add(self, other: u64) -> Self becomes fn
                                // add(self: u64, other: u64) -> u64
                                fn_decl.parameters.iter_mut().for_each(
                                    |FunctionParameter { ref mut r#type, .. }| {
                                        if r#type == &TypeInfo::SelfType {
                                            *r#type = type_implementing_for.clone();
                                        }
                                    },
                                );
                                if fn_decl.return_type == TypeInfo::SelfType {
                                    fn_decl.return_type = type_implementing_for.clone();
                                }

                                functions_buf.push(type_check!(
                                    TypedFunctionDeclaration::type_check(
                                        fn_decl,
                                        &namespace,
                                        None,
                                        "",
                                        &type_implementing_for_resolved
                                    ),
                                    continue,
                                    warnings,
                                    errors
                                ));
                            }
                            namespace.insert_trait_implementation(
                                CallPath {
                                    prefixes: vec![],
                                    suffix: Ident {
                                        primary_name: "r#Self",
                                        span: block_span.clone(),
                                    },
                                },
                                type_implementing_for_resolved,
                                functions_buf,
                            );
                            TypedDeclaration::SideEffect
                        }
                        Declaration::StructDeclaration(decl) => {
                            // look up any generic or struct types in the namespace
                            let fields = decl
                                .fields
                                .into_iter()
                                .map(|StructField { name, r#type, span }| TypedStructField {
                                    name,
                                    r#type: match namespace.resolve_type(&r#type, self_type) {
                                        MaybeResolvedType::Resolved(r) => r,
                                        MaybeResolvedType::Partial(
                                            crate::types::PartiallyResolvedType::Numeric,
                                        ) => ResolvedType::UnsignedInteger(IntegerBits::SixtyFour),
                                        MaybeResolvedType::Partial(p) => {
                                            errors.push(CompileError::TypeMustBeKnown {
                                                ty: p.friendly_type_str(),
                                                span: span.clone(),
                                            });
                                            ResolvedType::ErrorRecovery
                                        }
                                    },
                                    span,
                                })
                                .collect::<Vec<_>>();
                            let decl = TypedStructDeclaration {
                                name: decl.name.clone(),
                                type_parameters: decl.type_parameters.clone(),
                                fields,
                                visibility: decl.visibility,
                            };

                            // insert struct into namespace
                            namespace.insert(
                                decl.name.clone(),
                                TypedDeclaration::StructDeclaration(decl.clone()),
                            );

                            TypedDeclaration::StructDeclaration(decl)
                        }
                    })
                }
                AstNodeContent::Expression(a) => {
                    let inner = type_check!(
                        TypedExpression::type_check(a.clone(), &namespace, None, "", self_type),
                        error_recovery_expr(a.span()),
                        warnings,
                        errors
                    );
                    TypedAstNodeContent::Expression(inner)
                }
                AstNodeContent::ReturnStatement(ReturnStatement { expr }) => {
                    TypedAstNodeContent::ReturnStatement(TypedReturnStatement {
                        expr: type_check!(
                            TypedExpression::type_check(
                                expr.clone(),
                                &namespace,
                                return_type_annotation,
                                "Returned value must match up with the function return type \
                                 annotation.",
                                self_type
                            ),
                            error_recovery_expr(expr.span()),
                            warnings,
                            errors
                        ),
                    })
                }
                AstNodeContent::ImplicitReturnExpression(expr) => {
                    let typed_expr = type_check!(
                        TypedExpression::type_check(
                            expr.clone(),
                            &namespace,
                            return_type_annotation,
                            format!(
                                "Implicit return must match up with block's type. {}",
                                help_text.into()
                            ),
                            self_type
                        ),
                        error_recovery_expr(expr.span()),
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
                            Some(MaybeResolvedType::Resolved(ResolvedType::Boolean)),
                            "A while loop's loop condition must be a boolean expression.",
                            self_type
                        ),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    let (typed_body, _block_implicit_return) = type_check!(
                        TypedCodeBlock::type_check(
                            body.clone(),
                            &namespace,
                            Some(MaybeResolvedType::Resolved(ResolvedType::Unit)),
                            "A while loop's loop body cannot implicitly return a value.Try \
                             assigning it to a mutable variable declared outside of the loop \
                             instead.",
                            self_type
                        ),
                        (
                            TypedCodeBlock {
                                contents: vec![],
                                whole_block_span: body.whole_block_span.clone(),
                            },
                            Some(MaybeResolvedType::Resolved(ResolvedType::Unit))
                        ),
                        warnings,
                        errors
                    );
                    TypedAstNodeContent::WhileLoop(TypedWhileLoop {
                        condition: typed_condition,
                        body: typed_body,
                    })
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
                    node.type_info() == MaybeResolvedType::Resolved(ResolvedType::Unit)
                        || node.type_info()
                            == MaybeResolvedType::Resolved(ResolvedType::ErrorRecovery),
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
