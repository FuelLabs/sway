use crate::{
    ast_elements::type_parameter::ConstGenericParameter,
    attr_decls_to_attributes,
    compiler_generated::{
        generate_destructured_struct_var_name, generate_matched_value_var_name,
        generate_tuple_var_name,
    },
    decl_engine::{parsed_engine::ParsedDeclEngineInsert, parsed_id::ParsedDeclId},
    language::{parsed::*, *},
    transform::{attribute::*, to_parsed_lang::context::Context},
    type_system::*,
    BuildTarget, Engines,
};
use ast_elements::{
    type_argument::{GenericConstArgument, GenericTypeArgument},
    type_parameter::{ConstGenericExpr, GenericTypeParameter},
};
use itertools::Itertools;
use sway_ast::{
    assignable::ElementAccess,
    expr::{LoopControlFlow, ReassignmentOp, ReassignmentOpVariant},
    generics::GenericParam,
    ty::TyTupleDescriptor,
    AbiCastArgs, AngleBrackets, AsmBlock, Assignable, Braces, CodeBlockContents, CommaToken,
    DoubleColonToken, Expr, ExprArrayDescriptor, ExprStructField, ExprTupleDescriptor, FnArg,
    FnArgs, FnSignature, GenericArgs, GenericParams, IfCondition, IfExpr, Instruction, Intrinsic,
    Item, ItemAbi, ItemConfigurable, ItemConst, ItemEnum, ItemFn, ItemImpl, ItemKind, ItemStorage,
    ItemStruct, ItemTrait, ItemTraitItem, ItemTypeAlias, ItemUse, LitInt, LitIntType,
    MatchBranchKind, Module, ModuleKind, Parens, PathExpr, PathExprSegment, PathType,
    PathTypeSegment, Pattern, PatternStructField, PubToken, Punctuated, QualifiedPathRoot,
    Statement, StatementLet, Submodule, TraitType, Traits, Ty, TypeField, UseTree, WhereClause,
};
use sway_error::handler::{ErrorEmitted, Handler};
use sway_error::{convert_parse_tree_error::ConvertParseTreeError, error::CompileError};
use sway_features::ExperimentalFeatures;
use sway_types::{integer_bits::IntegerBits, BaseIdent};
use sway_types::{Ident, Span, Spanned};

use std::{collections::HashSet, convert::TryFrom, iter, mem::MaybeUninit, str::FromStr};

pub fn convert_parse_tree(
    context: &mut Context,
    handler: &Handler,
    engines: &Engines,
    module: Module,
) -> Result<(TreeType, ParseTree), ErrorEmitted> {
    let tree_type = convert_module_kind(&module.kind);
    context.set_program_type(tree_type);
    let tree = module_to_sway_parse_tree(context, handler, engines, module)?;
    Ok((tree_type, tree))
}

/// Converts the module kind of the AST to the tree type of the parsed tree.
pub fn convert_module_kind(kind: &ModuleKind) -> TreeType {
    match kind {
        ModuleKind::Script { .. } => TreeType::Script,
        ModuleKind::Contract { .. } => TreeType::Contract,
        ModuleKind::Predicate { .. } => TreeType::Predicate,
        ModuleKind::Library { .. } => TreeType::Library,
    }
}

pub fn module_to_sway_parse_tree(
    context: &mut Context,
    handler: &Handler,
    engines: &Engines,
    module: Module,
) -> Result<ParseTree, ErrorEmitted> {
    let span = module.span();
    let root_nodes = {
        let mut root_nodes: Vec<AstNode> = vec![];
        let mut item_can_be_submodule = true;
        for item in module.items {
            let previous_item_is_submodule = matches!(item.value, ItemKind::Submodule(_));
            let ast_nodes =
                item_to_ast_nodes(context, handler, engines, item, item_can_be_submodule, None)?;
            root_nodes.extend(ast_nodes);
            item_can_be_submodule = previous_item_is_submodule;
        }
        root_nodes
    };
    Ok(ParseTree { span, root_nodes })
}

pub fn item_to_ast_nodes(
    context: &mut Context,
    handler: &Handler,
    engines: &Engines,
    item: Item,
    // Submodules (`mod`) must be at the beginning of a file before any other items.
    // If an other non-`mod` item already appeared before this `item`,
    // or if the `item` is not a module item at all, but a nested one,
    // this parameter will be false.
    item_can_be_submodule: bool,
    override_kind: Option<FunctionDeclarationKind>,
) -> Result<Vec<AstNode>, ErrorEmitted> {
    let (attributes_handler, attributes) = attr_decls_to_attributes(
        &item.attributes,
        |attr| attr.can_annotate_item_kind(&item.value),
        item.value.friendly_name_with_acronym(),
    );
    // TODO: Remove the special handling for submodules (`mod`) once
    //       https://github.com/FuelLabs/sway/issues/6879 is fixed.
    if !matches!(item.value, ItemKind::Submodule(_))
        && !cfg_eval(context, handler, &attributes, context.experimental)?
    {
        return Ok(vec![]);
    }

    let attributes_error_emitted = handler.append(attributes_handler);

    let decl = |d| vec![AstNodeContent::Declaration(d)];

    let span = item.span();
    let contents = match item.value {
        ItemKind::Submodule(submodule) => {
            if !item_can_be_submodule {
                return Err(handler.emit_err(
                    (ConvertParseTreeError::ExpectedModuleAtBeginning {
                        span: submodule.span(),
                    })
                    .into(),
                ));
            }

            let incl_stmt = submodule_to_include_statement(&submodule);
            vec![AstNodeContent::IncludeStatement(incl_stmt)]
        }
        ItemKind::Use(item_use) => item_use_to_use_statements(context, handler, item_use)?
            .into_iter()
            .map(AstNodeContent::UseStatement)
            .collect(),
        ItemKind::Struct(item_struct) => {
            let struct_decl = Declaration::StructDeclaration(item_struct_to_struct_declaration(
                context,
                handler,
                engines,
                item_struct,
                attributes,
            )?);
            context.implementing_type = Some(struct_decl.clone());
            decl(struct_decl)
        }
        ItemKind::Enum(item_enum) => decl(Declaration::EnumDeclaration(
            item_enum_to_enum_declaration(context, handler, engines, item_enum, attributes)?,
        )),
        ItemKind::Fn(item_fn) => {
            let function_declaration_decl_id = item_fn_to_function_declaration(
                context,
                handler,
                engines,
                item_fn,
                attributes,
                None,
                None,
                override_kind,
            )?;
            let function_declaration = engines.pe().get_function(&function_declaration_decl_id);
            error_if_self_param_is_not_allowed(
                context,
                handler,
                engines,
                &function_declaration.parameters,
                "a free function",
            )?;
            decl(Declaration::FunctionDeclaration(
                function_declaration_decl_id,
            ))
        }
        ItemKind::Trait(item_trait) => {
            let trait_decl = Declaration::TraitDeclaration(item_trait_to_trait_declaration(
                context, handler, engines, item_trait, attributes,
            )?);
            context.implementing_type = Some(trait_decl.clone());
            decl(trait_decl)
        }
        ItemKind::Impl(item_impl) => {
            match handle_impl_contract(context, handler, engines, item_impl.clone(), span.clone()) {
                Ok(contents) if !contents.is_empty() => contents,
                _ => {
                    let impl_decl = item_impl_to_declaration(context, handler, engines, item_impl)?;
                    decl(impl_decl)
                }
            }
        }
        ItemKind::Abi(item_abi) => {
            let abi_decl = Declaration::AbiDeclaration(item_abi_to_abi_declaration(
                context, handler, engines, item_abi, attributes,
            )?);
            context.implementing_type = Some(abi_decl.clone());
            decl(abi_decl)
        }
        ItemKind::Const(item_const) => decl(Declaration::ConstantDeclaration({
            item_const_to_constant_declaration(
                context,
                handler,
                engines,
                item_const,
                Visibility::Private,
                attributes,
                true,
            )?
        })),
        ItemKind::Storage(item_storage) => decl(Declaration::StorageDeclaration(
            item_storage_to_storage_declaration(
                context,
                handler,
                engines,
                item_storage,
                attributes,
            )?,
        )),
        ItemKind::Configurable(item_configurable) => {
            item_configurable_to_configurable_declarations(
                context,
                handler,
                engines,
                item_configurable,
                &attributes,
            )?
            .into_iter()
            .map(|decl| AstNodeContent::Declaration(Declaration::ConfigurableDeclaration(decl)))
            .collect()
        }
        ItemKind::TypeAlias(item_type_alias) => decl(Declaration::TypeAliasDeclaration(
            item_type_alias_to_type_alias_declaration(
                context,
                handler,
                engines,
                item_type_alias,
                attributes,
            )?,
        )),
        ItemKind::Error(spans, error) => {
            vec![AstNodeContent::Error(spans, error)]
        }
    };

    match attributes_error_emitted {
        Some(err) => Err(err),
        None => Ok(contents
            .into_iter()
            .map(|content| AstNode {
                span: span.clone(),
                content,
            })
            .collect()),
    }
}

fn item_use_to_use_statements(
    _context: &mut Context,
    handler: &Handler,
    item_use: ItemUse,
) -> Result<Vec<UseStatement>, ErrorEmitted> {
    let mut ret = Vec::new();
    let mut prefix = Vec::new();
    let item_span = item_use.span();

    use_tree_to_use_statements(
        item_use.tree,
        item_use.root_import.is_some(),
        pub_token_opt_to_visibility(item_use.visibility),
        &mut prefix,
        &mut ret,
        item_span,
    );

    // Check that all use statements have a call_path
    // This is not the case for `use foo;`, which is currently not supported
    for use_stmt in ret.iter() {
        if use_stmt.call_path.is_empty() {
            let error = ConvertParseTreeError::ImportsWithoutItemsNotSupported {
                span: use_stmt.span.clone(),
            };
            return Err(handler.emit_err(error.into()));
        }
    }

    debug_assert!(prefix.is_empty());
    Ok(ret)
}

fn use_tree_to_use_statements(
    use_tree: UseTree,
    is_relative_to_package_root: bool,
    reexport: Visibility,
    path: &mut Vec<Ident>,
    ret: &mut Vec<UseStatement>,
    item_span: Span,
) {
    match use_tree {
        UseTree::Group { imports } => {
            for use_tree in imports.into_inner() {
                use_tree_to_use_statements(
                    use_tree,
                    is_relative_to_package_root,
                    reexport,
                    path,
                    ret,
                    item_span.clone(),
                );
            }
        }
        UseTree::Name { name } => {
            let import_type = if name.as_str() == "self" {
                ImportType::SelfImport(name.span())
            } else {
                ImportType::Item(name)
            };
            ret.push(UseStatement {
                call_path: path.clone(),
                span: item_span,
                import_type,
                is_relative_to_package_root,
                reexport,
                alias: None,
            });
        }
        UseTree::Rename { name, alias, .. } => {
            let import_type = if name.as_str() == "self" {
                ImportType::SelfImport(name.span())
            } else {
                ImportType::Item(name)
            };
            ret.push(UseStatement {
                call_path: path.clone(),
                span: item_span,
                import_type,
                is_relative_to_package_root,
                reexport,
                alias: Some(alias),
            });
        }
        UseTree::Glob { .. } => {
            ret.push(UseStatement {
                call_path: path.clone(),
                span: item_span,
                import_type: ImportType::Star,
                is_relative_to_package_root,
                reexport,
                alias: None,
            });
        }
        UseTree::Path { prefix, suffix, .. } => {
            path.push(prefix);
            use_tree_to_use_statements(
                *suffix,
                is_relative_to_package_root,
                reexport,
                path,
                ret,
                item_span,
            );
            path.pop().unwrap();
        }
        UseTree::Error { .. } => {
            // parsing error, nothing to push to the use statements collection
        }
    }
}

// TODO: Remove all usages of `emit_all` and replace the manual collection of errors with
//       the `Handler::scope`.
fn emit_all(handler: &Handler, errors: Vec<ConvertParseTreeError>) -> Option<ErrorEmitted> {
    errors
        .into_iter()
        .fold(None, |_, error| Some(handler.emit_err(error.into())))
}

fn item_struct_to_struct_declaration(
    context: &mut Context,
    handler: &Handler,
    engines: &Engines,
    item_struct: ItemStruct,
    attributes: Attributes,
) -> Result<ParsedDeclId<StructDeclaration>, ErrorEmitted> {
    let span = item_struct.span();
    let fields = item_struct
        .fields
        .into_inner()
        .into_iter()
        .map(|type_field| {
            let (attributes_handler, attributes) = attr_decls_to_attributes(
                &type_field.attributes,
                |attr| attr.can_annotate_struct_or_enum_field(StructOrEnumField::StructField),
                "struct field",
            );

            if !cfg_eval(context, handler, &attributes, context.experimental)? {
                return Ok(None);
            }

            let attributes_error_emitted = handler.append(attributes_handler);

            let struct_field = type_field_to_struct_field(
                context,
                handler,
                engines,
                type_field.value,
                attributes,
            )?;

            match attributes_error_emitted {
                Some(err) => Err(err),
                None => Ok(Some(struct_field)),
            }
        })
        .filter_map_ok(|field| field)
        .collect::<Result<Vec<_>, _>>()?;

    handler.scope(|handler| {
        if fields.iter().any(
            |field| matches!(&&*engines.te().get(field.type_argument.type_id()), TypeInfo::Custom { qualified_call_path, ..} if qualified_call_path.call_path.suffix == item_struct.name),
        ) {
            handler.emit_err(ConvertParseTreeError::RecursiveType { span: span.clone() }.into());
        }

        // Make sure each struct field is declared once
        let mut names_of_fields = std::collections::HashSet::new();
        for field in &fields {
            if !names_of_fields.insert(field.name.clone()) {
                handler.emit_err(ConvertParseTreeError::DuplicateStructField {
                    name: field.name.clone(),
                    span: field.name.span(),
                }.into());
            }
        }

        Ok(())
    })?;

    let generic_parameters = generic_params_opt_to_type_parameters(
        context,
        handler,
        engines,
        item_struct.generics,
        item_struct.where_clause_opt,
    )?;
    let struct_declaration_id = engines.pe().insert(StructDeclaration {
        name: item_struct.name,
        attributes,
        fields,
        type_parameters: generic_parameters,
        visibility: pub_token_opt_to_visibility(item_struct.visibility),
        span,
    });

    Ok(struct_declaration_id)
}

fn item_enum_to_enum_declaration(
    context: &mut Context,
    handler: &Handler,
    engines: &Engines,
    item_enum: ItemEnum,
    attributes: Attributes,
) -> Result<ParsedDeclId<EnumDeclaration>, ErrorEmitted> {
    let span = item_enum.span();
    let variants = item_enum
        .fields
        .into_inner()
        .into_iter()
        .enumerate()
        .map(|(tag, type_field)| {
            let (attributes_handler, attributes) = attr_decls_to_attributes(
                &type_field.attributes,
                |attr| attr.can_annotate_struct_or_enum_field(StructOrEnumField::EnumField),
                "enum variant",
            );

            if !cfg_eval(context, handler, &attributes, context.experimental)? {
                return Ok(None);
            }

            let attributes_error_emitted = handler.append(attributes_handler);

            let enum_variant = type_field_to_enum_variant(
                context,
                handler,
                engines,
                type_field.value,
                attributes,
                tag,
            )?;

            match attributes_error_emitted {
                Some(err) => Err(err),
                None => Ok(Some(enum_variant)),
            }
        })
        .filter_map_ok(|field| field)
        .collect::<Result<Vec<_>, _>>()?;

    handler.scope(|handler| {
        if variants.iter().any(|variant| {
        matches!(&&*engines.te().get(variant.type_argument.type_id()), TypeInfo::Custom { qualified_call_path, ..} if qualified_call_path.call_path.suffix == item_enum.name)
        }) {
            handler.emit_err(ConvertParseTreeError::RecursiveType { span: span.clone() }.into());
        }

        // Make sure each enum variant is declared once
        let mut names_of_variants = std::collections::HashSet::new();
        for v in variants.iter() {
            if !names_of_variants.insert(v.name.clone()) {
                handler.emit_err(ConvertParseTreeError::DuplicateEnumVariant {
                    name: v.name.clone(),
                    span: v.name.span(),
                }.into());
            }
        }

        Ok(())
    })?;

    let type_parameters = generic_params_opt_to_type_parameters(
        context,
        handler,
        engines,
        item_enum.generics,
        item_enum.where_clause_opt,
    )?;
    let enum_declaration_id = engines.pe().insert(EnumDeclaration {
        name: item_enum.name,
        type_parameters,
        variants,
        span,
        visibility: pub_token_opt_to_visibility(item_enum.visibility),
        attributes,
    });
    Ok(enum_declaration_id)
}

#[allow(clippy::too_many_arguments)]
pub fn item_fn_to_function_declaration(
    context: &mut Context,
    handler: &Handler,
    engines: &Engines,
    item_fn: ItemFn,
    attributes: Attributes,
    parent_generic_params_opt: Option<GenericParams>,
    parent_where_clause_opt: Option<WhereClause>,
    override_kind: Option<FunctionDeclarationKind>,
) -> Result<ParsedDeclId<FunctionDeclaration>, ErrorEmitted> {
    let span = item_fn.span();
    let return_type = match item_fn.fn_signature.return_type_opt {
        Some((_right_arrow, ty)) => ty_to_type_argument(context, handler, engines, ty)?,
        None => {
            let type_id = engines.te().id_of_unit();
            GenericArgument::Type(GenericTypeArgument {
                type_id,
                initial_type_id: type_id,
                span: item_fn.fn_signature.span(),
                call_path_tree: None,
            })
        }
    };

    let kind = if item_fn.fn_signature.name.as_str() == "main" {
        FunctionDeclarationKind::Main
    } else {
        FunctionDeclarationKind::Default
    };

    let kind = override_kind.unwrap_or(kind);
    let implementing_type = context.implementing_type.clone();

    let mut generic_parameters = generic_params_opt_to_type_parameters_with_parent(
        context,
        handler,
        engines,
        item_fn.fn_signature.generics,
        parent_generic_params_opt,
        item_fn.fn_signature.where_clause_opt.clone(),
        parent_where_clause_opt,
    )?;

    for p in generic_parameters.iter_mut() {
        match p {
            TypeParameter::Type(_) => {}
            TypeParameter::Const(p) => {
                p.id = Some(engines.pe().insert(ConstGenericDeclaration {
                    name: p.name.clone(),
                    ty: p.ty,
                    span: p.span.clone(),
                }));
            }
        }
    }

    let fn_decl = FunctionDeclaration {
        purity: attributes.purity(),
        attributes,
        name: item_fn.fn_signature.name,
        visibility: pub_token_opt_to_visibility(item_fn.fn_signature.visibility),
        body: braced_code_block_contents_to_code_block(context, handler, engines, item_fn.body)?,
        parameters: fn_args_to_function_parameters(
            context,
            handler,
            engines,
            item_fn.fn_signature.arguments.into_inner(),
        )?,
        span,
        return_type,
        type_parameters: generic_parameters,
        where_clause: item_fn
            .fn_signature
            .where_clause_opt
            .map(|where_clause| {
                where_clause_to_trait_constraints(context, handler, engines, where_clause)
            })
            .transpose()?
            .unwrap_or(vec![]),
        kind,
        implementing_type,
    };
    let decl_id = engines.pe().insert(fn_decl);
    Ok(decl_id)
}

fn where_clause_to_trait_constraints(
    context: &mut Context,
    handler: &Handler,
    engines: &Engines,
    where_clause: WhereClause,
) -> Result<Vec<(Ident, Vec<TraitConstraint>)>, ErrorEmitted> {
    where_clause
        .bounds
        .into_iter()
        .map(|bound| {
            Ok((
                bound.ty_name,
                traits_to_trait_constraints(context, handler, engines, bound.bounds)?,
            ))
        })
        .collect()
}

