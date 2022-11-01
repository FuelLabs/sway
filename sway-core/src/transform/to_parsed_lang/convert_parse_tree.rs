use crate::{
    language::{parsed::*, *},
    transform::attribute::*,
    type_system::*,
};

use sway_ast::{
    expr::{ReassignmentOp, ReassignmentOpVariant},
    ty::TyTupleDescriptor,
    AbiCastArgs, AngleBrackets, AsmBlock, Assignable, AttributeDecl, Braces, CodeBlockContents,
    CommaToken, Dependency, DoubleColonToken, Expr, ExprArrayDescriptor, ExprStructField,
    ExprTupleDescriptor, FnArg, FnArgs, FnSignature, GenericArgs, GenericParams, IfCondition,
    IfExpr, Instruction, Intrinsic, Item, ItemAbi, ItemConst, ItemEnum, ItemFn, ItemImpl, ItemKind,
    ItemStorage, ItemStruct, ItemTrait, ItemUse, LitInt, LitIntType, MatchBranchKind, Module,
    ModuleKind, Parens, PathExpr, PathExprSegment, PathType, PathTypeSegment, Pattern,
    PatternStructField, PubToken, Punctuated, QualifiedPathRoot, Statement, StatementLet, Traits,
    Ty, TypeField, UseTree, WhereClause,
};
use sway_error::convert_parse_tree_error::ConvertParseTreeError;
use sway_error::handler::{ErrorEmitted, Handler};
use sway_error::warning::{CompileWarning, Warning};
use sway_types::{
    constants::{
        DESTRUCTURE_PREFIX, DOC_ATTRIBUTE_NAME, INLINE_ATTRIBUTE_NAME,
        MATCH_RETURN_VAR_NAME_PREFIX, STORAGE_PURITY_ATTRIBUTE_NAME, STORAGE_PURITY_READ_NAME,
        STORAGE_PURITY_WRITE_NAME, TEST_ATTRIBUTE_NAME, TUPLE_NAME_PREFIX, VALID_ATTRIBUTE_NAMES,
    },
    integer_bits::IntegerBits,
};
use sway_types::{Ident, Span, Spanned};

