mod code_block;
pub mod declaration;
pub mod expression;
pub mod impl_trait;
mod return_statement;
pub mod while_loop;

pub(crate) use code_block::*;
pub use declaration::*;
pub(crate) use expression::*;
pub(crate) use impl_trait::*;
pub(crate) use return_statement::*;
pub(crate) use while_loop::*;

use super::ERROR_RECOVERY_DECLARATION;

use crate::{
    error::*, parse_tree::*, semantic_analysis::*, style::*, type_engine::*, AstNode,
    AstNodeContent, Ident, ReturnStatement,
};

use sway_types::{span::Span, state::StateIndex};

use derivative::Derivative;

/// whether or not something is constantly evaluatable (if the result is known at compile
/// time)
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub(crate) enum IsConstant {
    Yes,
    No,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypedAstNodeContent {
    ReturnStatement(TypedReturnStatement),
    Declaration(TypedDeclaration),
    Expression(TypedExpression),
    ImplicitReturnExpression(TypedExpression),
    WhileLoop(TypedWhileLoop),
    // a no-op node used for something that just issues a side effect, like an import statement.
    SideEffect,
}

#[derive(Clone, Debug, Eq, Derivative)]
#[derivative(PartialEq)]
pub struct TypedAstNode {
    pub content: TypedAstNodeContent,
    #[derivative(PartialEq = "ignore")]
    pub(crate) span: Span,
}

impl std::fmt::Display for TypedAstNode {
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

impl CopyTypes for TypedAstNode {
    fn copy_types(&mut self, type_mapping: &TypeMapping) {
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
}

impl TypedAstNode {
    /// Returns `true` if this AST node will be exported in a library, i.e. it is a public declaration.
    pub(crate) fn is_public(&self) -> bool {
        use TypedAstNodeContent::*;
        match &self.content {
            Declaration(decl) => decl.visibility().is_public(),
            ReturnStatement(_)
            | Expression(_)
            | WhileLoop(_)
            | SideEffect
            | ImplicitReturnExpression(_) => false,
        }
    }

    /// Naive check to see if this node is a function declaration of a function called `main` if
    /// the [TreeType] is Script or Predicate.
    pub(crate) fn is_main_function(&self, tree_type: TreeType) -> bool {
        match &self {
            TypedAstNode {
                content:
                    TypedAstNodeContent::Declaration(TypedDeclaration::FunctionDeclaration(
                        TypedFunctionDeclaration { name, .. },
                    )),
                ..
            } if name.as_str() == crate::constants::DEFAULT_ENTRY_POINT_FN_NAME => {
                matches!(tree_type, TreeType::Script | TreeType::Predicate)
            }
            _ => false,
        }
    }

    /// if this ast node _deterministically_ panics/aborts, then this is true.
    /// This is used to assist in type checking branches that abort control flow and therefore
    /// don't need to return a type.
    pub(crate) fn deterministically_aborts(&self) -> bool {
        use TypedAstNodeContent::*;
        match &self.content {
            ReturnStatement(_) => true,
            Declaration(_) => false,
            Expression(exp) | ImplicitReturnExpression(exp) => exp.deterministically_aborts(),
            WhileLoop(TypedWhileLoop { condition, body }) => {
                condition.deterministically_aborts() || body.deterministically_aborts()
            }
            SideEffect => false,
        }
    }

    /// recurse into `self` and get any return statements -- used to validate that all returns
    /// do indeed return the correct type
    /// This does _not_ extract implicit return statements as those are not control flow! This is
    /// _only_ for explicit returns.
    pub(crate) fn gather_return_statements(&self) -> Vec<&TypedReturnStatement> {
        match &self.content {
            TypedAstNodeContent::ReturnStatement(ref stmt) => vec![stmt],
            TypedAstNodeContent::ImplicitReturnExpression(ref exp) => {
                exp.gather_return_statements()
            }
            TypedAstNodeContent::WhileLoop(TypedWhileLoop {
                ref condition,
                ref body,
                ..
            }) => {
                let mut buf = condition.gather_return_statements();
                for node in &body.contents {
                    buf.append(&mut node.gather_return_statements())
                }
                buf
            }
            // assignments and  reassignments can happen during control flow and can abort
            TypedAstNodeContent::Declaration(TypedDeclaration::VariableDeclaration(
                TypedVariableDeclaration { body, .. },
            )) => body.gather_return_statements(),
            TypedAstNodeContent::Declaration(TypedDeclaration::Reassignment(
                TypedReassignment { rhs, .. },
            )) => rhs.gather_return_statements(),
            TypedAstNodeContent::Expression(exp) => exp.gather_return_statements(),
            TypedAstNodeContent::SideEffect | TypedAstNodeContent::Declaration(_) => vec![],
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
        let TypeCheckArguments {
            checkee: node,
            namespace,
            return_type_annotation,
            help_text,
            self_type,
            opts,
            ..
        } = arguments;
        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        // A little utility used to check an ascribed type matches its associated expression.
        let mut type_check_ascribed_expr =
            |namespace: &mut Namespace, type_ascription: TypeInfo, value| {
                let type_id = check!(
                    namespace.resolve_type_with_self(
                        type_ascription,
                        self_type,
                        &node.span,
                        EnforceTypeArguments::No
                    ),
                    insert_type(TypeInfo::ErrorRecovery),
                    warnings,
                    errors,
                );
                TypedExpression::type_check(TypeCheckArguments {
                    checkee: value,
                    namespace,
                    return_type_annotation: type_id,
                    help_text: "This declaration's type annotation  does \
                     not match up with the assigned expression's type.",
                    self_type,
                    mode: Mode::NonAbi,
                    opts,
                })
            };

        let node = TypedAstNode {
            content: match node.content.clone() {
                AstNodeContent::UseStatement(a) => {
                    let path = if a.is_absolute {
                        a.call_path.clone()
                    } else {
                        namespace.find_module_path(&a.call_path)
                    };
                    let mut res = match a.import_type {
                        ImportType::Star => namespace.star_import(&path),
                        ImportType::SelfImport => namespace.self_import(&path, a.alias),
                        ImportType::Item(s) => namespace.item_import(&path, &s, a.alias),
                    };
                    warnings.append(&mut res.warnings);
                    errors.append(&mut res.errors);
                    TypedAstNodeContent::SideEffect
                }
                AstNodeContent::IncludeStatement(_) => TypedAstNodeContent::SideEffect,
                AstNodeContent::Declaration(a) => {
                    TypedAstNodeContent::Declaration(match a {
                        Declaration::VariableDeclaration(VariableDeclaration {
                            name,
                            type_ascription,
                            type_ascription_span,
                            body,
                            is_mutable,
                        }) => {
                            check_if_name_is_invalid(&name).ok(&mut warnings, &mut errors);
                            let type_ascription_span = match type_ascription_span {
                                Some(type_ascription_span) => type_ascription_span,
                                None => name.span().clone(),
                            };
                            let type_ascription = match namespace
                                .resolve_type_with_self(
                                    type_ascription,
                                    self_type,
                                    &type_ascription_span,
                                    EnforceTypeArguments::Yes,
                                )
                                .value
                            {
                                Some(type_ascription) => type_ascription,
                                None => {
                                    errors.push(CompileError::UnknownType {
                                        span: type_ascription_span,
                                    });
                                    insert_type(TypeInfo::ErrorRecovery)
                                }
                            };
                            let result = {
                                TypedExpression::type_check(TypeCheckArguments {
                                    checkee: body,
                                    namespace,
                                    return_type_annotation: type_ascription,
                                    help_text: "Variable declaration's type annotation does \
                     not match up with the assigned expression's type.",
                                    self_type,
                                    mode: Mode::NonAbi,
                                    opts,
                                })
                            };
                            let body = check!(
                                result,
                                error_recovery_expr(name.span().clone()),
                                warnings,
                                errors
                            );
                            let typed_var_decl =
                                TypedDeclaration::VariableDeclaration(TypedVariableDeclaration {
                                    name: name.clone(),
                                    body,
                                    is_mutable: is_mutable.into(),
                                    const_decl_origin: false,
                                    type_ascription,
                                });
                            namespace.insert_symbol(name, typed_var_decl.clone());
                            typed_var_decl
                        }
                        Declaration::ConstantDeclaration(ConstantDeclaration {
                            name,
                            type_ascription,
                            value,
                            visibility,
                        }) => {
                            let result =
                                type_check_ascribed_expr(namespace, type_ascription.clone(), value);
                            is_screaming_snake_case(&name).ok(&mut warnings, &mut errors);
                            let value = check!(
                                result,
                                error_recovery_expr(name.span().clone()),
                                warnings,
                                errors
                            );
                            let typed_const_decl =
                                TypedDeclaration::VariableDeclaration(TypedVariableDeclaration {
                                    name: name.clone(),
                                    body: value,
                                    is_mutable: if visibility.is_public() {
                                        VariableMutability::ExportedConst
                                    } else {
                                        VariableMutability::Immutable
                                    },
                                    const_decl_origin: true,
                                    type_ascription: insert_type(type_ascription),
                                });
                            namespace.insert_symbol(name, typed_const_decl.clone());
                            typed_const_decl
                        }
                        Declaration::EnumDeclaration(decl) => {
                            let decl = check!(
                                TypedEnumDeclaration::type_check(decl, namespace, self_type),
                                return err(warnings, errors),
                                warnings,
                                errors
                            );
                            let name = decl.name.clone();
                            let decl = TypedDeclaration::EnumDeclaration(decl);
                            let _ = check!(
                                namespace.insert_symbol(name, decl.clone()),
                                return err(warnings, errors),
                                warnings,
                                errors
                            );
                            decl
                        }
                        Declaration::FunctionDeclaration(fn_decl) => {
                            for type_parameter in fn_decl.type_parameters.iter() {
                                if !type_parameter.trait_constraints.is_empty() {
                                    errors.push(CompileError::WhereClauseNotYetSupported {
                                        span: type_parameter.name_ident.span().clone(),
                                    });
                                    break;
                                }
                            }

                            let decl = check!(
                                TypedFunctionDeclaration::type_check(TypeCheckArguments {
                                    checkee: fn_decl.clone(),
                                    namespace,
                                    return_type_annotation: insert_type(TypeInfo::Unknown),
                                    help_text,
                                    self_type,
                                    mode: Mode::NonAbi,
                                    opts
                                }),
                                error_recovery_function_declaration(fn_decl),
                                warnings,
                                errors
                            );
                            namespace.insert_symbol(
                                decl.name.clone(),
                                TypedDeclaration::FunctionDeclaration(decl.clone()),
                            );
                            TypedDeclaration::FunctionDeclaration(decl)
                        }
                        Declaration::TraitDeclaration(trait_decl) => {
                            is_upper_camel_case(&trait_decl.name).ok(&mut warnings, &mut errors);
                            check!(
                                type_check_trait_decl(TypeCheckArguments {
                                    checkee: trait_decl,
                                    namespace,
                                    self_type,
                                    // this is unused by `type_check_trait`
                                    return_type_annotation: insert_type(TypeInfo::Unknown),
                                    help_text: Default::default(),
                                    mode: Mode::NonAbi,
                                    opts,
                                }),
                                return err(warnings, errors),
                                warnings,
                                errors
                            )
                        }
                        Declaration::Reassignment(Reassignment { lhs, rhs, span }) => {
                            check!(
                                reassignment(
                                    TypeCheckArguments {
                                        checkee: (lhs, rhs),
                                        namespace,
                                        self_type,
                                        // this is unused by `reassignment`
                                        return_type_annotation: insert_type(TypeInfo::Unknown),
                                        help_text: Default::default(),
                                        mode: Mode::NonAbi,
                                        opts,
                                    },
                                    span,
                                ),
                                return err(warnings, errors),
                                warnings,
                                errors
                            )
                        }
                        Declaration::ImplTrait(impl_trait) => check!(
                            implementation_of_trait(impl_trait, namespace, opts),
                            return err(warnings, errors),
                            warnings,
                            errors
                        ),

                        Declaration::ImplSelf(ImplSelf {
                            functions,
                            type_implementing_for,
                            block_span,
                            type_parameters,
                            ..
                        }) => {
                            for type_parameter in type_parameters.iter() {
                                if !type_parameter.trait_constraints.is_empty() {
                                    errors.push(CompileError::WhereClauseNotYetSupported {
                                        span: type_parameter.name_ident.span().clone(),
                                    });
                                    break;
                                }
                            }

                            // create the namespace for the impl
                            let mut impl_namespace = namespace.clone();
                            for type_parameter in type_parameters.iter() {
                                impl_namespace.insert_symbol(
                                    type_parameter.name_ident.clone(),
                                    type_parameter.into(),
                                );
                            }

                            // Resolve the Self type as it's most likely still 'Custom' and use the
                            // resolved type for self instead.
                            let implementing_for_type_id = check!(
                                impl_namespace.resolve_type_without_self(type_implementing_for),
                                return err(warnings, errors),
                                warnings,
                                errors
                            );
                            let type_implementing_for = look_up_type_id(implementing_for_type_id);
                            let mut functions_buf: Vec<TypedFunctionDeclaration> = vec![];
                            for mut fn_decl in functions.into_iter() {
                                // ensure this fn decl's parameters and signature lines up with the
                                // one in the trait

                                // replace SelfType with type of implementor
                                // i.e. fn add(self, other: u64) -> Self becomes fn
                                // add(self: u64, other: u64) -> u64
                                fn_decl.parameters.iter_mut().for_each(
                                    |FunctionParameter {
                                         ref mut type_id, ..
                                     }| {
                                        if look_up_type_id(*type_id) == TypeInfo::SelfType {
                                            *type_id = implementing_for_type_id;
                                        }
                                    },
                                );
                                if fn_decl.return_type == TypeInfo::SelfType {
                                    fn_decl.return_type = type_implementing_for.clone();
                                }
                                let args = TypeCheckArguments {
                                    checkee: fn_decl,
                                    namespace: &mut impl_namespace,
                                    return_type_annotation: insert_type(TypeInfo::Unknown),
                                    help_text: "",
                                    self_type: implementing_for_type_id,
                                    mode: Mode::NonAbi,
                                    opts,
                                };
                                functions_buf.push(check!(
                                    TypedFunctionDeclaration::type_check(args),
                                    continue,
                                    warnings,
                                    errors
                                ));
                            }
                            let trait_name = CallPath {
                                prefixes: vec![],
                                suffix: Ident::new_with_override("r#Self", block_span.clone()),
                                is_absolute: false,
                            };
                            namespace.insert_trait_implementation(
                                trait_name.clone(),
                                type_implementing_for.clone(),
                                functions_buf.clone(),
                            );
                            TypedDeclaration::ImplTrait {
                                trait_name,
                                span: block_span,
                                methods: functions_buf,
                                type_implementing_for,
                            }
                        }
                        Declaration::StructDeclaration(decl) => {
                            let decl = check!(
                                TypedStructDeclaration::type_check(decl, namespace, self_type),
                                return err(warnings, errors),
                                warnings,
                                errors
                            );
                            let name = decl.name.clone();
                            let decl = TypedDeclaration::StructDeclaration(decl);
                            // insert the struct decl into namespace
                            let _ = check!(
                                namespace.insert_symbol(name, decl.clone()),
                                return err(warnings, errors),
                                warnings,
                                errors
                            );
                            decl
                        }
                        Declaration::AbiDeclaration(AbiDeclaration {
                            name,
                            interface_surface,
                            methods,
                            span,
                        }) => {
                            // type check the interface surface and methods
                            // We don't want the user to waste resources by contract calling
                            // themselves, and we don't want to do more work in the compiler,
                            // so we don't support the case of calling a contract's own interface
                            // from itself. This is by design.
                            let interface_surface = check!(
                                type_check_interface_surface(interface_surface, namespace),
                                return err(warnings, errors),
                                warnings,
                                errors
                            );
                            // type check these for errors but don't actually use them yet -- the real
                            // ones will be type checked with proper symbols when the ABI is implemented
                            let _methods = check!(
                                type_check_trait_methods(methods.clone(), namespace, self_type,),
                                vec![],
                                warnings,
                                errors
                            );

                            let decl = TypedDeclaration::AbiDeclaration(TypedAbiDeclaration {
                                interface_surface,
                                methods,
                                name: name.clone(),
                                span,
                            });
                            namespace.insert_symbol(name, decl.clone());
                            decl
                        }
                        Declaration::StorageDeclaration(StorageDeclaration { span, fields }) => {
                            let mut fields_buf = Vec::with_capacity(fields.len());
                            for StorageField { name, r#type } in fields {
                                let r#type = check!(
                                    namespace.resolve_type_without_self(r#type),
                                    return err(warnings, errors),
                                    warnings,
                                    errors
                                );
                                fields_buf.push(TypedStorageField::new(name, r#type, span.clone()));
                            }

                            let decl = TypedStorageDeclaration::new(fields_buf, span);
                            // insert the storage declaration into the symbols
                            // if there already was one, return an error that duplicate storage

                            // declarations are not allowed
                            check!(
                                namespace.set_storage_declaration(decl.clone()),
                                return err(warnings, errors),
                                warnings,
                                errors
                            );
                            TypedDeclaration::StorageDeclaration(decl)
                        }
                    })
                }
                AstNodeContent::Expression(a) => {
                    let inner = check!(
                        TypedExpression::type_check(TypeCheckArguments {
                            checkee: a.clone(),
                            namespace,
                            return_type_annotation: insert_type(TypeInfo::Unknown),
                            help_text: Default::default(),
                            self_type,
                            mode: Mode::NonAbi,
                            opts
                        }),
                        error_recovery_expr(a.span()),
                        warnings,
                        errors
                    );
                    TypedAstNodeContent::Expression(inner)
                }
                AstNodeContent::ReturnStatement(ReturnStatement { expr }) => {
                    TypedAstNodeContent::ReturnStatement(TypedReturnStatement {
                        expr: check!(
                            TypedExpression::type_check(TypeCheckArguments {
                                checkee: expr.clone(),
                                namespace,
                                // we use "unknown" here because return statements do not
                                // necessarily follow the type annotation of their immediate
                                // surrounding context. Because a return statement is control flow
                                // that breaks out to the nearest function, we need to type check
                                // it against the surrounding function.
                                // That is impossible here, as we don't have that information. It
                                // is the responsibility of the function declaration to type check
                                // all return statements contained within it.
                                return_type_annotation: insert_type(TypeInfo::Unknown),
                                help_text:
                                    "Returned value must match up with the function return type \
                                 annotation.",
                                self_type,
                                mode: Mode::NonAbi,
                                opts
                            }),
                            error_recovery_expr(expr.span()),
                            warnings,
                            errors
                        ),
                    })
                }
                AstNodeContent::ImplicitReturnExpression(expr) => {
                    let typed_expr = check!(
                        TypedExpression::type_check(TypeCheckArguments {
                            checkee: expr.clone(),
                            namespace,
                            return_type_annotation,
                            help_text: "Implicit return must match up with block's type.",
                            self_type,
                            mode: Mode::NonAbi,
                            opts,
                        }),
                        error_recovery_expr(expr.span()),
                        warnings,
                        errors
                    );
                    TypedAstNodeContent::ImplicitReturnExpression(typed_expr)
                }
                AstNodeContent::WhileLoop(WhileLoop { condition, body }) => {
                    let typed_condition = check!(
                        TypedExpression::type_check(TypeCheckArguments {
                            checkee: condition,
                            namespace,
                            return_type_annotation: insert_type(TypeInfo::Boolean),
                            help_text:
                                "A while loop's loop condition must be a boolean expression.",
                            self_type,
                            mode: Mode::NonAbi,
                            opts
                        }),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    let (typed_body, _block_implicit_return) = check!(
                        TypedCodeBlock::type_check(TypeCheckArguments {
                            checkee: body,
                            namespace,
                            return_type_annotation: insert_type(TypeInfo::Tuple(Vec::new())),
                            help_text:
                                "A while loop's loop body cannot implicitly return a value.Try \
                             assigning it to a mutable variable declared outside of the loop \
                             instead.",
                            self_type,
                            mode: Mode::NonAbi,
                            opts,
                        }),
                        (
                            TypedCodeBlock { contents: vec![] },
                            insert_type(TypeInfo::Tuple(Vec::new()))
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

        if let TypedAstNode {
            content: TypedAstNodeContent::Expression(TypedExpression { .. }),
            ..
        } = node
        {
            let warning = Warning::UnusedReturnValue {
                r#type: Box::new(node.type_info()),
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

fn reassignment(
    arguments: TypeCheckArguments<'_, (ReassignmentTarget, Expression)>,
    span: Span,
) -> CompileResult<TypedDeclaration> {
    let TypeCheckArguments {
        checkee: (lhs, rhs),
        namespace,
        self_type,
        opts,
        ..
    } = arguments;
    let mut errors = vec![];
    let mut warnings = vec![];
    // ensure that the lhs is a variable expression or struct field access
    match lhs {
        ReassignmentTarget::VariableExpression(var) => {
            match *var {
                Expression::VariableExpression { name, span } => {
                    // check that the reassigned name exists
                    let unknown_decl = check!(
                        namespace.resolve_symbol(&name).cloned(),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    let variable_decl = check!(
                        unknown_decl.expect_variable().cloned(),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    if !variable_decl.is_mutable.is_mutable() {
                        errors.push(CompileError::AssignmentToNonMutable { name });
                    }
                    // the RHS is a ref type to the LHS
                    let rhs_type_id = insert_type(TypeInfo::Ref(
                        variable_decl.body.return_type,
                        variable_decl.body.span.clone(),
                    ));
                    // type check the reassignment
                    let rhs = check!(
                        TypedExpression::type_check(TypeCheckArguments {
                            checkee: rhs,
                            namespace,
                            return_type_annotation: rhs_type_id,
                            help_text:
                                "You can only reassign a value of the same type to a variable.",
                            self_type,
                            mode: Mode::NonAbi,
                            opts
                        }),
                        error_recovery_expr(span),
                        warnings,
                        errors
                    );

                    ok(
                        TypedDeclaration::Reassignment(TypedReassignment {
                            lhs_base_name: variable_decl.name,
                            lhs_type: rhs_type_id,
                            lhs_indices: vec![],
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
                    let (base_name, final_return_type) = loop {
                        let type_checked = check!(
                            TypedExpression::type_check(TypeCheckArguments {
                                checkee: expr.clone(),
                                namespace,
                                return_type_annotation: insert_type(TypeInfo::Unknown),
                                help_text: Default::default(),
                                self_type,
                                mode: Mode::NonAbi,
                                opts
                            }),
                            error_recovery_expr(expr.span()),
                            warnings,
                            errors
                        );

                        match expr {
                            Expression::VariableExpression { name, .. } => {
                                match namespace.clone().resolve_symbol(&name).value {
                                    Some(TypedDeclaration::VariableDeclaration(
                                        TypedVariableDeclaration { is_mutable, .. },
                                    )) => {
                                        if !is_mutable.is_mutable() {
                                            errors.push(CompileError::AssignmentToNonMutable {
                                                name: name.clone(),
                                            });
                                        }
                                    }
                                    Some(other) => {
                                        errors.push(CompileError::ReassignmentToNonVariable {
                                            name: name.clone(),
                                            kind: other.friendly_name(),
                                            span,
                                        });
                                        return err(warnings, errors);
                                    }
                                    None => {
                                        errors
                                            .push(CompileError::UnknownVariable { var_name: name });
                                        return err(warnings, errors);
                                    }
                                }
                                break (name, type_checked.return_type);
                            }
                            Expression::SubfieldExpression {
                                field_to_access,
                                prefix,
                                ..
                            } => {
                                names_vec.push(ReassignmentLhs {
                                    kind: ReassignmentLhsKind::StructField {
                                        name: field_to_access,
                                    },
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
                        kind: ReassignmentLhsKind::StructField {
                            name: field_to_access,
                        },
                        r#type: final_return_type,
                    });

                    let (ty_of_field, _ty_of_parent) = check!(
                        namespace.find_subfield_type(
                            std::iter::once(base_name.clone())
                                .chain(names_vec.iter().map(|ReassignmentLhs { kind, .. }| {
                                    match kind {
                                        ReassignmentLhsKind::StructField { name } => name.clone(),
                                    }
                                }))
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
                            return_type_annotation: ty_of_field,
                            help_text: Default::default(),
                            self_type,
                            mode: Mode::NonAbi,
                            opts,
                        }),
                        error_recovery_expr(span),
                        warnings,
                        errors
                    );

                    ok(
                        TypedDeclaration::Reassignment(TypedReassignment {
                            lhs_base_name: base_name,
                            lhs_type: final_return_type,
                            lhs_indices: names_vec,
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
        ReassignmentTarget::StorageField(fields) => reassign_storage_subfield(TypeCheckArguments {
            checkee: (fields, span, rhs),
            namespace,
            return_type_annotation: insert_type(TypeInfo::Unknown),
            help_text: Default::default(),
            self_type,
            mode: Mode::NonAbi,
            opts,
        })
        .map(TypedDeclaration::StorageReassignment),
    }
}

/// Recursively handle supertraits by adding all their interfaces and methods to some namespace
/// which is meant to be the namespace of the subtrait in question
fn handle_supertraits(
    supertraits: &[Supertrait],
    trait_namespace: &mut Namespace,
) -> CompileResult<()> {
    let mut warnings = Vec::new();
    let mut errors = Vec::new();

    for supertrait in supertraits.iter() {
        match trait_namespace
            .resolve_call_path(&supertrait.name)
            .ok(&mut warnings, &mut errors)
            .cloned()
        {
            Some(TypedDeclaration::TraitDeclaration(TypedTraitDeclaration {
                ref interface_surface,
                ref methods,
                ref supertraits,
                ..
            })) => {
                // insert dummy versions of the interfaces for all of the supertraits
                trait_namespace.insert_trait_implementation(
                    supertrait.name.clone(),
                    TypeInfo::SelfType,
                    interface_surface
                        .iter()
                        .map(|x| x.to_dummy_func(Mode::NonAbi))
                        .collect(),
                );

                // insert dummy versions of the methods of all of the supertraits
                let dummy_funcs = check!(
                    convert_trait_methods_to_dummy_funcs(methods, trait_namespace),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                trait_namespace.insert_trait_implementation(
                    supertrait.name.clone(),
                    TypeInfo::SelfType,
                    dummy_funcs,
                );

                // Recurse to insert dummy versions of interfaces and methods of the *super*
                // supertraits
                check!(
                    handle_supertraits(supertraits, trait_namespace),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
            }
            Some(TypedDeclaration::AbiDeclaration(_)) => {
                errors.push(CompileError::AbiAsSupertrait {
                    span: supertrait.name.span().clone(),
                })
            }
            _ => errors.push(CompileError::TraitNotFound {
                name: supertrait.name.clone(),
            }),
        }
    }

    ok((), warnings, errors)
}

fn type_check_trait_decl(
    arguments: TypeCheckArguments<'_, TraitDeclaration>,
) -> CompileResult<TypedDeclaration> {
    let TypeCheckArguments {
        checkee: trait_decl,
        namespace,
        return_type_annotation: _return_type_annotation,
        help_text: _help_text,
        ..
    } = arguments;

    let mut warnings = Vec::new();
    let mut errors = Vec::new();

    // type check the interface surface
    let interface_surface = check!(
        type_check_interface_surface(trait_decl.interface_surface.to_vec(), namespace),
        return err(warnings, errors),
        warnings,
        errors
    );

    // A temporary namespace for checking within the trait's scope.
    let mut trait_namespace = namespace.clone();

    // Recursively handle supertraits: make their interfaces and methods available to this trait
    check!(
        handle_supertraits(&trait_decl.supertraits, &mut trait_namespace),
        return err(warnings, errors),
        warnings,
        errors
    );

    // insert placeholder functions representing the interface surface
    // to allow methods to use those functions
    trait_namespace.insert_trait_implementation(
        CallPath {
            prefixes: vec![],
            suffix: trait_decl.name.clone(),
            is_absolute: false,
        },
        TypeInfo::SelfType,
        interface_surface
            .iter()
            .map(|x| x.to_dummy_func(Mode::NonAbi))
            .collect(),
    );
    // check the methods for errors but throw them away and use vanilla [FunctionDeclaration]s
    let _methods = check!(
        type_check_trait_methods(
            trait_decl.methods.clone(),
            &mut trait_namespace,
            insert_type(TypeInfo::SelfType),
        ),
        vec![],
        warnings,
        errors
    );
    let typed_trait_decl = TypedDeclaration::TraitDeclaration(TypedTraitDeclaration {
        name: trait_decl.name.clone(),
        interface_surface,
        methods: trait_decl.methods.to_vec(),
        supertraits: trait_decl.supertraits.to_vec(),
        visibility: trait_decl.visibility,
    });
    namespace.insert_symbol(trait_decl.name, typed_trait_decl.clone());
    ok(typed_trait_decl, warnings, errors)
}

fn type_check_interface_surface(
    interface_surface: Vec<TraitFn>,
    namespace: &mut Namespace,
) -> CompileResult<Vec<TypedTraitFn>> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let interface_surface = interface_surface
        .into_iter()
        .map(
            |TraitFn {
                 name,
                 purity,
                 parameters,
                 return_type,
                 return_type_span,
             }| TypedTraitFn {
                name,
                purity,
                return_type_span: return_type_span.clone(),
                parameters: parameters
                    .into_iter()
                    .map(
                        |FunctionParameter {
                             name,
                             type_id,
                             type_span,
                         }| TypedFunctionParameter {
                            name,
                            r#type: check!(
                                namespace.resolve_type_with_self(
                                    look_up_type_id(type_id),
                                    insert_type(TypeInfo::SelfType),
                                    &type_span,
                                    EnforceTypeArguments::Yes
                                ),
                                insert_type(TypeInfo::ErrorRecovery),
                                warnings,
                                errors,
                            ),
                            type_span,
                        },
                    )
                    .collect(),
                return_type: check!(
                    namespace.resolve_type_with_self(
                        return_type,
                        insert_type(TypeInfo::SelfType),
                        &return_type_span,
                        EnforceTypeArguments::Yes
                    ),
                    insert_type(TypeInfo::ErrorRecovery),
                    warnings,
                    errors,
                ),
            },
        )
        .collect::<Vec<_>>();
    ok(interface_surface, warnings, errors)
}

fn type_check_trait_methods(
    methods: Vec<FunctionDeclaration>,
    namespace: &mut Namespace,
    self_type: TypeId,
) -> CompileResult<Vec<TypedFunctionDeclaration>> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let mut methods_buf = Vec::new();
    for FunctionDeclaration {
        body,
        name: fn_name,
        parameters,
        span,
        return_type,
        type_parameters,
        return_type_span,
        purity,
        ..
    } in methods
    {
        parameters.clone().into_iter().for_each(
            |FunctionParameter {
                 name,
                 type_id: ref r#type,
                 ..
             }| {
                let r#type = check!(
                    namespace.resolve_type_with_self(
                        look_up_type_id(*r#type),
                        insert_type(TypeInfo::SelfType),
                        name.span(),
                        EnforceTypeArguments::Yes
                    ),
                    insert_type(TypeInfo::ErrorRecovery),
                    warnings,
                    errors,
                );
                namespace.insert_symbol(
                    name.clone(),
                    TypedDeclaration::VariableDeclaration(TypedVariableDeclaration {
                        name: name.clone(),
                        body: TypedExpression {
                            expression: TypedExpressionVariant::FunctionParameter,
                            return_type: r#type,
                            is_constant: IsConstant::No,
                            span: name.span().clone(),
                        },
                        // TODO allow mutable function params?
                        is_mutable: VariableMutability::Immutable,
                        const_decl_origin: false,
                        type_ascription: r#type,
                    }),
                );
            },
        );
        // check the generic types in the arguments, make sure they are in
        // the type scope
        let mut generic_params_buf_for_error_message = Vec::new();
        for param in parameters.iter() {
            if let TypeInfo::Custom { ref name, .. } = look_up_type_id(param.type_id) {
                generic_params_buf_for_error_message.push(name.to_string());
            }
        }
        let comma_separated_generic_params = generic_params_buf_for_error_message.join(", ");
        for FunctionParameter {
            ref type_id, name, ..
        } in parameters.iter()
        {
            let span = name.span().clone();
            if let TypeInfo::Custom { name, .. } = look_up_type_id(*type_id) {
                let args_span = parameters.iter().fold(
                    parameters[0].name.span().clone(),
                    |acc, FunctionParameter { name, .. }| Span::join(acc, name.span().clone()),
                );
                if type_parameters.iter().any(|TypeParameter { type_id, .. }| {
                    if let TypeInfo::Custom {
                        name: this_name, ..
                    } = look_up_type_id(*type_id)
                    {
                        this_name == name.clone()
                    } else {
                        false
                    }
                }) {
                    errors.push(CompileError::TypeParameterNotInTypeScope {
                        name: name.clone(),
                        span: span.clone(),
                        comma_separated_generic_params: comma_separated_generic_params.clone(),
                        fn_name: fn_name.clone(),
                        args: args_span.as_str().to_string(),
                    });
                }
            }
        }
        let parameters = parameters
            .into_iter()
            .map(
                |FunctionParameter {
                     name,
                     type_id,
                     type_span,
                 }| {
                    TypedFunctionParameter {
                        name,
                        r#type: check!(
                            namespace.resolve_type_with_self(
                                look_up_type_id(type_id),
                                crate::type_engine::insert_type(TypeInfo::SelfType),
                                &type_span,
                                EnforceTypeArguments::Yes
                            ),
                            insert_type(TypeInfo::ErrorRecovery),
                            warnings,
                            errors,
                        ),
                        type_span,
                    }
                },
            )
            .collect::<Vec<_>>();

        // TODO check code block implicit return
        let return_type = check!(
            namespace.resolve_type_with_self(
                return_type,
                self_type,
                &return_type_span,
                EnforceTypeArguments::Yes
            ),
            insert_type(TypeInfo::ErrorRecovery),
            warnings,
            errors,
        );
        let (body, _code_block_implicit_return) = check!(
            TypedCodeBlock::type_check(TypeCheckArguments {
                checkee: body,
                namespace,
                return_type_annotation: return_type,
                help_text: "Trait method body's return type does not match up with \
                                         its return type annotation.",
                self_type,
                mode: Mode::NonAbi,
                opts: TCOpts { purity }
            }),
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
            is_contract_call: false,
            purity,
        });
    }
    ok(methods_buf, warnings, errors)
}

/// Convert a vector of FunctionDeclarations into a vector of TypedFunctionDeclarations where only
/// the parameters and the return types are type checked.
fn convert_trait_methods_to_dummy_funcs(
    methods: &[FunctionDeclaration],
    trait_namespace: &mut Namespace,
) -> CompileResult<Vec<TypedFunctionDeclaration>> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let dummy_funcs = methods
        .iter()
        .map(
            |FunctionDeclaration {
                 name,
                 parameters,
                 return_type,
                 return_type_span,
                 ..
             }| TypedFunctionDeclaration {
                purity: Default::default(),
                name: name.clone(),
                body: TypedCodeBlock { contents: vec![] },
                parameters: parameters
                    .iter()
                    .map(
                        |FunctionParameter {
                             name,
                             type_id,
                             type_span,
                         }| TypedFunctionParameter {
                            name: name.clone(),
                            r#type: check!(
                                trait_namespace.resolve_type_with_self(
                                    look_up_type_id(*type_id),
                                    insert_type(TypeInfo::SelfType),
                                    type_span,
                                    EnforceTypeArguments::Yes
                                ),
                                insert_type(TypeInfo::ErrorRecovery),
                                warnings,
                                errors,
                            ),
                            type_span: type_span.clone(),
                        },
                    )
                    .collect(),
                span: name.span().clone(),
                return_type: check!(
                    trait_namespace.resolve_type_with_self(
                        return_type.clone(),
                        insert_type(TypeInfo::SelfType),
                        return_type_span,
                        EnforceTypeArguments::Yes
                    ),
                    insert_type(TypeInfo::ErrorRecovery),
                    warnings,
                    errors,
                ),
                return_type_span: return_type_span.clone(),
                visibility: Visibility::Public,
                type_parameters: vec![],
                is_contract_call: false,
            },
        )
        .collect::<Vec<_>>();

    ok(dummy_funcs, warnings, errors)
}

/// Used to create a stubbed out function when the function fails to compile, preventing cascading
/// namespace errors
fn error_recovery_function_declaration(decl: FunctionDeclaration) -> TypedFunctionDeclaration {
    let FunctionDeclaration {
        name,
        return_type,
        span,
        return_type_span,
        visibility,
        ..
    } = decl;
    TypedFunctionDeclaration {
        purity: Default::default(),
        name,
        body: TypedCodeBlock {
            contents: Default::default(),
        },
        span,
        is_contract_call: false,
        return_type_span,
        parameters: Default::default(),
        visibility,
        return_type: insert_type(return_type),
        type_parameters: Default::default(),
    }
}

/// Describes each field being drilled down into in storage and its type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeCheckedStorageReassignment {
    pub fields: Vec<TypeCheckedStorageReassignDescriptor>,
    pub(crate) ix: StateIndex,
    pub rhs: TypedExpression,
}

impl TypeCheckedStorageReassignment {
    pub fn span(&self) -> Span {
        self.fields
            .iter()
            .fold(self.fields[0].span.clone(), |acc, field| {
                Span::join(acc, field.span.clone())
            })
    }
    pub fn names(&self) -> Vec<Ident> {
        self.fields
            .iter()
            .map(|f| f.name.clone())
            .collect::<Vec<_>>()
    }
}

/// Describes a single subfield access in the sequence when reassigning to a subfield within
/// storage.
#[derive(Clone, Debug, Eq)]
pub struct TypeCheckedStorageReassignDescriptor {
    pub name: Ident,
    pub r#type: TypeId,
    pub(crate) span: Span,
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl PartialEq for TypeCheckedStorageReassignDescriptor {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && look_up_type_id(self.r#type) == look_up_type_id(other.r#type)
    }
}

fn reassign_storage_subfield(
    arguments: TypeCheckArguments<'_, (Vec<Ident>, Span, Expression)>,
) -> CompileResult<TypeCheckedStorageReassignment> {
    let TypeCheckArguments {
        checkee: (fields, span, rhs),
        namespace,
        return_type_annotation: _return_type_annotation,
        help_text: _help_text,
        self_type,
        opts,
        ..
    } = arguments;
    let mut errors = vec![];
    let mut warnings = vec![];
    if !namespace.has_storage_declared() {
        errors.push(CompileError::NoDeclaredStorage { span });

        return err(warnings, errors);
    }

    let storage_fields = check!(
        namespace.get_storage_field_descriptors(),
        return err(warnings, errors),
        warnings,
        errors
    );
    let mut type_checked_buf = vec![];
    let mut fields: Vec<_> = fields.into_iter().rev().collect();

    let first_field = fields.pop().expect("guaranteed by grammar");
    let (ix, initial_field_type) = match storage_fields
        .iter()
        .enumerate()
        .find(|(_, TypedStorageField { name, .. })| name == &first_field)
    {
        Some((ix, TypedStorageField { r#type, .. })) => (StateIndex::new(ix), r#type),
        None => {
            errors.push(CompileError::StorageFieldDoesNotExist {
                name: first_field.clone(),
            });
            return err(warnings, errors);
        }
    };

    type_checked_buf.push(TypeCheckedStorageReassignDescriptor {
        name: first_field.clone(),
        r#type: *initial_field_type,
        span: first_field.span().clone(),
    });

    fn update_available_struct_fields(id: TypeId) -> Vec<TypedStructField> {
        match look_up_type_id(id) {
            TypeInfo::Struct { fields, .. } => fields,
            _ => vec![],
        }
    }
    let mut curr_type = *initial_field_type;

    // if the previously iterated type was a struct, put its fields here so we know that,
    // in the case of a subfield, we can type check the that the subfield exists and its type.
    let mut available_struct_fields = update_available_struct_fields(*initial_field_type);

    // get the initial field's type
    // make sure the next field exists in that type
    for field in fields.into_iter().rev() {
        match available_struct_fields
            .iter()
            .find(|x| x.name.as_str() == field.as_str())
        {
            Some(struct_field) => {
                curr_type = struct_field.r#type;
                type_checked_buf.push(TypeCheckedStorageReassignDescriptor {
                    name: field.clone(),
                    r#type: struct_field.r#type,
                    span: field.span().clone(),
                });
                available_struct_fields = update_available_struct_fields(struct_field.r#type);
            }
            None => {
                let available_fields = available_struct_fields
                    .iter()
                    .map(|x| x.name.as_str())
                    .collect::<Vec<_>>();
                errors.push(CompileError::FieldNotFound {
                    field_name: field.clone(),
                    available_fields: available_fields.join(", "),
                    struct_name: type_checked_buf.last().unwrap().name.clone(),
                });
                return err(warnings, errors);
            }
        }
    }
    let rhs = check!(
        TypedExpression::type_check(TypeCheckArguments {
            checkee: rhs,
            namespace,
            return_type_annotation: curr_type,
            help_text: Default::default(),
            self_type,
            mode: Mode::NonAbi,
            opts,
        }),
        error_recovery_expr(span),
        warnings,
        errors
    );

    ok(
        TypeCheckedStorageReassignment {
            fields: type_checked_buf,
            ix,
            rhs,
        },
        warnings,
        errors,
    )
}
