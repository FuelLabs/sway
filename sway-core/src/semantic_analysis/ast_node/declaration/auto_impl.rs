use crate::{
    decl_engine::{parsed_engine::ParsedDeclEngineInsert, DeclEngineGet},
    language::{
        parsed::{
            AstNode, AstNodeContent, CodeBlock, Declaration, Expression, ExpressionKind,
            FunctionDeclaration, FunctionParameter, ImplItem, ImplTrait, MatchBranch,
            MatchExpression, MethodApplicationExpression, MethodName, Scrutinee,
            SubfieldExpression,
        },
        ty::{self, TyAstNode, TyDecl},
        CallPath, QualifiedCallPath,
    },
    semantic_analysis::{type_check_context::EnforceTypeArguments, TypeCheckContext},
    transform::AttributesMap,
    Engines, TraitConstraint, TypeArgs, TypeArgument, TypeBinding, TypeId, TypeInfo, TypeParameter,
};
use sway_error::handler::Handler;
use sway_types::{Ident, Span};

/// Contains all information needed to implement AbiEncode
pub struct AutoImplAbiEncodeContext<'a, 'b> {
    ctx: &'b mut TypeCheckContext<'a>,
    buffer_type_id: TypeId,
    abi_encode_call_path: CallPath,
}

impl<'a, 'b> AutoImplAbiEncodeContext<'a, 'b> {
    /// This function fails if the context does not have access to the "core" module
    pub fn new(ctx: &'b mut TypeCheckContext<'a>) -> Option<Self> {
        let buffer_type_id = Self::resolve_core_codec_buffer(ctx)?;
        Some(Self {
            ctx,
            buffer_type_id,
            abi_encode_call_path: CallPath::absolute(&["core", "codec", "AbiEncode"]),
        })
    }

    fn resolve_core_codec_buffer(ctx: &mut TypeCheckContext<'_>) -> Option<TypeId> {
        let handler = Handler::default();
        let buffer_type_id = ctx.engines.te().insert(
            ctx.engines,
            TypeInfo::Custom {
                qualified_call_path: QualifiedCallPath {
                    call_path: CallPath::absolute(&["core", "codec", "Buffer"]),
                    qualified_path_root: None,
                },
                type_arguments: None,
                root_type_id: None,
            },
            None,
        );
        let buffer_type_id = ctx
            .resolve_type(
                &handler,
                buffer_type_id,
                &Span::dummy(),
                EnforceTypeArguments::No,
                None,
            )
            .ok()?;
        Some(buffer_type_id)
    }

    fn import_core_codec(&mut self) -> bool {
        // Check if the compilation context has acces to the
        // core library.
        let handler = Handler::default();
        let _ = self.ctx.star_import(
            &handler,
            &[
                Ident::new_no_span("core".into()),
                Ident::new_no_span("codec".into()),
            ],
            true,
        );

        !handler.has_errors()
    }

    /// Verify with a enum has all variants that can be auto implemented.
    fn can_enum_auto_impl_abi_encode(&mut self, decl: &ty::TyDecl) -> bool {
        let handler = Handler::default();
        let Ok(enum_decl) = decl
            .to_enum_ref(&handler, self.ctx.engines())
            .map(|enum_ref| self.ctx.engines().de().get(enum_ref.id()))
        else {
            return false;
        };

        let all_variants_are_abi_encode = enum_decl.variants.iter().all(|variant| {
            // If the variant is the generic argument of the enum, we are ok
            // because we will constraint it later
            if self
                .ctx
                .engines()
                .te()
                .get(variant.type_argument.type_id)
                .is_unknown_generic()
            {
                return true;
            }

            // Check variant implements AbiEncode
            self.ctx.check_type_impls_traits(
                variant.type_argument.type_id,
                &[TraitConstraint {
                    trait_name: self.abi_encode_call_path.clone(),
                    type_arguments: vec![],
                }],
            )
        });

        all_variants_are_abi_encode
    }