fn item_trait_to_trait_declaration(
    context: &mut Context,
    handler: &Handler,
    engines: &Engines,
    item_trait: ItemTrait,
    attributes: Attributes,
) -> Result<ParsedDeclId<TraitDeclaration>, ErrorEmitted> {
    let span = item_trait.span();
    let type_parameters = generic_params_opt_to_type_parameters(
        context,
        handler,
        engines,
        item_trait.generics.clone(),
        item_trait.where_clause_opt.clone(),
    )?;
    let interface_surface = item_trait
        .trait_items
        .into_inner()
        .into_iter()
        .map(|annotated| {
            let (attributes_handler, attributes) = attr_decls_to_attributes(
                &annotated.attributes,
                |attr| {
                    attr.can_annotate_abi_or_trait_item(&annotated.value, TraitItemParent::Trait)
                },
                annotated.value.friendly_name(),
            );

            if !cfg_eval(context, handler, &attributes, context.experimental)? {
                return Ok(None);
            }

            let attributes_error_emitted = handler.append(attributes_handler);

            let trait_item = match annotated.value {
                ItemTraitItem::Fn(fn_sig, _) => {
                    fn_signature_to_trait_fn(context, handler, engines, fn_sig, attributes)
                        .map(TraitItem::TraitFn)
                }
                ItemTraitItem::Const(const_decl, _) => item_const_to_constant_declaration(
                    context,
                    handler,
                    engines,
                    const_decl,
                    Visibility::Public,
                    attributes,
                    false,
                )
                .map(TraitItem::Constant),
                ItemTraitItem::Type(trait_type, _) => trait_type_to_trait_type_declaration(
                    context, handler, engines, trait_type, attributes,
                )
                .map(TraitItem::Type),
                ItemTraitItem::Error(spans, error) => Ok(TraitItem::Error(spans, error)),
            }?;

            match attributes_error_emitted {
                Some(err) => Err(err),
                None => Ok(Some(trait_item)),
            }
        })
        .filter_map_ok(|item| item)
        .collect::<Result<_, _>>()?;

    let methods = match item_trait.trait_defs_opt {
        None => Vec::new(),
        Some(trait_defs) => trait_defs
            .into_inner()
            .into_iter()
            .map(|item_fn| {
                let (attributes_handler, attributes) = attr_decls_to_attributes(
                    &item_fn.attributes,
                    |attr| attr.can_annotate_abi_or_trait_item_fn(TraitItemParent::Trait),
                    "provided trait function",
                );

                if !cfg_eval(context, handler, &attributes, context.experimental)? {
                    return Ok(None);
                }

                let attributes_error_emitted = handler.append(attributes_handler);

                let function_declaration_id = item_fn_to_function_declaration(
                    context,
                    handler,
                    engines,
                    item_fn.value,
                    attributes,
                    item_trait.generics.clone(),
                    item_trait.where_clause_opt.clone(),
                    None,
                )?;

                match attributes_error_emitted {
                    Some(err) => Err(err),
                    None => Ok(Some(function_declaration_id)),
                }
            })
            .filter_map_ok(|fn_decl| fn_decl)
            .collect::<Result<_, _>>()?,
    };
    let supertraits = match item_trait.super_traits {
        None => Vec::new(),
        Some((_colon_token, traits)) => traits_to_supertraits(context, handler, traits)?,
    };
    let visibility = pub_token_opt_to_visibility(item_trait.visibility);
    let trait_decl_id = engines.pe().insert(TraitDeclaration {
        name: item_trait.name,
        type_parameters,
        interface_surface,
        methods,
        supertraits,
        visibility,
        attributes,
        span,
    });
    Ok(trait_decl_id)
}

pub fn item_impl_to_declaration(
    context: &mut Context,
    handler: &Handler,
    engines: &Engines,
    item_impl: ItemImpl,
) -> Result<Declaration, ErrorEmitted> {
    let block_span = item_impl.span();
    let implementing_for = ty_to_type_argument(context, handler, engines, item_impl.ty)?;
    let impl_item_parent = (&*engines.te().get(implementing_for.type_id())).into();

    let items = item_impl
        .contents
        .into_inner()
        .into_iter()
        .map(|item| {
            let (attributes_handler, attributes) = attr_decls_to_attributes(
                &item.attributes,
                |attr| attr.can_annotate_impl_item(&item.value, impl_item_parent),
                item.value.friendly_name(impl_item_parent),
            );

            if !cfg_eval(context, handler, &attributes, context.experimental)? {
                return Ok(None);
            }

            let attributes_error_emitted = handler.append(attributes_handler);

            let impl_item = match item.value {
                sway_ast::ItemImplItem::Fn(fn_item) => item_fn_to_function_declaration(
                    context,
                    handler,
                    engines,
                    fn_item,
                    attributes,
                    item_impl.generic_params_opt.clone(),
                    item_impl.where_clause_opt.clone(),
                    None,
                )
                .map(ImplItem::Fn),
                sway_ast::ItemImplItem::Const(const_item) => item_const_to_constant_declaration(
                    context,
                    handler,
                    engines,
                    const_item,
                    Visibility::Private,
                    attributes,
                    false,
                )
                .map(ImplItem::Constant),
                sway_ast::ItemImplItem::Type(type_item) => trait_type_to_trait_type_declaration(
                    context, handler, engines, type_item, attributes,
                )
                .map(ImplItem::Type),
            }?;

            match attributes_error_emitted {
                Some(err) => Err(err),
                None => Ok(Some(impl_item)),
            }
        })
        .filter_map_ok(|item| item)
        .collect::<Result<_, _>>()?;

    let mut impl_type_parameters = generic_params_opt_to_type_parameters(
        context,
        handler,
        engines,
        item_impl.generic_params_opt,
        item_impl.where_clause_opt,
    )?;

    for p in impl_type_parameters.iter_mut() {
        match p {
            TypeParameter::Type(_) => {}
            TypeParameter::Const(p) => {
                p.id = Some(engines.pe().insert(ConstGenericDeclaration {
                    name: p.name.clone(),
                    ty: p.ty,
                    span: p.span.clone(),
                }));
            }
        }
    }

    match item_impl.trait_opt {
        Some((path_type, _)) => {
            let (trait_name, trait_type_arguments) =
                path_type_to_call_path_and_type_arguments(context, handler, engines, path_type)?;
            let impl_trait = ImplSelfOrTrait {
                is_self: false,
                impl_type_parameters,
                trait_name: trait_name.to_call_path(handler)?,
                trait_type_arguments,
                trait_decl_ref: None,
                implementing_for,
                items,
                block_span,
            };
            let impl_trait = engines.pe().insert(impl_trait);
            Ok(Declaration::ImplSelfOrTrait(impl_trait))
        }
        None => match &*engines.te().get(implementing_for.type_id()) {
            TypeInfo::Contract => Err(handler
                .emit_err(ConvertParseTreeError::SelfImplForContract { span: block_span }.into())),
            _ => {
                let impl_self = ImplSelfOrTrait {
                    is_self: true,
                    trait_name: CallPath {
                        callpath_type: CallPathType::Ambiguous,
                        prefixes: vec![],
                        suffix: BaseIdent::dummy(),
                    },
                    trait_decl_ref: None,
                    trait_type_arguments: vec![],
                    implementing_for,
                    impl_type_parameters,
                    items,
                    block_span,
                };
                let impl_self = engines.pe().insert(impl_self);
                Ok(Declaration::ImplSelfOrTrait(impl_self))
            }
        },
    }
}

fn handle_impl_contract(
    context: &mut Context,
    handler: &Handler,
    engines: &Engines,
    item_impl: ItemImpl,
    span: Span,
) -> Result<Vec<AstNodeContent>, ErrorEmitted> {
    let implementing_for = ty_to_type_argument(context, handler, engines, item_impl.ty)?;

    // Only handle if this is an impl Contract block
    if let TypeInfo::Contract = &*engines.te().get(implementing_for.type_id) {
        // Check if there's an explicit trait being implemented
        match item_impl.trait_opt {
            Some((_, _)) => return Ok(vec![]),
            None => {
                // Generate unique name for anonymous ABI
                let anon_abi_name = Ident::new_with_override(
                    format!("_AnonymousAbi_{}", context.next_anon_suffix()),
                    span.clone(),
                );

                // Convert the methods to ABI interface
                let mut interface_surface = Vec::new();
                for item in &item_impl.contents.inner {
                    match &item.value {
                        sway_ast::ItemImplItem::Fn(fn_item) => {
                            let fn_decl = fn_signature_to_trait_fn(
                                context,
                                handler,
                                engines,
                                fn_item.fn_signature.clone(),
                                Attributes::default(),
                            )?;

                            // Validate parameters for mutability
                            let fn_decl_ref = engines.pe().get_trait_fn(&fn_decl);
                            error_if_self_param_is_not_allowed(
                                context,
                                handler,
                                engines,
                                &fn_decl_ref.parameters,
                                "an ABI method signature",
                            )?;

                            interface_surface.push(TraitItem::TraitFn(fn_decl));
                        }
                        _ => continue,
                    }
                }

                // Create ABI declaration
                let abi_decl = AbiDeclaration {
                    name: anon_abi_name.clone(),
                    attributes: Attributes::default(),
                    interface_surface: interface_surface.clone(),
                    methods: vec![],
                    supertraits: vec![],
                    span: span.clone(),
                };

                // Insert ABI declaration
                let abi_decl_id = engines.pe().insert(abi_decl);
                let impl_item_parent = (&*engines.te().get(implementing_for.type_id)).into();

                // Convert original impl items to ImplItems
                let items = item_impl
                    .contents
                    .inner
                    .into_iter()
                    .filter_map(|item| {
                        let (_, attributes) = attr_decls_to_attributes(
                            &item.attributes,
                            |attr| attr.can_annotate_impl_item(&item.value, impl_item_parent),
                            item.value.friendly_name(impl_item_parent),
                        );
                        match item.value {
                            sway_ast::ItemImplItem::Fn(fn_item) => item_fn_to_function_declaration(
                                context, handler, engines, fn_item, attributes, None, None, None,
                            )
                            .ok()
                            .map(ImplItem::Fn),
                            _ => None,
                        }
                    })
                    .collect();

                // Convert impl Contract to impl trait
                let impl_trait = ImplSelfOrTrait {
                    is_self: false,
                    impl_type_parameters: vec![],
                    trait_name: CallPath {
                        prefixes: vec![],
                        suffix: anon_abi_name,
                        callpath_type: CallPathType::Ambiguous,
                    },
                    trait_type_arguments: vec![],
                    trait_decl_ref: Some(crate::decl_engine::ParsedInterfaceDeclId::Abi(
                        abi_decl_id,
                    )),
                    implementing_for,
                    items,
                    block_span: span.clone(),
                };

                let impl_trait_id = engines.pe().insert(impl_trait);

                // Return both declarations as AST nodes
                return Ok(vec![
                    AstNodeContent::Declaration(Declaration::AbiDeclaration(abi_decl_id)),
                    AstNodeContent::Declaration(Declaration::ImplSelfOrTrait(impl_trait_id)),
                ]);
            }
        }
    }

    // Not a Contract impl, return None
    Ok(vec![])
}

fn path_type_to_call_path_and_type_arguments(
    context: &mut Context,
    handler: &Handler,
    engines: &Engines,
    path_type: PathType,
) -> Result<(QualifiedCallPath, Vec<GenericArgument>), ErrorEmitted> {
    let root_opt = path_type.root_opt.clone();
    let (prefixes, suffix) = path_type_to_prefixes_and_suffix(context, handler, path_type.clone())?;

    let (is_relative_to_root, qualified_path) =
        path_root_opt_to_bool_and_qualified_path_root(context, handler, engines, root_opt)?;

    let callpath_type = if is_relative_to_root {
        CallPathType::RelativeToPackageRoot
    } else {
        CallPathType::Ambiguous
    };

    let qualified_call_path = QualifiedCallPath {
        call_path: CallPath {
            prefixes,
            suffix: suffix.name,
            callpath_type,
        },
        qualified_path_root: qualified_path.map(Box::new),
    };

    let ty_args = match suffix.generics_opt {
        Some((_, generic_args)) => {
            generic_args_to_type_arguments(context, handler, engines, generic_args)?
        }
        None => vec![],
    };

    Ok((qualified_call_path, ty_args))
}

fn item_abi_to_abi_declaration(
    context: &mut Context,
    handler: &Handler,
    engines: &Engines,
    item_abi: ItemAbi,
    attributes: Attributes,
) -> Result<ParsedDeclId<AbiDeclaration>, ErrorEmitted> {
    let span = item_abi.span();
    let abi_decl = AbiDeclaration {
        name: item_abi.name,
        interface_surface: {
            item_abi
                .abi_items
                .into_inner()
                .into_iter()
                .map(|annotated| {
                    let (attributes_handler, attributes) = attr_decls_to_attributes(
                        &annotated.attributes,
                        |attr| {
                            attr.can_annotate_abi_or_trait_item(
                                &annotated.value,
                                TraitItemParent::Abi,
                            )
                        },
                        annotated.value.friendly_name(),
                    );

                    if !cfg_eval(context, handler, &attributes, context.experimental)? {
                        return Ok(None);
                    }

                    let attributes_error_emitted = handler.append(attributes_handler);

                    let trait_item = match annotated.value {
                        ItemTraitItem::Fn(fn_signature, _) => {
                            let trait_fn = fn_signature_to_trait_fn(
                                context,
                                handler,
                                engines,
                                fn_signature,
                                attributes,
                            )?;
                            error_if_self_param_is_not_allowed(
                                context,
                                handler,
                                engines,
                                &engines.pe().get_trait_fn(&trait_fn).parameters,
                                "an ABI method signature",
                            )?;
                            Ok(TraitItem::TraitFn(trait_fn))
                        }
                        ItemTraitItem::Const(const_decl, _) => item_const_to_constant_declaration(
                            context,
                            handler,
                            engines,
                            const_decl,
                            Visibility::Public,
                            attributes,
                            false,
                        )
                        .map(TraitItem::Constant),
                        ItemTraitItem::Type(type_decl, _) => trait_type_to_trait_type_declaration(
                            context, handler, engines, type_decl, attributes,
                        )
                        .map(TraitItem::Type),
                        ItemTraitItem::Error(spans, error) => Ok(TraitItem::Error(spans, error)),
                    }?;

                    match attributes_error_emitted {
                        Some(err) => Err(err),
                        None => Ok(Some(trait_item)),
                    }
                })
                .filter_map_ok(|item| item)
                .collect::<Result<_, _>>()?
        },
        supertraits: match item_abi.super_traits {
            None => Vec::new(),
            Some((_colon_token, traits)) => traits_to_supertraits(context, handler, traits)?,
        },
        methods: match item_abi.abi_defs_opt {
            None => Vec::new(),
            Some(abi_defs) => abi_defs
                .into_inner()
                .into_iter()
                .map(|item_fn| {
                    let (attributes_handler, attributes) = attr_decls_to_attributes(
                        &item_fn.attributes,
                        |attr| attr.can_annotate_abi_or_trait_item_fn(TraitItemParent::Abi),
                        "provided ABI function",
                    );

                    if !cfg_eval(context, handler, &attributes, context.experimental)? {
                        return Ok(None);
                    }

                    let attributes_error_emitted = handler.append(attributes_handler);

                    let function_declaration_id = item_fn_to_function_declaration(
                        context,
                        handler,
                        engines,
                        item_fn.value,
                        attributes,
                        None,
                        None,
                        None,
                    )?;

                    let function_declaration = engines.pe().get_function(&function_declaration_id);
                    error_if_self_param_is_not_allowed(
                        context,
                        handler,
                        engines,
                        &function_declaration.parameters,
                        "a method provided by ABI",
                    )?;

                    match attributes_error_emitted {
                        Some(err) => Err(err),
                        None => Ok(Some(function_declaration_id)),
                    }
                })
                .filter_map_ok(|fn_decl| fn_decl)
                .collect::<Result<_, _>>()?,
        },
        span,
        attributes,
    };
    let abi_decl = engines.pe().insert(abi_decl);
    Ok(abi_decl)
}

pub(crate) fn item_const_to_constant_declaration(
    context: &mut Context,
    handler: &Handler,
    engines: &Engines,
    item_const: ItemConst,
    default_visibility: Visibility,
    attributes: Attributes,
    require_expression: bool,
) -> Result<ParsedDeclId<ConstantDeclaration>, ErrorEmitted> {
    let span = item_const.span();

    let expr = match item_const.expr_opt {
        Some(expr) => Some(expr_to_expression(context, handler, engines, expr)?),
        None => {
            if require_expression {
                let err = ConvertParseTreeError::ConstantRequiresExpression { span: span.clone() };
                if let Some(errors) = emit_all(handler, vec![err]) {
                    return Err(errors);
                }
            }
            None
        }
    };

    let type_ascription = match item_const.ty_opt {
        Some((_colon_token, ty)) => ty_to_type_argument(context, handler, engines, ty)?,
        None => {
            if expr.is_none() {
                let err =
                    ConvertParseTreeError::ConstantRequiresTypeAscription { span: span.clone() };
                if let Some(errors) = emit_all(handler, vec![err]) {
                    return Err(errors);
                }
            }
            engines.te().new_unknown().into()
        }
    };

    let visibility = match item_const.pub_token {
        Some(pub_token) => pub_token_opt_to_visibility(Some(pub_token)),
        None => default_visibility,
    };

    let const_decl = ConstantDeclaration {
        name: item_const.name,
        type_ascription,
        value: expr,
        visibility,
        attributes,
        span,
    };
    let const_decl = engines.pe().insert(const_decl);

    Ok(const_decl)
}

pub(crate) fn trait_type_to_trait_type_declaration(
    context: &mut Context,
    handler: &Handler,
    engines: &Engines,
    trait_type: TraitType,
    attributes: Attributes,
) -> Result<ParsedDeclId<TraitTypeDeclaration>, ErrorEmitted> {
    let span = trait_type.span();
    let trait_type_decl = TraitTypeDeclaration {
        name: trait_type.name.clone(),
        attributes,
        ty_opt: if let Some(ty) = trait_type.ty_opt {
            Some(ty_to_type_argument(context, handler, engines, ty)?)
        } else {
            None
        },
        span,
    };
    let trait_type_decl = engines.pe().insert(trait_type_decl);
    Ok(trait_type_decl)
}

fn item_storage_to_storage_declaration(
    context: &mut Context,
    handler: &Handler,
    engines: &Engines,
    item_storage: ItemStorage,
    attributes: Attributes,
) -> Result<ParsedDeclId<StorageDeclaration>, ErrorEmitted> {
    let mut errors = Vec::new();
    let span = item_storage.span();
    let entries: Vec<StorageEntry> = item_storage
        .entries
        .into_inner()
        .into_iter()
        .map(|storage_entry| {
            let (attributes_handler, attributes) = attr_decls_to_attributes(
                &storage_entry.attributes,
                |attr| attr.can_annotate_storage_entry(),
                storage_entry.value.friendly_kind_name(),
            );

            if !cfg_eval(context, handler, &attributes, context.experimental)? {
                return Ok(None);
            }

            let attributes_error_emitted = handler.append(attributes_handler);

            let storage_entry = storage_entry_to_storage_entry(
                context,
                handler,
                engines,
                storage_entry.value,
                attributes,
            )?;

            match attributes_error_emitted {
                Some(err) => Err(err),
                None => Ok(Some(storage_entry)),
            }
        })
        .filter_map_ok(|entry| entry)
        .collect::<Result<_, _>>()?;

    fn check_duplicate_names(entries: Vec<StorageEntry>, errors: &mut Vec<ConvertParseTreeError>) {
        // Make sure each storage field is declared once
        let mut names_of_fields = std::collections::HashSet::new();
        for v in entries {
            if !names_of_fields.insert(v.name().clone()) {
                errors.push(ConvertParseTreeError::DuplicateStorageField {
                    name: v.name().clone(),
                    span: v.name().span(),
                });
            }
            if let StorageEntry::Namespace(namespace) = v {
                check_duplicate_names(
                    namespace
                        .entries
                        .iter()
                        .map(|e| (**e).clone())
                        .collect::<Vec<_>>(),
                    errors,
                );
            }
        }
    }

    check_duplicate_names(entries.clone(), &mut errors);

    if let Some(errors) = emit_all(handler, errors) {
        return Err(errors);
    }

    let storage_declaration = StorageDeclaration {
        attributes,
        span,
        entries,
        storage_keyword: item_storage.storage_token.into(),
    };
    let storage_declaration = engines.pe().insert(storage_declaration);
    Ok(storage_declaration)
}

fn item_configurable_to_configurable_declarations(
    context: &mut Context,
    handler: &Handler,
    engines: &Engines,
    item_configurable: ItemConfigurable,
    _attributes: &Attributes,
) -> Result<Vec<ParsedDeclId<ConfigurableDeclaration>>, ErrorEmitted> {
    let mut errors = Vec::new();

    if context.module_has_configurable_block() {
        errors.push(ConvertParseTreeError::MultipleConfigurableBlocksInModule {
            span: item_configurable.span(),
        });
    }

    if let Some(TreeType::Library) = context.program_type() {
        handler.emit_err(CompileError::ConfigurableInLibrary {
            span: item_configurable.span(),
        });
    }

    let item_configurable_keyword_span = item_configurable.configurable_token.span();
    let declarations: Vec<ParsedDeclId<ConfigurableDeclaration>> = item_configurable
        .fields
        .into_inner()
        .into_iter()
        .map(|configurable_field| {
            let (attributes_handler, attributes) = attr_decls_to_attributes(
                &configurable_field.attributes,
                |attr| attr.can_annotate_configurable_field(),
                "configurable field",
            );

            if !cfg_eval(context, handler, &attributes, context.experimental)? {
                return Ok(None);
            }

            let attributes_error_emitted = handler.append(attributes_handler);

            let configurable_decl = configurable_field_to_configurable_declaration(
                context,
                handler,
                engines,
                configurable_field.value,
                attributes,
                item_configurable_keyword_span.clone(),
            )?;

            match attributes_error_emitted {
                Some(err) => Err(err),
                None => Ok(Some(configurable_decl)),
            }
        })
        .filter_map_ok(|decl| decl)
        .collect::<Result<_, _>>()?;

    // Make sure each configurable is declared once
    let mut names_of_declarations = std::collections::HashSet::new();
    declarations.iter().for_each(|decl_id| {
        let v = engines.pe().get_configurable(decl_id);
        if !names_of_declarations.insert(v.name.clone()) {
            errors.push(ConvertParseTreeError::DuplicateConfigurable {
                name: v.name.clone(),
                span: v.name.span(),
            });
        }
    });

    if let Some(errors) = emit_all(handler, errors) {
        return Err(errors);
    }

    context.set_module_has_configurable_block(true);

    Ok(declarations)
}

