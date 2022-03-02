use super::{
    error_recovery_expr,
    impl_trait::{implementation_of_trait, Mode},
    IsConstant, TypedCodeBlock, TypedExpression, TypedExpressionVariant,
};
use crate::{
    control_flow_analysis::ControlFlowGraph,
    error::*,
    parse_tree::*,
    semantic_analysis::{ast_node::reassignment, create_new_scope, TCOpts, TypeCheckArguments},
    type_engine::*,
    BuildConfig, Ident, NamespaceRef, NamespaceWrapper,
};

use sway_types::{join_spans, span::Span, Property};

mod function;
mod variable;
pub use function::*;
pub use variable::*;

#[derive(Clone, Debug)]
pub enum TypedDeclaration {
    VariableDeclaration(TypedVariableDeclaration),
    ConstantDeclaration(TypedConstantDeclaration),
    FunctionDeclaration(TypedFunctionDeclaration),
    TraitDeclaration(TypedTraitDeclaration),
    StructDeclaration(TypedStructDeclaration),
    EnumDeclaration(TypedEnumDeclaration),
    Reassignment(TypedReassignment),
    ImplTrait {
        trait_name: CallPath,
        span: Span,
        methods: Vec<TypedFunctionDeclaration>,
        type_implementing_for: TypeInfo,
    },
    AbiDeclaration(TypedAbiDeclaration),
    // If type parameters are defined for a function, they are put in the namespace just for
    // the body of that function.
    GenericTypeForFunctionScope {
        name: Ident,
    },
    ErrorRecovery,
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

impl TypedDeclaration {
    /// The entry point to monomorphizing typed declarations. Instantiates all new type ids,
    /// assuming `self` has already been copied.
    pub(crate) fn copy_types(&mut self, type_mapping: &[(TypeParameter, TypeId)]) {
        use TypedDeclaration::*;
        match self {
            VariableDeclaration(ref mut var_decl) => var_decl.copy_types(type_mapping),
            ConstantDeclaration(ref mut const_decl) => const_decl.copy_types(type_mapping),
            FunctionDeclaration(ref mut fn_decl) => fn_decl.copy_types(type_mapping),
            TraitDeclaration(ref mut trait_decl) => trait_decl.copy_types(type_mapping),
            StructDeclaration(ref mut struct_decl) => struct_decl.copy_types(type_mapping),
            EnumDeclaration(ref mut enum_decl) => enum_decl.copy_types(type_mapping),
            Reassignment(ref mut reassignment) => reassignment.copy_types(type_mapping),
            ImplTrait {
                ref mut methods, ..
            } => {
                methods.iter_mut().for_each(|x| x.copy_types(type_mapping));
            }
            // generics in an ABI is unsupported by design
            AbiDeclaration(..) => (),
            GenericTypeForFunctionScope { .. } | ErrorRecovery => (),
        }
    }