    /// Auto implements AbiEncode for structs
    fn enum_auto_impl_abi_encode(&mut self, engines: &Engines, decl: &TyDecl) -> Option<TyAstNode> {
        if !self.can_enum_auto_impl_abi_encode(decl) {
            return None;
        }

        let implementing_for_decl_ref = decl.get_enum_decl_ref().unwrap();

        let unit_type_id =
            self.ctx
                .engines
                .te()
                .insert(self.ctx.engines, TypeInfo::Tuple(vec![]), None);

        let enum_decl = self.ctx.engines().de().get(implementing_for_decl_ref.id());

        if !self.import_core_codec() {
            return None;
        }

        // If the enum has generic parameters, they must have AbiEncode appended
        // as new constraint
        let impl_type_parameters: Vec<_> = enum_decl
            .type_parameters
            .iter()
            .map(|type_parameter| {
                let type_id = self.ctx.engines.te().insert(
                    self.ctx.engines(),
                    TypeInfo::Custom {
                        qualified_call_path: QualifiedCallPath {
                            call_path: CallPath {
                                prefixes: vec![],
                                suffix: Ident::new_no_span(
                                    type_parameter.name_ident.as_str().into(),
                                ),
                                is_absolute: false,
                            },
                            qualified_path_root: None,
                        },
                        type_arguments: None,
                        root_type_id: None,
                    },
                    None,
                );

                let mut trait_constraints: Vec<_> = type_parameter
                    .trait_constraints
                    .iter()
                    .map(|x| TraitConstraint {
                        trait_name: CallPath {
                            prefixes: vec![],
                            suffix: Ident::new_no_span(x.trait_name.suffix.as_str().into()),
                            is_absolute: false,
                        },
                        type_arguments: x
                            .type_arguments
                            .iter()
                            .map(|x| {
                                let name = match &*self.ctx.engines.te().get(x.type_id) {
                                    TypeInfo::Custom {
                                        qualified_call_path,
                                        ..
                                    } => Ident::new_no_span(
                                        qualified_call_path.call_path.suffix.as_str().into(),
                                    ),
                                    _ => todo!(),
                                };

                                let type_id = self.ctx.engines.te().insert(
                                    self.ctx.engines(),
                                    TypeInfo::Custom {
                                        qualified_call_path: QualifiedCallPath {
                                            call_path: CallPath {
                                                prefixes: vec![],
                                                suffix: name,
                                                is_absolute: false,
                                            },
                                            qualified_path_root: None,
                                        },
                                        type_arguments: None,
                                        root_type_id: None,
                                    },
                                    None,
                                );

                                TypeArgument {
                                    type_id,
                                    initial_type_id: type_id,
                                    span: Span::dummy(),
                                    call_path_tree: None,
                                }
                            })
                            .collect(),
                    })
                    .collect();

                trait_constraints.push(TraitConstraint {
                    trait_name: CallPath {
                        prefixes: vec![],
                        suffix: Ident::new_no_span("AbiEncode".into()),
                        is_absolute: false,
                    },
                    type_arguments: vec![],
                });

                TypeParameter {
                    type_id,
                    initial_type_id: type_id,
                    name_ident: Ident::new_no_span(type_parameter.name_ident.as_str().into()),
                    trait_constraints,
                    trait_constraints_span: Span::dummy(),
                    is_from_parent: false,
                }
            })
            .collect();

        let implementing_for_type_id = self.ctx.engines.te().insert(
            self.ctx.engines,
            TypeInfo::Custom {
                qualified_call_path: QualifiedCallPath {
                    call_path: CallPath {
                        prefixes: vec![],
                        suffix: enum_decl.call_path.suffix.clone(),
                        is_absolute: false,
                    },
                    qualified_path_root: None,
                },
                type_arguments: if impl_type_parameters.is_empty() {
                    None
                } else {
                    Some(
                        impl_type_parameters
                            .iter()
                            .map(|x| {
                                let type_id = self.ctx.engines().te().insert(
                                    self.ctx.engines(),
                                    TypeInfo::Custom {
                                        qualified_call_path: QualifiedCallPath {
                                            call_path: CallPath {
                                                prefixes: vec![],
                                                suffix: x.name_ident.clone(),
                                                is_absolute: false,
                                            },
                                            qualified_path_root: None,
                                        },
                                        type_arguments: None,
                                        root_type_id: None,
                                    },
                                    None,
                                );

                                TypeArgument {
                                    type_id,
                                    initial_type_id: type_id,
                                    span: Span::dummy(),
                                    call_path_tree: None,
                                }
                            })
                            .collect(),
                    )
                },
                root_type_id: None,
            },
            None,
        );

        let implementing_for = TypeArgument {
            type_id: implementing_for_type_id,
            initial_type_id: implementing_for_type_id,
            span: Span::dummy(),
            call_path_tree: None,
        };

        let impl_trait_item = FunctionDeclaration {
            purity: crate::language::Purity::Pure,
            attributes: AttributesMap::default(),
            name: Ident::new_no_span("abi_encode".into()),
            visibility: crate::language::Visibility::Public,
            body: CodeBlock {
                contents: vec![
                    AstNode {
                        content: AstNodeContent::Expression(
                            Expression {
                                kind: ExpressionKind::Match(
                                    MatchExpression {
                                        value: Box::new(Expression {
                                            kind: ExpressionKind::AmbiguousVariableExpression (
                                                Ident::new_no_span("self".into())
                                            ),
                                            span: Span::dummy()
                                        }),
                                        branches: enum_decl.variants.iter()
                                            .enumerate()
                                            .map(|(i, x)| {
                                                let variant_type = self.ctx.engines().te().get(x.type_argument.type_id);
                                                MatchBranch {
                                                    scrutinee: Scrutinee::EnumScrutinee {
                                                        call_path: CallPath {
                                                            prefixes: vec![
                                                                Ident::new_no_span("Self".into())
                                                            ],
                                                            suffix: Ident::new_no_span(
                                                                x.name.as_str().into()
                                                            ),
                                                            is_absolute: false
                                                        },
                                                        value: Box::new(if variant_type.is_unit() {
                                                            Scrutinee::CatchAll {
                                                                span: Span::dummy()
                                                            }
                                                        } else {
                                                            Scrutinee::Variable {
                                                                name: Ident::new_no_span("x".into()),
                                                                span: Span::dummy()
                                                            }
                                                        }),
                                                        span: Span::dummy(),
                                                    },
                                                    result: Expression {
                                                        kind: ExpressionKind::CodeBlock(
                                                            CodeBlock {
                                                                contents: {
                                                                    let mut contents = vec![];

                                                                    // discriminant
                                                                    contents.push(
                                                                        AstNode {
                                                                            content: AstNodeContent::Expression(
                                                                                Expression {
                                                                                    kind: ExpressionKind::MethodApplication(
                                                                                        Box::new(MethodApplicationExpression {
                                                                                            method_name_binding: TypeBinding {
                                                                                                inner: MethodName::FromModule {
                                                                                                    method_name: Ident::new_no_span("abi_encode".into())
                                                                                                },
                                                                                                type_arguments: TypeArgs::Regular(vec![]),
                                                                                                span: Span::dummy()
                                                                                            },
                                                                                            arguments: vec![
                                                                                                Expression {
                                                                                                    kind: ExpressionKind::Literal(
                                                                                                        crate::language::Literal::U64(
                                                                                                            i as u64
                                                                                                        )
                                                                                                    ),
                                                                                                    span: Span::dummy()
                                                                                                },
                                                                                                Expression {
                                                                                                    kind: ExpressionKind::AmbiguousVariableExpression(
                                                                                                        Ident::new_no_span("buffer".into())
                                                                                                    ),
                                                                                                    span: Span::dummy()
                                                                                                }
                                                                                            ],
                                                                                            contract_call_params: vec![],
                                                                                        })
                                                                                    ),
                                                                                    span: Span::dummy()
                                                                                }
                                                                            ),
                                                                            span: Span::dummy()
                                                                    });

                                                                    // variant data
                                                                    if !variant_type.is_unit() {
                                                                        contents.push(
                                                                            AstNode {
                                                                                content: AstNodeContent::Expression(
                                                                                    Expression {
                                                                                        kind: ExpressionKind::MethodApplication(
                                                                                            Box::new(MethodApplicationExpression {
                                                                                                method_name_binding: TypeBinding {
                                                                                                    inner: MethodName::FromModule {
                                                                                                        method_name: Ident::new_no_span("abi_encode".into())
                                                                                                    },
                                                                                                    type_arguments: TypeArgs::Regular(vec![]),
                                                                                                    span: Span::dummy()
                                                                                                },
                                                                                                arguments: vec![
                                                                                                    Expression {
                                                                                                        kind: ExpressionKind::AmbiguousVariableExpression (
                                                                                                            Ident::new_no_span("x".into())
                                                                                                        ),
                                                                                                        span: Span::dummy()
                                                                                                    },
                                                                                                    Expression {
                                                                                                        kind: ExpressionKind::AmbiguousVariableExpression(
                                                                                                            Ident::new_no_span("buffer".into())
                                                                                                        ),
                                                                                                        span: Span::dummy()
                                                                                                    }
                                                                                                ],
                                                                                                contract_call_params: vec![],
                                                                                            })
                                                                                        ),
                                                                                        span: Span::dummy()
                                                                                    }
                                                                                ),
                                                                                span: Span::dummy()
                                                                            }
                                                                        );
                                                                    }

                                                                    contents
                                                                },
                                                                whole_block_span: Span::dummy()
                                                            }
                                                        ),
                                                        span: Span::dummy()
                                                    },
                                                    span: Span::dummy()
                                                }
                                            }).collect()
                                    }
                                ),
                                span: Span::dummy()
                            }
                        ),
                        span: Span::dummy()
                    }
                ],
                whole_block_span: Span::dummy(),
            },
            parameters: vec![
                FunctionParameter {
                    name: Ident::new_no_span("self".into()),
                    is_reference: false,
                    is_mutable: false,
                    mutability_span: Span::dummy(),
                    type_argument: TypeArgument {
                        type_id: implementing_for_type_id,
                        initial_type_id: implementing_for_type_id,
                        span: Span::dummy(),
                        call_path_tree: None,
                    },
                },
                FunctionParameter {
                    name: Ident::new_no_span("buffer".into()),
                    is_reference: true,
                    is_mutable: true,
                    mutability_span: Span::dummy(),
                    type_argument: TypeArgument {
                        type_id: self.buffer_type_id,
                        initial_type_id: self.buffer_type_id,
                        span: Span::dummy(),
                        call_path_tree: None,
                    },
                },
            ],
            span: Span::dummy(),
            return_type: TypeArgument {
                type_id: unit_type_id,
                initial_type_id: unit_type_id,
                span: Span::dummy(),
                call_path_tree: None,
            },
            type_parameters: vec![],
            where_clause: vec![],
        };
        let impl_trait_item = engines.pe().insert(impl_trait_item);

        let impl_trait = ImplTrait {
            impl_type_parameters,
            trait_name: self.abi_encode_call_path.clone(),
            trait_type_arguments: vec![],
            implementing_for,
            items: vec![ImplItem::Fn(impl_trait_item)],
            block_span: Span::dummy(),
        };
        let impl_trait = engines.pe().insert(impl_trait);
        let impl_trait = Declaration::ImplTrait(impl_trait);

        let handler = Handler::default();
        TyAstNode::type_check(
            &handler,
            self.ctx.by_ref(),
            AstNode {
                content: AstNodeContent::Declaration(impl_trait),
                span: Span::dummy(),
            },
        )
        .ok()
    }