fn item_type_alias_to_type_alias_declaration(
    context: &mut Context,
    handler: &Handler,
    engines: &Engines,
    item_type_alias: ItemTypeAlias,
    attributes: Attributes,
) -> Result<ParsedDeclId<TypeAliasDeclaration>, ErrorEmitted> {
    let span = item_type_alias.span();
    let type_alias_decl = TypeAliasDeclaration {
        name: item_type_alias.name.clone(),
        attributes,
        ty: ty_to_type_argument(context, handler, engines, item_type_alias.ty)?,
        visibility: pub_token_opt_to_visibility(item_type_alias.visibility),
        span,
    };
    let type_alias_decl = engines.pe().insert(type_alias_decl);
    Ok(type_alias_decl)
}

fn type_field_to_struct_field(
    context: &mut Context,
    handler: &Handler,
    engines: &Engines,
    type_field: TypeField,
    attributes: Attributes,
) -> Result<StructField, ErrorEmitted> {
    let span = type_field.span();
    let struct_field = StructField {
        visibility: pub_token_opt_to_visibility(type_field.visibility),
        name: type_field.name,
        attributes,
        type_argument: ty_to_type_argument(context, handler, engines, type_field.ty)?,
        span,
    };
    Ok(struct_field)
}

fn generic_params_opt_to_type_parameters(
    context: &mut Context,
    handler: &Handler,
    engines: &Engines,
    generic_params_opt: Option<GenericParams>,
    where_clause_opt: Option<WhereClause>,
) -> Result<Vec<TypeParameter>, ErrorEmitted> {
    generic_params_opt_to_type_parameters_with_parent(
        context,
        handler,
        engines,
        generic_params_opt,
        None,
        where_clause_opt,
        None,
    )
}

fn generic_params_opt_to_type_parameters_with_parent(
    context: &mut Context,
    handler: &Handler,
    engines: &Engines,
    generic_params_opt: Option<GenericParams>,
    parent_generic_params_opt: Option<GenericParams>,
    where_clause_opt: Option<WhereClause>,
    parent_where_clause_opt: Option<WhereClause>,
) -> Result<Vec<TypeParameter>, ErrorEmitted> {
    let type_engine = engines.te();

    let trait_constraints = match where_clause_opt {
        Some(where_clause) => where_clause
            .bounds
            .into_iter()
            .map(|where_bound| (where_bound.ty_name, where_bound.bounds))
            .collect::<Vec<_>>(),
        None => Vec::new(),
    };

    let parent_trait_constraints = match parent_where_clause_opt {
        Some(where_clause) => where_clause
            .bounds
            .into_iter()
            .map(|where_bound| (where_bound.ty_name, where_bound.bounds))
            .collect::<Vec<_>>(),
        None => Vec::new(),
    };

    let generics_to_params = |generics: Option<GenericParams>, is_from_parent: bool| match generics
    {
        Some(generic_params) => generic_params
            .parameters
            .into_inner()
            .into_iter()
            .map(|param| {
                match param {
                    GenericParam::Trait { ident } => {
                        let custom_type = type_engine.new_custom_from_name(engines, ident.clone());
                        TypeParameter::Type(GenericTypeParameter {
                            type_id: custom_type,
                            initial_type_id: custom_type,
                            name: ident,
                            trait_constraints: Vec::new(),
                            trait_constraints_span: Span::dummy(),
                            is_from_parent,
                        })
                    }
                    GenericParam::Const { ident, .. } => {
                        // let the compilation continue,
                        // but error the user for each const generic being used
                        // if the feature is disabled
                        if !context.experimental.const_generics {
                            handler.emit_err(
                                sway_features::Feature::ConstGenerics
                                    .error_because_is_disabled(&ident.span()),
                            );
                        }
                        TypeParameter::Const(ConstGenericParameter {
                            span: ident.span().clone(),
                            name: ident,
                            ty: type_engine.id_of_u64(),
                            is_from_parent,
                            id: None,
                            expr: None,
                        })
                    }
                }
            })
            .collect(),
        None => vec![],
    };

    let mut params = generics_to_params(generic_params_opt, false);
    let parent_params = generics_to_params(parent_generic_params_opt, true);
    params.extend(
        parent_params
            .iter()
            .filter(|x| x.as_const_parameter().is_some())
            .cloned(),
    );

    let mut errors = Vec::new();
    for (ty_name, bounds) in trait_constraints
        .into_iter()
        .chain(parent_trait_constraints)
    {
        let param_to_edit = if let Some(o) = params
            .iter_mut()
            .filter_map(|x| x.as_type_parameter_mut())
            .find(|GenericTypeParameter { name, .. }| name.as_str() == ty_name.as_str())
        {
            o
        } else if let Some(o2) = parent_params.iter().find(|t| match t {
            TypeParameter::Type(p) => p.name.as_str() == ty_name.as_str(),
            TypeParameter::Const(_) => false,
        }) {
            params.push(o2.clone());
            params
                .last_mut()
                .unwrap()
                .as_type_parameter_mut()
                .expect("must be type parameter")
        } else {
            errors.push(ConvertParseTreeError::ConstrainedNonExistentType {
                ty_name: ty_name.clone(),
                span: ty_name.span().clone(),
            });
            continue;
        };

        param_to_edit.trait_constraints_span = Span::join(ty_name.span(), &bounds.span());
        param_to_edit
            .trait_constraints
            .extend(traits_to_trait_constraints(
                context, handler, engines, bounds,
            )?);
    }
    if let Some(errors) = emit_all(handler, errors) {
        return Err(errors);
    }

    Ok(params)
}

fn pub_token_opt_to_visibility(pub_token_opt: Option<PubToken>) -> Visibility {
    match pub_token_opt {
        Some(..) => Visibility::Public,
        None => Visibility::Private,
    }
}

fn type_field_to_enum_variant(
    context: &mut Context,
    handler: &Handler,
    engines: &Engines,
    type_field: TypeField,
    attributes: Attributes,
    tag: usize,
) -> Result<EnumVariant, ErrorEmitted> {
    let span = type_field.span();

    let enum_variant = EnumVariant {
        name: type_field.name,
        attributes,
        type_argument: ty_to_type_argument(context, handler, engines, type_field.ty)?,
        tag,
        span,
    };
    Ok(enum_variant)
}

fn braced_code_block_contents_to_code_block(
    context: &mut Context,
    handler: &Handler,
    engines: &Engines,
    braced_code_block_contents: Braces<CodeBlockContents>,
) -> Result<CodeBlock, ErrorEmitted> {
    let whole_block_span = braced_code_block_contents.span();
    let code_block_contents = braced_code_block_contents.into_inner();
    let contents = {
        let mut error = None;

        let mut contents = Vec::new();
        for statement in code_block_contents.statements {
            match statement_to_ast_nodes(context, handler, engines, statement) {
                Ok(mut ast_nodes) => contents.append(&mut ast_nodes),
                Err(e) => error = Some(e),
            }
        }

        if let Some(expr) = code_block_contents.final_expr_opt {
            let final_ast_node = expr_to_ast_node(context, handler, engines, *expr, false)?;
            contents.push(final_ast_node);
        }

        if let Some(error) = error {
            return Err(error);
        } else {
            contents
        }
    };

    Ok(CodeBlock {
        contents,
        whole_block_span,
    })
}

fn fn_args_to_function_parameters(
    context: &mut Context,
    handler: &Handler,
    engines: &Engines,
    fn_args: FnArgs,
) -> Result<Vec<FunctionParameter>, ErrorEmitted> {
    let function_parameters = match fn_args {
        FnArgs::Static(args) => args
            .into_iter()
            .map(|fn_arg| fn_arg_to_function_parameter(context, handler, engines, fn_arg))
            .collect::<Result<_, _>>()?,
        FnArgs::NonStatic {
            self_token,
            ref_self,
            mutable_self,
            args_opt,
        } => {
            let mutability_span = match (&ref_self, &mutable_self) {
                (None, None) => Span::dummy(),
                (None, Some(mutable)) => mutable.span(),
                (Some(reference), None) => reference.span(),
                (Some(reference), Some(mutable)) => Span::join(reference.span(), &mutable.span()),
            };
            let type_id = engines.te().new_self_type(engines, self_token.span());
            let mut function_parameters = vec![FunctionParameter {
                name: Ident::new(self_token.span()),
                is_reference: ref_self.is_some(),
                is_mutable: mutable_self.is_some(),
                mutability_span,
                type_argument: GenericArgument::Type(GenericTypeArgument {
                    type_id,
                    initial_type_id: type_id,
                    span: self_token.span(),
                    call_path_tree: None,
                }),
            }];
            if let Some((_comma_token, args)) = args_opt {
                for arg in args {
                    let function_parameter =
                        fn_arg_to_function_parameter(context, handler, engines, arg)?;
                    function_parameters.push(function_parameter);
                }
            }
            function_parameters
        }
    };

    let mut unique_params = HashSet::<Ident>::default();
    for fn_param in &function_parameters {
        let already_used = !unique_params.insert(fn_param.name.clone());
        if already_used {
            let error = ConvertParseTreeError::DuplicateParameterIdentifier {
                name: fn_param.name.clone(),
                span: fn_param.name.span(),
            };
            return Err(handler.emit_err(error.into()));
        }
    }

    Ok(function_parameters)
}

pub(crate) fn type_name_to_type_info_opt(name: &Ident) -> Option<TypeInfo> {
    match name.as_str() {
        "u8" => Some(TypeInfo::UnsignedInteger(IntegerBits::Eight)),
        "u16" => Some(TypeInfo::UnsignedInteger(IntegerBits::Sixteen)),
        "u32" => Some(TypeInfo::UnsignedInteger(IntegerBits::ThirtyTwo)),
        "u64" => Some(TypeInfo::UnsignedInteger(IntegerBits::SixtyFour)),
        "u256" => Some(TypeInfo::UnsignedInteger(IntegerBits::V256)),
        "bool" => Some(TypeInfo::Boolean),
        "unit" => Some(TypeInfo::Tuple(Vec::new())),
        "b256" => Some(TypeInfo::B256),
        "str" => Some(TypeInfo::StringSlice),
        "raw_ptr" => Some(TypeInfo::RawUntypedPtr),
        "raw_slice" => Some(TypeInfo::RawUntypedSlice),
        "Self" => Some(TypeInfo::new_self_type(name.span())),
        "Contract" => Some(TypeInfo::Contract),
        _other => None,
    }
}

fn ty_to_type_info(
    context: &mut Context,
    handler: &Handler,
    engines: &Engines,
    ty: Ty,
) -> Result<TypeInfo, ErrorEmitted> {
    let type_info = match ty {
        Ty::Path(path_type) => path_type_to_type_info(context, handler, engines, path_type)?,
        Ty::Tuple(parenthesized_ty_tuple_descriptor) => {
            TypeInfo::Tuple(ty_tuple_descriptor_to_type_arguments(
                context,
                handler,
                engines,
                parenthesized_ty_tuple_descriptor.into_inner(),
            )?)
        }
        Ty::Array(bracketed_ty_array_descriptor) => {
            let ty_array_descriptor = bracketed_ty_array_descriptor.into_inner();
            TypeInfo::Array(
                ty_to_type_argument(context, handler, engines, *ty_array_descriptor.ty)?,
                Length(expr_to_const_generic_expr(
                    context,
                    engines,
                    handler,
                    ty_array_descriptor.length.as_ref(),
                )?),
            )
        }
        Ty::StringSlice(..) => TypeInfo::StringSlice,
        Ty::StringArray { length, .. } => TypeInfo::StringArray(Length(
            expr_to_const_generic_expr(context, engines, handler, length.get())?,
        )),
        Ty::Infer { .. } => TypeInfo::Unknown,
        Ty::Ptr { ty, .. } => {
            let type_argument = ty_to_type_argument(context, handler, engines, *ty.into_inner())?;
            TypeInfo::Ptr(type_argument)
        }
        Ty::Slice { ty, .. } => {
            let type_argument = ty_to_type_argument(context, handler, engines, *ty.into_inner())?;
            TypeInfo::Slice(type_argument)
        }
        Ty::Ref { mut_token, ty, .. } => {
            let type_argument = ty_to_type_argument(context, handler, engines, *ty)?;
            TypeInfo::Ref {
                to_mutable_value: mut_token.is_some(),
                referenced_type: type_argument,
            }
        }
        Ty::Never { .. } => TypeInfo::Never,
        Ty::Expr(_) => todo!("Will be implemented by https://github.com/FuelLabs/sway/issues/6860"),
    };
    Ok(type_info)
}

fn path_type_to_prefixes_and_suffix(
    context: &mut Context,
    handler: &Handler,
    PathType {
        root_opt: _,
        prefix,
        mut suffix,
    }: PathType,
) -> Result<(Vec<Ident>, PathTypeSegment), ErrorEmitted> {
    Ok(match suffix.pop() {
        None => (Vec::new(), prefix),
        Some((_, last)) => {
            // Gather the idents of the prefix, i.e. all segments but the last one.
            let mut before = Vec::with_capacity(suffix.len() + 1);
            before.push(path_type_segment_to_ident(context, handler, prefix)?);
            for (_, seg) in suffix {
                before.push(path_type_segment_to_ident(context, handler, seg)?);
            }
            (before, last)
        }
    })
}

fn ty_to_call_path_tree(
    context: &mut Context,
    handler: &Handler,
    engines: &Engines,
    ty: Ty,
) -> Result<Option<CallPathTree>, ErrorEmitted> {
    if let Ty::Path(path_type) = ty {
        let root_opt = path_type.root_opt.clone();
        let (prefixes, suffix) = path_type_to_prefixes_and_suffix(context, handler, path_type)?;

        let children = if let Some((_, generic_args)) = suffix.generics_opt {
            generic_args
                .parameters
                .inner
                .into_iter()
                .filter_map(|ty| ty_to_call_path_tree(context, handler, engines, ty).transpose())
                .collect::<Result<Vec<_>, _>>()?
        } else {
            vec![]
        };

        let (is_relative_to_root, qualified_path) =
            path_root_opt_to_bool_and_qualified_path_root(context, handler, engines, root_opt)?;

        let callpath_type = if is_relative_to_root {
            CallPathType::RelativeToPackageRoot
        } else {
            CallPathType::Ambiguous
        };

        let call_path = QualifiedCallPath {
            call_path: CallPath {
                prefixes,
                suffix: suffix.name,
                callpath_type,
            },
            qualified_path_root: qualified_path.map(Box::new),
        };

        Ok(Some(CallPathTree {
            qualified_call_path: call_path,
            children,
        }))
    } else {
        Ok(None)
    }
}

fn ty_to_type_argument(
    context: &mut Context,
    handler: &Handler,
    engines: &Engines,
    ty: Ty,
) -> Result<GenericArgument, ErrorEmitted> {
    let type_engine = engines.te();
    let span = ty.span();

    match ty {
        Ty::Expr(expr) => Ok(GenericArgument::Const(GenericConstArgument {
            expr: expr_to_const_generic_expr(context, engines, handler, expr.as_ref())?,
        })),
        ty => {
            let call_path_tree = ty_to_call_path_tree(context, handler, engines, ty.clone())?;
            let initial_type_id = type_engine.insert(
                engines,
                ty_to_type_info(context, handler, engines, ty.clone())?,
                ty.span().source_id(),
            );

            let type_argument = GenericArgument::Type(GenericTypeArgument {
                type_id: initial_type_id,
                initial_type_id,
                call_path_tree,
                span,
            });
            Ok(type_argument)
        }
    }
}

fn fn_signature_to_trait_fn(
    context: &mut Context,
    handler: &Handler,
    engines: &Engines,
    fn_signature: FnSignature,
    attributes: Attributes,
) -> Result<ParsedDeclId<TraitFn>, ErrorEmitted> {
    let return_type = match &fn_signature.return_type_opt {
        Some((_right_arrow, ty)) => ty_to_type_argument(context, handler, engines, ty.clone())?,
        None => {
            let type_id = engines.te().id_of_unit();
            GenericArgument::Type(GenericTypeArgument {
                type_id,
                initial_type_id: type_id,
                // TODO: Fix as part of https://github.com/FuelLabs/sway/issues/3635
                span: fn_signature.span(),
                call_path_tree: None,
            })
        }
    };

    let trait_fn = TraitFn {
        name: fn_signature.name.clone(),
        span: fn_signature.span(),
        purity: attributes.purity(),
        attributes,
        parameters: fn_args_to_function_parameters(
            context,
            handler,
            engines,
            fn_signature.arguments.into_inner(),
        )?,
        return_type,
    };
    let trait_fn = engines.pe().insert(trait_fn);
    Ok(trait_fn)
}

fn traits_to_trait_constraints(
    context: &mut Context,
    handler: &Handler,
    engines: &Engines,
    traits: Traits,
) -> Result<Vec<TraitConstraint>, ErrorEmitted> {
    let mut parsed_traits = vec![path_type_to_call_path_and_type_arguments(
        context,
        handler,
        engines,
        traits.prefix,
    )?];
    for (_add_token, suffix) in traits.suffixes {
        let supertrait =
            path_type_to_call_path_and_type_arguments(context, handler, engines, suffix)?;
        parsed_traits.push(supertrait);
    }
    let mut trait_constraints = vec![];
    for (trait_name, type_arguments) in parsed_traits {
        trait_constraints.push(TraitConstraint {
            trait_name: trait_name.to_call_path(handler)?,
            type_arguments,
        })
    }
    Ok(trait_constraints)
}

fn traits_to_supertraits(
    context: &mut Context,
    handler: &Handler,
    traits: Traits,
) -> Result<Vec<Supertrait>, ErrorEmitted> {
    let mut supertraits = vec![path_type_to_supertrait(context, handler, traits.prefix)?];
    for (_add_token, suffix) in traits.suffixes {
        let supertrait = path_type_to_supertrait(context, handler, suffix)?;
        supertraits.push(supertrait);
    }
    Ok(supertraits)
}

fn path_type_to_call_path(
    context: &mut Context,
    handler: &Handler,
    path_type: PathType,
) -> Result<CallPath, ErrorEmitted> {
    let PathType {
        root_opt,
        prefix,
        mut suffix,
    } = path_type;
    let is_relative_to_root = path_root_opt_to_bool(context, handler, root_opt)?;
    let callpath_type = if is_relative_to_root {
        CallPathType::RelativeToPackageRoot
    } else {
        CallPathType::Ambiguous
    };
    let call_path = match suffix.pop() {
        Some((_double_colon_token, call_path_suffix)) => {
            let mut prefixes = vec![path_type_segment_to_ident(context, handler, prefix)?];
            for (_double_colon_token, call_path_prefix) in suffix {
                let ident = path_type_segment_to_ident(context, handler, call_path_prefix)?;
                prefixes.push(ident);
            }
            CallPath {
                prefixes,
                suffix: path_type_segment_to_ident(context, handler, call_path_suffix)?,
                callpath_type,
            }
        }
        None => CallPath {
            prefixes: Vec::new(),
            suffix: path_type_segment_to_ident(context, handler, prefix)?,
            callpath_type,
        },
    };
    Ok(call_path)
}

fn expr_to_ast_node(
    context: &mut Context,
    handler: &Handler,
    engines: &Engines,
    expr: Expr,
    is_statement: bool,
) -> Result<AstNode, ErrorEmitted> {
    let span = expr.span();
    let ast_node = {
        let expression = expr_to_expression(context, handler, engines, expr)?;
        if !is_statement {
            AstNode {
                content: AstNodeContent::Expression(Expression {
                    kind: ExpressionKind::ImplicitReturn(Box::new(expression)),
                    span: span.clone(),
                }),
                span,
            }
        } else {
            AstNode {
                content: AstNodeContent::Expression(expression),
                span,
            }
        }
    };
    Ok(ast_node)
}

fn abi_cast_args_to_abi_cast_expression(
    context: &mut Context,
    handler: &Handler,
    engines: &Engines,
    args: Parens<AbiCastArgs>,
) -> Result<Box<AbiCastExpression>, ErrorEmitted> {
    let AbiCastArgs { name, address, .. } = args.into_inner();
    let abi_name = path_type_to_call_path(context, handler, name)?;
    let address = Box::new(expr_to_expression(context, handler, engines, *address)?);
    Ok(Box::new(AbiCastExpression { abi_name, address }))
}

