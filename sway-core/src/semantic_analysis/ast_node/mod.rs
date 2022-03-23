use super::ERROR_RECOVERY_DECLARATION;

use crate::{
    build_config::BuildConfig,
    control_flow_analysis::ControlFlowGraph,
    error::*,
    parse_tree::*,
    semantic_analysis::{ast_node::declaration::insert_type_parameters, *},
    type_engine::*,
    AstNode, AstNodeContent, Ident, ReturnStatement,
};

use sway_types::{
    span::{join_spans, Span},
    state::StateIndex,
};

use std::sync::Arc;

pub(crate) use crate::semantic_analysis::ast_node::declaration::ReassignmentLhs;

pub mod declaration;
use declaration::TypedTraitFn;
pub use declaration::{
    TypedAbiDeclaration, TypedConstantDeclaration, TypedDeclaration, TypedEnumDeclaration,
    TypedEnumVariant, TypedFunctionDeclaration, TypedFunctionParameter, TypedStructDeclaration,
    TypedStructField,
};
pub(crate) use declaration::{
    TypedReassignment, TypedTraitDeclaration, TypedVariableDeclaration, VariableMutability,
};

pub mod impl_trait;
use impl_trait::implementation_of_trait;
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
            ..
        } = arguments;
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        // A little utility used to check an ascribed type matches its associated expression.
        let mut type_check_ascribed_expr = |namespace: crate::semantic_analysis::NamespaceRef,
                                            crate_namespace: NamespaceRef,
                                            type_ascription: TypeInfo,
                                            value| {
            let type_id = namespace
                .resolve_type_with_self(type_ascription, self_type)
                .unwrap_or_else(|_| {
                    errors.push(CompileError::UnknownType {
                        span: node.span.clone(),
                    });
                    insert_type(TypeInfo::ErrorRecovery)
                });
            TypedExpression::type_check(TypeCheckArguments {
                checkee: value,
                namespace,
                crate_namespace,
                return_type_annotation: type_id,
                help_text: "This declaration's type annotation  does \
                     not match up with the assigned expression's type.",
                self_type,
                build_config,
                dead_code_graph,
                mode: Mode::NonAbi,
                opts,
            })
        };

        let node = TypedAstNode {
            content: match node.content.clone() {
                AstNodeContent::UseStatement(a) => {
                    let from_module = if a.is_absolute {
                        Some(crate_namespace)
                    } else {
                        None
                    };
                    let mut res = match a.import_type {
                        ImportType::Star => namespace.star_import(from_module, a.call_path),
                        ImportType::SelfImport => {
                            namespace.self_import(from_module, a.call_path, a.alias)
                        }
                        ImportType::Item(s) => {
                            namespace.item_import(from_module, a.call_path, &s, a.alias)
                        }
                    };
                    warnings.append(&mut res.warnings);
                    errors.append(&mut res.errors);
                    TypedAstNodeContent::SideEffect
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
                    TypedAstNodeContent::Declaration(match a {
                        Declaration::VariableDeclaration(VariableDeclaration {
                            name,
                            type_ascription,
                            type_ascription_span,
                            body,
                            is_mutable,
                        }) => {
                            let type_ascription = namespace
                                .resolve_type_with_self(type_ascription, self_type)
                                .unwrap_or_else(|_| {
                                    errors.push(CompileError::UnknownType {
                                        span: type_ascription_span.expect("Invariant violated: type checked an annotation that did not exist in the source"),
                                    });
                                    insert_type(TypeInfo::ErrorRecovery)
                                });
                            let result = {
                                TypedExpression::type_check(TypeCheckArguments {
                                    checkee: body,
                                    namespace,
                                    crate_namespace,
                                    return_type_annotation: type_ascription,
                                    help_text: "Variable declaration's type annotation does \
                     not match up with the assigned expression's type.",
                                    self_type,
                                    build_config,
                                    dead_code_graph,
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
                            namespace.insert(name, typed_var_decl.clone());
                            typed_var_decl
                        }
                        Declaration::ConstantDeclaration(ConstantDeclaration {
                            name,
                            type_ascription,
                            value,
                            visibility,
                        }) => {
                            let result = type_check_ascribed_expr(
                                namespace,
                                crate_namespace,
                                type_ascription.clone(),
                                value,
                            );
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
                            namespace.insert(name, typed_const_decl.clone());
                            typed_const_decl
                        }
                        Declaration::EnumDeclaration(e) => {
                            let decl = TypedDeclaration::EnumDeclaration(
                                e.to_typed_decl(namespace, self_type),
                            );

                            let _ = check!(
                                namespace.insert(e.name, decl.clone()),
                                return err(warnings, errors),
                                warnings,
                                errors
                            );
                            decl
                        }
                        Declaration::FunctionDeclaration(fn_decl) => {
                            let decl = check!(
                                TypedFunctionDeclaration::type_check(TypeCheckArguments {
                                    checkee: fn_decl.clone(),
                                    namespace,
                                    crate_namespace,
                                    return_type_annotation: insert_type(TypeInfo::Unknown),
                                    help_text,
                                    self_type,
                                    build_config,
                                    dead_code_graph,
                                    mode: Mode::NonAbi,
                                    opts
                                }),
                                error_recovery_function_declaration(fn_decl),
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
                            supertraits,
                            visibility,
                        }) => {
                            // type check the interface surface
                            let interface_surface = check!(
                                type_check_interface_surface(interface_surface, namespace),
                                return err(warnings, errors),
                                warnings,
                                errors
                            );

                            // Error checking. Make sure that each supertrait exists and that none
                            // of the supertraits are actually an ABI declaration
                            for supertrait in &supertraits {
                                match namespace
                                    .get_call_path(&supertrait.name)
                                    .ok(&mut warnings, &mut errors)
                                {
                                    Some(TypedDeclaration::TraitDeclaration(_)) => (),
                                    Some(TypedDeclaration::AbiDeclaration(_)) => {
                                        errors.push(CompileError::AbiAsSupertrait {
                                            span: name.span().clone(),
                                        })
                                    }
                                    _ => errors.push(CompileError::TraitNotFound {
                                        name: supertrait.name.span().as_str().to_string(),
                                        span: name.span().clone(),
                                    }),
                                }
                            }

                            let trait_namespace = create_new_scope(namespace);
                            // insert placeholder functions representing the interface surface
                            // to allow methods to use those functions
                            trait_namespace.insert_trait_implementation(
                                CallPath {
                                    prefixes: vec![],
                                    suffix: name.clone(),
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
                                    methods.clone(),
                                    trait_namespace,
                                    crate_namespace,
                                    insert_type(TypeInfo::SelfType),
                                    build_config,
                                    dead_code_graph,
                                ),
                                vec![],
                                warnings,
                                errors
                            );
                            let trait_decl =
                                TypedDeclaration::TraitDeclaration(TypedTraitDeclaration {
                                    name: name.clone(),
                                    interface_surface,
                                    methods,
                                    type_parameters,
                                    supertraits,
                                    visibility,
                                });
                            namespace.insert(name, trait_decl.clone());
                            trait_decl
                        }
                        Declaration::Reassignment(Reassignment { lhs, rhs, span }) => {
                            check!(
                                reassignment(
                                    TypeCheckArguments {
                                        checkee: (lhs, rhs),
                                        namespace,
                                        crate_namespace,
                                        self_type,
                                        build_config,
                                        dead_code_graph,
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
                            implementation_of_trait(
                                impl_trait,
                                namespace,
                                crate_namespace,
                                build_config,
                                dead_code_graph,
                                opts,
                            ),
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
                            // Resolve the Self type as it's most likely still 'Custom' and use the
                            // resolved type for self instead.
                            let implementing_for_type_id =
                                namespace.resolve_type_without_self(&type_implementing_for);
                            let type_implementing_for = look_up_type_id(implementing_for_type_id);

                            let mut functions_buf: Vec<TypedFunctionDeclaration> = vec![];
                            if !type_arguments.is_empty() {
                                errors.push(CompileError::Internal(
                                    "Where clauses are not supported yet.",
                                    type_arguments[0].clone().name_ident.span().clone(),
                                ));
                            }
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

                                functions_buf.push(check!(
                                    TypedFunctionDeclaration::type_check(TypeCheckArguments {
                                        checkee: fn_decl,
                                        namespace,
                                        crate_namespace,
                                        return_type_annotation: insert_type(TypeInfo::Unknown),
                                        help_text: Default::default(),
                                        self_type: implementing_for_type_id,
                                        build_config,
                                        dead_code_graph,
                                        mode: Mode::NonAbi,
                                        opts
                                    }),
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
                                look_up_type_id(implementing_for_type_id),
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
                            // look up any generic or struct types in the namespace
                            // insert type parameters
                            let type_mapping = insert_type_parameters(&decl.type_parameters);
                            let fields = decl
                                .fields
                                .into_iter()
                                .map(
                                    |StructField {
                                         name,
                                         r#type,
                                         span,
                                         type_span,
                                     }| TypedStructField {
                                        name,
                                        r#type: if let Some(matching_id) =
                                            r#type.matches_type_parameter(&type_mapping)
                                        {
                                            insert_type(TypeInfo::Ref(matching_id))
                                        } else {
                                            namespace
                                                .resolve_type_with_self(r#type, self_type)
                                                .unwrap_or_else(|_| {
                                                    errors.push(CompileError::UnknownType {
                                                        span: type_span.clone(),
                                                    });
                                                    insert_type(TypeInfo::ErrorRecovery)
                                                })
                                        },
                                        span,
                                    },
                                )
                                .collect::<Vec<_>>();
                            let decl = TypedStructDeclaration {
                                name: decl.name.clone(),
                                type_parameters: decl.type_parameters.clone(),
                                fields,
                                visibility: decl.visibility,
                            };

                            // insert struct into namespace
                            let _ = check!(
                                namespace.insert(
                                    decl.name.clone(),
                                    TypedDeclaration::StructDeclaration(decl.clone()),
                                ),
                                return err(warnings, errors),
                                warnings,
                                errors
                            );

                            TypedDeclaration::StructDeclaration(decl)
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
                                type_check_trait_methods(
                                    methods.clone(),
                                    namespace,
                                    crate_namespace,
                                    self_type,
                                    build_config,
                                    dead_code_graph,
                                ),
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
                            namespace.insert(name, decl.clone());
                            decl
                        }
                        Declaration::StorageDeclaration(StorageDeclaration { span, .. }) => {
                            errors.push(CompileError::Unimplemented(
                                "Storage declarations are not supported yet. Coming soon!",
                                span,
                            ));
                            return err(warnings, errors);
                        }
                    })
                }
                AstNodeContent::Expression(a) => {
                    let inner = check!(
                        TypedExpression::type_check(TypeCheckArguments {
                            checkee: a.clone(),
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
                                crate_namespace,
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
                                build_config,
                                dead_code_graph,
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
                            crate_namespace,
                            return_type_annotation,
                            help_text: "Implicit return must match up with block's type.",
                            self_type,
                            build_config,
                            dead_code_graph,
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
                            crate_namespace,
                            return_type_annotation: insert_type(TypeInfo::Boolean),
                            help_text:
                                "A while loop's loop condition must be a boolean expression.",
                            self_type,
                            build_config,
                            dead_code_graph,
                            mode: Mode::NonAbi,
                            opts
                        }),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    let (typed_body, _block_implicit_return) = check!(
                        TypedCodeBlock::type_check(TypeCheckArguments {
                            checkee: body.clone(),
                            namespace,
                            crate_namespace,
                            return_type_annotation: insert_type(TypeInfo::Tuple(Vec::new())),
                            help_text:
                                "A while loop's loop body cannot implicitly return a value.Try \
                             assigning it to a mutable variable declared outside of the loop \
                             instead.",
                            self_type,
                            build_config,
                            dead_code_graph,
                            mode: Mode::NonAbi,
                            opts,
                        }),
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

fn type_check_interface_surface(
    interface_surface: Vec<TraitFn>,
    namespace: crate::semantic_analysis::NamespaceRef,
) -> CompileResult<Vec<TypedTraitFn>> {
    let mut errors = vec![];
    ok(
        interface_surface
            .into_iter()
            .map(
                |TraitFn {
                     name,
                     parameters,
                     return_type,
                     return_type_span,
                 }| TypedTraitFn {
                    name,
                    return_type_span: return_type_span.clone(),
                    parameters: parameters
                        .into_iter()
                        .map(
                            |FunctionParameter {
                                 name,
                                 r#type,
                                 type_span,
                             }| TypedFunctionParameter {
                                name,
                                r#type: namespace
                                    .resolve_type_with_self(
                                        r#type,
                                        crate::type_engine::insert_type(TypeInfo::SelfType),
                                    )
                                    .unwrap_or_else(|_| {
                                        errors.push(CompileError::UnknownType {
                                            span: type_span.clone(),
                                        });
                                        insert_type(TypeInfo::ErrorRecovery)
                                    }),
                                type_span,
                            },
                        )
                        .collect(),
                    return_type: namespace
                        .resolve_type_with_self(
                            return_type,
                            crate::type_engine::insert_type(TypeInfo::SelfType),
                        )
                        .unwrap_or_else(|_| {
                            errors.push(CompileError::UnknownType {
                                span: return_type_span,
                            });
                            insert_type(TypeInfo::ErrorRecovery)
                        }),
                },
            )
            .collect::<Vec<_>>(),
        vec![],
        errors,
    )
}

fn type_check_trait_methods(
    methods: Vec<FunctionDeclaration>,
    namespace: crate::semantic_analysis::NamespaceRef,
    crate_namespace: NamespaceRef,
    self_type: TypeId,
    build_config: &BuildConfig,
    dead_code_graph: &mut ControlFlowGraph,
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
        let function_namespace = namespace;
        parameters.clone().into_iter().for_each(
            |FunctionParameter {
                 name, ref r#type, ..
             }| {
                let r#type = function_namespace
                    .resolve_type_with_self(
                        r#type.clone(),
                        crate::type_engine::insert_type(TypeInfo::SelfType),
                    )
                    .unwrap_or_else(|_| {
                        errors.push(CompileError::UnknownType {
                            span: name.span().clone(),
                        });
                        insert_type(TypeInfo::ErrorRecovery)
                    });
                function_namespace.insert(
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
            if let TypeInfo::Custom { ref name } = param.r#type {
                generic_params_buf_for_error_message.push(name.to_string());
            }
        }
        let comma_separated_generic_params = generic_params_buf_for_error_message.join(", ");
        for FunctionParameter {
            ref r#type, name, ..
        } in parameters.iter()
        {
            let span = name.span().clone();
            if let TypeInfo::Custom { name, .. } = r#type {
                let args_span = parameters.iter().fold(
                    parameters[0].name.span().clone(),
                    |acc, FunctionParameter { name, .. }| join_spans(acc, name.span().clone()),
                );
                if type_parameters.iter().any(
                    |TypeParameter {
                         name: this_name, ..
                     }| {
                        if let TypeInfo::Custom { name: this_name } = this_name {
                            this_name == name
                        } else {
                            false
                        }
                    },
                ) {
                    errors.push(CompileError::TypeParameterNotInTypeScope {
                        name: name.to_string(),
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
                     r#type,
                     type_span,
                 }| {
                    TypedFunctionParameter {
                        name,
                        r#type: function_namespace
                            .resolve_type_with_self(
                                r#type,
                                crate::type_engine::insert_type(TypeInfo::SelfType),
                            )
                            .unwrap_or_else(|_| {
                                errors.push(CompileError::UnknownType {
                                    span: type_span.clone(),
                                });
                                insert_type(TypeInfo::ErrorRecovery)
                            }),
                        type_span,
                    }
                },
            )
            .collect::<Vec<_>>();

        // TODO check code block implicit return
        let return_type = function_namespace
            .resolve_type_with_self(return_type, self_type)
            .unwrap_or_else(|_| {
                errors.push(CompileError::UnknownType {
                    span: return_type_span.clone(),
                });
                insert_type(TypeInfo::ErrorRecovery)
            });
        let (body, _code_block_implicit_return) = check!(
            TypedCodeBlock::type_check(TypeCheckArguments {
                checkee: body,
                namespace: function_namespace,
                crate_namespace,
                return_type_annotation: return_type,
                help_text: "Trait method body's return type does not match up with \
                                         its return type annotation.",
                self_type,
                build_config,
                dead_code_graph,
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
            whole_block_span: span.clone(),
        },
        span,
        is_contract_call: false,
        return_type_span,
        parameters: Default::default(),
        visibility,
        return_type: crate::type_engine::insert_type(return_type),
        type_parameters: Default::default(),
    }
}

/// Describes each field being drilled down into in storage and its type.
#[derive(Clone, Debug)]
pub struct TypeCheckedStorageReassignment {
    pub(crate) fields: Vec<TypeCheckedStorageReassignDescriptor>,
    pub(crate) ix: StateIndex,
    pub(crate) rhs: TypedExpression,
}

impl TypeCheckedStorageReassignment {
    pub fn span(&self) -> Span {
        self.fields
            .iter()
            .fold(self.fields[0].span.clone(), |acc, field| {
                join_spans(acc, field.span.clone())
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
#[derive(Clone, Debug)]
pub struct TypeCheckedStorageReassignDescriptor {
    pub(crate) name: Ident,
    pub(crate) r#type: TypeId,
    pub(crate) span: Span,
}
