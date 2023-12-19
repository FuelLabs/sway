use sway_error::handler::{ErrorEmitted, Handler};
use sway_types::{Ident, Named, Span, Spanned};

use crate::{
    decl_engine::{DeclEngineGet, DeclEngineInsert, DeclRef, ReplaceFunctionImplementingType},
    language::{
        parsed::{self, FunctionDeclaration, FunctionParameter, SubfieldExpression},
        ty::{self, TyDecl},
        CallPath, QualifiedCallPath,
    },
    namespace::{IsExtendingExistingImpl, IsImplSelf},
    semantic_analysis::{
        type_check_context::EnforceTypeArguments, TypeCheckAnalysis, TypeCheckAnalysisContext,
        TypeCheckContext, TypeCheckFinalization, TypeCheckFinalizationContext,
    },
    transform::AttributesMap,
    type_system::*,
};

struct AutoImplAbiEncodeContext {
    buffer_type_id: TypeId,
}

fn can_auto_impl_abi_encode(
    ctx: &mut TypeCheckContext,
    decl: ty::TyDecl,
) -> Option<AutoImplAbiEncodeContext> {
    // skip module "core"
    if matches!(ctx.namespace.root().name.as_ref(), Some(x) if x.as_str() == "core") {
        return None;
    }

    // if Buffer type is not reacheable, we cannot auto impl
    let handler = Handler::default();
    let buffer_type_id = ctx.engines.te().insert(
        ctx.engines,
        TypeInfo::Custom {
            qualified_call_path: QualifiedCallPath {
                call_path: CallPath {
                    prefixes: vec![
                        Ident::new_no_span("core".into()),
                        Ident::new_no_span("codec".into()),
                    ],
                    suffix: Ident::new_no_span("Buffer".into()),
                    is_absolute: true,
                },
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
            &decl.span(),
            EnforceTypeArguments::No,
            None,
        )
        .ok()?;

    let decl_ref = decl.get_struct_decl_ref()?;
    let struct_ref = ctx.engines().de().get(decl_ref.id());

    // Do not support types with generic constraints
    // because this generates a circular impl trait
    if struct_ref.type_parameters.iter().any(|x| {
        x.trait_constraints
            .iter()
            .any(|c| !c.type_arguments.is_empty())
    }) {
        return None;
    }

    let all_fields_are_abi_encode = struct_ref.fields.iter().all(|field| {
        if let TypeInfo::UnknownGeneric { .. } =
            &*ctx.engines().te().get(field.type_argument.type_id)
        {
            return true;
        }

        let handler = Handler::default();
        ctx.namespace
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
                &decl.span(),
                ctx.engines,
                crate::namespace::TryInsertingTraitImplOnFailure::Yes,
            )
            .is_ok()
    });

    all_fields_are_abi_encode.then_some(AutoImplAbiEncodeContext { buffer_type_id })
}

fn auto_impl_abi_encode(
    handler: &Handler,
    ctx: &mut TypeCheckContext,
    decl: ty::TyDecl,
    buffer_type_id: TypeId,
) {
    let implementing_for_decl_ref = decl.get_struct_decl_ref().unwrap();

    let unit_type_id = ctx
        .engines
        .te()
        .insert(ctx.engines, TypeInfo::Tuple(vec![]), None);

    let struct_ref = ctx.engines().de().get(implementing_for_decl_ref.id());

    let import_handler = Handler::default();
    let _ = ctx.star_import(
        &import_handler,
        &[
            Ident::new_no_span("core".into()),
            Ident::new_no_span("codec".into()),
        ],
        true,
    );

    if import_handler.has_errors() {
        return;
    }

    let abi_encode_trait_name = CallPath {
        prefixes: vec![
            Ident::new_no_span("core".into()),
            Ident::new_no_span("codec".into()),
        ],
        suffix: Ident::new_no_span("AbiEncode".into()),
        is_absolute: true,
    };

    let implementing_for_decl = ctx.engines.de().get(implementing_for_decl_ref.id());
    let impl_type_parameters: Vec<_> = implementing_for_decl
        .type_parameters
        .iter()
        .map(|x| {
            let type_id = ctx.engines.te().insert(
                ctx.engines(),
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
                            let name = match &*ctx.engines.te().get(x.type_id) {
                                TypeInfo::Custom {
                                    qualified_call_path,
                                    ..
                                } => Ident::new_no_span(
                                    qualified_call_path.call_path.suffix.as_str().into(),
                                ),
                                _ => todo!(),
                            };

                            let type_id = ctx.engines.te().insert(
                                ctx.engines(),
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

    let implementing_for_type_id = ctx.engines.te().insert(
        ctx.engines,
        TypeInfo::Custom {
            qualified_call_path: QualifiedCallPath {
                call_path: CallPath {
                    prefixes: vec![],
                    suffix: implementing_for_decl.call_path.suffix.clone(),
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
                            let type_id = ctx.engines().te().insert(
                                ctx.engines(),
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

    if implementing_for_decl.name().as_str() == "A" {
        // todo!();
    }

    let implementing_for = TypeArgument {
        type_id: implementing_for_type_id,
        initial_type_id: implementing_for_type_id,
        span: Span::dummy(),
        call_path_tree: None,
    };

    let impl_trait =  parsed::Declaration::ImplTrait(
        parsed::ImplTrait {
            impl_type_parameters,
            trait_name: abi_encode_trait_name,
            trait_type_arguments: vec![],
            implementing_for,
            items: vec![
                parsed::ImplItem::Fn(
                    FunctionDeclaration {
                        purity: crate::language::Purity::Pure,
                        attributes: AttributesMap::default(),
                        name: Ident::new_no_span("abi_encode".into()),
                        visibility: crate::language::Visibility::Public,
                        body: parsed::CodeBlock {
                            contents: struct_ref.fields
                                .iter()
                                .map(|x| {
                                    parsed::AstNode {
                                        content: parsed::AstNodeContent::Expression(
                                            parsed::Expression {
                                                kind: parsed::ExpressionKind::MethodApplication(
                                                    Box::new(parsed::MethodApplicationExpression {
                                                        method_name_binding: TypeBinding {
                                                            inner: parsed::MethodName::FromModule {
                                                                method_name: Ident::new_no_span("abi_encode".into())
                                                            },
                                                            type_arguments: TypeArgs::Regular(vec![]),
                                                            span: Span::dummy()
                                                        },
                                                        arguments: vec![
                                                            parsed::Expression {
                                                                kind: parsed::ExpressionKind::Subfield (
                                                                    SubfieldExpression {
                                                                        prefix: Box::new(
                                                                            parsed::Expression {
                                                                                kind: parsed::ExpressionKind::AmbiguousVariableExpression (
                                                                                    Ident::new_no_span("self".into())
                                                                                ),
                                                                                span: Span::dummy()
                                                                            }
                                                                        ),
                                                                        field_to_access: x.name.clone()
                                                                    }
                                                                ),
                                                                span: Span::dummy()
                                                            },
                                                            parsed::Expression {
                                                                kind: parsed::ExpressionKind::AmbiguousVariableExpression(
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
                                })
                                .collect(),
                            whole_block_span: Span::dummy()
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
                                    call_path_tree: None
                                }
                            },
                            FunctionParameter {
                                name: Ident::new_no_span("buffer".into()),
                                is_reference: true,
                                is_mutable: true,
                                mutability_span: Span::dummy(),
                                type_argument: TypeArgument {
                                    type_id: buffer_type_id,
                                    initial_type_id: buffer_type_id,
                                    span: Span::dummy(),
                                    call_path_tree: None
                                }
                            },
                        ],
                        span: Span::dummy(),
                        return_type: TypeArgument {
                            type_id: unit_type_id,
                            initial_type_id: unit_type_id,
                            span: Span::dummy(),
                            call_path_tree: None
                        },
                        type_parameters: vec![],
                        where_clause: vec![]
                    }
                )
            ],
            block_span: Span::dummy(),
        }
    );

    let _ = TyDecl::type_check(handler, ctx.by_ref(), impl_trait);
}

impl TyDecl {
    pub(crate) fn type_check(
        handler: &Handler,
        mut ctx: TypeCheckContext,
        decl: parsed::Declaration,
    ) -> Result<ty::TyDecl, ErrorEmitted> {
        let type_engine = ctx.engines.te();
        let decl_engine = ctx.engines.de();
        let engines = ctx.engines();

        let decl = match decl {
            parsed::Declaration::VariableDeclaration(parsed::VariableDeclaration {
                name,
                mut type_ascription,
                body,
                is_mutable,
            }) => {
                type_ascription.type_id = ctx
                    .resolve_type(
                        handler,
                        type_ascription.type_id,
                        &type_ascription.span,
                        EnforceTypeArguments::Yes,
                        None,
                    )
                    .unwrap_or_else(|err| {
                        type_engine.insert(engines, TypeInfo::ErrorRecovery(err), None)
                    });
                let mut ctx = ctx
                    .with_type_annotation(type_ascription.type_id)
                    .with_help_text(
                        "Variable declaration's type annotation does not match up \
                        with the assigned expression's type.",
                    );
                let result = ty::TyExpression::type_check(handler, ctx.by_ref(), body);
                let body =
                    result.unwrap_or_else(|err| ty::TyExpression::error(err, name.span(), engines));

                // Integers are special in the sense that we can't only rely on the type of `body`
                // to get the type of the variable. The type of the variable *has* to follow
                // `type_ascription` if `type_ascription` is a concrete integer type that does not
                // conflict with the type of `body` (i.e. passes the type checking above).
                let return_type = match &*type_engine.get(type_ascription.type_id) {
                    TypeInfo::UnsignedInteger(_) => type_ascription.type_id,
                    _ => body.return_type,
                };
                let typed_var_decl = ty::TyDecl::VariableDecl(Box::new(ty::TyVariableDecl {
                    name: name.clone(),
                    body,
                    mutability: ty::VariableMutability::new_from_ref_mut(false, is_mutable),
                    return_type,
                    type_ascription,
                }));
                ctx.insert_symbol(handler, name, typed_var_decl.clone())?;
                typed_var_decl
            }
            parsed::Declaration::ConstantDeclaration(decl) => {
                let span = decl.span.clone();
                let const_decl = match ty::TyConstantDecl::type_check(handler, ctx.by_ref(), decl) {
                    Ok(res) => res,
                    Err(err) => return Ok(ty::TyDecl::ErrorRecovery(span, err)),
                };
                let typed_const_decl: ty::TyDecl = decl_engine.insert(const_decl.clone()).into();
                ctx.insert_symbol(handler, const_decl.name().clone(), typed_const_decl.clone())?;
                typed_const_decl
            }
            parsed::Declaration::TraitTypeDeclaration(decl) => {
                let span = decl.span.clone();
                let type_decl = match ty::TyTraitType::type_check(handler, ctx.by_ref(), decl) {
                    Ok(res) => res,
                    Err(err) => return Ok(ty::TyDecl::ErrorRecovery(span, err)),
                };
                let typed_type_decl: ty::TyDecl = decl_engine.insert(type_decl.clone()).into();
                ctx.insert_symbol(handler, type_decl.name().clone(), typed_type_decl.clone())?;
                typed_type_decl
            }
            parsed::Declaration::EnumDeclaration(decl) => {
                let span = decl.span.clone();
                let enum_decl = match ty::TyEnumDecl::type_check(handler, ctx.by_ref(), decl) {
                    Ok(res) => res,
                    Err(err) => return Ok(ty::TyDecl::ErrorRecovery(span, err)),
                };
                let call_path = enum_decl.call_path.clone();
                let decl: ty::TyDecl = decl_engine.insert(enum_decl).into();
                ctx.insert_symbol(handler, call_path.suffix, decl.clone())?;
                decl
            }
            parsed::Declaration::FunctionDeclaration(fn_decl) => {
                let span = fn_decl.span.clone();

                let mut ctx =
                    ctx.with_type_annotation(type_engine.insert(engines, TypeInfo::Unknown, None));
                let fn_decl = match ty::TyFunctionDecl::type_check(
                    handler,
                    ctx.by_ref(),
                    fn_decl,
                    false,
                    false,
                ) {
                    Ok(res) => res,
                    Err(err) => return Ok(ty::TyDecl::ErrorRecovery(span, err)),
                };

                let name = fn_decl.name.clone();
                let decl: ty::TyDecl = decl_engine.insert(fn_decl).into();
                let _ = ctx.insert_symbol(handler, name, decl.clone());
                decl
            }
            parsed::Declaration::TraitDeclaration(trait_decl) => {
                let span = trait_decl.span.clone();
                let mut trait_decl =
                    match ty::TyTraitDecl::type_check(handler, ctx.by_ref(), trait_decl) {
                        Ok(res) => res,
                        Err(err) => return Ok(ty::TyDecl::ErrorRecovery(span, err)),
                    };
                let name = trait_decl.name.clone();

                // save decl_refs for the LSP
                for supertrait in trait_decl.supertraits.iter_mut() {
                    let _ = ctx
                        .namespace
                        .resolve_call_path(handler, engines, &supertrait.name, ctx.self_type())
                        .map(|supertrait_decl| {
                            if let ty::TyDecl::TraitDecl(ty::TraitDecl {
                                name: supertrait_name,
                                decl_id: supertrait_decl_id,
                                subst_list: _,
                                decl_span: supertrait_decl_span,
                            }) = supertrait_decl
                            {
                                supertrait.decl_ref = Some(DeclRef::new(
                                    supertrait_name,
                                    supertrait_decl_id,
                                    supertrait_decl_span,
                                ));
                            }
                        });
                }

                let decl: ty::TyDecl = decl_engine.insert(trait_decl.clone()).into();

                trait_decl
                    .items
                    .iter_mut()
                    .for_each(|item| item.replace_implementing_type(engines, decl.clone()));
                ctx.insert_symbol(handler, name, decl.clone())?;
                decl
            }
            parsed::Declaration::ImplTrait(impl_trait) => {
                let span = impl_trait.block_span.clone();
                let mut impl_trait =
                    match ty::TyImplTrait::type_check_impl_trait(handler, ctx.by_ref(), impl_trait)
                    {
                        Ok(res) => res,
                        Err(err) => return Ok(ty::TyDecl::ErrorRecovery(span, err)),
                    };
                // if this ImplTrait implements a trait and not an ABI,
                // we insert its methods into the context
                // otherwise, if it implements an ABI, we do not
                // insert those since we do not allow calling contract methods
                // from contract methods
                let emp_vec = vec![];
                let impl_trait_items = if let Ok(ty::TyDecl::TraitDecl { .. }) =
                    ctx.namespace.resolve_call_path(
                        &Handler::default(),
                        engines,
                        &impl_trait.trait_name,
                        ctx.self_type(),
                    ) {
                    &impl_trait.items
                } else {
                    &emp_vec
                };

                ctx.insert_trait_implementation(
                    handler,
                    impl_trait.trait_name.clone(),
                    impl_trait.trait_type_arguments.clone(),
                    impl_trait.implementing_for.type_id,
                    impl_trait_items,
                    &impl_trait.span,
                    impl_trait
                        .trait_decl_ref
                        .as_ref()
                        .map(|decl_ref| decl_ref.decl_span().clone()),
                    IsImplSelf::No,
                    IsExtendingExistingImpl::No,
                )?;
                let impl_trait_decl: ty::TyDecl = decl_engine.insert(impl_trait.clone()).into();
                impl_trait.items.iter_mut().for_each(|item| {
                    item.replace_implementing_type(engines, impl_trait_decl.clone());
                });
                impl_trait_decl
            }
            parsed::Declaration::ImplSelf(impl_self) => {
                let span = impl_self.block_span.clone();
                let impl_trait_decl =
                    match ty::TyImplTrait::type_check_impl_self(handler, ctx.by_ref(), impl_self) {
                        Ok(val) => val,
                        Err(err) => return Ok(ty::TyDecl::ErrorRecovery(span, err)),
                    };
                let impl_trait = if let TyDecl::ImplTrait(impl_trait_id) = &impl_trait_decl {
                    decl_engine.get_impl_trait(&impl_trait_id.decl_id)
                } else {
                    unreachable!();
                };
                ctx.insert_trait_implementation(
                    handler,
                    impl_trait.trait_name.clone(),
                    impl_trait.trait_type_arguments.clone(),
                    impl_trait.implementing_for.type_id,
                    &impl_trait.items,
                    &impl_trait.span,
                    impl_trait
                        .trait_decl_ref
                        .as_ref()
                        .map(|decl_ref| decl_ref.decl_span().clone()),
                    IsImplSelf::Yes,
                    IsExtendingExistingImpl::No,
                )?;
                impl_trait_decl
            }
            parsed::Declaration::StructDeclaration(decl) => {
                let span = decl.span.clone();
                let decl: ty::TyStructDecl =
                    match ty::TyStructDecl::type_check(handler, ctx.by_ref(), decl) {
                        Ok(res) => res,
                        Err(err) => {
                            return Ok(ty::TyDecl::ErrorRecovery(span, err));
                        }
                    };
                let call_path = decl.call_path.clone();
                let decl: ty::TyDecl = decl_engine.insert(decl).into();

                // insert the struct decl into namespace
                ctx.insert_symbol(handler, call_path.suffix, decl.clone())?;

                if let Some(AutoImplAbiEncodeContext { buffer_type_id }) =
                    can_auto_impl_abi_encode(&mut ctx, decl.clone())
                {
                    auto_impl_abi_encode(handler, &mut ctx, decl.clone(), buffer_type_id);
                }

                decl
            }
            parsed::Declaration::AbiDeclaration(abi_decl) => {
                let span = abi_decl.span.clone();
                let mut abi_decl = match ty::TyAbiDecl::type_check(handler, ctx.by_ref(), abi_decl)
                {
                    Ok(res) => res,
                    Err(err) => {
                        return Ok(ty::TyDecl::ErrorRecovery(span, err));
                    }
                };
                let name = abi_decl.name.clone();

                // save decl_refs for the LSP
                for supertrait in abi_decl.supertraits.iter_mut() {
                    let _ = ctx
                        .namespace
                        .resolve_call_path(handler, engines, &supertrait.name, ctx.self_type())
                        .map(|supertrait_decl| {
                            if let ty::TyDecl::TraitDecl(ty::TraitDecl {
                                name: supertrait_name,
                                decl_id: supertrait_decl_id,
                                subst_list: _,
                                decl_span: supertrait_decl_span,
                            }) = supertrait_decl
                            {
                                supertrait.decl_ref = Some(DeclRef::new(
                                    supertrait_name,
                                    supertrait_decl_id,
                                    supertrait_decl_span,
                                ));
                            }
                        });
                }

                let decl: ty::TyDecl = decl_engine.insert(abi_decl.clone()).into();
                abi_decl
                    .items
                    .iter_mut()
                    .for_each(|item| item.replace_implementing_type(engines, decl.clone()));
                ctx.insert_symbol(handler, name, decl.clone())?;
                decl
            }
            parsed::Declaration::StorageDeclaration(parsed::StorageDeclaration {
                span,
                fields,
                attributes,
                storage_keyword,
            }) => {
                let mut fields_buf = Vec::with_capacity(fields.len());
                for parsed::StorageField {
                    name,
                    initializer,
                    mut type_argument,
                    attributes,
                    span: field_span,
                    ..
                } in fields
                {
                    type_argument.type_id = ctx.resolve_type(
                        handler,
                        type_argument.type_id,
                        &name.span(),
                        EnforceTypeArguments::Yes,
                        None,
                    )?;

                    let mut ctx = ctx.by_ref().with_type_annotation(type_argument.type_id);
                    let initializer =
                        ty::TyExpression::type_check(handler, ctx.by_ref(), initializer)?;

                    fields_buf.push(ty::TyStorageField {
                        name,
                        type_argument,
                        initializer,
                        span: field_span,
                        attributes,
                    });
                }
                let decl = ty::TyStorageDecl {
                    fields: fields_buf,
                    span,
                    attributes,
                    storage_keyword,
                };
                let decl_ref = decl_engine.insert(decl);
                // insert the storage declaration into the symbols
                // if there already was one, return an error that duplicate storage

                // declarations are not allowed
                ctx.namespace
                    .set_storage_declaration(handler, decl_ref.clone())?;
                decl_ref.into()
            }
            parsed::Declaration::TypeAliasDeclaration(decl) => {
                let span = decl.name.span();
                let name = decl.name.clone();
                let ty = decl.ty;

                // Resolve the type that the type alias replaces
                let new_ty = ctx
                    .resolve_type(handler, ty.type_id, &span, EnforceTypeArguments::Yes, None)
                    .unwrap_or_else(|err| {
                        type_engine.insert(engines, TypeInfo::ErrorRecovery(err), None)
                    });

                // create the type alias decl using the resolved type above
                let decl = ty::TyTypeAliasDecl {
                    name: name.clone(),
                    call_path: CallPath::from(name.clone()).to_fullpath(ctx.namespace),
                    attributes: decl.attributes,
                    ty: TypeArgument {
                        initial_type_id: ty.initial_type_id,
                        type_id: new_ty,
                        call_path_tree: ty.call_path_tree,
                        span: ty.span,
                    },
                    visibility: decl.visibility,
                    span,
                };

                let decl: ty::TyDecl = decl_engine.insert(decl).into();

                // insert the type alias name and decl into namespace
                ctx.insert_symbol(handler, name, decl.clone())?;
                decl
            }
        };

        Ok(decl)
    }
}

impl TypeCheckAnalysis for TyDecl {
    fn type_check_analyze(
        &self,
        handler: &Handler,
        ctx: &mut TypeCheckAnalysisContext,
    ) -> Result<(), ErrorEmitted> {
        match self {
            TyDecl::VariableDecl(node) => {
                node.type_check_analyze(handler, ctx)?;
            }
            TyDecl::ConstantDecl(node) => {
                let const_decl = ctx.engines.de().get_constant(&node.decl_id);
                const_decl.type_check_analyze(handler, ctx)?;
            }
            TyDecl::FunctionDecl(node) => {
                let fn_decl = ctx.engines.de().get_function(&node.decl_id);
                fn_decl.type_check_analyze(handler, ctx)?;
            }
            TyDecl::TraitDecl(node) => {
                let trait_decl = ctx.engines.de().get_trait(&node.decl_id);
                trait_decl.type_check_analyze(handler, ctx)?;
            }
            TyDecl::StructDecl(node) => {
                let struct_decl = ctx.engines.de().get_struct(&node.decl_id);
                struct_decl.type_check_analyze(handler, ctx)?;
            }
            TyDecl::EnumDecl(node) => {
                let enum_decl = ctx.engines.de().get_enum(&node.decl_id);
                enum_decl.type_check_analyze(handler, ctx)?;
            }
            TyDecl::EnumVariantDecl(_) => {}
            TyDecl::ImplTrait(node) => {
                node.type_check_analyze(handler, ctx)?;
            }
            TyDecl::AbiDecl(node) => {
                let abi_decl = ctx.engines.de().get_abi(&node.decl_id);
                abi_decl.type_check_analyze(handler, ctx)?;
            }
            TyDecl::GenericTypeForFunctionScope(_) => {}
            TyDecl::ErrorRecovery(_, _) => {}
            TyDecl::StorageDecl(node) => {
                let storage_decl = ctx.engines.de().get_storage(&node.decl_id);
                storage_decl.type_check_analyze(handler, ctx)?;
            }
            TyDecl::TypeAliasDecl(_) => {}
            TyDecl::TraitTypeDecl(_) => {}
        }

        Ok(())
    }
}

impl TypeCheckFinalization for TyDecl {
    fn type_check_finalize(
        &mut self,
        handler: &Handler,
        ctx: &mut TypeCheckFinalizationContext,
    ) -> Result<(), ErrorEmitted> {
        let decl_engine = ctx.engines.de();
        match self {
            TyDecl::VariableDecl(node) => {
                node.type_check_finalize(handler, ctx)?;
            }
            TyDecl::ConstantDecl(node) => {
                let mut const_decl = (*ctx.engines.de().get_constant(&node.decl_id)).clone();
                const_decl.type_check_finalize(handler, ctx)?;
            }
            TyDecl::FunctionDecl(node) => {
                let mut fn_decl = (*ctx.engines.de().get_function(&node.decl_id)).clone();
                fn_decl.type_check_finalize(handler, ctx)?;
            }
            TyDecl::TraitDecl(node) => {
                let mut trait_decl = (*ctx.engines.de().get_trait(&node.decl_id)).clone();
                trait_decl.type_check_finalize(handler, ctx)?;
            }
            TyDecl::StructDecl(node) => {
                let mut struct_decl = (*ctx.engines.de().get_struct(&node.decl_id)).clone();
                struct_decl.type_check_finalize(handler, ctx)?;
            }
            TyDecl::EnumDecl(node) => {
                let mut enum_decl = (*ctx.engines.de().get_enum(&node.decl_id)).clone();
                enum_decl.type_check_finalize(handler, ctx)?;
            }
            TyDecl::EnumVariantDecl(_) => {}
            TyDecl::ImplTrait(node) => {
                let mut impl_trait = (*decl_engine.get_impl_trait(&node.decl_id)).clone();
                impl_trait.type_check_finalize(handler, ctx)?;
            }
            TyDecl::AbiDecl(node) => {
                let mut abi_decl = (*decl_engine.get_abi(&node.decl_id)).clone();
                abi_decl.type_check_finalize(handler, ctx)?;
            }
            TyDecl::GenericTypeForFunctionScope(_) => {}
            TyDecl::ErrorRecovery(_, _) => {}
            TyDecl::StorageDecl(node) => {
                let mut storage_decl = (*decl_engine.get_storage(&node.decl_id)).clone();
                storage_decl.type_check_finalize(handler, ctx)?;
            }
            TyDecl::TypeAliasDecl(node) => {
                let mut type_alias_decl = (*decl_engine.get_type_alias(&node.decl_id)).clone();
                type_alias_decl.type_check_finalize(handler, ctx)?;
            }
            TyDecl::TraitTypeDecl(_node) => {}
        }

        Ok(())
    }
}