fn struct_path_and_fields_to_struct_expression(
    context: &mut Context,
    handler: &Handler,
    engines: &Engines,
    path: PathExpr,
    fields: Braces<Punctuated<ExprStructField, CommaToken>>,
) -> Result<Box<StructExpression>, ErrorEmitted> {
    let call_path_binding = path_expr_to_call_path_binding(context, handler, engines, path)?;
    let fields = {
        fields
            .into_inner()
            .into_iter()
            .map(|expr_struct_field| {
                expr_struct_field_to_struct_expression_field(
                    context,
                    handler,
                    engines,
                    expr_struct_field,
                )
            })
            .collect::<Result<_, _>>()?
    };
    Ok(Box::new(StructExpression {
        call_path_binding,
        resolved_call_path_binding: None,
        fields,
    }))
}

fn method_call_fields_to_method_application_expression(
    context: &mut Context,
    handler: &Handler,
    engines: &Engines,
    target: Box<Expr>,
    path_seg: PathExprSegment,
    contract_args_opt: Option<Braces<Punctuated<ExprStructField, CommaToken>>>,
    args: Parens<Punctuated<Expr, CommaToken>>,
) -> Result<Box<MethodApplicationExpression>, ErrorEmitted> {
    let (method_name, type_arguments) =
        path_expr_segment_to_ident_or_type_argument(context, handler, engines, path_seg)?;

    let span = match &*type_arguments {
        [] => method_name.span(),
        [.., last] => Span::join(method_name.span(), &last.span()),
    };

    let method_name_binding = TypeBinding {
        inner: MethodName::FromModule { method_name },
        type_arguments: TypeArgs::Regular(type_arguments),
        span,
    };
    let contract_call_params = match contract_args_opt {
        None => Vec::new(),
        Some(contract_args) => contract_args
            .into_inner()
            .into_iter()
            .map(|expr_struct_field| {
                expr_struct_field_to_struct_expression_field(
                    context,
                    handler,
                    engines,
                    expr_struct_field,
                )
            })
            .collect::<Result<_, _>>()?,
    };
    let arguments = iter::once(*target)
        .chain(args.into_inner())
        .map(|expr| expr_to_expression(context, handler, engines, expr))
        .collect::<Result<_, _>>()?;
    Ok(Box::new(MethodApplicationExpression {
        method_name_binding,
        contract_call_params,
        arguments,
    }))
}

fn expr_func_app_to_expression_kind(
    context: &mut Context,
    handler: &Handler,
    engines: &Engines,
    func: Box<Expr>,
    args: Parens<Punctuated<Expr, CommaToken>>,
) -> Result<ExpressionKind, ErrorEmitted> {
    let span = Span::join(func.span(), &args.span());

    // For now, the callee has to be a path to a function.
    let PathExpr {
        root_opt,
        prefix,
        mut suffix,
        ..
    } = match *func {
        Expr::Path(path_expr) => path_expr,
        Expr::Error(_, err) => {
            // FIXME we can do better here and return function application expression here
            // if there are no parsing errors in the arguments
            return Ok(ExpressionKind::Error(Box::new([span]), err));
        }
        _ => {
            let error = ConvertParseTreeError::FunctionArbitraryExpression { span: func.span() };
            return Err(handler.emit_err(error.into()));
        }
    };

    let (is_relative_to_root, qualified_path_root) =
        path_root_opt_to_bool_and_qualified_path_root(context, handler, engines, root_opt)?;

    let convert_ty_args = |context: &mut Context, generics_opt: Option<(_, GenericArgs)>| {
        Ok(match generics_opt {
            Some((_, generic_args)) => {
                let span = generic_args.span();
                let ty_args =
                    generic_args_to_type_arguments(context, handler, engines, generic_args)?;
                (ty_args, Some(span))
            }
            None => <_>::default(),
        })
    };

    let (prefixes, last, call_seg) = match suffix.pop() {
        None => (Vec::new(), None, prefix),
        Some((_, call_path_suffix)) => {
            // Gather the idents of the prefix, i.e. all segments but the last one.
            let mut last = prefix;
            let mut prefix = Vec::with_capacity(suffix.len());
            for (_, seg) in suffix {
                prefix.push(path_expr_segment_to_ident(context, handler, &last)?);
                last = seg;
            }
            (prefix, Some(last), call_path_suffix)
        }
    };

    let arguments: Vec<Expression> = args
        .into_inner()
        .into_iter()
        .map(|expr| expr_to_expression(context, handler, engines, expr))
        .collect::<Result<_, _>>()?;

    let name_args_span = |start, end: Option<_>| match end {
        Some(end) => Span::join(start, &end),
        None => start,
    };

    let (type_arguments, type_arguments_span) = convert_ty_args(context, call_seg.generics_opt)?;

    // Transform the AST of some intrinsics
    match Intrinsic::try_from_str(call_seg.name.as_str()) {
        // "__log(arg)" becomes "__log(encode(arg))"
        Some(Intrinsic::Log)
            if context.experimental.new_encoding && last.is_none() && !is_relative_to_root =>
        {
            if arguments.len() != 1 {
                return Err(handler.emit_err(CompileError::IntrinsicIncorrectNumArgs {
                    name: Intrinsic::Log.to_string(),
                    expected: 1,
                    span: span.clone(),
                }));
            }

            let span = name_args_span(span, type_arguments_span);
            return Ok(ExpressionKind::IntrinsicFunction(
                IntrinsicFunctionExpression {
                    name: call_seg.name,
                    kind_binding: TypeBinding {
                        inner: Intrinsic::Log,
                        type_arguments: TypeArgs::Regular(vec![]),
                        span,
                    },
                    arguments: vec![wrap_logged_expr_into_encode_call(
                        arguments.into_iter().next().unwrap(),
                    )],
                },
            ));
        }
        // "__dbg(arg)" in debug becomes "{
        //      let mut f = Formatter { };
        //      f.print_str("[{current_file}:{current_line}:{current_col}] {span} = ");
        //      let arg = arg;
        //      arg.fmt(f);
        //      f.flush();
        //      arg
        // }"
        Some(Intrinsic::Dbg)
            if context.is_dbg_generation_full() && last.is_none() && !is_relative_to_root =>
        {
            if arguments.len() != 1 {
                return Err(handler.emit_err(CompileError::IntrinsicIncorrectNumArgs {
                    name: Intrinsic::Dbg.to_string(),
                    expected: 1,
                    span,
                }));
            }

            let f_id: String = format!("f_{}", context.next_for_unique_suffix());
            let f_ident = BaseIdent::new_no_span(f_id.to_string());

            let f_tid = engines.te().new_unknown();
            let f_decl_pid = engines.pe().insert(VariableDeclaration {
                name: f_ident.clone(),
                type_ascription: GenericArgument::Type(GenericTypeArgument {
                    type_id: f_tid,
                    initial_type_id: f_tid,
                    span: span.clone(),
                    call_path_tree: None,
                }),
                body: Expression {
                    kind: ExpressionKind::Struct(Box::new(StructExpression {
                        resolved_call_path_binding: None,
                        call_path_binding: TypeBinding {
                            inner: CallPath {
                                prefixes: vec![],
                                suffix: BaseIdent::new_no_span("Formatter".into()),
                                callpath_type: CallPathType::Ambiguous,
                            },
                            type_arguments: TypeArgs::Regular(vec![]),
                            span: span.clone(),
                        },
                        fields: vec![],
                    })),
                    span: span.clone(),
                },
                is_mutable: true,
            });

            fn ast_node_to_print_str(f_ident: BaseIdent, s: &str, span: &Span) -> AstNode {
                AstNode {
                    content: AstNodeContent::Expression(Expression {
                        kind: ExpressionKind::MethodApplication(Box::new(
                            MethodApplicationExpression {
                                method_name_binding: TypeBinding {
                                    inner: MethodName::FromModule {
                                        method_name: BaseIdent::new_no_span("print_str".into()),
                                    },
                                    type_arguments: TypeArgs::Regular(vec![]),
                                    span: span.clone(),
                                },
                                contract_call_params: vec![],
                                arguments: vec![
                                    Expression {
                                        kind: ExpressionKind::Variable(f_ident.clone()),
                                        span: span.clone(),
                                    },
                                    Expression {
                                        kind: ExpressionKind::Literal(Literal::String(
                                            Span::from_string(s.to_string()),
                                        )),
                                        span: span.clone(),
                                    },
                                ],
                            },
                        )),
                        span: span.clone(),
                    }),
                    span: span.clone(),
                }
            }

            let source_location = engines.se().get_source_location(&span);

            let arg_id: String = format!("arg_{}", context.next_for_unique_suffix());
            let arg_ident = BaseIdent::new_no_span(arg_id.to_string());

            let block = CodeBlock {
                contents: vec![
                    // let arg = arguments[0];
                    statement_let_to_ast_nodes_unfold(
                        context,
                        handler,
                        engines,
                        Pattern::AmbiguousSingleIdent(arg_ident.clone()),
                        None,
                        arguments[0].clone(),
                        Span::dummy(),
                    )
                    .unwrap()
                    .pop()
                    .unwrap(),
                    // let mut f = Formatter { };
                    AstNode {
                        content: AstNodeContent::Declaration(Declaration::VariableDeclaration(
                            f_decl_pid,
                        )),
                        span: Span::dummy(),
                    },
                    // f.print_str("[{current_file}:{current_line}:{current_col}] {span} = ");
                    ast_node_to_print_str(
                        f_ident.clone(),
                        &format!(
                            "[{}:{}:{}] {} = ",
                            source_location.file,
                            source_location.loc.line,
                            source_location.loc.col,
                            match swayfmt::parse::parse_format::<Expr>(
                                arguments[0].span.as_str(),
                                context.experimental
                            ) {
                                Ok(s) => s,
                                _ => arguments[0].span.as_str().to_string(),
                            }
                        ),
                        &span,
                    ),
                    // arg.fmt(f);
                    AstNode {
                        content: AstNodeContent::Expression(Expression {
                            kind: ExpressionKind::MethodApplication(Box::new(
                                MethodApplicationExpression {
                                    method_name_binding: TypeBinding {
                                        inner: MethodName::FromModule {
                                            method_name: BaseIdent::new_no_span("fmt".into()),
                                        },
                                        type_arguments: TypeArgs::Regular(vec![]),
                                        span: Span::dummy(),
                                    },
                                    contract_call_params: vec![],
                                    arguments: vec![
                                        Expression {
                                            kind: ExpressionKind::Variable(arg_ident.clone()),
                                            span: Span::dummy(),
                                        },
                                        Expression {
                                            kind: ExpressionKind::Variable(f_ident.clone()),
                                            span: Span::dummy(),
                                        },
                                    ],
                                },
                            )),
                            span: Span::dummy(),
                        }),
                        span: Span::dummy(),
                    },
                    // f.print_str(<newline>);
                    ast_node_to_print_str(f_ident.clone(), "\n", &span),
                    // f.flush();
                    AstNode {
                        content: AstNodeContent::Expression(Expression {
                            kind: ExpressionKind::MethodApplication(Box::new(
                                MethodApplicationExpression {
                                    method_name_binding: TypeBinding {
                                        inner: MethodName::FromModule {
                                            method_name: BaseIdent::new_no_span("flush".into()),
                                        },
                                        type_arguments: TypeArgs::Regular(vec![]),
                                        span: span.clone(),
                                    },
                                    contract_call_params: vec![],
                                    arguments: vec![Expression {
                                        kind: ExpressionKind::Variable(f_ident.clone()),
                                        span: span.clone(),
                                    }],
                                },
                            )),
                            span: span.clone(),
                        }),
                        span: span.clone(),
                    },
                    // arg
                    AstNode {
                        content: AstNodeContent::Expression(Expression {
                            kind: ExpressionKind::ImplicitReturn(Box::new(Expression {
                                kind: ExpressionKind::AmbiguousVariableExpression(
                                    arg_ident.clone(),
                                ),
                                span: Span::dummy(),
                            })),
                            span: Span::dummy(),
                        }),
                        span: Span::dummy(),
                    },
                ],
                whole_block_span: Span::dummy(),
            };

            return Ok(ExpressionKind::CodeBlock(block));
        }
        // ... and in release becomes "arg"
        Some(Intrinsic::Dbg)
            if !context.is_dbg_generation_full() && last.is_none() && !is_relative_to_root =>
        {
            if arguments.len() != 1 {
                return Err(handler.emit_err(CompileError::IntrinsicIncorrectNumArgs {
                    name: Intrinsic::Dbg.to_string(),
                    expected: 1,
                    span,
                }));
            }
            return Ok(arguments[0].kind.clone());
        }
        Some(intrinsic) if last.is_none() && !is_relative_to_root => {
            return Ok(ExpressionKind::IntrinsicFunction(
                IntrinsicFunctionExpression {
                    name: call_seg.name,
                    kind_binding: TypeBinding {
                        inner: intrinsic,
                        type_arguments: TypeArgs::Regular(type_arguments),
                        span: name_args_span(span, type_arguments_span),
                    },
                    arguments,
                },
            ));
        }
        _ => {}
    }

    let callpath_type = if is_relative_to_root {
        CallPathType::RelativeToPackageRoot
    } else {
        CallPathType::Ambiguous
    };

    // Only `foo(args)`? It could either be a function application or an enum variant.
    let last = match last {
        Some(last) => last,
        None => {
            let suffix = AmbiguousSuffix {
                before: None,
                suffix: call_seg.name,
            };
            let call_path = CallPath {
                prefixes,
                suffix,
                callpath_type,
            };
            let span = match type_arguments_span {
                Some(span) => Span::join(call_path.span(), &span),
                None => call_path.span(),
            };
            let call_path_binding = TypeBinding {
                inner: call_path,
                type_arguments: TypeArgs::Regular(type_arguments),
                span,
            };
            return Ok(ExpressionKind::AmbiguousPathExpression(Box::new(
                AmbiguousPathExpression {
                    args: arguments,
                    call_path_binding,
                    qualified_path_root,
                },
            )));
        }
    };

    // Ambiguous call. Could be a method call or a normal function call.
    // We don't know until type checking what `last` refers to, so let's defer.
    let (last_ty_args, last_ty_args_span) = convert_ty_args(context, last.generics_opt)?;
    let before = Some(TypeBinding {
        span: name_args_span(last.name.span(), last_ty_args_span),
        inner: last.name,
        type_arguments: TypeArgs::Regular(last_ty_args),
    });
    let suffix = AmbiguousSuffix {
        before,
        suffix: call_seg.name,
    };
    let call_path = CallPath {
        prefixes,
        suffix,
        callpath_type,
    };
    let call_path_binding = TypeBinding {
        span: name_args_span(call_path.span(), type_arguments_span),
        inner: call_path,
        type_arguments: TypeArgs::Regular(type_arguments),
    };
    Ok(ExpressionKind::AmbiguousPathExpression(Box::new(
        AmbiguousPathExpression {
            args: arguments,
            call_path_binding,
            qualified_path_root,
        },
    )))
}

