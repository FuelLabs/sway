use crate::{
    compiler_generated::{
        generate_destructured_struct_var_name, generate_matched_value_var_name,
        generate_tuple_var_name,
    },
    decl_engine::{parsed_engine::ParsedDeclEngineInsert, parsed_id::ParsedDeclId},
    language::{parsed::*, *},
    transform::{attribute::*, to_parsed_lang::context::Context},
    type_system::*,
    BuildTarget, Engines, ExperimentalFlags,
};

use indexmap::IndexMap;
use itertools::Itertools;
use sway_ast::{
    attribute::Annotated,
    expr::{LoopControlFlow, ReassignmentOp, ReassignmentOpVariant},
    ty::TyTupleDescriptor,
    AbiCastArgs, AngleBrackets, AsmBlock, Assignable, AttributeDecl, Braces, CodeBlockContents,
    CommaToken, DoubleColonToken, Expr, ExprArrayDescriptor, ExprStructField, ExprTupleDescriptor,
    FnArg, FnArgs, FnSignature, GenericArgs, GenericParams, IfCondition, IfExpr, Instruction,
    Intrinsic, Item, ItemAbi, ItemConfigurable, ItemConst, ItemEnum, ItemFn, ItemImpl, ItemKind,
    ItemStorage, ItemStruct, ItemTrait, ItemTraitItem, ItemTypeAlias, ItemUse, LitInt, LitIntType,
    MatchBranchKind, Module, ModuleKind, Parens, PathExpr, PathExprSegment, PathType,
    PathTypeSegment, Pattern, PatternStructField, PubToken, Punctuated, QualifiedPathRoot,
    Statement, StatementLet, Submodule, TraitType, Traits, Ty, TypeField, UseTree, WhereClause,
};
use sway_error::convert_parse_tree_error::ConvertParseTreeError;
use sway_error::handler::{ErrorEmitted, Handler};
use sway_error::warning::{CompileWarning, Warning};
use sway_types::{
    constants::{
        ALLOW_ATTRIBUTE_NAME, CFG_ATTRIBUTE_NAME, CFG_EXPERIMENTAL_NEW_ENCODING,
        CFG_PROGRAM_TYPE_ARG_NAME, CFG_TARGET_ARG_NAME, DEPRECATED_ATTRIBUTE_NAME,
        DOC_ATTRIBUTE_NAME, DOC_COMMENT_ATTRIBUTE_NAME, INLINE_ATTRIBUTE_NAME,
        PAYABLE_ATTRIBUTE_NAME, STORAGE_PURITY_ATTRIBUTE_NAME, STORAGE_PURITY_READ_NAME,
        STORAGE_PURITY_WRITE_NAME, TEST_ATTRIBUTE_NAME, VALID_ATTRIBUTE_NAMES,
    },
    integer_bits::IntegerBits,
};
use sway_types::{Ident, Span, Spanned};

use std::{
    collections::HashSet, convert::TryFrom, iter, mem::MaybeUninit, str::FromStr, sync::Arc,
};

pub fn convert_parse_tree(
    context: &mut Context,
    handler: &Handler,
    engines: &Engines,
    module: Module,
) -> Result<(TreeType, ParseTree), ErrorEmitted> {
    let tree_type = convert_module_kind(&module.kind);
    context.set_program_type(tree_type.clone());
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
        let mut prev_item: Option<Annotated<ItemKind>> = None;
        for item in module.items {
            let ast_nodes =
                item_to_ast_nodes(context, handler, engines, item.clone(), true, prev_item)?;
            root_nodes.extend(ast_nodes);
            prev_item = Some(item);
        }
        root_nodes
    };
    Ok(ParseTree { span, root_nodes })
}

fn ast_node_is_test_fn(engines: &Engines, node: &AstNode) -> bool {
    if let AstNodeContent::Declaration(Declaration::FunctionDeclaration(decl_id)) = node.content {
        let decl = engines.pe().get_function(&decl_id);
        if decl.attributes.contains_key(&AttributeKind::Test) {
            return true;
        }
    }
    false
}