use std::{
    collections::{HashMap, HashSet},
    convert::TryFrom,
    iter,
    mem::MaybeUninit,
    ops::ControlFlow,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

pub fn convert_parse_tree(
    handler: &Handler,
    module: Module,
    include_test_fns: bool,
) -> Result<(TreeType, ParseTree), ErrorEmitted> {
    let tree_type = match module.kind {
        ModuleKind::Script { .. } => TreeType::Script,
        ModuleKind::Contract { .. } => TreeType::Contract,
        ModuleKind::Predicate { .. } => TreeType::Predicate,
        ModuleKind::Library { ref name, .. } => TreeType::Library { name: name.clone() },
    };
    let tree = module_to_sway_parse_tree(handler, module, include_test_fns)?;
    Ok((tree_type, tree))
}

pub fn module_to_sway_parse_tree(
    handler: &Handler,
    module: Module,
    include_test_fns: bool,
) -> Result<ParseTree, ErrorEmitted> {
    let span = module.span();
    let root_nodes = {
        let mut root_nodes: Vec<AstNode> = {
            module
                .dependencies
                .iter()
                .map(|dependency| {
                    let span = dependency.span();
                    let incl_stmt = dependency_to_include_statement(dependency);
                    let content = AstNodeContent::IncludeStatement(incl_stmt);
                    AstNode { content, span }
                })
                .collect()
        };
        for item in module.items {
            let mut ast_nodes = item_to_ast_nodes(handler, item)?;
            if !include_test_fns {
                ast_nodes.retain(|node| !ast_node_is_test_fn(node));
            }
            root_nodes.extend(ast_nodes);
        }
        root_nodes
    };
    Ok(ParseTree { span, root_nodes })
}

fn ast_node_is_test_fn(node: &AstNode) -> bool {
    if let AstNodeContent::Declaration(Declaration::FunctionDeclaration(ref decl)) = node.content {
        if decl.attributes.contains_key(&AttributeKind::Test) {
            return true;
        }
    }
    false
}

fn item_to_ast_nodes(handler: &Handler, item: Item) -> Result<Vec<AstNode>, ErrorEmitted> {
    let attributes = item_attrs_to_map(handler, &item.attribute_list)?;

    let decl = |d| vec![AstNodeContent::Declaration(d)];

    let span = item.span();
    let contents = match item.value {
        ItemKind::Use(item_use) => item_use_to_use_statements(handler, item_use)?
            .into_iter()
            .map(AstNodeContent::UseStatement)
            .collect(),
        ItemKind::Struct(item_struct) => decl(Declaration::StructDeclaration(
            item_struct_to_struct_declaration(handler, item_struct, attributes)?,
        )),
        ItemKind::Enum(item_enum) => decl(Declaration::EnumDeclaration(
            item_enum_to_enum_declaration(handler, item_enum, attributes)?,
        )),
        ItemKind::Fn(item_fn) => {
            let function_declaration =
                item_fn_to_function_declaration(handler, item_fn, attributes)?;
            for param in &function_declaration.parameters {
                if matches!(param.type_info, TypeInfo::SelfType) {
                    let error = ConvertParseTreeError::SelfParameterNotAllowedForFreeFn {
                        span: param.type_span.clone(),
                    };
                    return Err(handler.emit_err(error.into()));
                }
            }
            decl(Declaration::FunctionDeclaration(function_declaration))
        }
        ItemKind::Trait(item_trait) => decl(Declaration::TraitDeclaration(
            item_trait_to_trait_declaration(handler, item_trait, attributes)?,
        )),
        ItemKind::Impl(item_impl) => decl(item_impl_to_declaration(handler, item_impl)?),
        ItemKind::Abi(item_abi) => decl(Declaration::AbiDeclaration(item_abi_to_abi_declaration(
            handler, item_abi, attributes,
        )?)),
        ItemKind::Const(item_const) => decl(Declaration::ConstantDeclaration(
            item_const_to_constant_declaration(handler, item_const, attributes)?,
        )),
        ItemKind::Storage(item_storage) => decl(Declaration::StorageDeclaration(
            item_storage_to_storage_declaration(handler, item_storage, attributes)?,
        )),
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
    use_tree_to_use_statements(
        item_use.tree,
        item_use.root_import.is_some(),
        &mut prefix,
        &mut ret,
    );
    debug_assert!(prefix.is_empty());
    Ok(ret)
}

fn use_tree_to_use_statements(
    use_tree: UseTree,
    is_absolute: bool,
    path: &mut Vec<Ident>,
    ret: &mut Vec<UseStatement>,
) {
    match use_tree {
        UseTree::Group { imports } => {
            for use_tree in imports.into_inner() {
                use_tree_to_use_statements(use_tree, is_absolute, path, ret);
            }
        }
        UseTree::Name { name } => {
            let import_type = if name.as_str() == "self" {
                ImportType::SelfImport
            } else {
                ImportType::Item(name)
            };
            ret.push(UseStatement {
                call_path: path.clone(),
                import_type,
                is_absolute,
                alias: None,
            });
        }
        UseTree::Rename { name, alias, .. } => {
            let import_type = if name.as_str() == "self" {
                ImportType::SelfImport
            } else {
                ImportType::Item(name)
            };
            ret.push(UseStatement {
                call_path: path.clone(),
                import_type,
                is_absolute,
                alias: Some(alias),
            });
        }
        UseTree::Glob { .. } => {
            ret.push(UseStatement {
                call_path: path.clone(),
                import_type: ImportType::Star,
                is_absolute,
                alias: None,
            });
        }
        UseTree::Path { prefix, suffix, .. } => {
            path.push(prefix);
            use_tree_to_use_statements(*suffix, is_absolute, path, ret);
            path.pop().unwrap();
        }
    }
}

fn emit_all(handler: &Handler, errors: Vec<ConvertParseTreeError>) -> Option<ErrorEmitted> {
    errors
        .into_iter()
        .fold(None, |_, error| Some(handler.emit_err(error.into())))
}

fn item_struct_to_struct_declaration(
    handler: &Handler,
    item_struct: ItemStruct,
    attributes: AttributesMap,
) -> Result<StructDeclaration, ErrorEmitted> {
    // FIXME(Centril): We shoudln't be collecting into a temporary  `errors` here. Recover instead!
    let mut errors = Vec::new();
    let span = item_struct.span();
    let fields = item_struct
        .fields
        .into_inner()
        .into_iter()
        .map(|type_field| {
            let attributes = item_attrs_to_map(handler, &type_field.attribute_list)?;
            type_field_to_struct_field(handler, type_field.value, attributes)
        })
        .collect::<Result<Vec<_>, _>>()?;

    if fields.iter().any(
        |field| matches!(&field.type_info, TypeInfo::Custom { name, ..} if name == &item_struct.name),
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

    let struct_declaration = StructDeclaration {
        name: item_struct.name,
        attributes,
        fields,
        type_parameters: generic_params_opt_to_type_parameters(
            handler,
            item_struct.generics,
            item_struct.where_clause_opt,
        )?,
        visibility: pub_token_opt_to_visibility(item_struct.visibility),
        span,
    };
    Ok(struct_declaration)
}

fn item_enum_to_enum_declaration(
    handler: &Handler,
    item_enum: ItemEnum,
    attributes: AttributesMap,
) -> Result<EnumDeclaration, ErrorEmitted> {
    let mut errors = Vec::new();
    let span = item_enum.span();
    let variants = item_enum
        .fields
        .into_inner()
        .into_iter()
        .enumerate()
        .map(|(tag, type_field)| {
            let attributes = item_attrs_to_map(handler, &type_field.attribute_list)?;
            type_field_to_enum_variant(handler, type_field.value, attributes, tag)
        })
        .collect::<Result<Vec<_>, _>>()?;

    if variants.iter().any(|variant| {
       matches!(&variant.type_info, TypeInfo::Custom { name, ..} if name == &item_enum.name)
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

    let enum_declaration = EnumDeclaration {
        name: item_enum.name,
        type_parameters: generic_params_opt_to_type_parameters(
            handler,
            item_enum.generics,
            item_enum.where_clause_opt,
        )?,
        variants,
        span,
        visibility: pub_token_opt_to_visibility(item_enum.visibility),
        attributes,
    };
    Ok(enum_declaration)
}

fn item_fn_to_function_declaration(
    handler: &Handler,
    item_fn: ItemFn,
    attributes: AttributesMap,
) -> Result<FunctionDeclaration, ErrorEmitted> {
    let span = item_fn.span();
    let return_type_span = match &item_fn.fn_signature.return_type_opt {
        Some((_right_arrow_token, ty)) => ty.span(),
        None => item_fn.fn_signature.span(),
    };
    Ok(FunctionDeclaration {
        purity: get_attributed_purity(handler, &attributes)?,
        attributes,
        name: item_fn.fn_signature.name,
        visibility: pub_token_opt_to_visibility(item_fn.fn_signature.visibility),
        body: braced_code_block_contents_to_code_block(handler, item_fn.body)?,
        parameters: fn_args_to_function_parameters(
            handler,
            item_fn.fn_signature.arguments.into_inner(),
        )?,
        span,
        return_type: match item_fn.fn_signature.return_type_opt {
            Some((_right_arrow, ty)) => ty_to_type_info(handler, ty)?,
            None => TypeInfo::Tuple(Vec::new()),
        },
        type_parameters: generic_params_opt_to_type_parameters(
            handler,
            item_fn.fn_signature.generics,
            item_fn.fn_signature.where_clause_opt,
        )?,
        return_type_span,
    })
}

fn get_attributed_purity(
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
                match arg.as_str() {
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

fn item_trait_to_trait_declaration(
    handler: &Handler,
    item_trait: ItemTrait,
    attributes: AttributesMap,
) -> Result<TraitDeclaration, ErrorEmitted> {
    let span = item_trait.span();
    let type_parameters = generic_params_opt_to_type_parameters(
        handler,
        item_trait.generics,
        item_trait.where_clause_opt,
    )?;
    let interface_surface = {
        item_trait
            .trait_items
            .into_inner()
            .into_iter()
            .map(|(fn_signature, _)| {
                let attributes = item_attrs_to_map(handler, &fn_signature.attribute_list)?;
                fn_signature_to_trait_fn(handler, fn_signature.value, attributes)
            })
            .collect::<Result<_, _>>()?
    };
    let methods = match item_trait.trait_defs_opt {
        None => Vec::new(),
        Some(trait_defs) => trait_defs
            .into_inner()
            .into_iter()
            .map(|item_fn| {
                let attributes = item_attrs_to_map(handler, &item_fn.attribute_list)?;
                item_fn_to_function_declaration(handler, item_fn.value, attributes)
            })
            .collect::<Result<_, _>>()?,
    };
    let supertraits = match item_trait.super_traits {
        None => Vec::new(),
        Some((_colon_token, traits)) => traits_to_supertraits(handler, traits)?,
    };
    let visibility = pub_token_opt_to_visibility(item_trait.visibility);
    Ok(TraitDeclaration {
        name: item_trait.name,
        type_parameters,
        interface_surface,
        methods,
        supertraits,
        visibility,
        attributes,
        span,
    })
}

fn item_impl_to_declaration(
    handler: &Handler,
    item_impl: ItemImpl,
) -> Result<Declaration, ErrorEmitted> {
    let block_span = item_impl.span();
    let type_implementing_for_span = item_impl.ty.span();
    let type_implementing_for = ty_to_type_info(handler, item_impl.ty)?;
    let functions = item_impl
        .contents
        .into_inner()
        .into_iter()
        .map(|item| {
            let attributes = item_attrs_to_map(handler, &item.attribute_list)?;
            item_fn_to_function_declaration(handler, item.value, attributes)
        })
        .collect::<Result<_, _>>()?;

    let impl_type_parameters = generic_params_opt_to_type_parameters(
        handler,
        item_impl.generic_params_opt,
        item_impl.where_clause_opt,
    )?;

    match item_impl.trait_opt {
        Some((path_type, _)) => {
            let (trait_name, trait_type_arguments) =
                path_type_to_call_path_and_type_arguments(handler, path_type)?;
            let impl_trait = ImplTrait {
                impl_type_parameters,
                trait_name,
                trait_type_arguments,
                type_implementing_for,
                type_implementing_for_span,
                functions,
                block_span,
            };
            Ok(Declaration::ImplTrait(impl_trait))
        }
        None => match type_implementing_for {
            TypeInfo::Contract => Err(handler
                .emit_err(ConvertParseTreeError::SelfImplForContract { span: block_span }.into())),
            _ => {
                let impl_self = ImplSelf {
                    type_implementing_for,
                    type_implementing_for_span,
                    impl_type_parameters,
                    functions,
                    block_span,
                };
                Ok(Declaration::ImplSelf(impl_self))
            }
        },
    }
}

fn path_type_to_call_path_and_type_arguments(
    handler: &Handler,
    PathType {
        root_opt,
        prefix,
        mut suffix,
    }: PathType,
) -> Result<(CallPath, Vec<TypeArgument>), ErrorEmitted> {
    let (prefixes, suffix) = match suffix.pop() {
        None => (Vec::new(), prefix),
        Some((_, last)) => {
            // Gather the idents of the prefix, i.e. all segments but the last one.
            let mut before = Vec::with_capacity(suffix.len() + 1);
            before.push(path_type_segment_to_ident(handler, prefix)?);
            for (_, seg) in suffix {
                before.push(path_type_segment_to_ident(handler, seg)?);
            }
            (before, last)
        }
    };

    let call_path = CallPath {
        prefixes,
        suffix: suffix.name,
        is_absolute: path_root_opt_to_bool(handler, root_opt)?,
    };

    let ty_args = match suffix.generics_opt {
        Some((_, generic_args)) => generic_args_to_type_arguments(handler, generic_args)?,
        None => vec![],
    };

    Ok((call_path, ty_args))
}

fn item_abi_to_abi_declaration(
    handler: &Handler,
    item_abi: ItemAbi,
    attributes: AttributesMap,
) -> Result<AbiDeclaration, ErrorEmitted> {
    let span = item_abi.span();
    Ok(AbiDeclaration {
        name: item_abi.name,
        interface_surface: {
            item_abi
                .abi_items
                .into_inner()
                .into_iter()
                .map(|(fn_signature, _semicolon_token)| {
                    let attributes = item_attrs_to_map(handler, &fn_signature.attribute_list)?;
                    fn_signature_to_trait_fn(handler, fn_signature.value, attributes)
                })
                .collect::<Result<_, _>>()?
        },
        methods: match item_abi.abi_defs_opt {
            None => Vec::new(),
            Some(abi_defs) => abi_defs
                .into_inner()
                .into_iter()
                .map(|item_fn| {
                    let attributes = item_attrs_to_map(handler, &item_fn.attribute_list)?;
                    item_fn_to_function_declaration(handler, item_fn.value, attributes)
                })
                .collect::<Result<_, _>>()?,
        },
        span,
        attributes,
    })
}

pub(crate) fn item_const_to_constant_declaration(
    handler: &Handler,
    item_const: ItemConst,
    attributes: AttributesMap,
) -> Result<ConstantDeclaration, ErrorEmitted> {
    let span = item_const.span();
    let (type_ascription, type_ascription_span) = match item_const.ty_opt {
        Some((_colon_token, ty)) => {
            let type_ascription = ty_to_type_info(handler, ty.clone())?;
            let type_ascription_span = if let Ty::Path(path_type) = &ty {
                path_type.prefix.name.span()
            } else {
                ty.span()
            };
            (type_ascription, Some(type_ascription_span))
        }
        None => (TypeInfo::Unknown, None),
    };

    Ok(ConstantDeclaration {
        name: item_const.name,
        type_ascription,
        type_ascription_span,
        value: expr_to_expression(handler, item_const.expr)?,
        visibility: pub_token_opt_to_visibility(item_const.visibility),
        attributes,
        span,
    })
}

fn item_storage_to_storage_declaration(
    handler: &Handler,
    item_storage: ItemStorage,
    attributes: AttributesMap,
) -> Result<StorageDeclaration, ErrorEmitted> {
    let mut errors = Vec::new();
    let span = item_storage.span();
    let fields: Vec<StorageField> = item_storage
        .fields
        .into_inner()
        .into_iter()
        .map(|storage_field| {
            let attributes = item_attrs_to_map(handler, &storage_field.attribute_list)?;
            storage_field_to_storage_field(handler, storage_field.value, attributes)
        })
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
    };
    Ok(storage_declaration)
}

fn type_field_to_struct_field(
    handler: &Handler,
    type_field: TypeField,
    attributes: AttributesMap,
) -> Result<StructField, ErrorEmitted> {
    let span = type_field.span();
    let type_span = type_field.ty.span();
    let struct_field = StructField {
        name: type_field.name,
        attributes,
        type_info: ty_to_type_info(handler, type_field.ty)?,
        span,
        type_span,
    };
    Ok(struct_field)
}

fn generic_params_opt_to_type_parameters(
    handler: &Handler,
    generic_params_opt: Option<GenericParams>,
    where_clause_opt: Option<WhereClause>,
) -> Result<Vec<TypeParameter>, ErrorEmitted> {
    let trait_constraints = match where_clause_opt {
        Some(where_clause) => where_clause
            .bounds
            .into_iter()
            .map(|where_bound| (where_bound.ty_name, where_bound.bounds))
            .collect::<Vec<_>>(),
        None => Vec::new(),
    };

    let mut params = match generic_params_opt {
        Some(generic_params) => generic_params
            .parameters
            .into_inner()
            .into_iter()
            .map(|ident| {
                let custom_type = insert_type(TypeInfo::Custom {
                    name: ident.clone(),
                    type_arguments: None,
                });
                TypeParameter {
                    type_id: custom_type,
                    initial_type_id: custom_type,
                    name_ident: ident,
                    trait_constraints: Vec::new(),
                    trait_constraints_span: Span::dummy(),
                }
            })
            .collect::<Vec<_>>(),
        None => Vec::new(),
    };

    let mut errors = Vec::new();
    for (ty_name, bounds) in trait_constraints.into_iter() {
        let param_to_edit = match params
            .iter_mut()
            .find(|TypeParameter { name_ident, .. }| name_ident.as_str() == ty_name.as_str())
        {
            Some(o) => o,
            None => {
                errors.push(ConvertParseTreeError::ConstrainedNonExistentType {
                    ty_name: ty_name.clone(),
                    span: ty_name.span().clone(),
                });
                continue;
            }
        };

        param_to_edit.trait_constraints_span = Span::join(ty_name.span(), bounds.span());

        param_to_edit.trait_constraints.extend(
            traits_to_call_paths(handler, bounds)?.into_iter().map(
                |(trait_name, type_arguments)| TraitConstraint {
                    trait_name,
                    type_arguments,
                },
            ),
        );
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
    handler: &Handler,
    type_field: TypeField,
    attributes: AttributesMap,
    tag: usize,
) -> Result<EnumVariant, ErrorEmitted> {
    let span = type_field.span();
    let type_span = if let Ty::Path(path_type) = &type_field.ty {
        path_type.prefix.name.span()
    } else {
        span.clone()
    };

    let enum_variant = EnumVariant {
        name: type_field.name,
        attributes,
        type_info: ty_to_type_info(handler, type_field.ty)?,
        type_span,
        tag,
        span,
    };
    Ok(enum_variant)
}

fn braced_code_block_contents_to_code_block(
    handler: &Handler,
    braced_code_block_contents: Braces<CodeBlockContents>,
) -> Result<CodeBlock, ErrorEmitted> {
    let whole_block_span = braced_code_block_contents.span();
    let code_block_contents = braced_code_block_contents.into_inner();
    let contents = {
        let mut contents = Vec::new();
        for statement in code_block_contents.statements {
            let ast_nodes = statement_to_ast_nodes(handler, statement)?;
            contents.extend(ast_nodes);
        }
        if let Some(expr) = code_block_contents.final_expr_opt {
            let final_ast_node = expr_to_ast_node(handler, *expr, false)?;
            contents.push(final_ast_node);
        }
        contents
    };
    Ok(CodeBlock {
        contents,
        whole_block_span,
    })
}

fn fn_args_to_function_parameters(
    handler: &Handler,
    fn_args: FnArgs,
) -> Result<Vec<FunctionParameter>, ErrorEmitted> {
    let function_parameters = match fn_args {
        FnArgs::Static(args) => args
            .into_iter()
            .map(|fn_arg| fn_arg_to_function_parameter(handler, fn_arg))
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
            let mut function_parameters = vec![FunctionParameter {
                name: Ident::new(self_token.span()),
                is_reference: ref_self.is_some(),
                is_mutable: mutable_self.is_some(),
                mutability_span,
                type_info: TypeInfo::SelfType,
                type_span: self_token.span(),
            }];
            if let Some((_comma_token, args)) = args_opt {
                for arg in args {
                    let function_parameter = fn_arg_to_function_parameter(handler, arg)?;
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
        "bool" => Some(TypeInfo::Boolean),
        "unit" => Some(TypeInfo::Tuple(Vec::new())),
        "b256" => Some(TypeInfo::B256),
        "raw_ptr" => Some(TypeInfo::RawUntypedPtr),
        "Self" | "self" => Some(TypeInfo::SelfType),
        "Contract" => Some(TypeInfo::Contract),
        _other => None,
    }
}

fn ty_to_type_info(handler: &Handler, ty: Ty) -> Result<TypeInfo, ErrorEmitted> {
    let type_info = match ty {
        Ty::Path(path_type) => path_type_to_type_info(handler, path_type)?,
        Ty::Tuple(parenthesized_ty_tuple_descriptor) => {
            TypeInfo::Tuple(ty_tuple_descriptor_to_type_arguments(
                handler,
                parenthesized_ty_tuple_descriptor.into_inner(),
            )?)
        }
        Ty::Array(bracketed_ty_array_descriptor) => {
            let ty_array_descriptor = bracketed_ty_array_descriptor.into_inner();
            let initial_elem_ty = insert_type(ty_to_type_info(handler, *ty_array_descriptor.ty)?);
            TypeInfo::Array(
                initial_elem_ty,
                expr_to_usize(handler, *ty_array_descriptor.length)?,
                initial_elem_ty,
            )
        }
        Ty::Str { length, .. } => TypeInfo::Str(expr_to_u64(handler, *length.into_inner())?),
        Ty::Infer { .. } => TypeInfo::Unknown,
    };
    Ok(type_info)
}

fn ty_to_type_argument(handler: &Handler, ty: Ty) -> Result<TypeArgument, ErrorEmitted> {
    let span = ty.span();
    let initial_type_id = insert_type(ty_to_type_info(handler, ty)?);
    let type_argument = TypeArgument {
        type_id: initial_type_id,
        initial_type_id,
        span,
    };
    Ok(type_argument)
}

fn fn_signature_to_trait_fn(
    handler: &Handler,
    fn_signature: FnSignature,
    attributes: AttributesMap,
) -> Result<TraitFn, ErrorEmitted> {
    let return_type_span = match &fn_signature.return_type_opt {
        Some((_right_arrow_token, ty)) => ty.span(),
        None => fn_signature.span(),
    };
    let trait_fn = TraitFn {
        name: fn_signature.name,
        purity: get_attributed_purity(handler, &attributes)?,
        attributes,
        parameters: fn_args_to_function_parameters(handler, fn_signature.arguments.into_inner())?,
        return_type: match fn_signature.return_type_opt {
            Some((_right_arrow_token, ty)) => ty_to_type_info(handler, ty)?,
            None => TypeInfo::Tuple(Vec::new()),
        },
        return_type_span,
    };
    Ok(trait_fn)
}

fn traits_to_call_paths(
    handler: &Handler,
    traits: Traits,
) -> Result<Vec<(CallPath, Vec<TypeArgument>)>, ErrorEmitted> {
    let mut parsed_traits = vec![path_type_to_call_path_and_type_arguments(
        handler,
        traits.prefix,
    )?];
    for (_add_token, suffix) in traits.suffixes {
        let supertrait = path_type_to_call_path_and_type_arguments(handler, suffix)?;
        parsed_traits.push(supertrait);
    }
    Ok(parsed_traits)
}

fn traits_to_supertraits(
    handler: &Handler,
    traits: Traits,
) -> Result<Vec<Supertrait>, ErrorEmitted> {
    let mut supertraits = vec![path_type_to_supertrait(handler, traits.prefix)?];
    for (_add_token, suffix) in traits.suffixes {
        let supertrait = path_type_to_supertrait(handler, suffix)?;
        supertraits.push(supertrait);
    }
    Ok(supertraits)
}

fn path_type_to_call_path(
    handler: &Handler,
    path_type: PathType,
) -> Result<CallPath, ErrorEmitted> {
    let PathType {
        root_opt,
        prefix,
        mut suffix,
    } = path_type;
    let is_absolute = path_root_opt_to_bool(handler, root_opt)?;
    let call_path = match suffix.pop() {
        Some((_double_colon_token, call_path_suffix)) => {
            let mut prefixes = vec![path_type_segment_to_ident(handler, prefix)?];
            for (_double_colon_token, call_path_prefix) in suffix {
                let ident = path_type_segment_to_ident(handler, call_path_prefix)?;
                prefixes.push(ident);
            }
            CallPath {
                prefixes,
                suffix: path_type_segment_to_ident(handler, call_path_suffix)?,
                is_absolute,
            }
        }
        None => CallPath {
            prefixes: Vec::new(),
            suffix: path_type_segment_to_ident(handler, prefix)?,
            is_absolute,
        },
    };
    Ok(call_path)
}

fn expr_to_ast_node(
    handler: &Handler,
    expr: Expr,
    is_statement: bool,
) -> Result<AstNode, ErrorEmitted> {
    let span = expr.span();
    let ast_node = {
        let expression = expr_to_expression(handler, expr)?;
        if !is_statement {
            AstNode {
                content: AstNodeContent::ImplicitReturnExpression(expression),
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
    handler: &Handler,
    args: Parens<AbiCastArgs>,
) -> Result<Box<AbiCastExpression>, ErrorEmitted> {
    let AbiCastArgs { name, address, .. } = args.into_inner();
    let abi_name = path_type_to_call_path(handler, name)?;
    let address = Box::new(expr_to_expression(handler, *address)?);
    Ok(Box::new(AbiCastExpression { abi_name, address }))
}

fn struct_path_and_fields_to_struct_expression(
    handler: &Handler,
    path: PathExpr,
    fields: Braces<Punctuated<ExprStructField, CommaToken>>,
) -> Result<Box<StructExpression>, ErrorEmitted> {
    let call_path_binding = path_expr_to_call_path_binding(handler, path)?;
    let fields = {
        fields
            .into_inner()
            .into_iter()
            .map(|expr_struct_field| {
                expr_struct_field_to_struct_expression_field(handler, expr_struct_field)
            })
            .collect::<Result<_, _>>()?
    };
    Ok(Box::new(StructExpression {
        call_path_binding,
        fields,
    }))
}

fn method_call_fields_to_method_application_expression(
    handler: &Handler,
    target: Box<Expr>,
    path_seg: PathExprSegment,
    contract_args_opt: Option<Braces<Punctuated<ExprStructField, CommaToken>>>,
    args: Parens<Punctuated<Expr, CommaToken>>,
) -> Result<Box<MethodApplicationExpression>, ErrorEmitted> {
    let (method_name, type_arguments) =
        path_expr_segment_to_ident_or_type_argument(handler, path_seg)?;

    let span = match &*type_arguments {
        [] => method_name.span(),
        [.., last] => Span::join(method_name.span(), last.span.clone()),
    };

    let method_name_binding = TypeBinding {
        inner: MethodName::FromModule { method_name },
        type_arguments,
        span,
    };
    let contract_call_params = match contract_args_opt {
        None => Vec::new(),
        Some(contract_args) => contract_args
            .into_inner()
            .into_iter()
            .map(|expr_struct_field| {
                expr_struct_field_to_struct_expression_field(handler, expr_struct_field)
            })
            .collect::<Result<_, _>>()?,
    };
    let arguments = iter::once(*target)
        .chain(args.into_inner().into_iter())
        .map(|expr| expr_to_expression(handler, expr))
        .collect::<Result<_, _>>()?;
    Ok(Box::new(MethodApplicationExpression {
        method_name_binding,
        contract_call_params,
        arguments,
    }))
}

fn expr_func_app_to_expression_kind(
    handler: &Handler,
    func: Box<Expr>,
    args: Parens<Punctuated<Expr, CommaToken>>,
) -> Result<ExpressionKind, ErrorEmitted> {
    let span = Span::join(func.span(), args.span());

    // For now, the callee has to be a path to a function.
    let PathExpr {
        root_opt,
        prefix,
        mut suffix,
    } = match *func {
        Expr::Path(path_expr) => path_expr,
        _ => {
            let error = ConvertParseTreeError::FunctionArbitraryExpression { span: func.span() };
            return Err(handler.emit_err(error.into()));
        }
    };

    let is_absolute = path_root_opt_to_bool(handler, root_opt)?;

    let convert_ty_args = |generics_opt: Option<(_, GenericArgs)>| {
        Ok(match generics_opt {
            Some((_, generic_args)) => {
                let span = generic_args.span();
                let ty_args = generic_args_to_type_arguments(handler, generic_args)?;
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
                prefix.push(path_expr_segment_to_ident(handler, &last)?);
                last = seg;
            }
            (prefix, Some(last), call_path_suffix)
        }
    };

    let arguments = args
        .into_inner()
        .into_iter()
        .map(|expr| expr_to_expression(handler, expr))
        .collect::<Result<_, _>>()?;

    let name_args_span = |start, end: Option<_>| match end {
        Some(end) => Span::join(start, end),
        None => start,
    };

    let (type_arguments, type_arguments_span) = convert_ty_args(call_seg.generics_opt)?;

    // Route intrinsic calls to different AST node.
    match Intrinsic::try_from_str(call_seg.name.as_str()) {
        Some(intrinsic) if last.is_none() && !is_absolute => {
            return Ok(ExpressionKind::IntrinsicFunction(
                IntrinsicFunctionExpression {
                    kind_binding: TypeBinding {
                        inner: intrinsic,
                        type_arguments,
                        span: name_args_span(span, type_arguments_span),
                    },
                    arguments,
                },
            ));
        }
        _ => {}
    }

    // Only `foo(args)`? It's a simple function call and not delineated / ambiguous.
    let last = match last {
        Some(last) => last,
        None => {
            let call_path = CallPath {
                prefixes,
                suffix: call_seg.name,
                is_absolute,
            };
            let span = match type_arguments_span {
                Some(span) => Span::join(call_path.span(), span),
                None => call_path.span(),
            };
            let call_path_binding = TypeBinding {
                inner: call_path,
                type_arguments,
                span,
            };
            return Ok(ExpressionKind::FunctionApplication(Box::new(
                FunctionApplicationExpression {
                    call_path_binding,
                    arguments,
                },
            )));
        }
    };

    // Ambiguous call. Could be a method call or a normal function call.
    // We don't know until type checking what `last` refers to, so let's defer.
    let (last_ty_args, last_ty_args_span) = convert_ty_args(last.generics_opt)?;
    let before = TypeBinding {
        span: name_args_span(last.name.span(), last_ty_args_span),
        inner: last.name,
        type_arguments: last_ty_args,
    };
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
        type_arguments,
    };
    Ok(ExpressionKind::AmbiguousPathExpression(Box::new(
        AmbiguousPathExpression {
            args: arguments,
            call_path_binding,
        },
    )))
}

fn expr_to_expression(handler: &Handler, expr: Expr) -> Result<Expression, ErrorEmitted> {
    let span = expr.span();
    let expression = match expr {
        Expr::Error(part_spans) => Expression {
            kind: ExpressionKind::Error(part_spans),
            span,
        },
        Expr::Path(path_expr) => path_expr_to_expression(handler, path_expr)?,
        Expr::Literal(literal) => Expression {
            kind: ExpressionKind::Literal(literal_to_literal(handler, literal)?),
            span,
        },
        Expr::AbiCast { args, .. } => {
            let abi_cast_expression = abi_cast_args_to_abi_cast_expression(handler, args)?;
            Expression {
                kind: ExpressionKind::AbiCast(abi_cast_expression),
                span,
            }
        }
        Expr::Struct { path, fields } => {
            let struct_expression =
                struct_path_and_fields_to_struct_expression(handler, path, fields)?;
            Expression {
                kind: ExpressionKind::Struct(struct_expression),
                span,
            }
        }
        Expr::Tuple(parenthesized_expr_tuple_descriptor) => {
            let fields = expr_tuple_descriptor_to_expressions(
                handler,
                parenthesized_expr_tuple_descriptor.into_inner(),
            )?;
            Expression {
                kind: ExpressionKind::Tuple(fields),
                span,
            }
        }
        Expr::Parens(parens) => expr_to_expression(handler, *parens.into_inner())?,
        Expr::Block(braced_code_block_contents) => {
            braced_code_block_contents_to_expression(handler, braced_code_block_contents)?
        }
        Expr::Array(bracketed_expr_array_descriptor) => {
            match bracketed_expr_array_descriptor.into_inner() {
                ExprArrayDescriptor::Sequence(exprs) => {
                    let contents = exprs
                        .into_iter()
                        .map(|expr| expr_to_expression(handler, expr))
                        .collect::<Result<_, _>>()?;
                    Expression {
                        kind: ExpressionKind::Array(contents),
                        span,
                    }
                }
                ExprArrayDescriptor::Repeat { value, length, .. } => {
                    let expression = expr_to_expression(handler, *value)?;
                    let length = expr_to_usize(handler, *length)?;
                    let contents = iter::repeat_with(|| expression.clone())
                        .take(length)
                        .collect();
                    Expression {
                        kind: ExpressionKind::Array(contents),
                        span,
                    }
                }
            }
        }
        Expr::Asm(asm_block) => {
            let asm_expression = asm_block_to_asm_expression(handler, asm_block)?;
            Expression {
                kind: ExpressionKind::Asm(asm_expression),
                span,
            }
        }
        Expr::Return { expr_opt, .. } => {
            let expression = match expr_opt {
                Some(expr) => expr_to_expression(handler, *expr)?,
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
        Expr::If(if_expr) => if_expr_to_expression(handler, if_expr)?,
        Expr::Match {
            value, branches, ..
        } => {
            let value = expr_to_expression(handler, *value)?;
            let var_decl_span = value.span();

            // Generate a deterministic name for the variable returned by the match expression.
            // Because the parser is single threaded, the name generated below will be stable.
            static COUNTER: AtomicUsize = AtomicUsize::new(0);
            let match_return_var_name = format!(
                "{}{}",
                MATCH_RETURN_VAR_NAME_PREFIX,
                COUNTER.load(Ordering::SeqCst)
            );
            COUNTER.fetch_add(1, Ordering::SeqCst);
            let var_decl_name = Ident::new_with_override(
                Box::leak(match_return_var_name.into_boxed_str()),
                var_decl_span.clone(),
            );

            let var_decl_exp = Expression {
                kind: ExpressionKind::Variable(var_decl_name.clone()),
                span: var_decl_span,
            };
            let branches = {
                branches
                    .into_inner()
                    .into_iter()
                    .map(|match_branch| match_branch_to_match_branch(handler, match_branch))
                    .collect::<Result<_, _>>()?
            };
            Expression {
                kind: ExpressionKind::CodeBlock(CodeBlock {
                    contents: vec![
                        AstNode {
                            content: AstNodeContent::Declaration(Declaration::VariableDeclaration(
                                VariableDeclaration {
                                    name: var_decl_name,
                                    type_ascription: TypeInfo::Unknown,
                                    type_ascription_span: None,
                                    is_mutable: false,
                                    body: value,
                                },
                            )),
                            span: span.clone(),
                        },
                        AstNode {
                            content: AstNodeContent::ImplicitReturnExpression(Expression {
                                kind: ExpressionKind::Match(MatchExpression {
                                    value: Box::new(var_decl_exp),
                                    branches,
                                }),
                                span: span.clone(),
                            }),
                            span: span.clone(),
                        },
                    ],
                    whole_block_span: span.clone(),
                }),
                span,
            }
        }
        Expr::While {
            condition, block, ..
        } => Expression {
            kind: ExpressionKind::WhileLoop(WhileLoopExpression {
                condition: Box::new(expr_to_expression(handler, *condition)?),
                body: braced_code_block_contents_to_code_block(handler, block)?,
            }),
            span,
        },
        Expr::FuncApp { func, args } => {
            let kind = expr_func_app_to_expression_kind(handler, func, args)?;
            Expression { kind, span }
        }
        Expr::Index { target, arg } => Expression {
            kind: ExpressionKind::ArrayIndex(ArrayIndexExpression {
                prefix: Box::new(expr_to_expression(handler, *target)?),
                index: Box::new(expr_to_expression(handler, *arg.into_inner())?),
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
                    handler,
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
            let storage_access_field_names = loop {
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
                        break Some(idents)
                    }
                    // We'll never find `storage`, so stop here.
                    _ => break None,
                }
            };

            let kind = match storage_access_field_names {
                Some(field_names) => ExpressionKind::StorageAccess(StorageAccessExpression {
                    field_names: field_names.into_iter().rev().cloned().collect(),
                }),
                None => ExpressionKind::Subfield(SubfieldExpression {
                    prefix: Box::new(expr_to_expression(handler, *target)?),
                    field_to_access: name,
                }),
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
                prefix: Box::new(expr_to_expression(handler, *target)?),
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
        Expr::Ref { .. } => unimplemented!(),
        Expr::Deref { .. } => unimplemented!(),
        Expr::Not { bang_token, expr } => {
            let expr = expr_to_expression(handler, *expr)?;
            op_call("not", bang_token.span(), span, &[expr])?
        }
        Expr::Pow {
            lhs,
            double_star_token,
            rhs,
        } => {
            let lhs = expr_to_expression(handler, *lhs)?;
            let rhs = expr_to_expression(handler, *rhs)?;
            op_call("pow", double_star_token.span(), span, &vec![lhs, rhs])?
        }
        Expr::Mul {
            lhs,
            star_token,
            rhs,
        } => {
            let lhs = expr_to_expression(handler, *lhs)?;
            let rhs = expr_to_expression(handler, *rhs)?;
            op_call("multiply", star_token.span(), span, &vec![lhs, rhs])?
        }
        Expr::Div {
            lhs,
            forward_slash_token,
            rhs,
        } => {
            let lhs = expr_to_expression(handler, *lhs)?;
            let rhs = expr_to_expression(handler, *rhs)?;
            op_call("divide", forward_slash_token.span(), span, &vec![lhs, rhs])?
        }
        Expr::Modulo {
            lhs,
            percent_token,
            rhs,
        } => {
            let lhs = expr_to_expression(handler, *lhs)?;
            let rhs = expr_to_expression(handler, *rhs)?;
            op_call("modulo", percent_token.span(), span, &vec![lhs, rhs])?
        }
        Expr::Add {
            lhs,
            add_token,
            rhs,
        } => {
            let lhs = expr_to_expression(handler, *lhs)?;
            let rhs = expr_to_expression(handler, *rhs)?;
            op_call("add", add_token.span(), span, &vec![lhs, rhs])?
        }
        Expr::Sub {
            lhs,
            sub_token,
            rhs,
        } => {
            let lhs = expr_to_expression(handler, *lhs)?;
            let rhs = expr_to_expression(handler, *rhs)?;
            op_call("subtract", sub_token.span(), span, &vec![lhs, rhs])?
        }
        Expr::Shl {
            lhs,
            shl_token,
            rhs,
        } => {
            let lhs = expr_to_expression(handler, *lhs)?;
            let rhs = expr_to_expression(handler, *rhs)?;
            op_call("lsh", shl_token.span(), span, &vec![lhs, rhs])?
        }
        Expr::Shr {
            lhs,
            shr_token,
            rhs,
        } => {
            let lhs = expr_to_expression(handler, *lhs)?;
            let rhs = expr_to_expression(handler, *rhs)?;
            op_call("rsh", shr_token.span(), span, &vec![lhs, rhs])?
        }
        Expr::BitAnd {
            lhs,
            ampersand_token,
            rhs,
        } => {
            let lhs = expr_to_expression(handler, *lhs)?;
            let rhs = expr_to_expression(handler, *rhs)?;
            op_call("binary_and", ampersand_token.span(), span, &vec![lhs, rhs])?
        }
        Expr::BitXor {
            lhs,
            caret_token,
            rhs,
        } => {
            let lhs = expr_to_expression(handler, *lhs)?;
            let rhs = expr_to_expression(handler, *rhs)?;
            op_call("binary_xor", caret_token.span(), span, &vec![lhs, rhs])?
        }
        Expr::BitOr {
            lhs,
            pipe_token,
            rhs,
        } => {
            let lhs = expr_to_expression(handler, *lhs)?;
            let rhs = expr_to_expression(handler, *rhs)?;
            op_call("binary_or", pipe_token.span(), span, &vec![lhs, rhs])?
        }
        Expr::Equal {
            lhs,
            double_eq_token,
            rhs,
        } => {
            let lhs = expr_to_expression(handler, *lhs)?;
            let rhs = expr_to_expression(handler, *rhs)?;
            op_call("eq", double_eq_token.span(), span, &vec![lhs, rhs])?
        }
        Expr::NotEqual {
            lhs,
            bang_eq_token,
            rhs,
        } => {
            let lhs = expr_to_expression(handler, *lhs)?;
            let rhs = expr_to_expression(handler, *rhs)?;
            op_call("neq", bang_eq_token.span(), span, &vec![lhs, rhs])?
        }
        Expr::LessThan {
            lhs,
            less_than_token,
            rhs,
        } => {
            let lhs = expr_to_expression(handler, *lhs)?;
            let rhs = expr_to_expression(handler, *rhs)?;
            op_call("lt", less_than_token.span(), span, &vec![lhs, rhs])?
        }
        Expr::GreaterThan {
            lhs,
            greater_than_token,
            rhs,
        } => {
            let lhs = expr_to_expression(handler, *lhs)?;
            let rhs = expr_to_expression(handler, *rhs)?;
            op_call("gt", greater_than_token.span(), span, &vec![lhs, rhs])?
        }
        Expr::LessThanEq {
            lhs,
            less_than_eq_token,
            rhs,
        } => {
            let lhs = expr_to_expression(handler, *lhs)?;
            let rhs = expr_to_expression(handler, *rhs)?;
            op_call("le", less_than_eq_token.span(), span, &vec![lhs, rhs])?
        }
        Expr::GreaterThanEq {
            lhs,
            greater_than_eq_token,
            rhs,
        } => {
            let lhs = expr_to_expression(handler, *lhs)?;
            let rhs = expr_to_expression(handler, *rhs)?;
            op_call("ge", greater_than_eq_token.span(), span, &vec![lhs, rhs])?
        }
        Expr::LogicalAnd { lhs, rhs, .. } => Expression {
            kind: ExpressionKind::LazyOperator(LazyOperatorExpression {
                op: LazyOp::And,
                lhs: Box::new(expr_to_expression(handler, *lhs)?),
                rhs: Box::new(expr_to_expression(handler, *rhs)?),
            }),
            span,
        },
        Expr::LogicalOr { lhs, rhs, .. } => Expression {
            kind: ExpressionKind::LazyOperator(LazyOperatorExpression {
                op: LazyOp::Or,
                lhs: Box::new(expr_to_expression(handler, *lhs)?),
                rhs: Box::new(expr_to_expression(handler, *rhs)?),
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
                    lhs: assignable_to_reassignment_target(handler, assignable)?,
                    rhs: Box::new(expr_to_expression(handler, *expr)?),
                }),
                span,
            },
            op_variant => {
                let lhs = assignable_to_reassignment_target(handler, assignable.clone())?;
                let rhs = Box::new(op_call(
                    op_variant.core_name(),
                    op_span,
                    span.clone(),
                    &vec![
                        assignable_to_expression(handler, assignable)?,
                        expr_to_expression(handler, *expr)?,
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
                    Ident::new_with_override("core", op_span.clone()),
                    Ident::new_with_override("ops", op_span.clone()),
                ],
                suffix: Ident::new_with_override(name, op_span.clone()),
                is_absolute: true,
            },
        },
        type_arguments: vec![],
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
    handler: &Handler,
    storage_field: sway_ast::StorageField,
    attributes: AttributesMap,
) -> Result<StorageField, ErrorEmitted> {
    let type_info_span = if let Ty::Path(path_type) = &storage_field.ty {
        path_type.prefix.name.span()
    } else {
        storage_field.ty.span()
    };
    let storage_field = StorageField {
        attributes,
        name: storage_field.name,
        type_info: ty_to_type_info(handler, storage_field.ty)?,
        type_info_span,
        initializer: expr_to_expression(handler, storage_field.initializer)?,
    };
    Ok(storage_field)
}

fn statement_to_ast_nodes(
    handler: &Handler,
    statement: Statement,
) -> Result<Vec<AstNode>, ErrorEmitted> {
    let ast_nodes = match statement {
        Statement::Let(statement_let) => statement_let_to_ast_nodes(handler, statement_let)?,
        Statement::Item(item) => {
            let nodes = item_to_ast_nodes(handler, item)?;
            nodes.iter().fold(Ok(()), |res, node| {
                if ast_node_is_test_fn(node) {
                    let span = node.span.clone();
                    let error = ConvertParseTreeError::TestFnOnlyAllowedAtModuleLevel { span };
                    Err(handler.emit_err(error.into()))
                } else {
                    res
                }
            })?;
            nodes
        }
        Statement::Expr { expr, .. } => vec![expr_to_ast_node(handler, expr, true)?],
    };
    Ok(ast_nodes)
}

fn fn_arg_to_function_parameter(
    handler: &Handler,
    fn_arg: FnArg,
) -> Result<FunctionParameter, ErrorEmitted> {
    let type_span = fn_arg.ty.span();
    let pat_span = fn_arg.pattern.span();
    let (reference, mutable, name) = match fn_arg.pattern {
        Pattern::Wildcard { .. } => {
            let error = ConvertParseTreeError::WildcardPatternsNotSupportedHere { span: pat_span };
            return Err(handler.emit_err(error.into()));
        }
        Pattern::Var {
            reference,
            mutable,
            name,
        } => (reference, mutable, name),
        Pattern::Literal(..) => {
            let error = ConvertParseTreeError::LiteralPatternsNotSupportedHere { span: pat_span };
            return Err(handler.emit_err(error.into()));
        }
        Pattern::Constant(..) => {
            let error = ConvertParseTreeError::ConstantPatternsNotSupportedHere { span: pat_span };
            return Err(handler.emit_err(error.into()));
        }
        Pattern::Constructor { .. } => {
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
        type_info: ty_to_type_info(handler, fn_arg.ty)?,
        type_span,
    };
    Ok(function_parameter)
}

fn expr_to_usize(handler: &Handler, expr: Expr) -> Result<usize, ErrorEmitted> {
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

fn expr_to_u64(handler: &Handler, expr: Expr) -> Result<u64, ErrorEmitted> {
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
            match u64::try_from(lit_int.parsed) {
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
    handler: &Handler,
    path_type: PathType,
) -> Result<Supertrait, ErrorEmitted> {
    let PathType {
        root_opt,
        prefix,
        mut suffix,
    } = path_type;
    let is_absolute = path_root_opt_to_bool(handler, root_opt)?;
    let (prefixes, call_path_suffix) = match suffix.pop() {
        Some((_, call_path_suffix)) => {
            let mut prefixes = vec![path_type_segment_to_ident(handler, prefix)?];
            for (_, call_path_prefix) in suffix {
                let ident = path_type_segment_to_ident(handler, call_path_prefix)?;
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
        //type_parameters,
    };
    Ok(supertrait)
}

fn path_type_segment_to_ident(
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
    handler: &Handler,
    PathExprSegment { name, generics_opt }: PathExprSegment,
) -> Result<(Ident, Vec<TypeArgument>), ErrorEmitted> {
    let type_args = match generics_opt {
        Some((_, x)) => generic_args_to_type_arguments(handler, x)?,
        None => Default::default(),
    };
    Ok((name, type_args))
}

fn path_expr_segment_to_ident(
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
    handler: &Handler,
    path_expr: PathExpr,
) -> Result<Expression, ErrorEmitted> {
    let span = path_expr.span();
    let expression = if path_expr.root_opt.is_none() && path_expr.suffix.is_empty() {
        let name = path_expr_segment_to_ident(handler, &path_expr.prefix)?;
        Expression {
            kind: ExpressionKind::Variable(name),
            span,
        }
    } else {
        let call_path = path_expr_to_call_path(handler, path_expr)?;
        let call_path_binding = TypeBinding {
            inner: call_path.clone(),
            type_arguments: vec![],
            span: call_path.span(),
        };
        Expression {
            kind: ExpressionKind::DelineatedPath(Box::new(DelineatedPathExpression {
                call_path_binding,
                args: Vec::new(),
            })),
            span,
        }
    };
    Ok(expression)
}

fn braced_code_block_contents_to_expression(
    handler: &Handler,
    braced_code_block_contents: Braces<CodeBlockContents>,
) -> Result<Expression, ErrorEmitted> {
    let span = braced_code_block_contents.span();
    let code_block = braced_code_block_contents_to_code_block(handler, braced_code_block_contents)?;
    Ok(Expression {
        kind: ExpressionKind::CodeBlock(code_block),
        span,
    })
}

fn if_expr_to_expression(handler: &Handler, if_expr: IfExpr) -> Result<Expression, ErrorEmitted> {
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
            handler, then_block,
        )?),
        span: then_block_span.clone(),
    };
    let else_block = match else_opt {
        None => None,
        Some((_else_token, tail)) => {
            let expression = match tail {
                ControlFlow::Break(braced_code_block_contents) => {
                    braced_code_block_contents_to_expression(handler, braced_code_block_contents)?
                }
                ControlFlow::Continue(if_expr) => if_expr_to_expression(handler, *if_expr)?,
            };
            Some(expression)
        }
    };
    let expression = match condition {
        IfCondition::Expr(condition) => Expression {
            kind: ExpressionKind::If(IfExpression {
                condition: Box::new(expr_to_expression(handler, *condition)?),
                then: Box::new(then_block),
                r#else: else_block.map(Box::new),
            }),
            span,
        },
        IfCondition::Let { lhs, rhs, .. } => {
            let scrutinee = pattern_to_scrutinee(handler, *lhs)?;
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
            Expression {
                kind: ExpressionKind::Match(MatchExpression {
                    value: Box::new(expr_to_expression(handler, *rhs)?),
                    branches,
                }),
                span,
            }
        }
    };
    Ok(expression)
}

/// Determine if the path is in absolute form, e.g., `::foo::bar`.
///
/// Throws an error when given `<Foo as Bar>::baz`.
fn path_root_opt_to_bool(
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

fn literal_to_literal(
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
                full_span.path().cloned(),
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
                            1..=16 => Literal::U64(u64::try_from(parsed).unwrap()),
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
                            1..=64 => Literal::U64(u64::try_from(parsed).unwrap()),
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
                Some((lit_int_type, _span)) => match lit_int_type {
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
fn path_expr_to_call_path_binding(
    handler: &Handler,
    path_expr: PathExpr,
) -> Result<TypeBinding<CallPath>, ErrorEmitted> {
    let PathExpr {
        root_opt,
        prefix,
        mut suffix,
    } = path_expr;
    let is_absolute = path_root_opt_to_bool(handler, root_opt)?;
    let (prefixes, suffix, span, type_arguments) = match suffix.pop() {
        Some((_, call_path_suffix)) => {
            let mut prefixes = vec![path_expr_segment_to_ident(handler, &prefix)?];
            for (_, call_path_prefix) in suffix {
                let ident = path_expr_segment_to_ident(handler, &call_path_prefix)?;
                // note that call paths only support one set of type arguments per call path right
                // now
                prefixes.push(ident);
            }
            let span = call_path_suffix.span();
            let (suffix, ty_args) =
                path_expr_segment_to_ident_or_type_argument(handler, call_path_suffix)?;
            (prefixes, suffix, span, ty_args)
        }
        None => {
            let span = prefix.span();
            let (suffix, ty_args) = path_expr_segment_to_ident_or_type_argument(handler, prefix)?;
            (vec![], suffix, span, ty_args)
        }
    };
    Ok(TypeBinding {
        inner: CallPath {
            prefixes,
            suffix,
            is_absolute,
        },
        type_arguments,
        span,
    })
}

fn path_expr_to_call_path(
    handler: &Handler,
    path_expr: PathExpr,
) -> Result<CallPath, ErrorEmitted> {
    let PathExpr {
        root_opt,
        prefix,
        mut suffix,
    } = path_expr;
    let is_absolute = path_root_opt_to_bool(handler, root_opt)?;
    let call_path = match suffix.pop() {
        Some((_double_colon_token, call_path_suffix)) => {
            let mut prefixes = vec![path_expr_segment_to_ident(handler, &prefix)?];
            for (_double_colon_token, call_path_prefix) in suffix {
                let ident = path_expr_segment_to_ident(handler, &call_path_prefix)?;
                prefixes.push(ident);
            }
            CallPath {
                prefixes,
                suffix: path_expr_segment_to_ident(handler, &call_path_suffix)?,
                is_absolute,
            }
        }
        None => CallPath {
            prefixes: Vec::new(),
            suffix: path_expr_segment_to_ident(handler, &prefix)?,
            is_absolute,
        },
    };
    Ok(call_path)
}

fn expr_struct_field_to_struct_expression_field(
    handler: &Handler,
    expr_struct_field: ExprStructField,
) -> Result<StructExpressionField, ErrorEmitted> {
    let span = expr_struct_field.span();
    let value = match expr_struct_field.expr_opt {
        Some((_colon_token, expr)) => expr_to_expression(handler, *expr)?,
        None => Expression {
            kind: ExpressionKind::Variable(expr_struct_field.field_name.clone()),
            span: span.clone(),
        },
    };
    Ok(StructExpressionField {
        name: expr_struct_field.field_name,
        value,
        span,
    })
}

fn expr_tuple_descriptor_to_expressions(
    handler: &Handler,
    expr_tuple_descriptor: ExprTupleDescriptor,
) -> Result<Vec<Expression>, ErrorEmitted> {
    let expressions = match expr_tuple_descriptor {
        ExprTupleDescriptor::Nil => Vec::new(),
        ExprTupleDescriptor::Cons { head, tail, .. } => {
            let mut expressions = vec![expr_to_expression(handler, *head)?];
            for expr in tail {
                expressions.push(expr_to_expression(handler, expr)?);
            }
            expressions
        }
    };
    Ok(expressions)
}

fn asm_block_to_asm_expression(
    handler: &Handler,
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
                Some((_colon_token, ty)) => ty_to_type_info(handler, ty)?,
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
                    handler,
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
    handler: &Handler,
    match_branch: sway_ast::MatchBranch,
) -> Result<MatchBranch, ErrorEmitted> {
    let span = match_branch.span();
    Ok(MatchBranch {
        scrutinee: pattern_to_scrutinee(handler, match_branch.pattern)?,
        result: match match_branch.kind {
            MatchBranchKind::Block { block, .. } => {
                let span = block.span();
                Expression {
                    kind: ExpressionKind::CodeBlock(braced_code_block_contents_to_code_block(
                        handler, block,
                    )?),
                    span,
                }
            }
            MatchBranchKind::Expr { expr, .. } => expr_to_expression(handler, expr)?,
        },
        span,
    })
}

fn statement_let_to_ast_nodes(
    handler: &Handler,
    statement_let: StatementLet,
) -> Result<Vec<AstNode>, ErrorEmitted> {
    fn unfold(
        handler: &Handler,
        pattern: Pattern,
        ty_opt: Option<Ty>,
        expression: Expression,
        span: Span,
    ) -> Result<Vec<AstNode>, ErrorEmitted> {
        let ast_nodes = match pattern {
            Pattern::Wildcard { .. } | Pattern::Var { .. } => {
                let (reference, mutable, name) = match pattern {
                    Pattern::Var {
                        reference,
                        mutable,
                        name,
                    } => (reference, mutable, name),
                    Pattern::Wildcard { .. } => (None, None, Ident::new_no_span("_")),
                    _ => unreachable!(),
                };
                if reference.is_some() {
                    let error = ConvertParseTreeError::RefVariablesNotSupported { span };
                    return Err(handler.emit_err(error.into()));
                }
                let (type_ascription, type_ascription_span) = match ty_opt {
                    Some(ty) => {
                        let type_ascription_span = ty.span();
                        let type_ascription = ty_to_type_info(handler, ty)?;
                        (type_ascription, Some(type_ascription_span))
                    }
                    None => (TypeInfo::Unknown, None),
                };
                let ast_node = AstNode {
                    content: AstNodeContent::Declaration(Declaration::VariableDeclaration(
                        VariableDeclaration {
                            name,
                            type_ascription,
                            type_ascription_span,
                            body: expression,
                            is_mutable: mutable.is_some(),
                        },
                    )),
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
            Pattern::Constructor { .. } => {
                let error = ConvertParseTreeError::ConstructorPatternsNotSupportedHere { span };
                return Err(handler.emit_err(error.into()));
            }
            Pattern::Struct { path, fields, .. } => {
                let mut ast_nodes = Vec::new();

                // Generate a deterministic name for the destructured struct
                // Because the parser is single threaded, the name generated below will be stable.
                static COUNTER: AtomicUsize = AtomicUsize::new(0);
                let destructured_name =
                    format!("{}{}", DESTRUCTURE_PREFIX, COUNTER.load(Ordering::SeqCst));
                COUNTER.fetch_add(1, Ordering::SeqCst);
                let destructure_name = Ident::new_with_override(
                    Box::leak(destructured_name.into_boxed_str()),
                    path.prefix.name.span(),
                );

                // Parse the type ascription and the type ascription span.
                // In the event that the user did not provide a type ascription,
                // it is set to TypeInfo::Unknown and the span to None.
                let (type_ascription, type_ascription_span) = match &ty_opt {
                    Some(ty) => {
                        let type_ascription_span = ty.span();
                        let type_ascription = ty_to_type_info(handler, ty.clone())?;
                        (type_ascription, Some(type_ascription_span))
                    }
                    None => (TypeInfo::Unknown, None),
                };

                // Save the destructure to the new name as a new variable declaration
                let save_body_first = VariableDeclaration {
                    name: destructure_name.clone(),
                    type_ascription,
                    type_ascription_span,
                    body: expression,
                    is_mutable: false,
                };
                ast_nodes.push(AstNode {
                    content: AstNodeContent::Declaration(Declaration::VariableDeclaration(
                        save_body_first,
                    )),
                    span: span.clone(),
                });

                // create a new variable expression that points to the new destructured struct name that we just created
                let new_expr = Expression {
                    kind: ExpressionKind::Variable(destructure_name),
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
                    ast_nodes.extend(unfold(
                        handler,
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
            Pattern::Tuple(pat_tuple) => {
                let mut ast_nodes = Vec::new();

                // Generate a deterministic name for the tuple.
                // Because the parser is single threaded, the name generated below will be stable.
                static COUNTER: AtomicUsize = AtomicUsize::new(0);
                let tuple_name = format!("{}{}", TUPLE_NAME_PREFIX, COUNTER.load(Ordering::SeqCst));
                COUNTER.fetch_add(1, Ordering::SeqCst);
                let tuple_name =
                    Ident::new_with_override(Box::leak(tuple_name.into_boxed_str()), span.clone());

                // Parse the type ascription and the type ascription span.
                // In the event that the user did not provide a type ascription,
                // it is set to TypeInfo::Unknown and the span to None.
                let (type_ascription, type_ascription_span) = match &ty_opt {
                    Some(ty) => {
                        let type_ascription_span = ty.span();
                        let type_ascription = ty_to_type_info(handler, ty.clone())?;
                        (type_ascription, Some(type_ascription_span))
                    }
                    None => (TypeInfo::Unknown, None),
                };

                // Save the tuple to the new name as a new variable declaration.
                let save_body_first = VariableDeclaration {
                    name: tuple_name.clone(),
                    type_ascription,
                    type_ascription_span,
                    body: expression,
                    is_mutable: false,
                };
                ast_nodes.push(AstNode {
                    content: AstNodeContent::Declaration(Declaration::VariableDeclaration(
                        save_body_first,
                    )),
                    span: span.clone(),
                });

                // create a variable expression that points to the new tuple name that we just created
                let new_expr = Expression {
                    kind: ExpressionKind::Variable(tuple_name),
                    span: span.clone(),
                };

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
                    ast_nodes.extend(unfold(
                        handler,
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
    let span = statement_let.span();
    let initial_expression = expr_to_expression(handler, statement_let.expr)?;
    unfold(
        handler,
        statement_let.pattern,
        statement_let.ty_opt.map(|(_colon_token, ty)| ty),
        initial_expression,
        span,
    )
}

fn dependency_to_include_statement(dependency: &Dependency) -> IncludeStatement {
    IncludeStatement {
        _alias: None,
        span: dependency.span(),
        _path_span: dependency.path.span(),
    }
}

#[allow(dead_code)]
fn generic_args_to_type_parameters(
    handler: &Handler,
    generic_args: GenericArgs,
) -> Result<Vec<TypeParameter>, ErrorEmitted> {
    generic_args
        .parameters
        .into_inner()
        .into_iter()
        .map(|x| ty_to_type_parameter(handler, x))
        .collect()
}

fn asm_register_declaration_to_asm_register_declaration(
    handler: &Handler,
    asm_register_declaration: sway_ast::AsmRegisterDeclaration,
) -> Result<AsmRegisterDeclaration, ErrorEmitted> {
    Ok(AsmRegisterDeclaration {
        name: asm_register_declaration.register,
        initializer: asm_register_declaration
            .value_opt
            .map(|(_colon_token, expr)| expr_to_expression(handler, *expr))
            .transpose()?,
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

fn pattern_to_scrutinee(handler: &Handler, pattern: Pattern) -> Result<Scrutinee, ErrorEmitted> {
    let span = pattern.span();
    let scrutinee = match pattern {
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
        Pattern::Literal(literal) => Scrutinee::Literal {
            value: literal_to_literal(handler, literal)?,
            span,
        },
        Pattern::Constant(path_expr) => {
            let call_path = path_expr_to_call_path(handler, path_expr)?;
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
                call_path: path_expr_to_call_path(handler, path)?,
                value: Box::new(pattern_to_scrutinee(handler, value)?),
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
                .map(|field| pattern_struct_field_to_struct_scrutinee_field(handler, field))
                .collect::<Result<_, _>>()?;

            Scrutinee::StructScrutinee {
                struct_name: path_expr_to_ident(handler, path)?,
                fields: { scrutinee_fields },
                span,
            }
        }
        Pattern::Tuple(pat_tuple) => Scrutinee::Tuple {
            elems: {
                pat_tuple
                    .into_inner()
                    .into_iter()
                    .map(|pattern| pattern_to_scrutinee(handler, pattern))
                    .collect::<Result<_, _>>()?
            },
            span,
        },
    };
    Ok(scrutinee)
}

#[allow(dead_code)]
fn ty_to_type_parameter(handler: &Handler, ty: Ty) -> Result<TypeParameter, ErrorEmitted> {
    let name_ident = match ty {
        Ty::Path(path_type) => path_type_to_ident(handler, path_type)?,
        Ty::Infer { underscore_token } => {
            let unknown_type = insert_type(TypeInfo::Unknown);
            return Ok(TypeParameter {
                type_id: unknown_type,
                initial_type_id: unknown_type,
                name_ident: underscore_token.into(),
                trait_constraints: Default::default(),
                trait_constraints_span: Span::dummy(),
            });
        }
        Ty::Tuple(..) => panic!("tuple types are not allowed in this position"),
        Ty::Array(..) => panic!("array types are not allowed in this position"),
        Ty::Str { .. } => panic!("str types are not allowed in this position"),
    };
    let custom_type = insert_type(TypeInfo::Custom {
        name: name_ident.clone(),
        type_arguments: None,
    });
    Ok(TypeParameter {
        type_id: custom_type,
        initial_type_id: custom_type,
        name_ident,
        trait_constraints: Vec::new(),
        trait_constraints_span: Span::dummy(),
    })
}

#[allow(dead_code)]
fn path_type_to_ident(handler: &Handler, path_type: PathType) -> Result<Ident, ErrorEmitted> {
    let PathType {
        root_opt,
        prefix,
        suffix,
    } = path_type;
    if root_opt.is_some() || !suffix.is_empty() {
        panic!("types with paths aren't currently supported");
    }
    path_type_segment_to_ident(handler, prefix)
}

fn path_expr_to_ident(handler: &Handler, path_expr: PathExpr) -> Result<Ident, ErrorEmitted> {
    let span = path_expr.span();
    let PathExpr {
        root_opt,
        prefix,
        suffix,
    } = path_expr;
    if root_opt.is_some() || !suffix.is_empty() {
        let error = ConvertParseTreeError::PathsNotSupportedHere { span };
        return Err(handler.emit_err(error.into()));
    }
    path_expr_segment_to_ident(handler, &prefix)
}

fn pattern_struct_field_to_struct_scrutinee_field(
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
                    .map(|(_colon_token, pattern)| pattern_to_scrutinee(handler, *pattern))
                    .transpose()?,
                span,
            };
            Ok(struct_scrutinee_field)
        }
    }
}

fn assignable_to_expression(
    handler: &Handler,
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
                prefix: Box::new(assignable_to_expression(handler, *target)?),
                index: Box::new(expr_to_expression(handler, *arg.into_inner())?),
            }),
            span,
        },
        Assignable::FieldProjection { target, name, .. } => {
            let mut idents = vec![&name];
            let mut base = &*target;
            let storage_access_field_names_opt = loop {
                match base {
                    Assignable::FieldProjection { target, name, .. } => {
                        idents.push(name);
                        base = target;
                    }
                    Assignable::Var(name) => {
                        if name.as_str() == "storage" {
                            break Some(idents);
                        }
                        break None;
                    }
                    _ => break None,
                }
            };
            match storage_access_field_names_opt {
                Some(field_names) => {
                    let field_names = field_names.into_iter().rev().cloned().collect();
                    Expression {
                        kind: ExpressionKind::StorageAccess(StorageAccessExpression {
                            field_names,
                        }),
                        span,
                    }
                }
                None => Expression {
                    kind: ExpressionKind::Subfield(SubfieldExpression {
                        prefix: Box::new(assignable_to_expression(handler, *target)?),
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
                    prefix: Box::new(assignable_to_expression(handler, *target)?),
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
    handler: &Handler,
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
            Assignable::Var(name) => {
                if name.as_str() == "storage" {
                    let idents = idents.into_iter().rev().cloned().collect();
                    return Ok(ReassignmentTarget::StorageField(idents));
                }
                break;
            }
            Assignable::Index { .. } => break,
            Assignable::TupleFieldProjection { .. } => break,
        }
    }
    let expression = assignable_to_expression(handler, assignable)?;
    Ok(ReassignmentTarget::VariableExpression(Box::new(expression)))
}

fn generic_args_to_type_arguments(
    handler: &Handler,
    generic_args: GenericArgs,
) -> Result<Vec<TypeArgument>, ErrorEmitted> {
    generic_args
        .parameters
        .into_inner()
        .into_iter()
        .map(|ty| {
            let span = ty.span();
            let type_id = insert_type(ty_to_type_info(handler, ty)?);
            Ok(TypeArgument {
                type_id,
                initial_type_id: type_id,
                span,
            })
        })
        .collect()
}

fn ty_tuple_descriptor_to_type_arguments(
    handler: &Handler,
    ty_tuple_descriptor: TyTupleDescriptor,
) -> Result<Vec<TypeArgument>, ErrorEmitted> {
    let type_arguments = match ty_tuple_descriptor {
        TyTupleDescriptor::Nil => vec![],
        TyTupleDescriptor::Cons { head, tail, .. } => {
            let mut type_arguments = vec![ty_to_type_argument(handler, *head)?];
            for ty in tail.into_iter() {
                type_arguments.push(ty_to_type_argument(handler, ty)?);
            }
            type_arguments
        }
    };
    Ok(type_arguments)
}

fn path_type_to_type_info(
    handler: &Handler,
    path_type: PathType,
) -> Result<TypeInfo, ErrorEmitted> {
    let span = path_type.span();
    let PathType {
        root_opt,
        prefix: PathTypeSegment { name, generics_opt },
        suffix,
    } = path_type;

    if root_opt.is_some() || !suffix.is_empty() {
        let error = ConvertParseTreeError::FullySpecifiedTypesNotSupported { span };
        return Err(handler.emit_err(error.into()));
    }

    let type_info = match type_name_to_type_info_opt(&name) {
        Some(type_info) => {
            if let Some((_, generic_args)) = generics_opt {
                let error = ConvertParseTreeError::GenericsNotSupportedHere {
                    span: generic_args.span(),
                };
                return Err(handler.emit_err(error.into()));
            }
            type_info
        }
        None => {
            if name.as_str() == "ContractCaller" {
                let generic_ty = match {
                    generics_opt.and_then(|(_, generic_args)| {
                        iter_to_array(generic_args.parameters.into_inner())
                    })
                } {
                    Some([ty]) => ty,
                    None => {
                        let error = ConvertParseTreeError::ContractCallerOneGenericArg { span };
                        return Err(handler.emit_err(error.into()));
                    }
                };
                let abi_name = match generic_ty {
                    Ty::Path(path_type) => {
                        let call_path = path_type_to_call_path(handler, path_type)?;
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
                let type_arguments = match generics_opt {
                    Some((_double_colon_token, generic_args)) => {
                        generic_args_to_type_arguments(handler, generic_args)?
                    }
                    None => Vec::new(),
                };
                TypeInfo::Custom {
                    name,
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

fn item_attrs_to_map(
    handler: &Handler,
    attribute_list: &[AttributeDecl],
) -> Result<AttributesMap, ErrorEmitted> {
    let mut attrs_map: HashMap<_, Vec<Attribute>> = HashMap::new();
    for attr_decl in attribute_list {
        let attr = attr_decl.attribute.get();
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
            .map(|parens| parens.get().into_iter().cloned().collect())
            .unwrap_or_else(Vec::new);

        let attribute = Attribute {
            name: attr.name.clone(),
            args,
        };

        if let Some(attr_kind) = match name {
            DOC_ATTRIBUTE_NAME => Some(AttributeKind::Doc),
            STORAGE_PURITY_ATTRIBUTE_NAME => Some(AttributeKind::Storage),
            INLINE_ATTRIBUTE_NAME => Some(AttributeKind::Inline),
            TEST_ATTRIBUTE_NAME => Some(AttributeKind::Test),
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
    Ok(Arc::new(attrs_map))
}