fn expr_to_expression(
    context: &mut Context,
    handler: &Handler,
    engines: &Engines,
    expr: Expr,
) -> Result<Expression, ErrorEmitted> {
    let span = expr.span();
    let expression = match expr {
        Expr::Error(part_spans, err) => Expression {
            kind: ExpressionKind::Error(part_spans, err),
            span,
        },
        Expr::Path(path_expr) => path_expr_to_expression(context, handler, engines, path_expr)?,
        Expr::Literal(literal) => Expression {
            kind: ExpressionKind::Literal(literal_to_literal(context, handler, literal)?),
            span,
        },
        Expr::AbiCast { args, .. } => {
            let abi_cast_expression =
                abi_cast_args_to_abi_cast_expression(context, handler, engines, args)?;
            Expression {
                kind: ExpressionKind::AbiCast(abi_cast_expression),
                span,
            }
        }
        Expr::Struct { path, fields } => {
            let struct_expression = struct_path_and_fields_to_struct_expression(
                context, handler, engines, path, fields,
            )?;
            Expression {
                kind: ExpressionKind::Struct(struct_expression),
                span,
            }
        }
        Expr::Tuple(parenthesized_expr_tuple_descriptor) => {
            let fields = expr_tuple_descriptor_to_expressions(
                context,
                handler,
                engines,
                parenthesized_expr_tuple_descriptor.into_inner(),
            )?;
            Expression {
                kind: ExpressionKind::Tuple(fields),
                span,
            }
        }
        Expr::Parens(parens) => {
            expr_to_expression(context, handler, engines, *parens.into_inner())?
        }
        Expr::Block(braced_code_block_contents) => braced_code_block_contents_to_expression(
            context,
            handler,
            engines,
            braced_code_block_contents,
        )?,
        Expr::Array(bracketed_expr_array_descriptor) => {
            match bracketed_expr_array_descriptor.into_inner() {
                ExprArrayDescriptor::Sequence(exprs) => {
                    let contents = exprs
                        .into_iter()
                        .map(|expr| expr_to_expression(context, handler, engines, expr))
                        .collect::<Result<_, _>>()?;
                    Expression {
                        kind: ExpressionKind::Array(ArrayExpression::Explicit {
                            contents,
                            length_span: None,
                        }),
                        span,
                    }
                }
                ExprArrayDescriptor::Repeat { value, length, .. } => {
                    let value = expr_to_expression(context, handler, engines, *value)?;
                    let length = expr_to_expression(context, handler, engines, *length)?;
                    Expression {
                        kind: ExpressionKind::Array(ArrayExpression::Repeat {
                            value: Box::new(value),
                            length: Box::new(length),
                        }),
                        span,
                    }
                }
            }
        }
        Expr::Asm(asm_block) => {
            let asm_expression = asm_block_to_asm_expression(context, handler, engines, asm_block)?;
            Expression {
                kind: ExpressionKind::Asm(asm_expression),
                span,
            }
        }
        Expr::Return { expr_opt, .. } => {
            let expression = match expr_opt {
                Some(expr) => expr_to_expression(context, handler, engines, *expr)?,
                None => Expression {
                    kind: ExpressionKind::Tuple(Vec::new()),
                    span: span.clone(),
                },
            };
            Expression {
                kind: ExpressionKind::Return(Box::new(expression)),
                span,
            }
        }
        Expr::Panic { expr_opt, .. } => {
            let expression = match expr_opt {
                Some(expr) => expr_to_expression(context, handler, engines, *expr)?,
                None => Expression {
                    kind: ExpressionKind::Tuple(Vec::new()),
                    span: span.clone(),
                },
            };

            let expression = if context.experimental.new_encoding {
                wrap_logged_expr_into_encode_call(expression)
            } else {
                expression
            };

            Expression {
                kind: ExpressionKind::Panic(Box::new(expression)),
                span,
            }
        }
        Expr::If(if_expr) => if_expr_to_expression(context, handler, engines, if_expr)?,
        Expr::Match {
            value, branches, ..
        } => {
            let branches = {
                branches
                    .into_inner()
                    .into_iter()
                    .map(|match_branch| {
                        match_branch_to_match_branch(context, handler, engines, match_branch)
                    })
                    .collect::<Result<_, _>>()?
            };

            match_expr_to_expression(context, handler, engines, *value, branches, span)?
        }
        Expr::While {
            condition, block, ..
        } => Expression {
            kind: ExpressionKind::WhileLoop(WhileLoopExpression {
                condition: Box::new(expr_to_expression(context, handler, engines, *condition)?),
                body: braced_code_block_contents_to_code_block(context, handler, engines, block)?,
                is_desugared_for_loop: false,
            }),
            span,
        },
        Expr::For {
            value_pattern,
            iterator,
            block,
            ..
        } => for_expr_to_expression(
            context,
            handler,
            engines,
            &value_pattern,
            iterator,
            block,
            span,
        )?,
        Expr::FuncApp { func, args } => {
            let kind = expr_func_app_to_expression_kind(context, handler, engines, func, args)?;
            Expression { kind, span }
        }
        Expr::Index { target, arg } => Expression {
            kind: ExpressionKind::ArrayIndex(ArrayIndexExpression {
                prefix: Box::new(expr_to_expression(context, handler, engines, *target)?),
                index: Box::new(expr_to_expression(
                    context,
                    handler,
                    engines,
                    *arg.into_inner(),
                )?),
            }),
            span,
        },
        Expr::MethodCall {
            target,
            path_seg,
            args,
            contract_args_opt,
            ..
        } => {
            let method_application_expression =
                method_call_fields_to_method_application_expression(
                    context,
                    handler,
                    engines,
                    target,
                    path_seg,
                    contract_args_opt,
                    args,
                )?;
            Expression {
                kind: ExpressionKind::MethodApplication(method_application_expression),
                span,
            }
        }
        Expr::FieldProjection { target, name, .. } => {
            // Walk through the `target` expressions until we find `storage.<...>`, if any.
            // For example, `storage.foo.bar` would result in `Some([foo, bar])`.
            let mut idents = vec![&name];
            let mut base = &*target;
            let kind = loop {
                match base {
                    // Parent is a projection itself, so check its parent.
                    Expr::FieldProjection { target, name, .. } => {
                        idents.push(name);
                        base = target;
                    }
                    // Parent is `storage`. We found what we were looking for.
                    Expr::Path(path_expr)
                        if path_expr.root_opt.is_none()
                            && path_expr.prefix.generics_opt.is_none()
                            && path_expr.prefix.name.as_str() == "storage" =>
                    {
                        break ExpressionKind::StorageAccess(StorageAccessExpression {
                            namespace_names: path_expr
                                .suffix
                                .iter()
                                .map(|s| s.1.name.clone())
                                .collect(),
                            field_names: idents.into_iter().rev().cloned().collect(),
                            storage_keyword_span: path_expr.prefix.name.span(),
                        })
                    }
                    // We'll never find `storage`, so stop here.
                    _ => {
                        break ExpressionKind::Subfield(SubfieldExpression {
                            prefix: Box::new(expr_to_expression(
                                context, handler, engines, *target,
                            )?),
                            field_to_access: name,
                        })
                    }
                }
            };
            Expression { kind, span }
        }
        Expr::TupleFieldProjection {
            target,
            field,
            field_span,
            ..
        } => Expression {
            kind: ExpressionKind::TupleIndex(TupleIndexExpression {
                prefix: Box::new(expr_to_expression(context, handler, engines, *target)?),
                index: match usize::try_from(field) {
                    Ok(index) => index,
                    Err(..) => {
                        let error =
                            ConvertParseTreeError::TupleIndexOutOfRange { span: field_span };
                        return Err(handler.emit_err(error.into()));
                    }
                },
                index_span: field_span,
            }),
            span,
        },
        Expr::Ref {
            mut_token, expr, ..
        } => Expression {
            kind: ExpressionKind::Ref(RefExpression {
                to_mutable_value: mut_token.is_some(),
                value: Box::new(expr_to_expression(context, handler, engines, *expr)?),
            }),
            span,
        },
        Expr::Deref { expr, .. } => Expression {
            kind: ExpressionKind::Deref(Box::new(expr_to_expression(
                context, handler, engines, *expr,
            )?)),
            span,
        },
        Expr::Not { bang_token, expr } => {
            let expr = expr_to_expression(context, handler, engines, *expr)?;
            op_call("not", bang_token.span(), span, &[expr])?
        }
        Expr::Pow {
            lhs,
            double_star_token,
            rhs,
        } => {
            let lhs = expr_to_expression(context, handler, engines, *lhs)?;
            let rhs = expr_to_expression(context, handler, engines, *rhs)?;
            op_call("pow", double_star_token.span(), span, &vec![lhs, rhs])?
        }
        Expr::Mul {
            lhs,
            star_token,
            rhs,
        } => {
            let lhs = expr_to_expression(context, handler, engines, *lhs)?;
            let rhs = expr_to_expression(context, handler, engines, *rhs)?;
            op_call("multiply", star_token.span(), span, &vec![lhs, rhs])?
        }
        Expr::Div {
            lhs,
            forward_slash_token,
            rhs,
        } => {
            let lhs = expr_to_expression(context, handler, engines, *lhs)?;
            let rhs = expr_to_expression(context, handler, engines, *rhs)?;
            op_call("divide", forward_slash_token.span(), span, &vec![lhs, rhs])?
        }
        Expr::Modulo {
            lhs,
            percent_token,
            rhs,
        } => {
            let lhs = expr_to_expression(context, handler, engines, *lhs)?;
            let rhs = expr_to_expression(context, handler, engines, *rhs)?;
            op_call("modulo", percent_token.span(), span, &vec![lhs, rhs])?
        }
        Expr::Add {
            lhs,
            add_token,
            rhs,
        } => {
            let lhs = expr_to_expression(context, handler, engines, *lhs)?;
            let rhs = expr_to_expression(context, handler, engines, *rhs)?;
            op_call("add", add_token.span(), span, &vec![lhs, rhs])?
        }
        Expr::Sub {
            lhs,
            sub_token,
            rhs,
        } => {
            let lhs = expr_to_expression(context, handler, engines, *lhs)?;
            let rhs = expr_to_expression(context, handler, engines, *rhs)?;
            op_call("subtract", sub_token.span(), span, &vec![lhs, rhs])?
        }
        Expr::Shl {
            lhs,
            shl_token,
            rhs,
        } => {
            let lhs = expr_to_expression(context, handler, engines, *lhs)?;
            let rhs = expr_to_expression(context, handler, engines, *rhs)?;
            op_call("lsh", shl_token.span(), span, &vec![lhs, rhs])?
        }
        Expr::Shr {
            lhs,
            shr_token,
            rhs,
        } => {
            let lhs = expr_to_expression(context, handler, engines, *lhs)?;
            let rhs = expr_to_expression(context, handler, engines, *rhs)?;
            op_call("rsh", shr_token.span(), span, &vec![lhs, rhs])?
        }
        Expr::BitAnd {
            lhs,
            ampersand_token,
            rhs,
        } => {
            let lhs = expr_to_expression(context, handler, engines, *lhs)?;
            let rhs = expr_to_expression(context, handler, engines, *rhs)?;
            op_call("binary_and", ampersand_token.span(), span, &vec![lhs, rhs])?
        }
        Expr::BitXor {
            lhs,
            caret_token,
            rhs,
        } => {
            let lhs = expr_to_expression(context, handler, engines, *lhs)?;
            let rhs = expr_to_expression(context, handler, engines, *rhs)?;
            op_call("binary_xor", caret_token.span(), span, &vec![lhs, rhs])?
        }
        Expr::BitOr {
            lhs,
            pipe_token,
            rhs,
        } => {
            let lhs = expr_to_expression(context, handler, engines, *lhs)?;
            let rhs = expr_to_expression(context, handler, engines, *rhs)?;
            op_call("binary_or", pipe_token.span(), span, &vec![lhs, rhs])?
        }
        Expr::Equal {
            lhs,
            double_eq_token,
            rhs,
        } => {
            let lhs = expr_to_expression(context, handler, engines, *lhs)?;
            let rhs = expr_to_expression(context, handler, engines, *rhs)?;
            op_call("eq", double_eq_token.span(), span, &vec![lhs, rhs])?
        }
        Expr::NotEqual {
            lhs,
            bang_eq_token,
            rhs,
        } => {
            let lhs = expr_to_expression(context, handler, engines, *lhs)?;
            let rhs = expr_to_expression(context, handler, engines, *rhs)?;
            op_call("neq", bang_eq_token.span(), span, &vec![lhs, rhs])?
        }
        Expr::LessThan {
            lhs,
            less_than_token,
            rhs,
        } => {
            let lhs = expr_to_expression(context, handler, engines, *lhs)?;
            let rhs = expr_to_expression(context, handler, engines, *rhs)?;
            op_call("lt", less_than_token.span(), span, &vec![lhs, rhs])?
        }
        Expr::GreaterThan {
            lhs,
            greater_than_token,
            rhs,
        } => {
            let lhs = expr_to_expression(context, handler, engines, *lhs)?;
            let rhs = expr_to_expression(context, handler, engines, *rhs)?;
            op_call("gt", greater_than_token.span(), span, &vec![lhs, rhs])?
        }
        Expr::LessThanEq {
            lhs,
            less_than_eq_token,
            rhs,
        } => {
            let lhs = expr_to_expression(context, handler, engines, *lhs)?;
            let rhs = expr_to_expression(context, handler, engines, *rhs)?;
            op_call("le", less_than_eq_token.span(), span, &vec![lhs, rhs])?
        }
        Expr::GreaterThanEq {
            lhs,
            greater_than_eq_token,
            rhs,
        } => {
            let lhs = expr_to_expression(context, handler, engines, *lhs)?;
            let rhs = expr_to_expression(context, handler, engines, *rhs)?;
            op_call("ge", greater_than_eq_token.span(), span, &vec![lhs, rhs])?
        }
        Expr::LogicalAnd { lhs, rhs, .. } => Expression {
            kind: ExpressionKind::LazyOperator(LazyOperatorExpression {
                op: LazyOp::And,
                lhs: Box::new(expr_to_expression(context, handler, engines, *lhs)?),
                rhs: Box::new(expr_to_expression(context, handler, engines, *rhs)?),
            }),
            span,
        },
        Expr::LogicalOr { lhs, rhs, .. } => Expression {
            kind: ExpressionKind::LazyOperator(LazyOperatorExpression {
                op: LazyOp::Or,
                lhs: Box::new(expr_to_expression(context, handler, engines, *lhs)?),
                rhs: Box::new(expr_to_expression(context, handler, engines, *rhs)?),
            }),
            span,
        },
        Expr::Reassignment {
            assignable,
            expr,
            reassignment_op:
                ReassignmentOp {
                    variant: op_variant,
                    span: op_span,
                },
        } => match op_variant {
            ReassignmentOpVariant::Equals => Expression {
                kind: ExpressionKind::Reassignment(ReassignmentExpression {
                    lhs: assignable_to_reassignment_target(context, handler, engines, assignable)?,
                    rhs: Box::new(expr_to_expression(context, handler, engines, *expr)?),
                }),
                span,
            },
            op_variant => {
                let lhs = assignable_to_reassignment_target(
                    context,
                    handler,
                    engines,
                    assignable.clone(),
                )?;
                let rhs = Box::new(op_call(
                    op_variant.std_name(),
                    op_span,
                    span.clone(),
                    &vec![
                        assignable_to_expression(context, handler, engines, assignable)?,
                        expr_to_expression(context, handler, engines, *expr)?,
                    ],
                )?);
                Expression {
                    kind: ExpressionKind::Reassignment(ReassignmentExpression { lhs, rhs }),
                    span,
                }
            }
        },
        Expr::Break { .. } => Expression {
            kind: ExpressionKind::Break,
            span,
        },
        Expr::Continue { .. } => Expression {
            kind: ExpressionKind::Continue,
            span,
        },
    };
    Ok(expression)
}

/// Wraps the `logged_expr` that needs to be logged into an `encode` call: `encode(<logged_expr>)`.
/// Wrapping is needed in the `__log` intrinsic and the `revert` expression in the case of
/// the new encoding.
fn wrap_logged_expr_into_encode_call(logged_expr: Expression) -> Expression {
    let expression_span = logged_expr.span();
    Expression {
        kind: ExpressionKind::FunctionApplication(Box::new(FunctionApplicationExpression {
            call_path_binding: TypeBinding {
                inner: CallPath {
                    prefixes: vec![],
                    suffix: Ident::new_no_span("encode".into()),
                    callpath_type: CallPathType::Ambiguous,
                },
                type_arguments: TypeArgs::Regular(vec![]),
                span: expression_span.clone(),
            },
            resolved_call_path_binding: None,
            arguments: vec![logged_expr],
        })),
        span: expression_span,
    }
}

fn op_call(
    name: &'static str,
    op_span: Span,
    span: Span,
    args: &[Expression],
) -> Result<Expression, ErrorEmitted> {
    let method_name_binding = TypeBinding {
        inner: MethodName::FromTrait {
            call_path: CallPath {
                prefixes: vec![
                    Ident::new_with_override("std".into(), op_span.clone()),
                    Ident::new_with_override("ops".into(), op_span.clone()),
                ],
                suffix: Ident::new_with_override(name.into(), op_span.clone()),
                callpath_type: CallPathType::Full,
            },
        },
        type_arguments: TypeArgs::Regular(vec![]),
        span: op_span,
    };
    Ok(Expression {
        kind: ExpressionKind::MethodApplication(Box::new(MethodApplicationExpression {
            method_name_binding,
            contract_call_params: Vec::new(),
            arguments: args.to_vec(),
        })),
        span,
    })
}

fn storage_entry_to_storage_entry(
    context: &mut Context,
    handler: &Handler,
    engines: &Engines,
    storage_entry: sway_ast::StorageEntry,
    attributes: Attributes,
) -> Result<StorageEntry, ErrorEmitted> {
    if let Some(storage_field) = storage_entry.field {
        Ok(StorageEntry::Field(storage_field_to_storage_field(
            context,
            handler,
            engines,
            storage_field,
            attributes,
        )?))
    } else {
        let mut entries = vec![];
        let namespace = storage_entry.namespace.unwrap();
        for entry in namespace
            .into_inner()
            .into_iter()
            .flat_map(|storage_entry| {
                let (attributes_handler, attributes) = attr_decls_to_attributes(
                    &storage_entry.attributes,
                    |attr| attr.can_annotate_storage_entry(),
                    storage_entry.value.friendly_kind_name(),
                );

                if !cfg_eval(context, handler, &attributes, context.experimental)? {
                    return Ok::<Option<StorageEntry>, ErrorEmitted>(None);
                }

                let attributes_error_emitted = handler.append(attributes_handler);

                let storage_entry = storage_entry_to_storage_entry(
                    context,
                    handler,
                    engines,
                    *storage_entry.value,
                    attributes,
                )?;

                match attributes_error_emitted {
                    Some(err) => Err(err),
                    None => Ok(Some(storage_entry)),
                }
            })
            .flatten()
        {
            entries.push(Box::new(entry));
        }
        Ok(StorageEntry::Namespace(StorageNamespace {
            name: storage_entry.name,
            entries,
        }))
    }
}

fn storage_field_to_storage_field(
    context: &mut Context,
    handler: &Handler,
    engines: &Engines,
    storage_field: sway_ast::StorageField,
    attributes: Attributes,
) -> Result<StorageField, ErrorEmitted> {
    let span = storage_field.span();
    let mut key_expr_opt = None;
    if let Some(key_expr) = storage_field.key_expr {
        key_expr_opt = Some(expr_to_expression(context, handler, engines, key_expr)?);
    }
    let storage_field = StorageField {
        attributes,
        name: storage_field.name,
        key_expression: key_expr_opt,
        type_argument: ty_to_type_argument(context, handler, engines, storage_field.ty)?,
        span,
        initializer: expr_to_expression(context, handler, engines, storage_field.initializer)?,
    };
    Ok(storage_field)
}

fn configurable_field_to_configurable_declaration(
    context: &mut Context,
    handler: &Handler,
    engines: &Engines,
    configurable_field: sway_ast::ConfigurableField,
    attributes: Attributes,
    item_configurable_keyword_span: Span,
) -> Result<ParsedDeclId<ConfigurableDeclaration>, ErrorEmitted> {
    let span = configurable_field.name.span();

    let type_ascription = ty_to_type_argument(context, handler, engines, configurable_field.ty)?;

    let value = expr_to_expression(context, handler, engines, configurable_field.initializer)?;
    let value = if context.experimental.new_encoding {
        let call_encode =
            ExpressionKind::FunctionApplication(Box::new(FunctionApplicationExpression {
                call_path_binding: TypeBinding {
                    inner: CallPath {
                        prefixes: vec![],
                        suffix: Ident::new_with_override("encode".into(), span.clone()),
                        callpath_type: CallPathType::Ambiguous,
                    },
                    type_arguments: TypeArgs::Regular(vec![type_ascription.clone()]),
                    span: span.clone(),
                },
                resolved_call_path_binding: None,
                arguments: vec![value],
            }));
        Expression {
            kind: call_encode,
            span: span.clone(),
        }
    } else {
        value
    };

    let config_decl = ConfigurableDeclaration {
        name: configurable_field.name,
        type_ascription,
        value: Some(value),
        visibility: Visibility::Public,
        attributes,
        span: span.clone(),
        block_keyword_span: item_configurable_keyword_span,
    };
    Ok(engines.pe().insert(config_decl))
}

fn statement_to_ast_nodes(
    context: &mut Context,
    handler: &Handler,
    engines: &Engines,
    statement: Statement,
) -> Result<Vec<AstNode>, ErrorEmitted> {
    let ast_nodes = match statement {
        Statement::Let(statement_let) => {
            statement_let_to_ast_nodes(context, handler, engines, statement_let)?
        }
        Statement::Item(item) => item_to_ast_nodes(context, handler, engines, item, false, None)?,
        Statement::Expr { expr, .. } => {
            vec![expr_to_ast_node(context, handler, engines, expr, true)?]
        }
        Statement::Error(spans, error) => {
            let span = Span::join_all(spans.iter().cloned());
            vec![AstNode {
                content: AstNodeContent::Error(spans, error),
                span,
            }]
        }
    };
    Ok(ast_nodes)
}

fn fn_arg_to_function_parameter(
    context: &mut Context,
    handler: &Handler,
    engines: &Engines,
    fn_arg: FnArg,
) -> Result<FunctionParameter, ErrorEmitted> {
    let pat_span = fn_arg.pattern.span();
    let (reference, mutable, name) = match fn_arg.pattern {
        Pattern::Wildcard { .. } => {
            let error = ConvertParseTreeError::WildcardPatternsNotSupportedHere { span: pat_span };
            return Err(handler.emit_err(error.into()));
        }
        Pattern::Or { .. } => {
            let error = ConvertParseTreeError::OrPatternsNotSupportedHere { span: pat_span };
            return Err(handler.emit_err(error.into()));
        }
        Pattern::Var {
            reference,
            mutable,
            name,
        } => (reference, mutable, name),
        Pattern::AmbiguousSingleIdent(ident) => (None, None, ident),
        Pattern::Literal(..) => {
            let error = ConvertParseTreeError::LiteralPatternsNotSupportedHere { span: pat_span };
            return Err(handler.emit_err(error.into()));
        }
        Pattern::Constant(..) => {
            let error = ConvertParseTreeError::ConstantPatternsNotSupportedHere { span: pat_span };
            return Err(handler.emit_err(error.into()));
        }
        Pattern::Constructor { .. } | Pattern::Error(..) => {
            let error =
                ConvertParseTreeError::ConstructorPatternsNotSupportedHere { span: pat_span };
            return Err(handler.emit_err(error.into()));
        }
        Pattern::Struct { .. } => {
            let error = ConvertParseTreeError::StructPatternsNotSupportedHere { span: pat_span };
            return Err(handler.emit_err(error.into()));
        }
        Pattern::Tuple(..) => {
            let error = ConvertParseTreeError::TuplePatternsNotSupportedHere { span: pat_span };
            return Err(handler.emit_err(error.into()));
        }
    };
    let mutability_span = match (&reference, &mutable) {
        (None, None) => Span::dummy(),
        (None, Some(mutable)) => mutable.span(),
        (Some(reference), None) => reference.span(),
        (Some(reference), Some(mutable)) => Span::join(reference.span(), &mutable.span()),
    };
    let function_parameter = FunctionParameter {
        name,
        is_reference: reference.is_some(),
        is_mutable: mutable.is_some(),
        mutability_span,
        type_argument: ty_to_type_argument(context, handler, engines, fn_arg.ty)?,
    };
    Ok(function_parameter)
}

fn expr_to_const_generic_expr(
    context: &mut Context,
    engines: &Engines,
    handler: &Handler,
    expr: &Expr,
) -> Result<ConstGenericExpr, ErrorEmitted> {
    let span = expr.span();
    match &expr {
        Expr::Literal(..) => Ok(ConstGenericExpr::literal(
            expr_to_usize(context, handler, expr.clone())?,
            Some(span),
        )),
        _ => {
            let expr = expr_to_expression(context, handler, engines, expr.clone())?;
            match expr.kind {
                ExpressionKind::AmbiguousVariableExpression(ident) => {
                    Ok(ConstGenericExpr::AmbiguousVariableExpression { ident })
                }
                _ => Err(handler.emit_err(CompileError::LengthExpressionNotSupported { span })),
            }
        }
    }
}

fn expr_to_usize(
    _context: &mut Context,
    handler: &Handler,
    expr: Expr,
) -> Result<usize, ErrorEmitted> {
    let span = expr.span();
    let value = match expr {
        Expr::Literal(sway_ast::Literal::Int(lit_int)) => {
            match lit_int.ty_opt {
                None => (),
                Some(..) => {
                    let error = ConvertParseTreeError::IntTySuffixNotSupported { span };
                    return Err(handler.emit_err(error.into()));
                }
            }
            match usize::try_from(lit_int.parsed) {
                Ok(value) => value,
                Err(..) => {
                    let error = ConvertParseTreeError::IntLiteralOutOfRange { span };
                    return Err(handler.emit_err(error.into()));
                }
            }
        }
        _ => {
            let error = ConvertParseTreeError::IntLiteralExpected { span };
            return Err(handler.emit_err(error.into()));
        }
    };
    Ok(value)
}