fn item_to_ast_nodes(
    context: &mut Context,
    handler: &Handler,
    engines: &Engines,
    item: Item,
    is_root: bool,
    prev_item: Option<Annotated<ItemKind>>,
) -> Result<Vec<AstNode>, ErrorEmitted> {
    let attributes = item_attrs_to_map(context, handler, &item.attribute_list)?;
    if !cfg_eval(context, handler, &attributes, context.experimental)? {
        return Ok(vec![]);
    }

    let decl = |d| vec![AstNodeContent::Declaration(d)];

    let span = item.span();
    let contents = match item.value {
        ItemKind::Submodule(submodule) => {
            // Check that Dependency is not annotated
            if attributes.contains_key(&AttributeKind::DocComment) {
                let error = ConvertParseTreeError::CannotDocCommentDependency {
                    span: attributes
                        .get(&AttributeKind::DocComment)
                        .unwrap()
                        .last()
                        .unwrap()
                        .span
                        .clone(),
                };
                handler.emit_err(error.into());
            }
            for (attribute_kind, attributes) in attributes.iter() {
                if attribute_kind != &AttributeKind::DocComment {
                    for attribute in attributes {
                        let error = ConvertParseTreeError::CannotAnnotateDependency {
                            span: attribute.span.clone(),
                        };
                        handler.emit_err(error.into());
                    }
                }
            }
            // Check that Dependency comes after only other Dependencies
            let emit_expected_dep_at_beginning = || {
                let error = ConvertParseTreeError::ExpectedDependencyAtBeginning {
                    span: submodule.span(),
                };
                handler.emit_err(error.into());
            };
            match prev_item {
                Some(Annotated {
                    value: ItemKind::Submodule(_),
                    ..
                }) => (),
                Some(_) => emit_expected_dep_at_beginning(),
                None => (),
            }
            if !is_root {
                emit_expected_dep_at_beginning();
            }
            let incl_stmt = submodule_to_include_statement(&submodule);
            vec![AstNodeContent::IncludeStatement(incl_stmt)]
        }
        ItemKind::Use(item_use) => item_use_to_use_statements(context, handler, item_use)?
            .into_iter()
            .map(AstNodeContent::UseStatement)
            .collect(),
        ItemKind::Struct(item_struct) => decl(Declaration::StructDeclaration(
            item_struct_to_struct_declaration(context, handler, engines, item_struct, attributes)?,
        )),
        ItemKind::Enum(item_enum) => decl(Declaration::EnumDeclaration(
            item_enum_to_enum_declaration(context, handler, engines, item_enum, attributes)?,
        )),
        ItemKind::Fn(item_fn) => {
            let function_declaration_decl_id = item_fn_to_function_declaration(
                context, handler, engines, item_fn, attributes, None, None,
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
        ItemKind::Trait(item_trait) => decl(Declaration::TraitDeclaration(
            item_trait_to_trait_declaration(context, handler, engines, item_trait, attributes)?,
        )),
        ItemKind::Impl(item_impl) => decl(item_impl_to_declaration(
            context, handler, engines, item_impl,
        )?),
        ItemKind::Abi(item_abi) => decl(Declaration::AbiDeclaration(item_abi_to_abi_declaration(
            context, handler, engines, item_abi, attributes,
        )?)),
        ItemKind::Const(item_const) => decl(Declaration::ConstantDeclaration({
            item_const_to_constant_declaration(
                context, handler, engines, item_const, attributes, true,
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
        ItemKind::Configurable(item_configurable) => item_configurable_to_constant_declarations(
            context,
            handler,
            engines,
            item_configurable,
            attributes,
        )?
        .into_iter()
        .map(|decl| AstNodeContent::Declaration(Declaration::ConstantDeclaration(decl)))
        .collect(),
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

    Ok(contents
        .into_iter()
        .map(|content| AstNode {
            span: span.clone(),
            content,
        })
        .collect())
}

fn item_use_to_use_statements(
    _context: &mut Context,
    handler: &Handler,
    item_use: ItemUse,
) -> Result<Vec<UseStatement>, ErrorEmitted> {
    if let Some(pub_token) = item_use.visibility {
        let error = ConvertParseTreeError::PubUseNotSupported {
            span: pub_token.span(),
        };
        return Err(handler.emit_err(error.into()));
    }
    let mut ret = Vec::new();
    let mut prefix = Vec::new();
    let item_span = item_use.span();

    use_tree_to_use_statements(
        item_use.tree,
        item_use.root_import.is_some(),
        &mut prefix,
        &mut ret,
        item_span,
    );
    debug_assert!(prefix.is_empty());
    Ok(ret)
}

fn use_tree_to_use_statements(
    use_tree: UseTree,
    is_absolute: bool,
    path: &mut Vec<Ident>,
    ret: &mut Vec<UseStatement>,
    item_span: Span,
) {
    match use_tree {
        UseTree::Group { imports } => {
            for use_tree in imports.into_inner() {
                use_tree_to_use_statements(use_tree, is_absolute, path, ret, item_span.clone());
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
                is_absolute,
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
                is_absolute,
                alias: Some(alias),
            });
        }
        UseTree::Glob { .. } => {
            ret.push(UseStatement {
                call_path: path.clone(),
                span: item_span,
                import_type: ImportType::Star,
                is_absolute,
                alias: None,
            });
        }
        UseTree::Path { prefix, suffix, .. } => {
            path.push(prefix);
            use_tree_to_use_statements(*suffix, is_absolute, path, ret, item_span);
            path.pop().unwrap();
        }
        UseTree::Error { .. } => {
            // parsing error, nothing to push to the use statements collection
        }
    }
}

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
    attributes: AttributesMap,
) -> Result<ParsedDeclId<StructDeclaration>, ErrorEmitted> {
    // FIXME(Centril): We shoudln't be collecting into a temporary  `errors` here. Recover instead!
    let mut errors = Vec::new();
    let span = item_struct.span();
    let fields = item_struct
        .fields
        .into_inner()
        .into_iter()
        .map(|type_field| {
            let attributes = item_attrs_to_map(context, handler, &type_field.attribute_list)?;
            if !cfg_eval(context, handler, &attributes, context.experimental)? {
                return Ok(None);
            }
            Ok(Some(type_field_to_struct_field(
                context,
                handler,
                engines,
                type_field.value,
                attributes,
            )?))
        })
        .filter_map_ok(|field| field)
        .collect::<Result<Vec<_>, _>>()?;

    if fields.iter().any(
        |field| matches!(&&*engines.te().get(field.type_argument.type_id), TypeInfo::Custom { qualified_call_path, ..} if qualified_call_path.call_path.suffix == item_struct.name),
    ) {
        errors.push(ConvertParseTreeError::RecursiveType { span: span.clone() });
    }

    // Make sure each struct field is declared once
    let mut names_of_fields = std::collections::HashSet::new();
    fields.iter().for_each(|v| {
        if !names_of_fields.insert(v.name.clone()) {
            errors.push(ConvertParseTreeError::DuplicateStructField {
                name: v.name.clone(),
                span: v.name.span(),
            });
        }
    });

    if let Some(emitted) = emit_all(handler, errors) {
        return Err(emitted);
    }

    let struct_declaration_id = engines.pe().insert(StructDeclaration {
        name: item_struct.name,
        attributes,
        fields,
        type_parameters: generic_params_opt_to_type_parameters(
            context,
            handler,
            engines,
            item_struct.generics,
            item_struct.where_clause_opt,
        )?,
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
    attributes: AttributesMap,
) -> Result<ParsedDeclId<EnumDeclaration>, ErrorEmitted> {
    let mut errors = Vec::new();
    let span = item_enum.span();
    let variants = item_enum
        .fields
        .into_inner()
        .into_iter()
        .enumerate()
        .map(|(tag, type_field)| {
            let attributes = item_attrs_to_map(context, handler, &type_field.attribute_list)?;
            if !cfg_eval(context, handler, &attributes, context.experimental)? {
                return Ok(None);
            }
            Ok(Some(type_field_to_enum_variant(
                context,
                handler,
                engines,
                type_field.value,
                attributes,
                tag,
            )?))
        })
        .filter_map_ok(|field| field)
        .collect::<Result<Vec<_>, _>>()?;

    if variants.iter().any(|variant| {
       matches!(&&*engines.te().get(variant.type_argument.type_id), TypeInfo::Custom { qualified_call_path, ..} if qualified_call_path.call_path.suffix == item_enum.name)
    }) {
        errors.push(ConvertParseTreeError::RecursiveType { span: span.clone() });
    }

    // Make sure each enum variant is declared once
    let mut names_of_variants = std::collections::HashSet::new();
    variants.iter().for_each(|v| {
        if !names_of_variants.insert(v.name.clone()) {
            errors.push(ConvertParseTreeError::DuplicateEnumVariant {
                name: v.name.clone(),
                span: v.name.span(),
            });
        }
    });

    if let Some(emitted) = emit_all(handler, errors) {
        return Err(emitted);
    }

    let enum_declaration_id = engines.pe().insert(EnumDeclaration {
        name: item_enum.name,
        type_parameters: generic_params_opt_to_type_parameters(
            context,
            handler,
            engines,
            item_enum.generics,
            item_enum.where_clause_opt,
        )?,
        variants,
        span,
        visibility: pub_token_opt_to_visibility(item_enum.visibility),
        attributes,
    });
    Ok(enum_declaration_id)
}

fn item_fn_to_function_declaration(
    context: &mut Context,
    handler: &Handler,
    engines: &Engines,
    item_fn: ItemFn,
    attributes: AttributesMap,
    parent_generic_params_opt: Option<GenericParams>,
    parent_where_clause_opt: Option<WhereClause>,
) -> Result<ParsedDeclId<FunctionDeclaration>, ErrorEmitted> {
    let span = item_fn.span();
    let return_type = match item_fn.fn_signature.return_type_opt {
        Some((_right_arrow, ty)) => ty_to_type_argument(context, handler, engines, ty)?,
        None => {
            let type_id = engines.te().insert(
                engines,
                TypeInfo::Tuple(Vec::new()),
                item_fn.fn_signature.span().source_id(),
            );
            TypeArgument {
                type_id,
                initial_type_id: type_id,
                span: item_fn.fn_signature.span(),
                call_path_tree: None,
            }
        }
    };
    let fn_decl = FunctionDeclaration {
        purity: get_attributed_purity(context, handler, &attributes)?,
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
        type_parameters: generic_params_opt_to_type_parameters_with_parent(
            context,
            handler,
            engines,
            item_fn.fn_signature.generics,
            parent_generic_params_opt,
            item_fn.fn_signature.where_clause_opt.clone(),
            parent_where_clause_opt,
        )?,
        where_clause: item_fn
            .fn_signature
            .where_clause_opt
            .map(|where_clause| {
                where_clause_to_trait_constraints(context, handler, engines, where_clause)
            })
            .transpose()?
            .unwrap_or(vec![]),
    };
    let decl_id = engines.pe().insert(fn_decl);
    Ok(decl_id)
}

fn get_attributed_purity(
    _context: &mut Context,
    handler: &Handler,
    attributes: &AttributesMap,
) -> Result<Purity, ErrorEmitted> {
    let mut purity = Purity::Pure;
    let mut add_impurity = |new_impurity, counter_impurity| {
        if purity == Purity::Pure {
            purity = new_impurity;
        } else if purity == counter_impurity {
            purity = Purity::ReadsWrites;
        }
    };
    match attributes.get(&AttributeKind::Storage) {
        Some(attrs) if !attrs.is_empty() => {
            for arg in attrs.iter().flat_map(|attr| &attr.args) {
                match arg.name.as_str() {
                    STORAGE_PURITY_READ_NAME => add_impurity(Purity::Reads, Purity::Writes),
                    STORAGE_PURITY_WRITE_NAME => add_impurity(Purity::Writes, Purity::Reads),
                    _otherwise => {
                        let error = ConvertParseTreeError::InvalidAttributeArgument {
                            attribute: "storage".to_owned(),
                            span: arg.span(),
                        };
                        return Err(handler.emit_err(error.into()));
                    }
                }
            }
            Ok(purity)
        }
        _otherwise => Ok(Purity::Pure),
    }
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
    attributes: AttributesMap,
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
            let attributes = item_attrs_to_map(context, handler, &annotated.attribute_list)?;
            if !cfg_eval(context, handler, &attributes, context.experimental)? {
                return Ok(None);
            }
            Ok(Some(match annotated.value {
                ItemTraitItem::Fn(fn_sig, _) => {
                    fn_signature_to_trait_fn(context, handler, engines, fn_sig, attributes)
                        .map(TraitItem::TraitFn)
                }
                ItemTraitItem::Const(const_decl, _) => item_const_to_constant_declaration(
                    context, handler, engines, const_decl, attributes, false,
                )
                .map(TraitItem::Constant),
                ItemTraitItem::Type(trait_type, _) => trait_type_to_trait_type_declaration(
                    context, handler, engines, trait_type, attributes,
                )
                .map(TraitItem::Type),
                ItemTraitItem::Error(spans, error) => Ok(TraitItem::Error(spans, error)),
            }?))
        })
        .filter_map_ok(|item| item)
        .collect::<Result<_, _>>()?;
    let methods = match item_trait.trait_defs_opt {
        None => Vec::new(),
        Some(trait_defs) => trait_defs
            .into_inner()
            .into_iter()
            .map(|item_fn| {
                let attributes = item_attrs_to_map(context, handler, &item_fn.attribute_list)?;
                if !cfg_eval(context, handler, &attributes, context.experimental)? {
                    return Ok(None);
                }
                Ok(Some(item_fn_to_function_declaration(
                    context,
                    handler,
                    engines,
                    item_fn.value,
                    attributes,
                    item_trait.generics.clone(),
                    item_trait.where_clause_opt.clone(),
                )?))
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

fn item_impl_to_declaration(
    context: &mut Context,
    handler: &Handler,
    engines: &Engines,
    item_impl: ItemImpl,
) -> Result<Declaration, ErrorEmitted> {
    let block_span = item_impl.span();
    let implementing_for = ty_to_type_argument(context, handler, engines, item_impl.ty)?;
    let items = item_impl
        .contents
        .into_inner()
        .into_iter()
        .map(|item| {
            let attributes = item_attrs_to_map(context, handler, &item.attribute_list)?;
            if !cfg_eval(context, handler, &attributes, context.experimental)? {
                return Ok(None);
            }
            Ok(Some(match item.value {
                sway_ast::ItemImplItem::Fn(fn_item) => item_fn_to_function_declaration(
                    context,
                    handler,
                    engines,
                    fn_item,
                    attributes,
                    item_impl.generic_params_opt.clone(),
                    item_impl.where_clause_opt.clone(),
                )
                .map(ImplItem::Fn),
                sway_ast::ItemImplItem::Const(const_item) => item_const_to_constant_declaration(
                    context, handler, engines, const_item, attributes, true,
                )
                .map(ImplItem::Constant),
                sway_ast::ItemImplItem::Type(type_item) => trait_type_to_trait_type_declaration(
                    context, handler, engines, type_item, attributes,
                )
                .map(ImplItem::Type),
            }?))
        })
        .filter_map_ok(|item| item)
        .collect::<Result<_, _>>()?;

    let impl_type_parameters = generic_params_opt_to_type_parameters(
        context,
        handler,
        engines,
        item_impl.generic_params_opt,
        item_impl.where_clause_opt,
    )?;

    match item_impl.trait_opt {
        Some((path_type, _)) => {
            let (trait_name, trait_type_arguments) =
                path_type_to_call_path_and_type_arguments(context, handler, engines, path_type)?;
            let impl_trait = ImplTrait {
                impl_type_parameters,
                trait_name: trait_name.to_call_path(handler)?,
                trait_type_arguments,
                implementing_for,
                items,
                block_span,
            };
            let impl_trait = engines.pe().insert(impl_trait);
            Ok(Declaration::ImplTrait(impl_trait))
        }
        None => match &*engines.te().get(implementing_for.type_id) {
            TypeInfo::Contract => Err(handler
                .emit_err(ConvertParseTreeError::SelfImplForContract { span: block_span }.into())),
            _ => {
                let impl_self = ImplSelf {
                    implementing_for,
                    impl_type_parameters,
                    items,
                    block_span,
                };
                let impl_self = engines.pe().insert(impl_self);
                Ok(Declaration::ImplSelf(impl_self))
            }
        },
    }
}

fn path_type_to_call_path_and_type_arguments(
    context: &mut Context,
    handler: &Handler,
    engines: &Engines,
    path_type: PathType,
) -> Result<(QualifiedCallPath, Vec<TypeArgument>), ErrorEmitted> {
    let root_opt = path_type.root_opt.clone();
    let (prefixes, suffix) = path_type_to_prefixes_and_suffix(context, handler, path_type)?;

    let (is_absolute, qualified_path) =
        path_root_opt_to_bool_and_qualified_path_root(context, handler, engines, root_opt)?;

    let qualified_call_path = QualifiedCallPath {
        call_path: CallPath {
            prefixes,
            suffix: suffix.name,
            is_absolute,
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
    attributes: AttributesMap,
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
                    let attributes =
                        item_attrs_to_map(context, handler, &annotated.attribute_list)?;
                    if !cfg_eval(context, handler, &attributes, context.experimental)? {
                        return Ok(None);
                    }
                    Ok(Some(match annotated.value {
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
                                &trait_fn.parameters,
                                "an ABI method signature",
                            )?;
                            Ok(TraitItem::TraitFn(trait_fn))
                        }
                        ItemTraitItem::Const(const_decl, _) => item_const_to_constant_declaration(
                            context, handler, engines, const_decl, attributes, false,
                        )
                        .map(TraitItem::Constant),
                        ItemTraitItem::Type(type_decl, _) => trait_type_to_trait_type_declaration(
                            context, handler, engines, type_decl, attributes,
                        )
                        .map(TraitItem::Type),
                        ItemTraitItem::Error(spans, error) => Ok(TraitItem::Error(spans, error)),
                    }?))
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
                    let attributes = item_attrs_to_map(context, handler, &item_fn.attribute_list)?;
                    if !cfg_eval(context, handler, &attributes, context.experimental)? {
                        return Ok(None);
                    }
                    let function_declaration_id = item_fn_to_function_declaration(
                        context,
                        handler,
                        engines,
                        item_fn.value,
                        attributes,
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
                    Ok(Some(function_declaration_id))
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
    attributes: AttributesMap,
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
            engines.te().insert(engines, TypeInfo::Unknown, None).into()
        }
    };

    let const_decl = ConstantDeclaration {
        name: item_const.name,
        type_ascription,
        value: expr,
        visibility: pub_token_opt_to_visibility(item_const.visibility),
        is_configurable: false,
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
    attributes: AttributesMap,
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
    attributes: AttributesMap,
) -> Result<ParsedDeclId<StorageDeclaration>, ErrorEmitted> {
    let mut errors = Vec::new();
    let span = item_storage.span();
    let fields: Vec<StorageField> = item_storage
        .fields
        .into_inner()
        .into_iter()
        .map(|storage_field| {
            let attributes = item_attrs_to_map(context, handler, &storage_field.attribute_list)?;
            if !cfg_eval(context, handler, &attributes, context.experimental)? {
                return Ok(None);
            }
            Ok(Some(storage_field_to_storage_field(
                context,
                handler,
                engines,
                storage_field.value,
                attributes,
            )?))
        })
        .filter_map_ok(|field| field)
        .collect::<Result<_, _>>()?;

    // Make sure each storage field is declared once
    let mut names_of_fields = std::collections::HashSet::new();
    fields.iter().for_each(|v| {
        if !names_of_fields.insert(v.name.clone()) {
            errors.push(ConvertParseTreeError::DuplicateStorageField {
                name: v.name.clone(),
                span: v.name.span(),
            });
        }
    });

    if let Some(errors) = emit_all(handler, errors) {
        return Err(errors);
    }

    let storage_declaration = StorageDeclaration {
        attributes,
        span,
        fields,
        storage_keyword: item_storage.storage_token.into(),
    };
    let storage_declaration = engines.pe().insert(storage_declaration);
    Ok(storage_declaration)
}

fn item_configurable_to_constant_declarations(
    context: &mut Context,
    handler: &Handler,
    engines: &Engines,
    item_configurable: ItemConfigurable,
    _attributes: AttributesMap,
) -> Result<Vec<ParsedDeclId<ConstantDeclaration>>, ErrorEmitted> {
    let mut errors = Vec::new();

    if context.module_has_configurable_block() {
        errors.push(ConvertParseTreeError::MultipleConfigurableBlocksInModule {
            span: item_configurable.span(),
        });
    }

    let declarations: Vec<ParsedDeclId<ConstantDeclaration>> = item_configurable
        .fields
        .into_inner()
        .into_iter()
        .map(|configurable_field| {
            let attributes =
                item_attrs_to_map(context, handler, &configurable_field.attribute_list)?;
            if !cfg_eval(context, handler, &attributes, context.experimental)? {
                return Ok(None);
            }
            Ok(Some(configurable_field_to_constant_declaration(
                context,
                handler,
                engines,
                configurable_field.value,
                attributes,
            )?))
        })
        .filter_map_ok(|decl| decl)
        .collect::<Result<_, _>>()?;

    // Make sure each configurable is declared once
    let mut names_of_declarations = std::collections::HashSet::new();
    declarations.iter().for_each(|decl_id| {
        let v = engines.pe().get_constant(decl_id);
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
    attributes: AttributesMap,
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
    attributes: AttributesMap,
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
            .map(|ident| {
                let custom_type = type_engine.insert(
                    engines,
                    TypeInfo::Custom {
                        qualified_call_path: ident.clone().into(),
                        type_arguments: None,
                        root_type_id: None,
                    },
                    ident.span().source_id(),
                );
                TypeParameter {
                    type_id: custom_type,
                    initial_type_id: custom_type,
                    name_ident: ident,
                    trait_constraints: Vec::new(),
                    trait_constraints_span: Span::dummy(),
                    is_from_parent,
                }
            })
            .collect::<Vec<_>>(),
        None => Vec::new(),
    };

    let mut params = generics_to_params(generic_params_opt, false);
    let parent_params = generics_to_params(parent_generic_params_opt, true);

    let mut errors = Vec::new();
    for (ty_name, bounds) in trait_constraints
        .into_iter()
        .chain(parent_trait_constraints)
    {
        let param_to_edit = if let Some(o) = params
            .iter_mut()
            .find(|TypeParameter { name_ident, .. }| name_ident.as_str() == ty_name.as_str())
        {
            o
        } else if let Some(o2) = parent_params
            .iter()
            .find(|TypeParameter { name_ident, .. }| name_ident.as_str() == ty_name.as_str())
        {
            params.push(o2.clone());
            params.last_mut().unwrap()
        } else {
            errors.push(ConvertParseTreeError::ConstrainedNonExistentType {
                ty_name: ty_name.clone(),
                span: ty_name.span().clone(),
            });
            continue;
        };

        param_to_edit.trait_constraints_span = Span::join(ty_name.span(), bounds.span());

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
    attributes: AttributesMap,
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
                (Some(reference), Some(mutable)) => Span::join(reference.span(), mutable.span()),
            };
            let type_id = engines.te().insert(
                engines,
                TypeInfo::new_self_type(self_token.span()),
                self_token.span().source_id(),
            );
            let mut function_parameters = vec![FunctionParameter {
                name: Ident::new(self_token.span()),
                is_reference: ref_self.is_some(),
                is_mutable: mutable_self.is_some(),
                mutability_span,
                type_argument: TypeArgument {
                    type_id,
                    initial_type_id: type_id,
                    span: self_token.span(),
                    call_path_tree: None,
                },
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
        "Self" | "self" => Some(TypeInfo::new_self_type(name.span())),
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
                expr_to_length(context, handler, *ty_array_descriptor.length)?,
            )
        }
        Ty::StringSlice(..) => TypeInfo::StringSlice,
        Ty::StringArray { length, .. } => {
            TypeInfo::StringArray(expr_to_length(context, handler, *length.into_inner())?)
        }
        Ty::Infer { .. } => TypeInfo::Unknown,
        Ty::Ptr { ty, .. } => {
            let type_argument = ty_to_type_argument(context, handler, engines, *ty.into_inner())?;
            TypeInfo::Ptr(type_argument)
        }
        Ty::Slice { ty, .. } => {
            let type_argument = ty_to_type_argument(context, handler, engines, *ty.into_inner())?;
            TypeInfo::Slice(type_argument)
        }
        Ty::Ref { ty, .. } => {
            let type_argument = ty_to_type_argument(context, handler, engines, *ty)?;
            TypeInfo::Ref(type_argument)
        }
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

        let (is_absolute, qualified_path) =
            path_root_opt_to_bool_and_qualified_path_root(context, handler, engines, root_opt)?;
        let call_path = QualifiedCallPath {
            call_path: CallPath {
                prefixes,
                suffix: suffix.name,
                is_absolute,
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
) -> Result<TypeArgument, ErrorEmitted> {
    let type_engine = engines.te();
    let span = ty.span();
    let call_path_tree = ty_to_call_path_tree(context, handler, engines, ty.clone())?;
    let initial_type_id = type_engine.insert(
        engines,
        ty_to_type_info(context, handler, engines, ty.clone())?,
        ty.span().source_id(),
    );

    let type_argument = TypeArgument {
        type_id: initial_type_id,
        initial_type_id,
        call_path_tree,
        span,
    };
    Ok(type_argument)
}

fn fn_signature_to_trait_fn(
    context: &mut Context,
    handler: &Handler,
    engines: &Engines,
    fn_signature: FnSignature,
    attributes: AttributesMap,
) -> Result<TraitFn, ErrorEmitted> {
    let return_type = match &fn_signature.return_type_opt {
        Some((_right_arrow, ty)) => ty_to_type_argument(context, handler, engines, ty.clone())?,
        None => {
            let type_id = engines.te().insert(
                engines,
                TypeInfo::Tuple(Vec::new()),
                fn_signature.span().source_id(),
            );
            TypeArgument {
                type_id,
                initial_type_id: type_id,
                // TODO: Fix as part of https://github.com/FuelLabs/sway/issues/3635
                span: fn_signature.span(),
                call_path_tree: None,
            }
        }
    };

    let trait_fn = TraitFn {
        name: fn_signature.name.clone(),
        span: fn_signature.span(),
        purity: get_attributed_purity(context, handler, &attributes)?,
        attributes,
        parameters: fn_args_to_function_parameters(
            context,
            handler,
            engines,
            fn_signature.arguments.into_inner(),
        )?,
        return_type,
    };
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
    let is_absolute = path_root_opt_to_bool(context, handler, root_opt)?;
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
                is_absolute,
            }
        }
        None => CallPath {
            prefixes: Vec::new(),
            suffix: path_type_segment_to_ident(context, handler, prefix)?,
            is_absolute,
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
        [.., last] => Span::join(method_name.span(), last.span.clone()),
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
    let span = Span::join(func.span(), args.span());

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

    let (is_absolute, qualified_path_root) =
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

    let arguments = args
        .into_inner()
        .into_iter()
        .map(|expr| expr_to_expression(context, handler, engines, expr))
        .collect::<Result<_, _>>()?;

    let name_args_span = |start, end: Option<_>| match end {
        Some(end) => Span::join(start, end),
        None => start,
    };

    let (type_arguments, type_arguments_span) = convert_ty_args(context, call_seg.generics_opt)?;

    // Route intrinsic calls to different AST node.
    match Intrinsic::try_from_str(call_seg.name.as_str()) {
        Some(Intrinsic::Log)
            if context.experimental.new_encoding && last.is_none() && !is_absolute =>
        {
            let span = name_args_span(span, type_arguments_span);
            return Ok(ExpressionKind::IntrinsicFunction(
                IntrinsicFunctionExpression {
                    name: call_seg.name,
                    kind_binding: TypeBinding {
                        inner: Intrinsic::Log,
                        type_arguments: TypeArgs::Regular(vec![]),
                        span: span.clone(),
                    },
                    arguments: vec![Expression {
                        kind: ExpressionKind::FunctionApplication(Box::new(
                            FunctionApplicationExpression {
                                call_path_binding: TypeBinding {
                                    inner: CallPath {
                                        prefixes: vec![],
                                        suffix: Ident::new_no_span("encode".into()),
                                        is_absolute: false,
                                    },
                                    type_arguments: TypeArgs::Regular(type_arguments),
                                    span: span.clone(),
                                },
                                arguments,
                            },
                        )),
                        span: span.clone(),
                    }],
                },
            ));
        }
        Some(intrinsic) if last.is_none() && !is_absolute => {
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
                is_absolute,
            };
            let span = match type_arguments_span {
                Some(span) => Span::join(call_path.span(), span),
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
        is_absolute,
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
                    let array_expression = ArrayExpression {
                        contents,
                        length_span: None,
                    };
                    Expression {
                        kind: ExpressionKind::Array(array_expression),
                        span,
                    }
                }
                ExprArrayDescriptor::Repeat { value, length, .. } => {
                    let expression = expr_to_expression(context, handler, engines, *value)?;
                    let length_span = length.span();
                    let length = expr_to_usize(context, handler, *length)?;
                    let contents = iter::repeat_with(|| expression.clone())
                        .take(length)
                        .collect();
                    let array_expression = ArrayExpression {
                        contents,
                        length_span: Some(length_span),
                    };
                    Expression {
                        kind: ExpressionKind::Array(array_expression),
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
            }),
            span,
        },
        Expr::For {
            value_pattern,
            iterator,
            block,
            ..
        } => {
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
            let value_opt_ident = Ident::new_no_span("__for_value_opt".into());
            let value_opt_expr = Expression {
                kind: ExpressionKind::Variable(value_opt_ident.clone()),
                span: Span::dummy(),
            };

            let iterable_ident = Ident::new_no_span("__for_iterable".into());
            let iterable_expr = Expression {
                kind: ExpressionKind::Variable(iterable_ident.clone()),
                span: Span::dummy(),
            };

            let iterator_expr = expr_to_expression(context, handler, engines, *iterator.clone())?;

            // Declare iterable with iterator return
            let iterable_decl = engines.pe().insert(VariableDeclaration {
                type_ascription: {
                    let type_id = engines.te().insert(engines, TypeInfo::Unknown, None);
                    TypeArgument {
                        type_id,
                        initial_type_id: type_id,
                        span: iterable_ident.clone().span(),
                        call_path_tree: None,
                    }
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
                    let type_id = engines.te().insert(engines, TypeInfo::Unknown, None);
                    TypeArgument {
                        type_id,
                        initial_type_id: type_id,
                        span: value_opt_ident.clone().span(),
                        call_path_tree: None,
                    }
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

            Expression {
                kind: ExpressionKind::ForLoop(ForLoopExpression {
                    desugared: Box::new(desugared),
                }),
                span,
            }
        }
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
                            && path_expr.suffix.is_empty()
                            && path_expr.prefix.generics_opt.is_none()
                            && path_expr.prefix.name.as_str() == "storage" =>
                    {
                        break ExpressionKind::StorageAccess(StorageAccessExpression {
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
        Expr::Ref { expr, .. } => Expression {
            kind: ExpressionKind::Ref(Box::new(expr_to_expression(
                context, handler, engines, *expr,
            )?)),
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
                    op_variant.core_name(),
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
                    Ident::new_with_override("core".into(), op_span.clone()),
                    Ident::new_with_override("ops".into(), op_span.clone()),
                ],
                suffix: Ident::new_with_override(name.into(), op_span.clone()),
                is_absolute: true,
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

fn storage_field_to_storage_field(
    context: &mut Context,
    handler: &Handler,
    engines: &Engines,
    storage_field: sway_ast::StorageField,
    attributes: AttributesMap,
) -> Result<StorageField, ErrorEmitted> {
    let span = storage_field.span();
    let storage_field = StorageField {
        attributes,
        name: storage_field.name,
        type_argument: ty_to_type_argument(context, handler, engines, storage_field.ty)?,
        span,
        initializer: expr_to_expression(context, handler, engines, storage_field.initializer)?,
    };
    Ok(storage_field)
}

fn configurable_field_to_constant_declaration(
    context: &mut Context,
    handler: &Handler,
    engines: &Engines,
    configurable_field: sway_ast::ConfigurableField,
    attributes: AttributesMap,
) -> Result<ParsedDeclId<ConstantDeclaration>, ErrorEmitted> {
    let span = configurable_field.name.span();
    let const_decl = ConstantDeclaration {
        name: configurable_field.name,
        type_ascription: ty_to_type_argument(context, handler, engines, configurable_field.ty)?,
        value: Some(expr_to_expression(
            context,
            handler,
            engines,
            configurable_field.initializer,
        )?),
        visibility: Visibility::Public,
        is_configurable: true,
        attributes,
        span,
    };
    let const_decl = engines.pe().insert(const_decl);
    Ok(const_decl)
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
        Statement::Item(item) => {
            let nodes = item_to_ast_nodes(context, handler, engines, item, false, None)?;
            nodes.iter().try_fold((), |res, node| {
                if ast_node_is_test_fn(engines, node) {
                    let span = node.span.clone();
                    let error = ConvertParseTreeError::TestFnOnlyAllowedAtModuleLevel { span };
                    Err(handler.emit_err(error.into()))
                } else {
                    Ok(res)
                }
            })?;
            nodes
        }
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
        (Some(reference), Some(mutable)) => Span::join(reference.span(), mutable.span()),
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

fn expr_to_length(
    context: &mut Context,
    handler: &Handler,
    expr: Expr,
) -> Result<Length, ErrorEmitted> {
    let span = expr.span();
    Ok(Length::new(expr_to_usize(context, handler, expr)?, span))
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
    let is_absolute = path_root_opt_to_bool(context, handler, root_opt)?;
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
        is_absolute,
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
) -> Result<(Ident, Vec<TypeArgument>), ErrorEmitted> {
    let type_args = match generics_opt {
        Some((_, x)) => generic_args_to_type_arguments(context, handler, engines, x)?,
        None => Default::default(),
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
        // only `foo`, it coult either be a variable or an enum variant

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
                span: Span::join(scrutinee_span, then_block_span),
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
            let type_id = engines.te().insert(engines, TypeInfo::Unknown, None);
            TypeArgument {
                type_id,
                initial_type_id: type_id,
                span: var_decl_name.span(),
                call_path_tree: None,
            }
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
) -> Result<(bool, Option<QualifiedPathRootTypes>), ErrorEmitted> {
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
        )) => (
            false,
            if let Some((_, path_type)) = as_trait {
                Some(QualifiedPathRootTypes {
                    ty: ty_to_type_argument(context, handler, engines, *ty)?,
                    as_trait: engines.te().insert(
                        engines,
                        path_type_to_type_info(context, handler, engines, *path_type.clone())?,
                        path_type.span().source_id(),
                    ),
                    as_trait_span: path_type.span(),
                })
            } else {
                None
            },
        ),
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
                full_span.source_id().cloned(),
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
    let (is_absolute, qualified_path_root) =
        path_root_opt_to_bool_and_qualified_path_root(context, handler, engines, root_opt)?;
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
                is_absolute,
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
    let is_absolute = path_root_opt_to_bool(context, handler, root_opt)?;
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
                is_absolute,
            }
        }
        None => CallPath {
            prefixes: Vec::new(),
            suffix: path_expr_segment_to_ident(context, handler, &prefix)?,
            is_absolute,
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
        Some(asm_final_expr) => {
            let asm_register = AsmRegister {
                name: asm_final_expr.register.as_str().to_owned(),
            };
            let returns = Some((asm_register, asm_final_expr.register.span()));
            let return_type = match asm_final_expr.ty_opt {
                Some((_colon_token, ty)) => ty_to_type_info(context, handler, engines, ty)?,
                None => TypeInfo::UnsignedInteger(IntegerBits::SixtyFour),
            };
            (returns, return_type)
        }
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
            .map(|(instruction, _semicolon_token)| instruction_to_asm_op(instruction))
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
                Pattern::Wildcard { .. } => (None, None, Ident::new_no_span("_".into())),
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
                    let type_id = engines.te().insert(engines, TypeInfo::Unknown, None);
                    TypeArgument {
                        type_id,
                        initial_type_id: type_id,
                        span: name.span(),
                        call_path_tree: None,
                    }
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
                    let type_id = engines.te().insert(engines, TypeInfo::Unknown, None);
                    TypeArgument {
                        type_id,
                        initial_type_id: type_id,
                        span: destructured_struct_name.span(),
                        call_path_tree: None,
                    }
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
            for pattern_struct_field in fields.into_inner().into_iter() {
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

            let tuple_name = Ident::new_with_override(tuple_name, span.clone());

            // Acript a second declaration to a tuple of placeholders to check that the tuple
            // is properly sized to the pattern
            let placeholders_type_ascription = {
                let type_id = engines.te().insert(
                    engines,
                    TypeInfo::Tuple(
                        pat_tuple
                            .clone()
                            .into_inner()
                            .into_iter()
                            .map(|_| {
                                let initial_type_id =
                                    engines.te().insert(engines, TypeInfo::Unknown, None);
                                let dummy_type_param = TypeParameter {
                                    type_id: initial_type_id,
                                    initial_type_id,
                                    name_ident: Ident::new_with_override("_".into(), span.clone()),
                                    trait_constraints: vec![],
                                    trait_constraints_span: Span::dummy(),
                                    is_from_parent: false,
                                };
                                let initial_type_id = engines.te().insert(
                                    engines,
                                    TypeInfo::Placeholder(dummy_type_param),
                                    None,
                                );
                                TypeArgument {
                                    type_id: initial_type_id,
                                    initial_type_id,
                                    call_path_tree: None,
                                    span: Span::dummy(),
                                }
                            })
                            .collect(),
                    ),
                    tuple_name.span().source_id(),
                );
                TypeArgument {
                    type_id,
                    initial_type_id: type_id,
                    span: tuple_name.span(),
                    call_path_tree: None,
                }
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

fn submodule_to_include_statement(dependency: &Submodule) -> IncludeStatement {
    IncludeStatement {
        span: dependency.span(),
        mod_name: dependency.name.clone(),
        visibility: pub_token_opt_to_visibility(dependency.visibility.clone()),
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

fn instruction_to_asm_op(instruction: Instruction) -> AsmOp {
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
            reference, name, ..
        } => {
            if reference.is_some() {
                let error = ConvertParseTreeError::RefPatternsNotSupportedHere { span };
                return Err(handler.emit_err(error.into()));
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

    let name_ident = match ty {
        Ty::Path(path_type) => path_type_to_ident(context, handler, path_type)?,
        Ty::Infer { underscore_token } => {
            let unknown_type = type_engine.insert(engines, TypeInfo::Unknown, None);
            return Ok(TypeParameter {
                type_id: unknown_type,
                initial_type_id: unknown_type,
                name_ident: underscore_token.into(),
                trait_constraints: Default::default(),
                trait_constraints_span: Span::dummy(),
                is_from_parent: false,
            });
        }
        Ty::Tuple(..) => panic!("tuple types are not allowed in this position"),
        Ty::Array(..) => panic!("array types are not allowed in this position"),
        Ty::StringSlice(..) => panic!("str types are not allowed in this position"),
        Ty::StringArray { .. } => panic!("str types are not allowed in this position"),
        Ty::Ptr { .. } => panic!("__ptr types are not allowed in this position"),
        Ty::Slice { .. } => panic!("__slice types are not allowed in this position"),
        Ty::Ref { .. } => panic!("ref types are not allowed in this position"),
    };
    let custom_type = type_engine.insert(
        engines,
        TypeInfo::Custom {
            qualified_call_path: name_ident.clone().into(),
            type_arguments: None,
            root_type_id: None,
        },
        name_ident.span().source_id(),
    );
    Ok(TypeParameter {
        type_id: custom_type,
        initial_type_id: custom_type,
        name_ident,
        trait_constraints: Vec::new(),
        trait_constraints_span: Span::dummy(),
        is_from_parent: false,
    })
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
        Assignable::Var(name) => Expression {
            kind: ExpressionKind::Variable(name),
            span,
        },
        Assignable::Index { target, arg } => Expression {
            kind: ExpressionKind::ArrayIndex(ArrayIndexExpression {
                prefix: Box::new(assignable_to_expression(
                    context, handler, engines, *target,
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
        Assignable::FieldProjection { target, name, .. } => {
            let mut idents = vec![&name];
            let mut base = &*target;
            let (storage_access_field_names_opt, storage_name_opt) = loop {
                match base {
                    Assignable::FieldProjection { target, name, .. } => {
                        idents.push(name);
                        base = target;
                    }
                    Assignable::Var(name) => {
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
                            field_names,
                            storage_keyword_span: storage_name.span(),
                        }),
                        span,
                    }
                }
                _ => Expression {
                    kind: ExpressionKind::Subfield(SubfieldExpression {
                        prefix: Box::new(assignable_to_expression(
                            context, handler, engines, *target,
                        )?),
                        field_to_access: name,
                    }),
                    span,
                },
            }
        }
        Assignable::TupleFieldProjection {
            target,
            field,
            field_span,
            ..
        } => {
            let index = match usize::try_from(field) {
                Ok(index) => index,
                Err(..) => {
                    let error = ConvertParseTreeError::TupleIndexOutOfRange { span: field_span };
                    return Err(handler.emit_err(error.into()));
                }
            };
            Expression {
                kind: ExpressionKind::TupleIndex(TupleIndexExpression {
                    prefix: Box::new(assignable_to_expression(
                        context, handler, engines, *target,
                    )?),
                    index,
                    index_span: field_span,
                }),
                span,
            }
        }
    };
    Ok(expression)
}

fn assignable_to_reassignment_target(
    context: &mut Context,
    handler: &Handler,
    engines: &Engines,
    assignable: Assignable,
) -> Result<ReassignmentTarget, ErrorEmitted> {
    let mut idents = Vec::new();
    let mut base = &assignable;

    loop {
        match base {
            Assignable::FieldProjection { target, name, .. } => {
                idents.push(name);
                base = target;
            }
            Assignable::Var(_) => break,
            Assignable::Index { .. } => break,
            Assignable::TupleFieldProjection { .. } => break,
        }
    }
    let expression = assignable_to_expression(context, handler, engines, assignable)?;
    Ok(ReassignmentTarget::VariableExpression(Box::new(expression)))
}

fn generic_args_to_type_arguments(
    context: &mut Context,
    handler: &Handler,
    engines: &Engines,
    generic_args: GenericArgs,
) -> Result<Vec<TypeArgument>, ErrorEmitted> {
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
) -> Result<Vec<TypeArgument>, ErrorEmitted> {
    let type_arguments = match ty_tuple_descriptor {
        TyTupleDescriptor::Nil => vec![],
        TyTupleDescriptor::Cons { head, tail, .. } => {
            let mut type_arguments = vec![ty_to_type_argument(context, handler, engines, *head)?];
            for ty in tail.into_iter() {
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
                let (mut call_path, type_arguments) = path_type_to_call_path_and_type_arguments(
                    context, handler, engines, path_type,
                )?;

                let mut root_type_id = None;
                if name.as_str() == "Self" {
                    call_path.call_path.prefixes.remove(0);
                    root_type_id = Some(engines.te().insert(
                        engines,
                        type_info,
                        name.span().source_id(),
                    ));
                }
                TypeInfo::Custom {
                    qualified_call_path: call_path,
                    type_arguments: Some(type_arguments),
                    root_type_id,
                }
            } else {
                type_info
            }
        }
        None => {
            if name.as_str() == "ContractCaller" {
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
                        let call_path = path_type_to_call_path(context, handler, path_type)?;
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
                    root_type_id: None,
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

fn item_attrs_to_map(
    _context: &mut Context,
    handler: &Handler,
    attribute_list: &[AttributeDecl],
) -> Result<AttributesMap, ErrorEmitted> {
    let mut attrs_map: IndexMap<_, Vec<Attribute>> = IndexMap::new();

    for attr_decl in attribute_list {
        let attrs = attr_decl.attribute.get().into_iter();
        for attr in attrs {
            let name = attr.name.as_str();
            if !VALID_ATTRIBUTE_NAMES.contains(&name) {
                handler.emit_warn(CompileWarning {
                    span: attr_decl.span().clone(),
                    warning_content: Warning::UnrecognizedAttribute {
                        attrib_name: attr.name.clone(),
                    },
                })
            }

            let args = attr
                .args
                .as_ref()
                .map(|parens| {
                    parens
                        .get()
                        .into_iter()
                        .cloned()
                        .map(|arg| AttributeArg {
                            name: arg.name.clone(),
                            value: arg.value.clone(),
                            span: arg.span(),
                        })
                        .collect()
                })
                .unwrap_or_else(Vec::new);

            let attribute = Attribute {
                name: attr.name.clone(),
                args,
                span: attr_decl.span(),
            };

            if let Some(attr_kind) = match name {
                DOC_ATTRIBUTE_NAME => Some(AttributeKind::Doc),
                DOC_COMMENT_ATTRIBUTE_NAME => Some(AttributeKind::DocComment),
                STORAGE_PURITY_ATTRIBUTE_NAME => Some(AttributeKind::Storage),
                INLINE_ATTRIBUTE_NAME => Some(AttributeKind::Inline),
                TEST_ATTRIBUTE_NAME => Some(AttributeKind::Test),
                PAYABLE_ATTRIBUTE_NAME => Some(AttributeKind::Payable),
                ALLOW_ATTRIBUTE_NAME => Some(AttributeKind::Allow),
                CFG_ATTRIBUTE_NAME => Some(AttributeKind::Cfg),
                DEPRECATED_ATTRIBUTE_NAME => Some(AttributeKind::Deprecated),
                _ => None,
            } {
                match attrs_map.get_mut(&attr_kind) {
                    Some(old_args) => {
                        old_args.push(attribute);
                    }
                    None => {
                        attrs_map.insert(attr_kind, vec![attribute]);
                    }
                }
            }
        }
    }

    // Check attribute arguments
    for (attribute_kind, attributes) in &attrs_map {
        for attribute in attributes {
            // check attribute arguments length
            let (expected_min_len, expected_max_len) =
                attribute_kind.clone().expected_args_len_min_max();
            if attribute.args.len() < expected_min_len
                || attribute.args.len() > expected_max_len.unwrap_or(usize::MAX)
            {
                handler.emit_warn(CompileWarning {
                    span: attribute.name.span().clone(),
                    warning_content: Warning::AttributeExpectedNumberOfArguments {
                        attrib_name: attribute.name.clone(),
                        received_args: attribute.args.len(),
                        expected_min_len,
                        expected_max_len,
                    },
                })
            }

            // check attribute argument value
            for (index, arg) in attribute.args.iter().enumerate() {
                let possible_values = attribute_kind.clone().expected_args_values(index);
                if let Some(possible_values) = possible_values {
                    if !possible_values.iter().any(|v| v == arg.name.as_str()) {
                        handler.emit_warn(CompileWarning {
                            span: attribute.name.span().clone(),
                            warning_content: Warning::UnexpectedAttributeArgumentValue {
                                attrib_name: attribute.name.clone(),
                                received_value: arg.name.as_str().to_string(),
                                expected_values: possible_values,
                            },
                        })
                    }
                }
            }
        }
    }

    Ok(AttributesMap::new(Arc::new(attrs_map)))
}

fn error_if_self_param_is_not_allowed(
    _context: &mut Context,
    handler: &Handler,
    engines: &Engines,
    parameters: &[FunctionParameter],
    fn_kind: &str,
) -> Result<(), ErrorEmitted> {
    for param in parameters {
        if engines.te().get(param.type_argument.type_id).is_self_type() {
            let error = ConvertParseTreeError::SelfParameterNotAllowedForFn {
                fn_kind: fn_kind.to_owned(),
                span: param.type_argument.span.clone(),
            };
            return Err(handler.emit_err(error.into()));
        }
    }
    Ok(())
}

/// Walks all the cfg attributes in a map, evaluating them
/// and returning false if any evaluated to false.
pub fn cfg_eval(
    context: &Context,
    handler: &Handler,
    attrs_map: &AttributesMap,
    experimental: ExperimentalFlags,
) -> Result<bool, ErrorEmitted> {
    if let Some(cfg_attrs) = attrs_map.get(&AttributeKind::Cfg) {
        for cfg_attr in cfg_attrs {
            for arg in &cfg_attr.args {
                match arg.name.as_str() {
                    CFG_TARGET_ARG_NAME => {
                        if let Some(value) = &arg.value {
                            if let sway_ast::Literal::String(value_str) = value {
                                if let Ok(target) = BuildTarget::from_str(value_str.parsed.as_str())
                                {
                                    if target != context.build_target() {
                                        return Ok(false);
                                    }
                                } else {
                                    let error = ConvertParseTreeError::InvalidCfgTargetArgValue {
                                        span: value.span(),
                                        value: value.span().str(),
                                    };
                                    return Err(handler.emit_err(error.into()));
                                }
                            } else {
                                let error = ConvertParseTreeError::InvalidCfgTargetArgValue {
                                    span: value.span(),
                                    value: value.span().str(),
                                };
                                return Err(handler.emit_err(error.into()));
                            }
                        } else {
                            let error = ConvertParseTreeError::ExpectedCfgTargetArgValue {
                                span: arg.span(),
                            };
                            return Err(handler.emit_err(error.into()));
                        }
                    }
                    CFG_PROGRAM_TYPE_ARG_NAME => {
                        if let Some(value) = &arg.value {
                            if let sway_ast::Literal::String(value_str) = value {
                                if let Ok(program_type) =
                                    TreeType::from_str(value_str.parsed.as_str())
                                {
                                    if program_type != context.program_type().unwrap() {
                                        return Ok(false);
                                    }
                                } else {
                                    let error =
                                        ConvertParseTreeError::InvalidCfgProgramTypeArgValue {
                                            span: value.span(),
                                            value: value.span().str(),
                                        };
                                    return Err(handler.emit_err(error.into()));
                                }
                            } else {
                                let error = ConvertParseTreeError::InvalidCfgProgramTypeArgValue {
                                    span: value.span(),
                                    value: value.span().str(),
                                };
                                return Err(handler.emit_err(error.into()));
                            }
                        } else {
                            let error = ConvertParseTreeError::ExpectedCfgTargetArgValue {
                                span: arg.span(),
                            };
                            return Err(handler.emit_err(error.into()));
                        }
                    }
                    CFG_EXPERIMENTAL_NEW_ENCODING => match &arg.value {
                        Some(sway_ast::Literal::Bool(v)) => {
                            let is_true = matches!(v.kind, sway_ast::literal::LitBoolType::True);
                            return Ok(experimental.new_encoding == is_true);
                        }
                        _ => {
                            let error =
                                ConvertParseTreeError::ExpectedExperimentalNewEncodingArgValue {
                                    span: arg.span(),
                                };
                            return Err(handler.emit_err(error.into()));
                        }
                    },
                    _ => {
                        // Already checked with `AttributeKind::expected_args_*`
                        unreachable!("cfg attribute should only have the `target` or the `program_type` argument");
                    }
                }
            }
        }
    }
    Ok(true)
}