    // Check if a struct can implement AbiEncode
    fn can_struct_auto_impl_abi_encode(&mut self, decl: &TyDecl) -> bool {
        // skip module "core"
        // Because of ordering, we cannot guarantee auto impl
        // for structs inside "core"
        if matches!(self.ctx.namespace.root_module_name(), Some(x) if x.as_str() == "core") {
            return false;
        }

        let Some(decl_ref) = decl.get_struct_decl_ref() else {
            return false;
        };
        let struct_ref = self.ctx.engines().de().get(decl_ref.id());

        // Do not support types with generic constraints
        // because this generates a circular impl trait
        if struct_ref.type_parameters.iter().any(|x| {
            x.trait_constraints
                .iter()
                .any(|c| !c.type_arguments.is_empty())
        }) {
            return false;
        }

        let all_fields_are_abi_encode = struct_ref.fields.iter().all(|field| {
            if let TypeInfo::UnknownGeneric { .. } =
                &*self.ctx.engines().te().get(field.type_argument.type_id)
            {
                return true;
            }

            let handler = Handler::default();
            self.ctx
                .namespace
                .module_mut()
                .current_items_mut()
                .implemented_traits
                .check_if_trait_constraints_are_satisfied_for_type(
                    &handler,
                    field.type_argument.type_id,
                    &[TraitConstraint {
                        trait_name: CallPath {
                            prefixes: vec![
                                Ident::new_no_span("core".into()),
                                Ident::new_no_span("codec".into()),
                            ],
                            suffix: Ident::new_no_span("AbiEncode".into()),
                            is_absolute: true,
                        },
                        type_arguments: vec![],
                    }],
                    &Span::dummy(),
                    self.ctx.engines,
                    crate::namespace::TryInsertingTraitImplOnFailure::Yes,
                )
                .is_ok()
        });

        all_fields_are_abi_encode
    }