fn path_type_to_supertrait(
    context: &mut Context,
    handler: &Handler,
    path_type: PathType,
) -> Result<Supertrait, ErrorEmitted> {
    let PathType {
        root_opt,
        prefix,
        mut suffix,
    } = path_type;
    let is_relative_to_root = path_root_opt_to_bool(context, handler, root_opt)?;
    let callpath_type = if is_relative_to_root {
        CallPathType::RelativeToPackageRoot
    } else {
        CallPathType::Ambiguous
    };
    let (prefixes, call_path_suffix) = match suffix.pop() {
        Some((_, call_path_suffix)) => {
            let mut prefixes = vec![path_type_segment_to_ident(context, handler, prefix)?];
            for (_, call_path_prefix) in suffix {
                let ident = path_type_segment_to_ident(context, handler, call_path_prefix)?;
                prefixes.push(ident);
            }
            (prefixes, call_path_suffix)
        }
        None => (Vec::new(), prefix),
    };
    let PathTypeSegment {
        name: suffix,
        generics_opt: _,
    } = call_path_suffix;
    let name = CallPath {
        prefixes,
        suffix,
        callpath_type,
    };
    /*
    let type_parameters = match generics_opt {
        Some((_double_colon_token_opt, generic_args)) => {
            generic_args_to_type_parameters(generic_args)
        },
        None => Vec::new(),
    };
    */
    let supertrait = Supertrait {
        name,
        decl_ref: None,
        //type_parameters,
    };
    Ok(supertrait)
}

fn path_type_segment_to_ident(
    _context: &mut Context,
    handler: &Handler,
    PathTypeSegment { name, generics_opt }: PathTypeSegment,
) -> Result<Ident, ErrorEmitted> {
    if let Some((_, generic_args)) = generics_opt {
        let error = ConvertParseTreeError::GenericsNotSupportedHere {
            span: generic_args.span(),
        };
        return Err(handler.emit_err(error.into()));
    }
    Ok(name)
}

/// Similar to [path_type_segment_to_ident],
/// but allows for the item to be either type arguments _or_ an ident.
fn path_expr_segment_to_ident_or_type_argument(
    context: &mut Context,
    handler: &Handler,
    engines: &Engines,
    PathExprSegment { name, generics_opt }: PathExprSegment,
) -> Result<(Ident, Vec<GenericArgument>), ErrorEmitted> {
    let type_args = match generics_opt {
        Some((_, x)) => generic_args_to_type_arguments(context, handler, engines, x)?,
        None => Vec::default(),
    };
    Ok((name, type_args))
}

fn path_expr_segment_to_ident(
    _context: &mut Context,
    handler: &Handler,
    PathExprSegment { name, generics_opt }: &PathExprSegment,
) -> Result<Ident, ErrorEmitted> {
    if let Some((_, generic_args)) = generics_opt {
        let error = ConvertParseTreeError::GenericsNotSupportedHere {
            span: generic_args.span(),
        };
        return Err(handler.emit_err(error.into()));
    }
    Ok(name.clone())
}

fn path_expr_to_expression(
    context: &mut Context,
    handler: &Handler,
    engines: &Engines,
    path_expr: PathExpr,
) -> Result<Expression, ErrorEmitted> {
    let span = path_expr.span();
    let expression = if path_expr.root_opt.is_none()
        && path_expr.suffix.is_empty()
        && path_expr.prefix.generics_opt.is_none()
    {
        // only `foo`, it could either be a variable or an enum variant

        let name = path_expr_segment_to_ident(context, handler, &path_expr.prefix)?;
        Expression {
            kind: ExpressionKind::AmbiguousVariableExpression(name),
            span,
        }
    } else {
        let call_path_binding =
            path_expr_to_qualified_call_path_binding(context, handler, engines, path_expr)?;
        Expression {
            kind: ExpressionKind::DelineatedPath(Box::new(DelineatedPathExpression {
                call_path_binding,
                args: None,
            })),
            span,
        }
    };
    Ok(expression)
}

fn braced_code_block_contents_to_expression(
    context: &mut Context,
    handler: &Handler,
    engines: &Engines,
    braced_code_block_contents: Braces<CodeBlockContents>,
) -> Result<Expression, ErrorEmitted> {
    let span = braced_code_block_contents.span();
    let code_block = braced_code_block_contents_to_code_block(
        context,
        handler,
        engines,
        braced_code_block_contents,
    )?;
    Ok(Expression {
        kind: ExpressionKind::CodeBlock(code_block),
        span,
    })
}

fn if_expr_to_expression(
    context: &mut Context,
    handler: &Handler,
    engines: &Engines,
    if_expr: IfExpr,
) -> Result<Expression, ErrorEmitted> {
    let span = if_expr.span();
    let IfExpr {
        condition,
        then_block,
        else_opt,
        ..
    } = if_expr;
    let then_block_span = then_block.span();
    let then_block = Expression {
        kind: ExpressionKind::CodeBlock(braced_code_block_contents_to_code_block(
            context, handler, engines, then_block,
        )?),
        span: then_block_span.clone(),
    };
    let else_block = match else_opt {
        None => None,
        Some((_else_token, tail)) => {
            let expression = match tail {
                LoopControlFlow::Break(braced_code_block_contents) => {
                    braced_code_block_contents_to_expression(
                        context,
                        handler,
                        engines,
                        braced_code_block_contents,
                    )?
                }
                LoopControlFlow::Continue(if_expr) => {
                    if_expr_to_expression(context, handler, engines, *if_expr)?
                }
            };
            Some(expression)
        }
    };
    let expression = match condition {
        IfCondition::Expr(condition) => Expression {
            kind: ExpressionKind::If(IfExpression {
                condition: Box::new(expr_to_expression(context, handler, engines, *condition)?),
                then: Box::new(then_block),
                r#else: else_block.map(Box::new),
            }),
            span,
        },
        IfCondition::Let { lhs, rhs, .. } => {
            let scrutinee = pattern_to_scrutinee(context, handler, *lhs)?;
            let scrutinee_span = scrutinee.span();
            let mut branches = vec![MatchBranch {
                scrutinee,
                result: then_block.clone(),
                span: Span::join(scrutinee_span, &then_block_span),
            }];
            branches.push(match else_block {
                Some(else_block) => {
                    let else_block_span = else_block.span();
                    MatchBranch {
                        scrutinee: Scrutinee::CatchAll {
                            span: else_block_span.clone(),
                        },
                        result: else_block,
                        span: else_block_span,
                    }
                }
                None => {
                    let else_block_span = then_block.span();
                    MatchBranch {
                        scrutinee: Scrutinee::CatchAll {
                            span: else_block_span.clone(),
                        },
                        // If there's no else in an `if-let` expression,
                        // then the else is equivalent to an empty block.
                        result: Expression {
                            kind: ExpressionKind::CodeBlock(CodeBlock {
                                contents: vec![],
                                whole_block_span: else_block_span.clone(),
                            }),
                            span: else_block_span.clone(),
                        },
                        span: else_block_span,
                    }
                }
            });

            match_expr_to_expression(context, handler, engines, *rhs, branches, span)?
        }
    };
    Ok(expression)
}

fn match_expr_to_expression(
    context: &mut Context,
    handler: &Handler,
    engines: &Engines,
    value: Expr,
    branches: Vec<MatchBranch>,
    span: Span,
) -> Result<Expression, ErrorEmitted> {
    let value = expr_to_expression(context, handler, engines, value)?;
    let var_decl_span = value.span();

    // Generate a deterministic name for the variable matched by the match expression.
    let matched_value_var_name = generate_matched_value_var_name(
        context.next_match_expression_matched_value_unique_suffix(),
    );

    let var_decl_name = Ident::new_with_override(matched_value_var_name, var_decl_span.clone());

    let var_decl_exp = Expression {
        kind: ExpressionKind::Variable(var_decl_name.clone()),
        span: var_decl_span,
    };

    let var_decl = engines.pe().insert(VariableDeclaration {
        type_ascription: {
            let type_id = engines.te().new_unknown();
            GenericArgument::Type(GenericTypeArgument {
                type_id,
                initial_type_id: type_id,
                span: var_decl_name.span(),
                call_path_tree: None,
            })
        },
        name: var_decl_name,
        is_mutable: false,
        body: value,
    });

    Ok(Expression {
        kind: ExpressionKind::CodeBlock(CodeBlock {
            contents: vec![
                AstNode {
                    content: AstNodeContent::Declaration(Declaration::VariableDeclaration(
                        var_decl,
                    )),
                    span: span.clone(),
                },
                AstNode {
                    content: AstNodeContent::Expression(Expression {
                        kind: ExpressionKind::ImplicitReturn(Box::new(Expression {
                            kind: ExpressionKind::Match(MatchExpression {
                                value: Box::new(var_decl_exp),
                                branches,
                            }),
                            span: span.clone(),
                        })),
                        span: span.clone(),
                    }),
                    span: span.clone(),
                },
            ],
            whole_block_span: span.clone(),
        }),
        span,
    })
}

fn for_expr_to_expression(
    context: &mut Context,
    handler: &Handler,
    engines: &Engines,
    value_pattern: &Pattern,
    iterator: Box<Expr>,
    block: Braces<CodeBlockContents>,
    span: Span,
) -> Result<Expression, ErrorEmitted> {
    // Desugar for loop into:
    //      let mut iterable = iterator;
    //    while true {
    //        let value_opt = iterable.next();
    //        if value_opt.is_none() {
    //             break;
    //        }
    //        let value = value_opt.unwrap();
    //        code_block
    //    }
    let value_opt_ident = Ident::new_no_span(format!(
        "__for_value_opt_{}",
        context.next_for_unique_suffix()
    ));
    let value_opt_expr = Expression {
        kind: ExpressionKind::Variable(value_opt_ident.clone()),
        span: Span::dummy(),
    };

    let iterable_ident = Ident::new_no_span(format!(
        "__for_iterable_{}",
        context.next_for_unique_suffix()
    ));
    let iterable_expr = Expression {
        kind: ExpressionKind::Variable(iterable_ident.clone()),
        span: Span::dummy(),
    };

    let iterator_expr = expr_to_expression(context, handler, engines, *iterator.clone())?;

    // Declare iterable with iterator return
    let iterable_decl = engines.pe().insert(VariableDeclaration {
        type_ascription: {
            let type_id = engines.te().new_unknown();
            GenericArgument::Type(GenericTypeArgument {
                type_id,
                initial_type_id: type_id,
                span: iterable_ident.clone().span(),
                call_path_tree: None,
            })
        },
        name: iterable_ident,
        is_mutable: true,
        body: iterator_expr.clone(),
    });

    // iterable.next() expression
    // We use iterator.span() so errors can point to it.
    let set_value_opt_to_next_body_expr = Expression {
        kind: ExpressionKind::MethodApplication(Box::new(MethodApplicationExpression {
            arguments: vec![iterable_expr],
            method_name_binding: TypeBinding {
                inner: MethodName::FromModule {
                    method_name: Ident::new_with_override("next".into(), iterator.span()),
                },
                type_arguments: TypeArgs::Regular(vec![]),
                span: iterator.span(),
            },
            contract_call_params: vec![],
        })),
        span: iterator.span(),
    };

    // Declare value_opt = iterable.next()
    let value_opt_to_next_decl = engines.pe().insert(VariableDeclaration {
        type_ascription: {
            let type_id = engines.te().new_unknown();
            GenericArgument::Type(GenericTypeArgument {
                type_id,
                initial_type_id: type_id,
                span: value_opt_ident.clone().span(),
                call_path_tree: None,
            })
        },
        name: value_opt_ident,
        is_mutable: true,
        body: set_value_opt_to_next_body_expr.clone(),
    });

    // Call value_opt.is_none()
    let value_opt_is_none = Expression {
        kind: ExpressionKind::MethodApplication(Box::new(MethodApplicationExpression {
            arguments: vec![value_opt_expr.clone()],
            method_name_binding: TypeBinding {
                inner: MethodName::FromModule {
                    method_name: Ident::new_no_span("is_none".into()),
                },
                type_arguments: TypeArgs::Regular(vec![]),
                span: Span::dummy(),
            },
            contract_call_params: vec![],
        })),
        span: Span::dummy(),
    };

    // Call value_opt.unwrap()
    // We use iterator.span() so mismatched types errors can point to it.
    let value_opt_unwarp = Expression {
        kind: ExpressionKind::MethodApplication(Box::new(MethodApplicationExpression {
            arguments: vec![value_opt_expr],
            method_name_binding: TypeBinding {
                inner: MethodName::FromModule {
                    method_name: Ident::new_with_override("unwrap".into(), iterator.span()),
                },
                type_arguments: TypeArgs::Regular(vec![]),
                span: iterator.span(),
            },
            contract_call_params: vec![],
        })),
        span: iterator.span(),
    };

    let pattern_ast_nodes = statement_let_to_ast_nodes_unfold(
        context,
        handler,
        engines,
        value_pattern.clone(),
        None,
        value_opt_unwarp,
        value_pattern.span(),
    )?;

    let mut while_body =
        braced_code_block_contents_to_code_block(context, handler, engines, block)?;

    //At the beginning of while block do:
    //    let value_opt = iterable.next();
    //    if value_opt.is_none() {
    //        break;
    //    }
    //    let value = value_opt.unwrap();
    // Note: Inserting in reverse order

    //    let value = value_opt.unwrap();
    for node in pattern_ast_nodes.iter().rev() {
        while_body.contents.insert(0, node.clone());
    }

    //    if value_opt.is_none() {
    //        break;
    //    }
    while_body.contents.insert(
        0,
        AstNode {
            content: AstNodeContent::Expression(Expression {
                kind: ExpressionKind::If(IfExpression {
                    condition: Box::new(value_opt_is_none),
                    then: Box::new(Expression {
                        kind: ExpressionKind::CodeBlock(CodeBlock {
                            contents: vec![AstNode {
                                content: AstNodeContent::Expression(Expression {
                                    kind: ExpressionKind::Break,
                                    span: Span::dummy(),
                                }),
                                span: Span::dummy(),
                            }],
                            whole_block_span: Span::dummy(),
                        }),
                        span: Span::dummy(),
                    }),
                    r#else: None,
                }),
                span: Span::dummy(),
            }),
            span: Span::dummy(),
        },
    );

    //    let value_opt = iterable.next();
    while_body.contents.insert(
        0,
        AstNode {
            content: AstNodeContent::Declaration(Declaration::VariableDeclaration(
                value_opt_to_next_decl,
            )),
            span: Span::dummy(),
        },
    );

    let desugared = Expression {
        kind: ExpressionKind::CodeBlock(CodeBlock {
            contents: vec![
                AstNode {
                    content: AstNodeContent::Declaration(Declaration::VariableDeclaration(
                        iterable_decl,
                    )),
                    span: Span::dummy(),
                },
                AstNode {
                    content: AstNodeContent::Expression(Expression {
                        kind: ExpressionKind::WhileLoop(WhileLoopExpression {
                            condition: Box::new(Expression {
                                kind: ExpressionKind::Literal(Literal::Boolean(true)),
                                span: Span::dummy(),
                            }),
                            body: while_body,
                            is_desugared_for_loop: true,
                        }),
                        span: Span::dummy(),
                    }),
                    span: Span::dummy(),
                },
            ],
            whole_block_span: Span::dummy(),
        }),
        span: span.clone(),
    };

    Ok(Expression {
        kind: ExpressionKind::ForLoop(ForLoopExpression {
            desugared: Box::new(desugared),
        }),
        span,
    })
}

/// Determine if the path is in absolute form, e.g., `::foo::bar`.
///
/// Throws an error when given `<Foo as Bar>::baz`.
fn path_root_opt_to_bool(
    _context: &mut Context,
    handler: &Handler,
    root_opt: Option<(Option<AngleBrackets<QualifiedPathRoot>>, DoubleColonToken)>,
) -> Result<bool, ErrorEmitted> {
    Ok(match root_opt {
        None => false,
        Some((None, _)) => true,
        Some((Some(qualified_path_root), _)) => {
            let error = ConvertParseTreeError::QualifiedPathRootsNotImplemented {
                span: qualified_path_root.span(),
            };
            return Err(handler.emit_err(error.into()));
        }
    })
}

fn path_root_opt_to_bool_and_qualified_path_root(
    context: &mut Context,
    handler: &Handler,
    engines: &Engines,
    root_opt: Option<(Option<AngleBrackets<QualifiedPathRoot>>, DoubleColonToken)>,
) -> Result<(bool, Option<QualifiedPathType>), ErrorEmitted> {
    Ok(match root_opt {
        None => (false, None),
        Some((None, _)) => (true, None),
        Some((
            Some(AngleBrackets {
                open_angle_bracket_token: _,
                inner: QualifiedPathRoot { ty, as_trait },
                close_angle_bracket_token: _,
            }),
            _,
        )) => (false, {
            let (_, path_type) = as_trait;
            Some(QualifiedPathType {
                ty: ty_to_type_argument(context, handler, engines, *ty)?,
                as_trait: engines.te().insert(
                    engines,
                    path_type_to_type_info(context, handler, engines, *path_type.clone())?,
                    path_type.span().source_id(),
                ),
                as_trait_span: path_type.span(),
            })
        }),
    })
}

fn literal_to_literal(
    _context: &mut Context,
    handler: &Handler,
    literal: sway_ast::Literal,
) -> Result<Literal, ErrorEmitted> {
    let literal = match literal {
        sway_ast::Literal::Bool(lit_bool) => Literal::Boolean(lit_bool.kind.into()),
        sway_ast::Literal::String(lit_string) => {
            let full_span = lit_string.span();
            let inner_span = Span::new(
                full_span.src().clone(),
                full_span.start() + 1,
                full_span.end() - 1,
                full_span.source_id().copied(),
            )
            .unwrap();
            Literal::String(inner_span)
        }
        sway_ast::Literal::Char(lit_char) => {
            let error = ConvertParseTreeError::CharLiteralsNotImplemented {
                span: lit_char.span(),
            };
            return Err(handler.emit_err(error.into()));
        }
        sway_ast::Literal::Int(lit_int) => {
            let LitInt {
                parsed,
                ty_opt,
                span,
                is_generated_b256: _,
            } = lit_int;
            match ty_opt {
                None => {
                    let orig_str = span.as_str();
                    if let Some(hex_digits) = orig_str.strip_prefix("0x") {
                        let num_digits = hex_digits.chars().filter(|c| *c != '_').count();
                        match num_digits {
                            1..=16 => Literal::Numeric(u64::try_from(parsed).unwrap()),
                            64 => {
                                let bytes = parsed.to_bytes_be();
                                let mut full_bytes = [0u8; 32];
                                full_bytes[(32 - bytes.len())..].copy_from_slice(&bytes);
                                Literal::B256(full_bytes)
                            }
                            _ => {
                                let error = ConvertParseTreeError::HexLiteralLength { span };
                                return Err(handler.emit_err(error.into()));
                            }
                        }
                    } else if let Some(bin_digits) = orig_str.strip_prefix("0b") {
                        let num_digits = bin_digits.chars().filter(|c| *c != '_').count();
                        match num_digits {
                            1..=64 => Literal::Numeric(u64::try_from(parsed).unwrap()),
                            256 => {
                                let bytes = parsed.to_bytes_be();
                                let mut full_bytes = [0u8; 32];
                                full_bytes[(32 - bytes.len())..].copy_from_slice(&bytes);
                                Literal::B256(full_bytes)
                            }
                            _ => {
                                let error = ConvertParseTreeError::BinaryLiteralLength { span };
                                return Err(handler.emit_err(error.into()));
                            }
                        }
                    } else {
                        match u64::try_from(&parsed) {
                            Ok(value) => Literal::Numeric(value),
                            Err(..) => {
                                let error = ConvertParseTreeError::IntLiteralOutOfRange { span };
                                return Err(handler.emit_err(error.into()));
                            }
                        }
                    }
                }
                Some((lit_int_type, _)) => match lit_int_type {
                    LitIntType::U8 => {
                        let value = match u8::try_from(parsed) {
                            Ok(value) => value,
                            Err(..) => {
                                let error = ConvertParseTreeError::U8LiteralOutOfRange { span };
                                return Err(handler.emit_err(error.into()));
                            }
                        };
                        Literal::U8(value)
                    }
                    LitIntType::U16 => {
                        let value = match u16::try_from(parsed) {
                            Ok(value) => value,
                            Err(..) => {
                                let error = ConvertParseTreeError::U16LiteralOutOfRange { span };
                                return Err(handler.emit_err(error.into()));
                            }
                        };
                        Literal::U16(value)
                    }
                    LitIntType::U32 => {
                        let value = match u32::try_from(parsed) {
                            Ok(value) => value,
                            Err(..) => {
                                let error = ConvertParseTreeError::U32LiteralOutOfRange { span };
                                return Err(handler.emit_err(error.into()));
                            }
                        };
                        Literal::U32(value)
                    }
                    LitIntType::U64 => {
                        let value = match u64::try_from(parsed) {
                            Ok(value) => value,
                            Err(..) => {
                                let error = ConvertParseTreeError::U64LiteralOutOfRange { span };
                                return Err(handler.emit_err(error.into()));
                            }
                        };
                        Literal::U64(value)
                    }
                    LitIntType::U256 => Literal::U256(parsed.into()),
                    LitIntType::I8 | LitIntType::I16 | LitIntType::I32 | LitIntType::I64 => {
                        let error = ConvertParseTreeError::SignedIntegersNotSupported { span };
                        return Err(handler.emit_err(error.into()));
                    }
                },
            }
        }
    };
    Ok(literal)
}