    pub(crate) fn type_check(
        arguments: TypeCheckArguments<'_, Declaration>,
        node_span: Span,
    ) -> CompileResult<Self> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let TypeCheckArguments {
            checkee: decl,
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
        match decl {
            Declaration::VariableDeclaration(VariableDeclaration {
                name,
                type_ascription,
                type_ascription_span,
                body,
                is_mutable,
            }) => {
                let args = TypeCheckArguments {
                    checkee: (
                        name,
                        type_ascription,
                        type_ascription_span,
                        body,
                        is_mutable,
                    ),
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
                Self::type_check_variable_declaration(args)
            }
            Declaration::ConstantDeclaration(ConstantDeclaration {
                name,
                type_ascription,
                value,
                visibility,
            }) => {
                let args = TypeCheckArguments {
                    checkee: (name, type_ascription, value, visibility),
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
                Self::type_check_constant_declaration(args, node_span)
            }
            Declaration::EnumDeclaration(e) => {
                let decl = TypedDeclaration::EnumDeclaration(e.to_typed_decl(namespace, self_type));
                let _ = check!(
                    namespace.insert(e.name, decl.clone()),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                ok(decl, warnings, errors)
            }
            Declaration::FunctionDeclaration(fn_decl) => {
                let args = TypeCheckArguments {
                    checkee: fn_decl.clone(),
                    namespace,
                    crate_namespace,
                    return_type_annotation: insert_type(TypeInfo::Unknown),
                    help_text,
                    self_type,
                    build_config,
                    dead_code_graph,
                    mode: Mode::NonAbi,
                    opts,
                };
                let decl = check!(
                    TypedFunctionDeclaration::type_check(args),
                    error_recovery_function_declaration(fn_decl),
                    warnings,
                    errors
                );
                namespace.insert(
                    decl.name.clone(),
                    TypedDeclaration::FunctionDeclaration(decl.clone()),
                );
                ok(
                    TypedDeclaration::FunctionDeclaration(decl),
                    warnings,
                    errors,
                )
            }
            Declaration::TraitDeclaration(TraitDeclaration {
                name,
                interface_surface,
                methods,
                type_parameters,
                supertraits,
                visibility,
            }) => {
                let args = TypeCheckArguments {
                    checkee: (
                        name,
                        interface_surface,
                        methods,
                        type_parameters,
                        supertraits,
                        visibility,
                    ),
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
                Self::type_check_trait_declaration(args)
            }
            Declaration::Reassignment(Reassignment { lhs, rhs, span }) => {
                let args = TypeCheckArguments {
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
                };
                reassignment(args, span)
            }
            Declaration::ImplTrait(impl_trait) => implementation_of_trait(
                impl_trait,
                namespace,
                crate_namespace,
                build_config,
                dead_code_graph,
                opts,
            ),
            Declaration::ImplSelf(ImplSelf {
                type_arguments,
                functions,
                type_implementing_for,
                block_span,
                ..
            }) => {
                let args = TypeCheckArguments {
                    checkee: (type_arguments, functions, type_implementing_for, block_span),
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
                Self::type_check_impl_self(args)
            }
            Declaration::StructDeclaration(decl) => {
                let args = TypeCheckArguments {
                    checkee: decl,
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
                Self::type_check_struct_declaration(args)
            }
            Declaration::AbiDeclaration(AbiDeclaration {
                name,
                interface_surface,
                methods,
                span,
            }) => {
                let args = TypeCheckArguments {
                    checkee: (name, interface_surface, methods, span),
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
                Self::type_check_abi_declaration(args)
            }
            Declaration::StorageDeclaration(StorageDeclaration { span, .. }) => {
                errors.push(CompileError::Unimplemented(
                    "Storage declarations are not supported yet. Coming soon!",
                    span,
                ));
                err(warnings, errors)
            }
        }
    }

    fn type_check_variable_declaration(
        arguments: TypeCheckArguments<'_, (Ident, TypeInfo, Option<Span>, Expression, bool)>,
    ) -> CompileResult<Self> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let TypeCheckArguments {
            checkee: (name, type_ascription, type_ascription_span, body, is_mutable),
            namespace,
            crate_namespace,
            self_type,
            build_config,
            dead_code_graph,
            opts,
            ..
        } = arguments;
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
        let typed_var_decl = TypedDeclaration::VariableDeclaration(TypedVariableDeclaration {
            name: name.clone(),
            body,
            is_mutable: is_mutable.into(),
            const_decl_origin: false,
            type_ascription,
        });
        namespace.insert(name, typed_var_decl.clone());
        ok(typed_var_decl, warnings, errors)
    }

    fn type_check_constant_declaration(
        arguments: TypeCheckArguments<'_, (Ident, TypeInfo, Expression, Visibility)>,
        node_span: Span,
    ) -> CompileResult<Self> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let TypeCheckArguments {
            checkee: (name, type_ascription, value, visibility),
            namespace,
            crate_namespace,
            self_type,
            build_config,
            dead_code_graph,
            opts,
            ..
        } = arguments;
        let type_id = namespace
            .resolve_type_with_self(type_ascription.clone(), self_type)
            .unwrap_or_else(|_| {
                errors.push(CompileError::UnknownType { span: node_span });
                insert_type(TypeInfo::ErrorRecovery)
            });
        let value_args = TypeCheckArguments {
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
        };
        let value = check!(
            TypedExpression::type_check(value_args),
            error_recovery_expr(name.span().clone()),
            warnings,
            errors
        );
        let typed_const_decl = TypedDeclaration::VariableDeclaration(TypedVariableDeclaration {
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
        ok(typed_const_decl, warnings, errors)
    }

    #[allow(clippy::type_complexity)]
    fn type_check_trait_declaration(
        arguments: TypeCheckArguments<
            '_,
            (
                Ident,
                Vec<TraitFn>,
                Vec<FunctionDeclaration>,
                Vec<TypeParameter>,
                Vec<Supertrait>,
                Visibility,
            ),
        >,
    ) -> CompileResult<Self> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let TypeCheckArguments {
            checkee: (name, interface_surface, methods, type_parameters, supertraits, visibility),
            namespace,
            crate_namespace,
            build_config,
            dead_code_graph,
            ..
        } = arguments;
        // type check the interface surface
        let interface_surface = check!(
            Self::type_check_interface_surface(interface_surface, namespace),
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
            Self::type_check_trait_methods(
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
        let trait_decl = TypedDeclaration::TraitDeclaration(TypedTraitDeclaration {
            name: name.clone(),
            interface_surface,
            methods,
            type_parameters,
            supertraits,
            visibility,
        });
        namespace.insert(name, trait_decl.clone());
        ok(trait_decl, warnings, errors)
    }

    fn type_check_impl_self(
        arguments: TypeCheckArguments<
            '_,
            (Vec<TypeParameter>, Vec<FunctionDeclaration>, TypeInfo, Span),
        >,
    ) -> CompileResult<Self> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let TypeCheckArguments {
            checkee: (type_arguments, functions, type_implementing_for, block_span),
            namespace,
            crate_namespace,
            build_config,
            dead_code_graph,
            opts,
            ..
        } = arguments;
        let implementing_for_type_id = namespace.resolve_type_without_self(&type_implementing_for);
        // check, if this is a custom type, if it is in scope or a generic.
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
                    help_text: "",
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
        let decl = TypedDeclaration::ImplTrait {
            trait_name,
            span: block_span,
            methods: functions_buf,
            type_implementing_for,
        };
        ok(decl, warnings, errors)
    }

    fn type_check_abi_declaration(
        arguments: TypeCheckArguments<'_, (Ident, Vec<TraitFn>, Vec<FunctionDeclaration>, Span)>,
    ) -> CompileResult<Self> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let TypeCheckArguments {
            checkee: (name, interface_surface, methods, span),
            namespace,
            crate_namespace,
            self_type,
            build_config,
            dead_code_graph,
            ..
        } = arguments;
        // type check the interface surface and methods
        // We don't want the user to waste resources by contract calling
        // themselves, and we don't want to do more work in the compiler,
        // so we don't support the case of calling a contract's own interface
        // from itself. This is by design.
        let interface_surface = check!(
            Self::type_check_interface_surface(interface_surface, namespace),
            return err(warnings, errors),
            warnings,
            errors
        );
        // type check these for errors but don't actually use them yet -- the real
        // ones will be type checked with proper symbols when the ABI is implemented
        let _methods = check!(
            Self::type_check_trait_methods(
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
        ok(decl, warnings, errors)
    }

    fn type_check_struct_declaration(
        arguments: TypeCheckArguments<'_, StructDeclaration>,
    ) -> CompileResult<Self> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let TypeCheckArguments {
            checkee: decl,
            namespace,
            self_type,
            ..
        } = arguments;
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
                    r#type: if let Some(matching_id) = r#type.matches_type_parameter(&type_mapping)
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
        ok(TypedDeclaration::StructDeclaration(decl), warnings, errors)
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
}

impl TypedDeclaration {
    /// friendly name string used for error reporting.
    pub(crate) fn friendly_name(&self) -> &'static str {
        use TypedDeclaration::*;
        match self {
            VariableDeclaration(_) => "variable",
            ConstantDeclaration(_) => "constant",
            FunctionDeclaration(_) => "function",
            TraitDeclaration(_) => "trait",
            StructDeclaration(_) => "struct",
            EnumDeclaration(_) => "enum",
            Reassignment(_) => "reassignment",
            ImplTrait { .. } => "impl trait",
            AbiDeclaration(..) => "abi",
            GenericTypeForFunctionScope { .. } => "generic type parameter",
            ErrorRecovery => "error",
        }
    }
    pub(crate) fn return_type(&self) -> CompileResult<TypeId> {
        ok(
            match self {
                TypedDeclaration::VariableDeclaration(TypedVariableDeclaration {
                    body, ..
                }) => body.return_type,
                TypedDeclaration::FunctionDeclaration { .. } => {
                    return err(
                        vec![],
                        vec![CompileError::Unimplemented(
                            "Function pointers have not yet been implemented.",
                            self.span(),
                        )],
                    )
                }
                TypedDeclaration::StructDeclaration(TypedStructDeclaration {
                    name,
                    fields,
                    ..
                }) => crate::type_engine::insert_type(TypeInfo::Struct {
                    name: name.as_str().to_string(),
                    fields: fields
                        .iter()
                        .map(TypedStructField::as_owned_typed_struct_field)
                        .collect(),
                }),
                TypedDeclaration::Reassignment(TypedReassignment { rhs, .. }) => rhs.return_type,
                TypedDeclaration::GenericTypeForFunctionScope { name } => {
                    insert_type(TypeInfo::UnknownGeneric { name: name.clone() })
                }
                decl => {
                    return err(
                        vec![],
                        vec![CompileError::NotAType {
                            span: decl.span(),
                            name: decl.pretty_print(),
                            actually_is: decl.friendly_name(),
                        }],
                    )
                }
            },
            vec![],
            vec![],
        )
    }

    pub(crate) fn span(&self) -> Span {
        use TypedDeclaration::*;
        match self {
            VariableDeclaration(TypedVariableDeclaration { name, .. }) => name.span().clone(),
            ConstantDeclaration(TypedConstantDeclaration { name, .. }) => name.span().clone(),
            FunctionDeclaration(TypedFunctionDeclaration { span, .. }) => span.clone(),
            TraitDeclaration(TypedTraitDeclaration { name, .. }) => name.span().clone(),
            StructDeclaration(TypedStructDeclaration { name, .. }) => name.span().clone(),
            EnumDeclaration(TypedEnumDeclaration { span, .. }) => span.clone(),
            Reassignment(TypedReassignment { lhs, .. }) => lhs
                .iter()
                .fold(lhs[0].span(), |acc, this| join_spans(acc, this.span())),
            AbiDeclaration(TypedAbiDeclaration { span, .. }) => span.clone(),
            ImplTrait { span, .. } => span.clone(),
            ErrorRecovery | GenericTypeForFunctionScope { .. } => {
                unreachable!("No span exists for these ast node types")
            }
        }
    }

    pub(crate) fn pretty_print(&self) -> String {
        format!(
            "{} declaration ({})",
            self.friendly_name(),
            match self {
                TypedDeclaration::VariableDeclaration(TypedVariableDeclaration {
                    is_mutable,
                    name,
                    ..
                }) => format!(
                    "{} {}",
                    match is_mutable {
                        VariableMutability::Mutable => "mut",
                        VariableMutability::Immutable => "",
                        VariableMutability::ExportedConst => "pub const",
                    },
                    name.as_str()
                ),
                TypedDeclaration::FunctionDeclaration(TypedFunctionDeclaration {
                    name, ..
                }) => {
                    name.as_str().into()
                }
                TypedDeclaration::TraitDeclaration(TypedTraitDeclaration { name, .. }) =>
                    name.as_str().into(),
                TypedDeclaration::StructDeclaration(TypedStructDeclaration { name, .. }) =>
                    name.as_str().into(),
                TypedDeclaration::EnumDeclaration(TypedEnumDeclaration { name, .. }) =>
                    name.as_str().into(),
                TypedDeclaration::Reassignment(TypedReassignment { lhs, .. }) => lhs
                    .iter()
                    .map(|x| x.name.as_str())
                    .collect::<Vec<_>>()
                    .join("."),
                _ => String::new(),
            }
        )
    }

    pub(crate) fn visibility(&self) -> Visibility {
        use TypedDeclaration::*;
        match self {
            GenericTypeForFunctionScope { .. }
            | Reassignment(..)
            | ImplTrait { .. }
            | AbiDeclaration(..)
            | ErrorRecovery => Visibility::Public,
            VariableDeclaration(TypedVariableDeclaration { is_mutable, .. }) => {
                is_mutable.visibility()
            }
            EnumDeclaration(TypedEnumDeclaration { visibility, .. })
            | ConstantDeclaration(TypedConstantDeclaration { visibility, .. })
            | FunctionDeclaration(TypedFunctionDeclaration { visibility, .. })
            | TraitDeclaration(TypedTraitDeclaration { visibility, .. })
            | StructDeclaration(TypedStructDeclaration { visibility, .. }) => *visibility,
        }
    }
}

/// A `TypedAbiDeclaration` contains the type-checked version of the parse tree's `AbiDeclaration`.
#[derive(Clone, Debug)]
pub struct TypedAbiDeclaration {
    /// The name of the abi trait (also known as a "contract trait")
    pub(crate) name: Ident,
    /// The methods a contract is required to implement in order opt in to this interface
    pub(crate) interface_surface: Vec<TypedTraitFn>,
    /// The methods provided to a contract "for free" upon opting in to this interface
    pub(crate) methods: Vec<FunctionDeclaration>,
    pub(crate) span: Span,
}

#[derive(Clone, Debug)]
pub struct TypedStructDeclaration {
    pub(crate) name: Ident,
    pub(crate) fields: Vec<TypedStructField>,
    pub(crate) type_parameters: Vec<TypeParameter>,
    pub(crate) visibility: Visibility,
}

impl TypedStructDeclaration {
    pub(crate) fn monomorphize(&self) -> Self {
        let mut new_decl = self.clone();
        let type_mapping = insert_type_parameters(&self.type_parameters);
        new_decl.copy_types(&type_mapping);
        new_decl
    }

    pub(crate) fn copy_types(&mut self, type_mapping: &[(TypeParameter, TypeId)]) {
        self.fields
            .iter_mut()
            .for_each(|x| x.copy_types(type_mapping));
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct TypedStructField {
    pub(crate) name: Ident,
    pub(crate) r#type: TypeId,
    pub(crate) span: Span,
}

// TODO(Static span) -- remove this type and use TypedStructField
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct OwnedTypedStructField {
    pub(crate) name: String,
    pub(crate) r#type: TypeId,
}

impl OwnedTypedStructField {
    pub(crate) fn copy_types(&mut self, type_mapping: &[(TypeParameter, TypeId)]) {
        self.r#type = if let Some(matching_id) =
            look_up_type_id(self.r#type).matches_type_parameter(type_mapping)
        {
            insert_type(TypeInfo::Ref(matching_id))
        } else {
            insert_type(look_up_type_id_raw(self.r#type))
        };
    }

    pub fn generate_json_abi(&self) -> Property {
        Property {
            name: self.name.clone(),
            type_field: self.r#type.json_abi_str(),
            components: self.r#type.generate_json_abi(),
        }
    }
}

impl TypedStructField {
    pub(crate) fn copy_types(&mut self, type_mapping: &[(TypeParameter, TypeId)]) {
        self.r#type = if let Some(matching_id) =
            look_up_type_id(self.r#type).matches_type_parameter(type_mapping)
        {
            insert_type(TypeInfo::Ref(matching_id))
        } else {
            insert_type(look_up_type_id_raw(self.r#type))
        };
    }
    pub(crate) fn as_owned_typed_struct_field(&self) -> OwnedTypedStructField {
        OwnedTypedStructField {
            name: self.name.as_str().to_string(),
            r#type: self.r#type,
        }
    }
}

#[derive(Clone, Debug)]
pub struct TypedEnumDeclaration {
    pub(crate) name: Ident,
    pub(crate) type_parameters: Vec<TypeParameter>,
    pub(crate) variants: Vec<TypedEnumVariant>,
    pub(crate) span: Span,
    pub(crate) visibility: Visibility,
}
impl TypedEnumDeclaration {
    pub(crate) fn monomorphize(&self) -> Self {
        let mut new_decl = self.clone();
        let type_mapping = insert_type_parameters(&self.type_parameters);
        new_decl.copy_types(&type_mapping);
        new_decl
    }
    pub(crate) fn copy_types(&mut self, type_mapping: &[(TypeParameter, TypeId)]) {
        self.variants
            .iter_mut()
            .for_each(|x| x.copy_types(type_mapping));
    }
    /// Returns the [ResolvedType] corresponding to this enum's type.
    pub(crate) fn as_type(&self) -> TypeId {
        crate::type_engine::insert_type(TypeInfo::Enum {
            name: self.name.as_str().to_string(),
            variant_types: self
                .variants
                .iter()
                .map(TypedEnumVariant::as_owned_typed_enum_variant)
                .collect(),
        })
    }
}
#[derive(Debug, Clone)]
pub struct TypedEnumVariant {
    pub(crate) name: Ident,
    pub(crate) r#type: TypeId,
    pub(crate) tag: usize,
    pub(crate) span: Span,
}

impl TypedEnumVariant {
    pub(crate) fn copy_types(&mut self, type_mapping: &[(TypeParameter, TypeId)]) {
        self.r#type = if let Some(matching_id) =
            look_up_type_id(self.r#type).matches_type_parameter(type_mapping)
        {
            insert_type(TypeInfo::Ref(matching_id))
        } else {
            insert_type(look_up_type_id_raw(self.r#type))
        };
    }
    pub(crate) fn as_owned_typed_enum_variant(&self) -> OwnedTypedEnumVariant {
        OwnedTypedEnumVariant {
            name: self.name.as_str().to_string(),
            r#type: self.r#type,
            tag: self.tag,
        }
    }
}

// TODO(Static span) -- remove this type and use TypedEnumVariant
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct OwnedTypedEnumVariant {
    pub(crate) name: String,
    pub(crate) r#type: TypeId,
    pub(crate) tag: usize,
}

impl OwnedTypedEnumVariant {
    pub fn generate_json_abi(&self) -> Property {
        Property {
            name: self.name.clone(),
            type_field: self.r#type.json_abi_str(),
            components: self.r#type.generate_json_abi(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct TypedConstantDeclaration {
    pub(crate) name: Ident,
    pub(crate) value: TypedExpression,
    pub(crate) visibility: Visibility,
}

impl TypedConstantDeclaration {
    pub(crate) fn copy_types(&mut self, type_mapping: &[(TypeParameter, TypeId)]) {
        self.value.copy_types(type_mapping);
    }
}

#[derive(Clone, Debug)]
pub struct TypedTraitDeclaration {
    pub(crate) name: Ident,
    pub(crate) interface_surface: Vec<TypedTraitFn>,
    pub(crate) methods: Vec<FunctionDeclaration>,
    pub(crate) type_parameters: Vec<TypeParameter>,
    pub(crate) supertraits: Vec<Supertrait>,
    pub(crate) visibility: Visibility,
}
impl TypedTraitDeclaration {
    pub(crate) fn copy_types(&mut self, type_mapping: &[(TypeParameter, TypeId)]) {
        let additional_type_map = insert_type_parameters(&self.type_parameters);
        let type_mapping = [type_mapping, &additional_type_map].concat();
        self.interface_surface
            .iter_mut()
            .for_each(|x| x.copy_types(&type_mapping[..]));
        // we don't have to type check the methods because it hasn't been type checked yet
    }
}
#[derive(Clone, Debug)]
pub struct TypedTraitFn {
    pub(crate) name: Ident,
    pub(crate) parameters: Vec<TypedFunctionParameter>,
    pub(crate) return_type: TypeId,
    pub(crate) return_type_span: Span,
}

/// Represents the left hand side of a reassignment -- a name to locate it in the
/// namespace, and the type that the name refers to. The type is used for memory layout
/// in asm generation.
#[derive(Clone, Debug)]
pub struct ReassignmentLhs {
    pub(crate) name: Ident,
    pub(crate) r#type: TypeId,
}

impl ReassignmentLhs {
    pub(crate) fn span(&self) -> Span {
        self.name.span().clone()
    }
}

#[derive(Clone, Debug)]
pub struct TypedReassignment {
    // either a direct variable, so length of 1, or
    // at series of struct fields/array indices (array syntax)
    pub(crate) lhs: Vec<ReassignmentLhs>,
    pub(crate) rhs: TypedExpression,
}

impl TypedReassignment {
    pub(crate) fn copy_types(&mut self, type_mapping: &[(TypeParameter, TypeId)]) {
        self.rhs.copy_types(type_mapping);
        self.lhs
            .iter_mut()
            .for_each(|ReassignmentLhs { ref mut r#type, .. }| {
                *r#type = if let Some(matching_id) =
                    look_up_type_id(*r#type).matches_type_parameter(type_mapping)
                {
                    insert_type(TypeInfo::Ref(matching_id))
                } else {
                    insert_type(look_up_type_id_raw(*r#type))
                };
            });
    }
}

impl TypedTraitFn {
    pub(crate) fn copy_types(&mut self, type_mapping: &[(TypeParameter, TypeId)]) {
        self.return_type = if let Some(matching_id) =
            look_up_type_id(self.return_type).matches_type_parameter(type_mapping)
        {
            insert_type(TypeInfo::Ref(matching_id))
        } else {
            insert_type(look_up_type_id_raw(self.return_type))
        };
    }
    /// This function is used in trait declarations to insert "placeholder" functions
    /// in the methods. This allows the methods to use functions declared in the
    /// interface surface.
    pub(crate) fn to_dummy_func(&self, mode: Mode) -> TypedFunctionDeclaration {
        TypedFunctionDeclaration {
            purity: Default::default(),
            name: self.name.clone(),
            body: TypedCodeBlock {
                contents: vec![],
                whole_block_span: self.name.span().clone(),
            },
            parameters: self.parameters.clone(),
            span: self.name.span().clone(),
            return_type: self.return_type,
            return_type_span: self.return_type_span.clone(),
            visibility: Visibility::Public,
            type_parameters: vec![],
            is_contract_call: mode == Mode::ImplAbiFn,
        }
    }
}