    // Auto implements AbiEncode for structs
    fn struct_auto_impl_abi_encode(
        &mut self,
        engines: &Engines,
        decl: &TyDecl,
    ) -> Option<TyAstNode> {
        if !self.can_struct_auto_impl_abi_encode(decl) {
            return None;
        }

        let implementing_for_decl_ref = decl.get_struct_decl_ref().unwrap();
        let struct_decl = self.ctx.engines().de().get(implementing_for_decl_ref.id());

        let unit_type_id =
            self.ctx
                .engines
                .te()
                .insert(self.ctx.engines, TypeInfo::Tuple(vec![]), None);

        let import_handler = Handler::default();
        let _ = self.ctx.star_import(
            &import_handler,
            &[
                Ident::new_no_span("core".into()),
                Ident::new_no_span("codec".into()),
            ],
            true,
        );

        if import_handler.has_errors() {
            return None;
        }

        let abi_encode_trait_name = CallPath {
            prefixes: vec![
                Ident::new_no_span("core".into()),
                Ident::new_no_span("codec".into()),
            ],
            suffix: Ident::new_no_span("AbiEncode".into()),
            is_absolute: true,
        };

        let impl_type_parameters: Vec<_> = struct_decl
            .type_parameters
            .iter()
            .map(|x| {
                let type_id = self.ctx.engines.te().insert(
                    self.ctx.engines(),
                    TypeInfo::Custom {
                        qualified_call_path: QualifiedCallPath {
                            call_path: CallPath {
                                prefixes: vec![],
                                suffix: Ident::new_no_span(x.name_ident.as_str().into()),
                                is_absolute: false,
                            },
                            qualified_path_root: None,
                        },
                        type_arguments: None,
                        root_type_id: None,
                    },
                    None,
                );

                let mut trait_constraints: Vec<_> = x
                    .trait_constraints
                    .iter()
                    .map(|x| TraitConstraint {
                        trait_name: CallPath {
                            prefixes: vec![],
                            suffix: Ident::new_no_span(x.trait_name.suffix.as_str().into()),
                            is_absolute: false,
                        },
                        type_arguments: x
                            .type_arguments
                            .iter()
                            .map(|x| {
                                let name = match &*self.ctx.engines.te().get(x.type_id) {
                                    TypeInfo::Custom {
                                        qualified_call_path,
                                        ..
                                    } => Ident::new_no_span(
                                        qualified_call_path.call_path.suffix.as_str().into(),
                                    ),
                                    _ => todo!(),
                                };

                                let type_id = self.ctx.engines.te().insert(
                                    self.ctx.engines(),
                                    TypeInfo::Custom {
                                        qualified_call_path: QualifiedCallPath {
                                            call_path: CallPath {
                                                prefixes: vec![],
                                                suffix: name,
                                                is_absolute: false,
                                            },
                                            qualified_path_root: None,
                                        },
                                        type_arguments: None,
                                        root_type_id: None,
                                    },
                                    None,
                                );

                                TypeArgument {
                                    type_id,
                                    initial_type_id: type_id,
                                    span: Span::dummy(),
                                    call_path_tree: None,
                                }
                            })
                            .collect(),
                    })
                    .collect();
                trait_constraints.push(TraitConstraint {
                    trait_name: CallPath {
                        prefixes: vec![],
                        suffix: Ident::new_no_span("AbiEncode".into()),
                        is_absolute: false,
                    },
                    type_arguments: vec![],
                });

                TypeParameter {
                    type_id,
                    initial_type_id: type_id,
                    name_ident: Ident::new_no_span(x.name_ident.as_str().into()),
                    trait_constraints,
                    trait_constraints_span: Span::dummy(),
                    is_from_parent: false,
                }
            })
            .collect();

        let implementing_for_type_id = self.ctx.engines.te().insert(
            self.ctx.engines,
            TypeInfo::Custom {
                qualified_call_path: QualifiedCallPath {
                    call_path: CallPath {
                        prefixes: vec![],
                        suffix: struct_decl.call_path.suffix.clone(),
                        is_absolute: false,
                    },
                    qualified_path_root: None,
                },
                type_arguments: if impl_type_parameters.is_empty() {
                    None
                } else {
                    Some(
                        impl_type_parameters
                            .iter()
                            .map(|x| {
                                let type_id = self.ctx.engines().te().insert(
                                    self.ctx.engines(),
                                    TypeInfo::Custom {
                                        qualified_call_path: QualifiedCallPath {
                                            call_path: CallPath {
                                                prefixes: vec![],
                                                suffix: x.name_ident.clone(),
                                                is_absolute: false,
                                            },
                                            qualified_path_root: None,
                                        },
                                        type_arguments: None,
                                        root_type_id: None,
                                    },
                                    None,
                                );

                                TypeArgument {
                                    type_id,
                                    initial_type_id: type_id,
                                    span: Span::dummy(),
                                    call_path_tree: None,
                                }
                            })
                            .collect(),
                    )
                },
                root_type_id: None,
            },
            None,
        );

        let implementing_for = TypeArgument {
            type_id: implementing_for_type_id,
            initial_type_id: implementing_for_type_id,
            span: Span::dummy(),
            call_path_tree: None,
        };

        let impl_trait_item = FunctionDeclaration {
            purity: crate::language::Purity::Pure,
            attributes: AttributesMap::default(),
            name: Ident::new_no_span("abi_encode".into()),
            visibility: crate::language::Visibility::Public,
            body: CodeBlock {
                contents: struct_decl
                    .fields
                    .iter()
                    .map(|x| AstNode {
                        content: AstNodeContent::Expression(Expression {
                            kind: ExpressionKind::MethodApplication(Box::new(
                                MethodApplicationExpression {
                                    method_name_binding: TypeBinding {
                                        inner: MethodName::FromModule {
                                            method_name: Ident::new_no_span("abi_encode".into()),
                                        },
                                        type_arguments: TypeArgs::Regular(vec![]),
                                        span: Span::dummy(),
                                    },
                                    arguments: vec![
                                        Expression {
                                            kind: ExpressionKind::Subfield(SubfieldExpression {
                                                prefix: Box::new(Expression {
                                                    kind:
                                                        ExpressionKind::AmbiguousVariableExpression(
                                                            Ident::new_no_span("self".into()),
                                                        ),
                                                    span: Span::dummy(),
                                                }),
                                                field_to_access: x.name.clone(),
                                            }),
                                            span: Span::dummy(),
                                        },
                                        Expression {
                                            kind: ExpressionKind::AmbiguousVariableExpression(
                                                Ident::new_no_span("buffer".into()),
                                            ),
                                            span: Span::dummy(),
                                        },
                                    ],
                                    contract_call_params: vec![],
                                },
                            )),
                            span: Span::dummy(),
                        }),
                        span: Span::dummy(),
                    })
                    .collect(),
                whole_block_span: Span::dummy(),
            },
            parameters: vec![
                FunctionParameter {
                    name: Ident::new_no_span("self".into()),
                    is_reference: false,
                    is_mutable: false,
                    mutability_span: Span::dummy(),
                    type_argument: TypeArgument {
                        type_id: implementing_for_type_id,
                        initial_type_id: implementing_for_type_id,
                        span: Span::dummy(),
                        call_path_tree: None,
                    },
                },
                FunctionParameter {
                    name: Ident::new_no_span("buffer".into()),
                    is_reference: true,
                    is_mutable: true,
                    mutability_span: Span::dummy(),
                    type_argument: TypeArgument {
                        type_id: self.buffer_type_id,
                        initial_type_id: self.buffer_type_id,
                        span: Span::dummy(),
                        call_path_tree: None,
                    },
                },
            ],
            span: Span::dummy(),
            return_type: TypeArgument {
                type_id: unit_type_id,
                initial_type_id: unit_type_id,
                span: Span::dummy(),
                call_path_tree: None,
            },
            type_parameters: vec![],
            where_clause: vec![],
        };
        let impl_trait_item = engines.pe().insert(impl_trait_item);

        let impl_trait = ImplTrait {
            impl_type_parameters,
            trait_name: abi_encode_trait_name,
            trait_type_arguments: vec![],
            implementing_for,
            items: vec![ImplItem::Fn(impl_trait_item)],
            block_span: Span::dummy(),
        };
        let impl_trait = engines.pe().insert(impl_trait);
        let impl_trait = Declaration::ImplTrait(impl_trait);

        let handler = Handler::default();
        TyAstNode::type_check(
            &handler,
            self.ctx.by_ref(),
            AstNode {
                content: AstNodeContent::Declaration(impl_trait),
                span: Span::dummy(),
            },
        )
        .ok()
    }

    pub fn auto_impl_abi_encode(
        &mut self,
        engines: &Engines,
        decl: &ty::TyDecl,
    ) -> Option<TyAstNode> {
        match decl {
            TyDecl::StructDecl(_) => self.struct_auto_impl_abi_encode(engines, decl),
            TyDecl::EnumDecl(_) => self.enum_auto_impl_abi_encode(engines, decl),
            _ => None,
        }
    }
}