/// Like [path_expr_to_call_path], but instead can potentially return type arguments.
/// Use this when converting a call path that could potentially include type arguments, i.e. the
/// turbofish.
fn path_expr_to_qualified_call_path_binding(
    context: &mut Context,
    handler: &Handler,
    engines: &Engines,
    path_expr: PathExpr,
) -> Result<TypeBinding<QualifiedCallPath>, ErrorEmitted> {
    let PathExpr {
        root_opt,
        prefix,
        mut suffix,
        ..
    } = path_expr;
    let (is_relative_to_root, qualified_path_root) =
        path_root_opt_to_bool_and_qualified_path_root(context, handler, engines, root_opt)?;
    let callpath_type = if is_relative_to_root {
        CallPathType::RelativeToPackageRoot
    } else {
        CallPathType::Ambiguous
    };
    let (prefixes, suffix, span, regular_type_arguments, prefix_type_arguments) = match suffix.pop()
    {
        Some((_, call_path_suffix)) => {
            let (prefix_ident, mut prefix_type_arguments) = if suffix.is_empty() {
                path_expr_segment_to_ident_or_type_argument(context, handler, engines, prefix)?
            } else {
                (
                    path_expr_segment_to_ident(context, handler, &prefix)?,
                    vec![],
                )
            };
            let mut prefixes = vec![prefix_ident];
            for (i, (_, call_path_prefix)) in suffix.iter().enumerate() {
                let ident = if i == suffix.len() - 1 {
                    let (prefix, prefix_ty_args) = path_expr_segment_to_ident_or_type_argument(
                        context,
                        handler,
                        engines,
                        call_path_prefix.clone(),
                    )?;
                    prefix_type_arguments = prefix_ty_args;
                    prefix
                } else {
                    path_expr_segment_to_ident(context, handler, call_path_prefix)?
                };
                // note that call paths only support one set of type arguments per call path right
                // now
                prefixes.push(ident);
            }
            let span = call_path_suffix.span();
            let (suffix, ty_args) = path_expr_segment_to_ident_or_type_argument(
                context,
                handler,
                engines,
                call_path_suffix,
            )?;
            (prefixes, suffix, span, ty_args, prefix_type_arguments)
        }
        None => {
            let span = prefix.span();
            let (suffix, ty_args) =
                path_expr_segment_to_ident_or_type_argument(context, handler, engines, prefix)?;
            (vec![], suffix, span, ty_args, vec![])
        }
    };

    let type_arguments = if !regular_type_arguments.is_empty() && !prefix_type_arguments.is_empty()
    {
        let error = ConvertParseTreeError::MultipleGenericsNotSupported { span };
        return Err(handler.emit_err(error.into()));
    } else if !prefix_type_arguments.is_empty() {
        TypeArgs::Prefix(prefix_type_arguments)
    } else {
        TypeArgs::Regular(regular_type_arguments)
    };

    Ok(TypeBinding {
        inner: QualifiedCallPath {
            call_path: CallPath {
                prefixes,
                suffix,
                callpath_type,
            },
            qualified_path_root: qualified_path_root.map(Box::new),
        },
        type_arguments,
        span,
    })
}

/// Like [path_expr_to_call_path], but instead can potentially return type arguments.
/// Use this when converting a call path that could potentially include type arguments, i.e. the
/// turbofish.
fn path_expr_to_call_path_binding(
    context: &mut Context,
    handler: &Handler,
    engines: &Engines,
    path_expr: PathExpr,
) -> Result<TypeBinding<CallPath>, ErrorEmitted> {
    let TypeBinding {
        inner: QualifiedCallPath {
            call_path,
            qualified_path_root,
        },
        type_arguments,
        span,
    } = path_expr_to_qualified_call_path_binding(context, handler, engines, path_expr)?;

    if let Some(qualified_path_root) = qualified_path_root {
        let error = ConvertParseTreeError::QualifiedPathRootsNotImplemented {
            span: qualified_path_root.as_trait_span,
        };
        return Err(handler.emit_err(error.into()));
    }

    Ok(TypeBinding {
        inner: call_path,
        type_arguments,
        span,
    })
}

fn path_expr_to_call_path(
    context: &mut Context,
    handler: &Handler,
    path_expr: PathExpr,
) -> Result<CallPath, ErrorEmitted> {
    let PathExpr {
        root_opt,
        prefix,
        mut suffix,
        ..
    } = path_expr;
    let is_relative_to_root = path_root_opt_to_bool(context, handler, root_opt)?;
    let callpath_type = if is_relative_to_root {
        CallPathType::RelativeToPackageRoot
    } else {
        CallPathType::Ambiguous
    };
    let call_path = match suffix.pop() {
        Some((_double_colon_token, call_path_suffix)) => {
            let mut prefixes = vec![path_expr_segment_to_ident(context, handler, &prefix)?];
            for (_double_colon_token, call_path_prefix) in suffix {
                let ident = path_expr_segment_to_ident(context, handler, &call_path_prefix)?;
                prefixes.push(ident);
            }
            CallPath {
                prefixes,
                suffix: path_expr_segment_to_ident(context, handler, &call_path_suffix)?,
                callpath_type,
            }
        }
        None => CallPath {
            prefixes: Vec::new(),
            suffix: path_expr_segment_to_ident(context, handler, &prefix)?,
            callpath_type,
        },
    };
    Ok(call_path)
}

fn expr_struct_field_to_struct_expression_field(
    context: &mut Context,
    handler: &Handler,
    engines: &Engines,
    expr_struct_field: ExprStructField,
) -> Result<StructExpressionField, ErrorEmitted> {
    let span = expr_struct_field.span();
    let value = match expr_struct_field.expr_opt {
        Some((_colon_token, expr)) => expr_to_expression(context, handler, engines, *expr)?,
        None => Expression {
            kind: ExpressionKind::Variable(expr_struct_field.field_name.clone()),
            span,
        },
    };
    Ok(StructExpressionField {
        name: expr_struct_field.field_name,
        value,
    })
}

fn expr_tuple_descriptor_to_expressions(
    context: &mut Context,
    handler: &Handler,
    engines: &Engines,
    expr_tuple_descriptor: ExprTupleDescriptor,
) -> Result<Vec<Expression>, ErrorEmitted> {
    let expressions = match expr_tuple_descriptor {
        ExprTupleDescriptor::Nil => Vec::new(),
        ExprTupleDescriptor::Cons { head, tail, .. } => {
            let mut expressions = vec![expr_to_expression(context, handler, engines, *head)?];
            for expr in tail {
                expressions.push(expr_to_expression(context, handler, engines, expr)?);
            }
            expressions
        }
    };
    Ok(expressions)
}

fn asm_block_to_asm_expression(
    context: &mut Context,
    handler: &Handler,
    engines: &Engines,
    asm_block: AsmBlock,
) -> Result<Box<AsmExpression>, ErrorEmitted> {
    let whole_block_span = asm_block.span();
    let asm_block_contents = asm_block.contents.into_inner();
    let (returns, return_type) = match asm_block_contents.final_expr_opt {
        // If the return register is specified.
        Some(asm_final_expr) => {
            let asm_register = AsmRegister {
                name: asm_final_expr.register.as_str().to_owned(),
            };
            let returns = Some((asm_register, asm_final_expr.register.span()));
            let return_type = match asm_final_expr.ty_opt {
                Some((_colon_token, ty)) => ty_to_type_info(context, handler, engines, ty)?,
                // If the return type is not specified, the ASM block returns `u64` as the default.
                None => TypeInfo::UnsignedInteger(IntegerBits::SixtyFour),
            };
            (returns, return_type)
        }
        // If the return register is not specified, the return type is unit, `()`.
        None => (None, TypeInfo::Tuple(Vec::new())),
    };
    let registers = {
        asm_block
            .registers
            .into_inner()
            .into_iter()
            .map(|asm_register_declaration| {
                asm_register_declaration_to_asm_register_declaration(
                    context,
                    handler,
                    engines,
                    asm_register_declaration,
                )
            })
            .collect::<Result<_, _>>()?
    };
    let body = {
        asm_block_contents
            .instructions
            .into_iter()
            .map(|(instruction, _semicolon_token)| instruction_to_asm_op(&instruction))
            .collect()
    };
    Ok(Box::new(AsmExpression {
        registers,
        body,
        returns,
        return_type,
        whole_block_span,
    }))
}

fn match_branch_to_match_branch(
    context: &mut Context,
    handler: &Handler,
    engines: &Engines,
    match_branch: sway_ast::MatchBranch,
) -> Result<MatchBranch, ErrorEmitted> {
    let span = match_branch.span();
    Ok(MatchBranch {
        scrutinee: pattern_to_scrutinee(context, handler, match_branch.pattern)?,
        result: match match_branch.kind {
            MatchBranchKind::Block { block, .. } => {
                let span = block.span();
                Expression {
                    kind: ExpressionKind::CodeBlock(braced_code_block_contents_to_code_block(
                        context, handler, engines, block,
                    )?),
                    span,
                }
            }
            MatchBranchKind::Expr { expr, .. } => {
                expr_to_expression(context, handler, engines, expr)?
            }
        },
        span,
    })
}

fn statement_let_to_ast_nodes(
    context: &mut Context,
    handler: &Handler,
    engines: &Engines,
    statement_let: StatementLet,
) -> Result<Vec<AstNode>, ErrorEmitted> {
    let span = statement_let.span();
    let initial_expression = expr_to_expression(context, handler, engines, statement_let.expr)?;
    statement_let_to_ast_nodes_unfold(
        context,
        handler,
        engines,
        statement_let.pattern,
        statement_let.ty_opt.map(|(_colon_token, ty)| ty),
        initial_expression,
        span,
    )
}

fn statement_let_to_ast_nodes_unfold(
    context: &mut Context,
    handler: &Handler,
    engines: &Engines,
    pattern: Pattern,
    ty_opt: Option<Ty>,
    expression: Expression,
    span: Span,
) -> Result<Vec<AstNode>, ErrorEmitted> {
    let ast_nodes = match pattern {
        Pattern::Wildcard { .. } | Pattern::Var { .. } | Pattern::AmbiguousSingleIdent(..) => {
            let (reference, mutable, name) = match pattern {
                Pattern::Var {
                    reference,
                    mutable,
                    name,
                } => (reference, mutable, name),
                Pattern::Wildcard { underscore_token } => (
                    None,
                    None,
                    Ident::new_with_override("_".to_string(), underscore_token.span()),
                ),
                Pattern::AmbiguousSingleIdent(ident) => (None, None, ident),
                _ => unreachable!(),
            };
            if reference.is_some() {
                let error = ConvertParseTreeError::RefVariablesNotSupported { span };
                return Err(handler.emit_err(error.into()));
            }
            let type_ascription = match ty_opt {
                Some(ty) => ty_to_type_argument(context, handler, engines, ty)?,
                None => {
                    let type_id = engines.te().new_unknown();
                    GenericArgument::Type(GenericTypeArgument {
                        type_id,
                        initial_type_id: type_id,
                        span: name.span(),
                        call_path_tree: None,
                    })
                }
            };
            let var_decl = engines.pe().insert(VariableDeclaration {
                name,
                type_ascription,
                body: expression,
                is_mutable: mutable.is_some(),
            });
            let ast_node = AstNode {
                content: AstNodeContent::Declaration(Declaration::VariableDeclaration(var_decl)),
                span,
            };
            vec![ast_node]
        }
        Pattern::Literal(..) => {
            let error = ConvertParseTreeError::LiteralPatternsNotSupportedHere { span };
            return Err(handler.emit_err(error.into()));
        }
        Pattern::Constant(..) => {
            let error = ConvertParseTreeError::ConstantPatternsNotSupportedHere { span };
            return Err(handler.emit_err(error.into()));
        }
        Pattern::Constructor { .. } | Pattern::Error(..) => {
            let error = ConvertParseTreeError::ConstructorPatternsNotSupportedHere { span };
            return Err(handler.emit_err(error.into()));
        }
        Pattern::Struct { path, fields, .. } => {
            let mut ast_nodes = Vec::new();

            // Generate a deterministic name for the destructured struct variable.
            let destructured_struct_name = generate_destructured_struct_var_name(
                context.next_destructured_struct_unique_suffix(),
            );

            let destructured_struct_name =
                Ident::new_with_override(destructured_struct_name, path.prefix.name.span());

            // Parse the type ascription and the type ascription span.
            // In the event that the user did not provide a type ascription,
            // it is set to TypeInfo::Unknown and the span to None.
            let type_ascription = match &ty_opt {
                Some(ty) => ty_to_type_argument(context, handler, engines, ty.clone())?,
                None => {
                    let type_id = engines.te().new_unknown();
                    GenericArgument::Type(GenericTypeArgument {
                        type_id,
                        initial_type_id: type_id,
                        span: destructured_struct_name.span(),
                        call_path_tree: None,
                    })
                }
            };

            // Save the destructure to the new name as a new variable declaration
            let save_body_first = engines.pe().insert(VariableDeclaration {
                name: destructured_struct_name.clone(),
                type_ascription,
                body: expression,
                is_mutable: false,
            });
            ast_nodes.push(AstNode {
                content: AstNodeContent::Declaration(Declaration::VariableDeclaration(
                    save_body_first,
                )),
                span: span.clone(),
            });

            // create a new variable expression that points to the new destructured struct name that we just created
            let new_expr = Expression {
                kind: ExpressionKind::Variable(destructured_struct_name),
                span: span.clone(),
            };

            // for all of the fields of the struct destructuring on the LHS,
            // recursively create variable declarations
            for pattern_struct_field in fields.into_inner() {
                let (field, recursive_pattern) = match pattern_struct_field {
                    PatternStructField::Field {
                        field_name,
                        pattern_opt,
                    } => {
                        let recursive_pattern = match pattern_opt {
                            Some((_colon_token, box_pattern)) => *box_pattern,
                            None => Pattern::Var {
                                reference: None,
                                mutable: None,
                                name: field_name.clone(),
                            },
                        };
                        (field_name, recursive_pattern)
                    }
                    PatternStructField::Rest { .. } => {
                        continue;
                    }
                };

                // recursively create variable declarations for the subpatterns on the LHS
                // and add them to the ast nodes
                ast_nodes.extend(statement_let_to_ast_nodes_unfold(
                    context,
                    handler,
                    engines,
                    recursive_pattern,
                    None,
                    Expression {
                        kind: ExpressionKind::Subfield(SubfieldExpression {
                            prefix: Box::new(new_expr.clone()),
                            field_to_access: field,
                        }),
                        span: span.clone(),
                    },
                    span.clone(),
                )?);
            }
            ast_nodes
        }
        Pattern::Or { .. } => {
            let error = ConvertParseTreeError::OrPatternsNotSupportedHere { span };
            return Err(handler.emit_err(error.into()));
        }
        Pattern::Tuple(pat_tuple) => {
            let mut ast_nodes = Vec::new();

            // Generate a deterministic name for the tuple.
            let tuple_name =
                generate_tuple_var_name(context.next_destructured_tuple_unique_suffix());

            let tuple_name = Ident::new_with_override(tuple_name, pat_tuple.span());

            // Ascribe a second declaration to a tuple of placeholders to check that the tuple
            // is properly sized to the pattern.
            let placeholders_type_ascription = {
                let type_id = engines.te().insert_tuple_without_annotations(
                    engines,
                    pat_tuple
                        .clone()
                        .into_inner()
                        .into_iter()
                        .map(|pat| {
                            // Since these placeholders are generated specifically for checks, the `pat.span()` must not
                            // necessarily point to a "_" string in code. E.g., in this example:
                            //   let (a, _) = (0, 0);
                            // The first `pat.span()` will point to "a", while the second one will indeed point to "_".
                            // However, their `pat.span()`s will always be in the source file in which the placeholder
                            // is logically situated.
                            engines.te().new_placeholder(TypeParameter::Type(
                                GenericTypeParameter::new_placeholder(
                                    engines.te().new_unknown(),
                                    pat.span(),
                                ),
                            ))
                        })
                        .collect(),
                );

                // The type argument is a tuple of place holders of unknowns pointing to
                // the tuple pattern.
                GenericArgument::Type(GenericTypeArgument {
                    type_id,
                    initial_type_id: type_id,
                    span: pat_tuple.span(),
                    call_path_tree: None,
                })
            };

            // Parse the type ascription and the type ascription span.
            // In the event that the user did not provide a type ascription,
            // it is set to TypeInfo::Unknown and the span to None.
            let type_ascription = match &ty_opt {
                Some(ty) => ty_to_type_argument(context, handler, engines, ty.clone())?,
                None => placeholders_type_ascription.clone(),
            };

            // Save the tuple to the new name as a new variable declaration.
            let save_body_first = engines.pe().insert(VariableDeclaration {
                name: tuple_name.clone(),
                type_ascription,
                body: expression,
                is_mutable: false,
            });
            ast_nodes.push(AstNode {
                content: AstNodeContent::Declaration(Declaration::VariableDeclaration(
                    save_body_first,
                )),
                span: span.clone(),
            });

            // create a variable expression that points to the new tuple name that we just created
            let new_expr = Expression {
                kind: ExpressionKind::Variable(tuple_name.clone()),
                span: span.clone(),
            };

            // Override the previous declaration with a tuple of placeholders to check the
            // shape of the tuple
            let check_tuple_shape_second = engines.pe().insert(VariableDeclaration {
                name: tuple_name,
                type_ascription: placeholders_type_ascription,
                body: new_expr.clone(),
                is_mutable: false,
            });
            ast_nodes.push(AstNode {
                content: AstNodeContent::Declaration(Declaration::VariableDeclaration(
                    check_tuple_shape_second,
                )),
                span: span.clone(),
            });

            // from the possible type annotation, if the annotation was a tuple annotation,
            // extract the internal types of the annotation
            let tuple_tys_opt = match ty_opt {
                Some(Ty::Tuple(tys)) => Some(tys.into_inner().to_tys()),
                _ => None,
            };

            // for all of the elements in the tuple destructuring on the LHS,
            // recursively create variable declarations
            for (index, pattern) in pat_tuple.into_inner().into_iter().enumerate() {
                // from the possible type annotation, grab the type at the index of the current element
                // we are processing
                let ty_opt = tuple_tys_opt
                    .as_ref()
                    .and_then(|tys| tys.get(index).cloned());

                // recursively create variable declarations for the subpatterns on the LHS
                // and add them to the ast nodes
                ast_nodes.extend(statement_let_to_ast_nodes_unfold(
                    context,
                    handler,
                    engines,
                    pattern,
                    ty_opt,
                    Expression {
                        kind: ExpressionKind::TupleIndex(TupleIndexExpression {
                            prefix: Box::new(new_expr.clone()),
                            index,
                            index_span: span.clone(),
                        }),
                        span: span.clone(),
                    },
                    span.clone(),
                )?);
            }
            ast_nodes
        }
    };
    Ok(ast_nodes)
}

fn submodule_to_include_statement(submodule: &Submodule) -> IncludeStatement {
    IncludeStatement {
        span: submodule.span(),
        mod_name: submodule.name.clone(),
        visibility: pub_token_opt_to_visibility(submodule.visibility.clone()),
    }
}

#[allow(dead_code)]
fn generic_args_to_type_parameters(
    context: &mut Context,
    handler: &Handler,
    engines: &Engines,
    generic_args: GenericArgs,
) -> Result<Vec<TypeParameter>, ErrorEmitted> {
    generic_args
        .parameters
        .into_inner()
        .into_iter()
        .map(|x| ty_to_type_parameter(context, handler, engines, x))
        .collect()
}

fn asm_register_declaration_to_asm_register_declaration(
    context: &mut Context,
    handler: &Handler,
    engines: &Engines,
    asm_register_declaration: sway_ast::AsmRegisterDeclaration,
) -> Result<AsmRegisterDeclaration, ErrorEmitted> {
    let initializer = asm_register_declaration
        .value_opt
        .map(|(_colon_token, expr)| expr_to_expression(context, handler, engines, *expr))
        .transpose()?;

    Ok(AsmRegisterDeclaration {
        name: asm_register_declaration.register,
        initializer,
    })
}

fn instruction_to_asm_op(instruction: &Instruction) -> AsmOp {
    AsmOp {
        op_name: instruction.op_code_ident(),
        op_args: instruction.register_arg_idents(),
        span: instruction.span(),
        immediate: instruction.immediate_ident_opt(),
    }
}

fn pattern_to_scrutinee(
    context: &mut Context,
    handler: &Handler,
    pattern: Pattern,
) -> Result<Scrutinee, ErrorEmitted> {
    let span = pattern.span();
    let scrutinee = match pattern {
        Pattern::Or {
            lhs,
            pipe_token: _,
            rhs,
        } => {
            let mut elems = vec![rhs];
            let mut current_lhs = lhs;

            while let Pattern::Or {
                lhs: new_lhs,
                pipe_token: _,
                rhs: new_rhs,
            } = *current_lhs
            {
                elems.push(new_rhs);
                current_lhs = new_lhs;
            }
            elems.push(current_lhs);

            let elems = elems
                .into_iter()
                .rev()
                .map(|p| pattern_to_scrutinee(context, handler, *p))
                .collect::<Result<Vec<_>, _>>()?;
            Scrutinee::Or { span, elems }
        }
        Pattern::Wildcard { underscore_token } => Scrutinee::CatchAll {
            span: underscore_token.span(),
        },
        Pattern::Var {
            reference,
            mutable,
            name,
        } => {
            let unsupported_ref_mut = match (reference, mutable) {
                (Some(reference), Some(mutable)) => Some((
                    "ref mut",
                    vec![
                        "If you want to change the matched value, or some of its parts, consider using the matched value".to_string(),
                        format!("directly in the block, instead of the pattern variable \"{name}\"."),
                    ],
                    Span::join(reference.span(), &mutable.span())
                )),
                (Some(reference), None) => Some((
                    "ref",
                    vec![
                        "If you want to avoid copying the matched value, or some of its parts, consider using the matched value".to_string(),
                        format!("directly in the block, instead of the pattern variable \"{name}\"."),
                    ],
                    reference.span()
                )),
                (None, Some(mutable)) => Some((
                    "mut",
                    vec![
                        format!("If you want change the value of the pattern variable \"{name}\", consider declaring its mutable copy within the block:"),
                        format!("  let mut {name} = {name};"),
                        format!("Note that the pattern variable \"{name}\" already contains a copy of the matched value, or some of its parts,"),
                        format!("and that the original matched value will not be affected when changing the mutable \"{name}\"."),
                        " ".to_string(),
                        "Alternatively, if you want to change the matched value, or some of its parts, consider using the matched value".to_string(),
                        format!("directly in the block, instead of the pattern variable \"{name}\"."),
                    ],
                    mutable.span()
                )),
                _ => None,
            };

            if let Some((msg, help, span)) = unsupported_ref_mut {
                return Err(handler.emit_err(CompileError::Unimplemented {
                    feature: format!("Using `{msg}` in pattern variables"),
                    help,
                    span,
                }));
            }

            Scrutinee::Variable { name, span }
        }
        Pattern::AmbiguousSingleIdent(ident) => Scrutinee::AmbiguousSingleIdent(ident),
        Pattern::Literal(literal) => Scrutinee::Literal {
            value: literal_to_literal(context, handler, literal)?,
            span,
        },
        Pattern::Constant(path_expr) => {
            let call_path = path_expr_to_call_path(context, handler, path_expr)?;
            let call_path_span = call_path.span();
            Scrutinee::EnumScrutinee {
                call_path,
                value: Box::new(Scrutinee::CatchAll {
                    span: call_path_span,
                }),
                span,
            }
        }
        Pattern::Constructor { path, args } => {
            let value = match iter_to_array(args.into_inner()) {
                Some([arg]) => arg,
                None => {
                    let error = ConvertParseTreeError::ConstructorPatternOneArg { span };
                    return Err(handler.emit_err(error.into()));
                }
            };
            Scrutinee::EnumScrutinee {
                call_path: path_expr_to_call_path(context, handler, path)?,
                value: Box::new(pattern_to_scrutinee(context, handler, value)?),
                span,
            }
        }
        Pattern::Struct { path, fields } => {
            let mut errors = Vec::new();
            let fields = fields.into_inner();

            // Make sure each struct field is declared once
            let mut names_of_fields = std::collections::HashSet::new();
            fields.clone().into_iter().for_each(|v| {
                if let PatternStructField::Field {
                    field_name,
                    pattern_opt: _,
                } = v
                {
                    if !names_of_fields.insert(field_name.clone()) {
                        errors.push(ConvertParseTreeError::DuplicateStructField {
                            name: field_name.clone(),
                            span: field_name.span(),
                        });
                    }
                }
            });

            if let Some(errors) = emit_all(handler, errors) {
                return Err(errors);
            }

            let scrutinee_fields = fields
                .into_iter()
                .map(|field| {
                    pattern_struct_field_to_struct_scrutinee_field(context, handler, field)
                })
                .collect::<Result<_, _>>()?;

            Scrutinee::StructScrutinee {
                struct_name: path_expr_to_ident(context, handler, path)?.into(),
                fields: { scrutinee_fields },
                span,
            }
        }
        Pattern::Tuple(pat_tuple) => Scrutinee::Tuple {
            elems: {
                pat_tuple
                    .into_inner()
                    .into_iter()
                    .map(|pattern| pattern_to_scrutinee(context, handler, pattern))
                    .collect::<Result<_, _>>()?
            },
            span,
        },
        Pattern::Error(spans, err) => Scrutinee::Error { spans, err },
    };
    Ok(scrutinee)
}

#[allow(dead_code)]
fn ty_to_type_parameter(
    context: &mut Context,
    handler: &Handler,
    engines: &Engines,
    ty: Ty,
) -> Result<TypeParameter, ErrorEmitted> {
    let type_engine = engines.te();

    let name = match ty {
        Ty::Path(path_type) => path_type_to_ident(context, handler, path_type)?,
        Ty::Infer { underscore_token } => {
            let unknown_type = type_engine.new_unknown();
            return Ok(TypeParameter::Type(GenericTypeParameter {
                type_id: unknown_type,
                initial_type_id: unknown_type,
                name: underscore_token.into(),
                trait_constraints: Vec::default(),
                trait_constraints_span: Span::dummy(),
                is_from_parent: false,
            }));
        }
        Ty::Expr(_) => panic!("expr are not allowed in this position"),
        Ty::Tuple(..) => panic!("tuple types are not allowed in this position"),
        Ty::Array(..) => panic!("array types are not allowed in this position"),
        Ty::StringSlice(..) => panic!("str slice types are not allowed in this position"),
        Ty::StringArray { .. } => panic!("str array types are not allowed in this position"),
        Ty::Ptr { .. } => panic!("__ptr types are not allowed in this position"),
        Ty::Slice { .. } => panic!("__slice types are not allowed in this position"),
        Ty::Ref { .. } => panic!("ref types are not allowed in this position"),
        Ty::Never { .. } => panic!("never types are not allowed in this position"),
    };
    let custom_type = type_engine.new_custom_from_name(engines, name.clone());
    Ok(TypeParameter::Type(GenericTypeParameter {
        type_id: custom_type,
        initial_type_id: custom_type,
        name,
        trait_constraints: Vec::new(),
        trait_constraints_span: Span::dummy(),
        is_from_parent: false,
    }))
}

fn path_type_to_ident(
    context: &mut Context,
    handler: &Handler,
    path_type: PathType,
) -> Result<Ident, ErrorEmitted> {
    let PathType {
        root_opt,
        prefix,
        suffix,
    } = path_type;
    if root_opt.is_some() || !suffix.is_empty() {
        panic!("types with paths aren't currently supported");
    }
    path_type_segment_to_ident(context, handler, prefix)
}

fn path_expr_to_ident(
    context: &mut Context,
    handler: &Handler,
    path_expr: PathExpr,
) -> Result<Ident, ErrorEmitted> {
    let span = path_expr.span();
    let PathExpr {
        root_opt,
        prefix,
        suffix,
        ..
    } = path_expr;
    if root_opt.is_some() || !suffix.is_empty() {
        let error = ConvertParseTreeError::PathsNotSupportedHere { span };
        return Err(handler.emit_err(error.into()));
    }
    path_expr_segment_to_ident(context, handler, &prefix)
}

fn pattern_struct_field_to_struct_scrutinee_field(
    context: &mut Context,
    handler: &Handler,
    pattern_struct_field: PatternStructField,
) -> Result<StructScrutineeField, ErrorEmitted> {
    let span = pattern_struct_field.span();
    match pattern_struct_field {
        PatternStructField::Rest { token } => {
            let struct_scrutinee_field = StructScrutineeField::Rest { span: token.span() };
            Ok(struct_scrutinee_field)
        }
        PatternStructField::Field {
            field_name,
            pattern_opt,
        } => {
            let struct_scrutinee_field = StructScrutineeField::Field {
                field: field_name,
                scrutinee: pattern_opt
                    .map(|(_colon_token, pattern)| pattern_to_scrutinee(context, handler, *pattern))
                    .transpose()?,
                span,
            };
            Ok(struct_scrutinee_field)
        }
    }
}

fn assignable_to_expression(
    context: &mut Context,
    handler: &Handler,
    engines: &Engines,
    assignable: Assignable,
) -> Result<Expression, ErrorEmitted> {
    let span = assignable.span();
    let expression = match assignable {
        Assignable::ElementAccess(element_access) => {
            element_access_to_expression(context, handler, engines, element_access, span)?
        }
        Assignable::Deref {
            star_token: _,
            expr,
        } => Expression {
            kind: ExpressionKind::Deref(Box::new(expr_to_expression(
                context, handler, engines, *expr,
            )?)),
            span,
        },
    };

    return Ok(expression);

    fn element_access_to_expression(
        context: &mut Context,
        handler: &Handler,
        engines: &Engines,
        element_access: ElementAccess,
        span: Span,
    ) -> Result<Expression, ErrorEmitted> {
        let expression = match element_access {
            ElementAccess::Var(name) => Expression {
                kind: ExpressionKind::Variable(name),
                span,
            },
            ElementAccess::Index { target, arg } => Expression {
                kind: ExpressionKind::ArrayIndex(ArrayIndexExpression {
                    prefix: Box::new(element_access_to_expression(
                        context,
                        handler,
                        engines,
                        *target,
                        span.clone(),
                    )?),
                    index: Box::new(expr_to_expression(
                        context,
                        handler,
                        engines,
                        *arg.into_inner(),
                    )?),
                }),
                span,
            },
            ElementAccess::FieldProjection { target, name, .. } => {
                let mut idents = vec![&name];
                let mut base = &*target;
                let (storage_access_field_names_opt, storage_name_opt) = loop {
                    match base {
                        ElementAccess::FieldProjection { target, name, .. } => {
                            idents.push(name);
                            base = target;
                        }
                        ElementAccess::Var(name) => {
                            if name.as_str() == "storage" {
                                break (Some(idents), Some(name.clone()));
                            }
                            break (None, None);
                        }
                        _ => break (None, None),
                    }
                };
                match (storage_access_field_names_opt, storage_name_opt) {
                    (Some(field_names), Some(storage_name)) => {
                        let field_names = field_names.into_iter().rev().cloned().collect();
                        Expression {
                            kind: ExpressionKind::StorageAccess(StorageAccessExpression {
                                namespace_names: vec![],
                                field_names,
                                storage_keyword_span: storage_name.span(),
                            }),
                            span,
                        }
                    }
                    _ => Expression {
                        kind: ExpressionKind::Subfield(SubfieldExpression {
                            prefix: Box::new(element_access_to_expression(
                                context,
                                handler,
                                engines,
                                *target,
                                span.clone(),
                            )?),
                            field_to_access: name,
                        }),
                        span,
                    },
                }
            }
            ElementAccess::TupleFieldProjection {
                target,
                field,
                field_span,
                ..
            } => {
                let index = match usize::try_from(field) {
                    Ok(index) => index,
                    Err(..) => {
                        let error =
                            ConvertParseTreeError::TupleIndexOutOfRange { span: field_span };
                        return Err(handler.emit_err(error.into()));
                    }
                };
                Expression {
                    kind: ExpressionKind::TupleIndex(TupleIndexExpression {
                        prefix: Box::new(element_access_to_expression(
                            context,
                            handler,
                            engines,
                            *target,
                            span.clone(),
                        )?),
                        index,
                        index_span: field_span,
                    }),
                    span,
                }
            }
            ElementAccess::Deref { target, .. } => Expression {
                kind: ExpressionKind::Deref(Box::new(element_access_to_expression(
                    context,
                    handler,
                    engines,
                    *target,
                    span.clone(),
                )?)),
                span,
            },
        };

        Ok(expression)
    }
}

fn assignable_to_reassignment_target(
    context: &mut Context,
    handler: &Handler,
    engines: &Engines,
    assignable: Assignable,
) -> Result<ReassignmentTarget, ErrorEmitted> {
    let expression = assignable_to_expression(context, handler, engines, assignable)?;
    Ok(match expression.kind {
        ExpressionKind::Deref(_) => ReassignmentTarget::Deref(Box::new(expression)),
        _ => ReassignmentTarget::ElementAccess(Box::new(expression)),
    })
}

fn generic_args_to_type_arguments(
    context: &mut Context,
    handler: &Handler,
    engines: &Engines,
    generic_args: GenericArgs,
) -> Result<Vec<GenericArgument>, ErrorEmitted> {
    generic_args
        .parameters
        .into_inner()
        .into_iter()
        .map(|ty| ty_to_type_argument(context, handler, engines, ty))
        .collect()
}

fn ty_tuple_descriptor_to_type_arguments(
    context: &mut Context,
    handler: &Handler,
    engines: &Engines,
    ty_tuple_descriptor: TyTupleDescriptor,
) -> Result<Vec<GenericArgument>, ErrorEmitted> {
    let type_arguments = match ty_tuple_descriptor {
        TyTupleDescriptor::Nil => vec![],
        TyTupleDescriptor::Cons { head, tail, .. } => {
            let mut type_arguments = vec![ty_to_type_argument(context, handler, engines, *head)?];
            for ty in tail {
                type_arguments.push(ty_to_type_argument(context, handler, engines, ty)?);
            }
            type_arguments
        }
    };
    Ok(type_arguments)
}

fn path_type_to_type_info(
    context: &mut Context,
    handler: &Handler,
    engines: &Engines,
    path_type: PathType,
) -> Result<TypeInfo, ErrorEmitted> {
    let span = path_type.span();
    let PathType {
        root_opt,
        prefix: PathTypeSegment { name, generics_opt },
        suffix,
    } = path_type.clone();

    let type_info = match type_name_to_type_info_opt(&name) {
        Some(type_info) => {
            if root_opt.is_some() {
                let error = ConvertParseTreeError::FullySpecifiedTypesNotSupported { span };
                return Err(handler.emit_err(error.into()));
            }

            if let Some((_, generic_args)) = generics_opt {
                let error = ConvertParseTreeError::GenericsNotSupportedHere {
                    span: generic_args.span(),
                };
                return Err(handler.emit_err(error.into()));
            }

            if !suffix.is_empty() {
                let (call_path, type_arguments) = path_type_to_call_path_and_type_arguments(
                    context, handler, engines, path_type,
                )?;
                TypeInfo::Custom {
                    qualified_call_path: call_path,
                    type_arguments: Some(type_arguments),
                }
            } else {
                type_info
            }
        }
        None => {
            if name.as_str() == "self" {
                let error = ConvertParseTreeError::UnknownTypeNameSelf { span };
                return Err(handler.emit_err(error.into()));
            } else if name.as_str() == "ContractCaller" {
                if root_opt.is_some() || !suffix.is_empty() {
                    let error = ConvertParseTreeError::FullySpecifiedTypesNotSupported { span };
                    return Err(handler.emit_err(error.into()));
                }
                let generic_ty = match generics_opt.and_then(|(_, generic_args)| {
                    iter_to_array(generic_args.parameters.into_inner())
                }) {
                    Some([ty]) => ty,
                    None => {
                        let error = ConvertParseTreeError::ContractCallerOneGenericArg { span };
                        return Err(handler.emit_err(error.into()));
                    }
                };
                let abi_name = match generic_ty {
                    Ty::Path(path_type) => {
                        let call_path =
                            path_type_to_call_path(context, handler, path_type.clone())?;
                        AbiName::Known(call_path)
                    }
                    Ty::Infer { .. } => AbiName::Deferred,
                    _ => {
                        let error =
                            ConvertParseTreeError::ContractCallerNamedTypeGenericArg { span };
                        return Err(handler.emit_err(error.into()));
                    }
                };
                TypeInfo::ContractCaller {
                    abi_name,
                    address: None,
                }
            } else {
                let (call_path, type_arguments) = path_type_to_call_path_and_type_arguments(
                    context, handler, engines, path_type,
                )?;
                TypeInfo::Custom {
                    qualified_call_path: call_path,
                    type_arguments: Some(type_arguments),
                }
            }
        }
    };
    Ok(type_info)
}

fn iter_to_array<I, T, const N: usize>(iter: I) -> Option<[T; N]>
where
    I: IntoIterator<Item = T>,
{
    let mut iter = iter.into_iter();
    let mut ret: MaybeUninit<[T; N]> = MaybeUninit::uninit();
    for i in 0..N {
        match iter.next() {
            Some(value) => {
                let array_ptr = ret.as_mut_ptr();
                let start_ptr: *mut T = array_ptr as *mut T;
                let value_ptr: *mut T = unsafe { start_ptr.add(i) };
                unsafe {
                    value_ptr.write(value);
                }
            }
            None => {
                for j in (0..i).rev() {
                    let array_ptr = ret.as_mut_ptr();
                    let start_ptr: *mut T = array_ptr as *mut T;
                    let value_ptr = unsafe { start_ptr.add(j) };
                    unsafe {
                        drop(value_ptr.read());
                    }
                }
                return None;
            }
        }
    }
    let ret = unsafe { ret.assume_init() };
    Some(ret)
}

fn error_if_self_param_is_not_allowed(
    _context: &mut Context,
    handler: &Handler,
    engines: &Engines,
    parameters: &[FunctionParameter],
    fn_kind: &str,
) -> Result<(), ErrorEmitted> {
    for param in parameters {
        if engines
            .te()
            .get(param.type_argument.type_id())
            .is_self_type()
        {
            let error = ConvertParseTreeError::SelfParameterNotAllowedForFn {
                fn_kind: fn_kind.to_owned(),
                span: param.type_argument.span(),
            };
            return Err(handler.emit_err(error.into()));
        }
    }
    Ok(())
}

/// Walks all the `#[cfg]` attributes in `attributes`, evaluating them
/// and returning false if any evaluated to false.
///
/// If the cfg-evaluation returns false, the annotated elements are excluded
/// from the tree, including their annotations. This is important, because, if
/// any `#[cfg]` evaluates to false, any error in the annotations will be ignored.
///
/// This also implies that all `#[cfg]` attributes must be valid. The evaluation
/// returns error if that's not the case.
pub fn cfg_eval(
    context: &Context,
    handler: &Handler,
    attributes: &Attributes,
    experimental: ExperimentalFeatures,
) -> Result<bool, ErrorEmitted> {
    for cfg_attr in attributes.of_kind(AttributeKind::Cfg) {
        cfg_attr.check_args_multiplicity(handler)?;
        assert_eq!(
            (1usize, 1usize),
            (&cfg_attr.args_multiplicity()).into(),
            "`#[cfg]` attribute must have argument multiplicity of exactly one"
        );

        let arg = &cfg_attr.args[0];
        if arg.is_cfg_target() {
            let cfg_target_val = arg.get_string(handler, cfg_attr)?;
            if let Ok(cfg_target) = BuildTarget::from_str(cfg_target_val) {
                if cfg_target != context.build_target() {
                    return Ok(false);
                }
            } else {
                return Err(handler.emit_err(
                    (ConvertParseTreeError::InvalidAttributeArgValue {
                        span: arg
                            .value
                            .as_ref()
                            .expect("`cfg_target` is the value of `arg`")
                            .span(),
                        arg: arg.name.clone(),
                        expected_values: BuildTarget::CFG.to_vec(),
                    })
                    .into(),
                ));
            }
        } else if arg.is_cfg_program_type() {
            let cfg_program_type_val = arg.get_string(handler, cfg_attr)?;
            if let Ok(cfg_program_type) = TreeType::from_str(cfg_program_type_val) {
                if cfg_program_type
                    != context
                        .program_type()
                        .expect("at this compilation stage the `program_type` is defined")
                {
                    return Ok(false);
                }
            } else {
                return Err(handler.emit_err(
                    (ConvertParseTreeError::InvalidAttributeArgValue {
                        span: arg
                            .value
                            .as_ref()
                            .expect("`cfg_target` is the value of `arg`")
                            .span(),
                        arg: arg.name.clone(),
                        expected_values: TreeType::CFG.to_vec(),
                    })
                    .into(),
                ));
            }
        } else if arg.is_cfg_experimental() {
            let cfg_experimental_val = arg.get_bool(handler, cfg_attr)?;
            let experimental_enabled = experimental
                .is_enabled_for_cfg(arg.name.as_str())
                .expect("`arg` is a known `cfg` experimental argument");
            if cfg_experimental_val != experimental_enabled {
                return Ok(false);
            }
        } else {
            return Err(handler.emit_err(
                ConvertParseTreeError::InvalidAttributeArg {
                    attribute: cfg_attr.name.clone(),
                    arg: (&arg.name).into(),
                    expected_args: cfg_attr.expected_args().args_names(),
                }
                .into(),
            ));
        }
    }

    Ok(true)
}
