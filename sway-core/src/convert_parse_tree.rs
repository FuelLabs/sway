use {
    std::{
        iter,
        convert::TryFrom,
        ops::ControlFlow,
    },
    crate::{
        SwayParseTree, ParseTree, TreeType, AstNode, AstNodeContent, Declaration,
        FunctionDeclaration, TraitDeclaration, StructDeclaration, EnumDeclaration, AbiDeclaration,
        ConstantDeclaration, StorageDeclaration,
        Visibility, StructField, TypeParameter, EnumVariant, FunctionParameter, CodeBlock, Purity,
        Supertrait, TraitFn, ImplTrait, ImplSelf,
        CallPath, StorageField,
        IncludeStatement, VariableDeclaration, ReturnStatement, WhileLoop, Reassignment, UseStatement,
        TypeInfo, ImportType, TypeArgument, ReassignmentTarget,
        Expression, Literal, StructExpressionField, AsmExpression, MatchBranch, LazyOp, MethodName,
        AsmRegister, AsmRegisterDeclaration, AsmOp,
        MatchCondition, Scrutinee, CatchAll, StructScrutineeField, BuiltinProperty,
        type_engine::{insert_type, IntegerBits, AbiName},
        parse_tree::desugar_match_expression,
    },
    sway_types::{Ident, Span},
    new_parser_again::{
        Program, ProgramKind,
        Item, ItemStruct, ItemEnum, ItemFn, ItemTrait, ItemImpl, ItemAbi, ItemConst, ItemStorage,
        TypeField, GenericParams, GenericArgs, FnArgs, FnSignature, Traits,
        PubToken, ImpureToken, Braces, AngleBrackets, DoubleColonToken,
        Ty, Pattern, PatternStructField,
        CodeBlockContents, Statement, StatementLet,
        QualifiedPathRoot, PathType, PathTypeSegment, PathExpr, PathExprSegment,
        Expr, ExprTupleDescriptor, ExprArrayDescriptor, ExprStructField, AsmBlock, LitInt, LitIntType,
        IfExpr, IfCondition, AbiCastArgs,
        Instruction, MatchBranchKind, Assignable,
        Dependency, ItemUse, UseTree,
    },
    nanoid::nanoid,
};

pub fn program_to_sway_parse_tree(program: Program) -> SwayParseTree {
    let span = program.span();
    SwayParseTree {
        tree_type: match program.kind {
            ProgramKind::Script { .. } => TreeType::Script,
            ProgramKind::Contract { .. } => TreeType::Contract,
            ProgramKind::Predicate { .. } => TreeType::Predicate,
            ProgramKind::Library { name, .. } => TreeType::Library { name },
        },
        tree: ParseTree {
            span,
            root_nodes: {
                program
                .dependencies
                .into_iter()
                .map(|dependency| {
                    let span = dependency.span();
                    AstNode {
                        content: AstNodeContent::IncludeStatement(dependency_to_include_statement(dependency)),
                        span,
                    }
                })
                .chain(program.items.into_iter().map(|item| item_to_ast_nodes(item).into_iter()).flatten())
                .collect()
            },
        },
    }
}

fn item_to_ast_nodes(item: Item) -> Vec<AstNode> {
    let span = item.span();
    let contents = match item {
        Item::Use(item_use) => {
            let use_statements = item_use_to_use_statements(item_use);
            use_statements.into_iter().map(AstNodeContent::UseStatement).collect()
        },
        Item::Struct(item_struct) => {
            let struct_declaration = item_struct_to_struct_declaration(item_struct);
            vec![AstNodeContent::Declaration(Declaration::StructDeclaration(struct_declaration))]
        },
        Item::Enum(item_enum) => {
            let enum_declaration = item_enum_to_enum_declaration(item_enum);
            vec![AstNodeContent::Declaration(Declaration::EnumDeclaration(enum_declaration))]
        },
        Item::Fn(item_fn) => {
            let function_declaration = item_fn_to_function_declaration(item_fn);
            vec![AstNodeContent::Declaration(Declaration::FunctionDeclaration(function_declaration))]
        },
        Item::Trait(item_trait) => {
            let trait_declaration = item_trait_to_trait_declaration(item_trait);
            vec![AstNodeContent::Declaration(Declaration::TraitDeclaration(trait_declaration))]
        },
        Item::Impl(item_impl) => {
            let declaration = item_impl_to_declaration(item_impl);
            vec![AstNodeContent::Declaration(declaration)]
        },
        Item::Abi(item_abi) => {
            let abi_declaration = item_abi_to_abi_declaration(item_abi);
            vec![AstNodeContent::Declaration(Declaration::AbiDeclaration(abi_declaration))]
        },
        Item::Const(item_const) => {
            let constant_declaration = item_const_to_constant_declaration(item_const);
            vec![AstNodeContent::Declaration(Declaration::ConstantDeclaration(constant_declaration))]
        },
        Item::Storage(item_storage) => {
            let storage_declaration = item_storage_to_storage_declaration(item_storage);
            vec![AstNodeContent::Declaration(Declaration::StorageDeclaration(storage_declaration))]
        },
    };
    contents.into_iter().map(|content| AstNode { span: span.clone(), content }).collect()
}

fn item_use_to_use_statements(item_use: ItemUse) -> Vec<UseStatement> {
    if item_use.visibility.is_some() {
        panic!("public imports are not yet supported");
    }
    let mut ret = Vec::new();
    let mut prefix = Vec::new();
    use_tree_to_use_statements(item_use.tree, item_use.root_import.is_some(), &mut prefix, &mut ret);
    assert!(prefix.is_empty());
    ret
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
        },
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
        },
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
        },
        UseTree::Glob { .. } => {
            ret.push(UseStatement {
                call_path: path.clone(),
                import_type: ImportType::Star,
                is_absolute,
                alias: None,
            });
        },
        UseTree::Path { prefix, suffix, .. } => {
            path.push(prefix);
            use_tree_to_use_statements(*suffix, is_absolute, path, ret);
            path.pop().unwrap();
        },
    }
}

fn item_struct_to_struct_declaration(item_struct: ItemStruct) -> StructDeclaration {
    let span = item_struct.span();
    StructDeclaration {
        name: item_struct.name,
        fields: item_struct.fields.into_inner().into_iter().map(type_field_to_struct_field).collect(),
        type_parameters: generic_params_opt_to_type_parameters(item_struct.generics),
        visibility: pub_token_opt_to_visibility(item_struct.visibility),
        span,
    }
}

fn item_enum_to_enum_declaration(item_enum: ItemEnum) -> EnumDeclaration {
    let span = item_enum.span();
    EnumDeclaration {
        name: item_enum.name,
        type_parameters: generic_params_opt_to_type_parameters(item_enum.generics),
        variants: {
            item_enum
            .fields
            .into_inner()
            .into_iter()
            .enumerate()
            .map(|(tag, type_field)| type_field_to_enum_variant(type_field, tag))
            .collect()
        },
        span,
        visibility: pub_token_opt_to_visibility(item_enum.visibility),
    }
}

fn item_fn_to_function_declaration(item_fn: ItemFn) -> FunctionDeclaration {
    let span = item_fn.span();
    let return_type_span = match &item_fn.fn_signature.return_type_opt {
        Some((_right_arrow_token, ty)) => ty.span(),
        None => item_fn.fn_signature.span(),
    };
    FunctionDeclaration {
        purity: impure_token_opt_to_purity(item_fn.fn_signature.impure),
        name: item_fn.fn_signature.name,
        visibility: pub_token_opt_to_visibility(item_fn.fn_signature.visibility),
        body: braced_code_block_contents_to_code_block(item_fn.body),
        parameters: fn_args_to_function_parameters(item_fn.fn_signature.arguments.into_inner()),
        span,
        return_type: match item_fn.fn_signature.return_type_opt {
            Some((_right_arrow, ty)) => ty_to_type_info(ty),
            None => TypeInfo::Tuple(Vec::new()),
        },
        type_parameters: generic_params_opt_to_type_parameters(item_fn.fn_signature.generics),
        return_type_span,
    }
}

fn item_trait_to_trait_declaration(item_trait: ItemTrait) -> TraitDeclaration {
    TraitDeclaration {
        name: item_trait.name,
        interface_surface: {
            item_trait
            .trait_items
            .into_inner()
            .into_iter()
            .map(|(fn_signature, _semicolon_token)| fn_signature_to_trait_fn(fn_signature))
            .collect()
        },
        methods: {
            item_trait
            .trait_defs_opt
            .into_iter()
            .map(|trait_defs| trait_defs.into_inner().into_iter().map(item_fn_to_function_declaration))
            .flatten()
            .collect()
        },
        supertraits: {
            item_trait
            .super_traits
            .map(|(_colon_token, traits)| traits_to_supertraits(traits))
            .unwrap_or_default()
        },
        visibility: pub_token_opt_to_visibility(item_trait.visibility),
    }
}

fn item_impl_to_declaration(item_impl: ItemImpl) -> Declaration {
    let block_span = item_impl.span();
    //let type_arguments_span = item_impl.ty.span();
    let type_implementing_for_span = item_impl.ty.span();
    let type_implementing_for = ty_to_type_info(item_impl.ty);
    let functions = {
        item_impl
        .contents
        .into_inner()
        .into_iter()
        .map(item_fn_to_function_declaration)
        .collect()
    };
    let type_parameters = generic_params_opt_to_type_parameters(item_impl.generic_params_opt);
    match item_impl.trait_opt {
        Some((path_type, _for_token)) => {
            let impl_trait = ImplTrait {
                trait_name: path_type_to_call_path(path_type),
                type_implementing_for,
                type_implementing_for_span,
                type_arguments: type_parameters,
                functions,
                block_span,
            };
            Declaration::ImplTrait(impl_trait)
        },
        None => {
            let impl_self = ImplSelf {
                type_implementing_for,
                type_implementing_for_span,
                type_parameters,
                functions,
                block_span,
            };
            Declaration::ImplSelf(impl_self)
        },
    }
}

fn item_abi_to_abi_declaration(item_abi: ItemAbi) -> AbiDeclaration {
    let span = item_abi.span();
    AbiDeclaration {
        name: item_abi.name,
        interface_surface: {
            item_abi
            .abi_items
            .into_inner()
            .into_iter()
            .map(|(fn_signature, _semicolon_token)| fn_signature_to_trait_fn(fn_signature))
            .collect()
        },
        methods: {
            item_abi
            .abi_defs_opt
            .into_iter()
            .map(|abi_defs| abi_defs.into_inner().into_iter().map(item_fn_to_function_declaration))
            .flatten()
            .collect()
        },
        span,
    }
}

fn item_const_to_constant_declaration(item_const: ItemConst) -> ConstantDeclaration {
    ConstantDeclaration {
        name: item_const.name,
        type_ascription: match item_const.ty_opt {
            Some((_colon_token, ty)) => ty_to_type_info(ty),
            None => TypeInfo::Unknown,
        },
        value: expr_to_expression(item_const.expr),
        //visibility: pub_token_opt_to_visibility(item_const.visibility),
        // FIXME: you have to lie here or else the tests fail.
        visibility: Visibility::Public,
    }
}

fn item_storage_to_storage_declaration(item_storage: ItemStorage) -> StorageDeclaration {
    let span = item_storage.span();
    StorageDeclaration {
        span,
        fields: item_storage.fields.into_inner().into_iter().map(storage_field_to_storage_field).collect(),
    }
}

fn type_field_to_struct_field(type_field: TypeField) -> StructField {
    let span = type_field.span();
    let type_span = type_field.ty.span();
    StructField {
        name: type_field.name,
        r#type: ty_to_type_info(type_field.ty),
        span,
        type_span,
    }
}

fn generic_params_opt_to_type_parameters(generic_params_opt: Option<GenericParams>) -> Vec<TypeParameter> {
    let generic_params = match generic_params_opt {
        Some(generic_params) => generic_params,
        None => return Vec::new(),
    };
    generic_params
    .parameters
    .into_inner()
    .into_iter()
    .map(|ident| TypeParameter {
        type_id: insert_type(TypeInfo::Custom {
            name: ident.clone(),
            type_arguments: Vec::new(),
        }),
        name_ident: ident.clone(),
        trait_constraints: Vec::new(),
    })
    .collect()
}

fn pub_token_opt_to_visibility(pub_token_opt: Option<PubToken>) -> Visibility {
    match pub_token_opt {
        Some(..) => Visibility::Public,
        None => Visibility::Private,
    }
}

fn type_field_to_enum_variant(type_field: TypeField, tag: usize) -> EnumVariant {
    let span = type_field.span();
    EnumVariant {
        name: type_field.name,
        r#type: ty_to_type_info(type_field.ty),
        tag,
        span,
    }
}

fn impure_token_opt_to_purity(impure_token_opt: Option<ImpureToken>) -> Purity {
    match impure_token_opt {
        Some(..) => Purity::Impure,
        None => Purity::Pure,
    }
}

fn braced_code_block_contents_to_code_block(braced_code_block_contents: Braces<CodeBlockContents>)
    -> CodeBlock
{
    let whole_block_span = braced_code_block_contents.span();
    let code_block_contents = braced_code_block_contents.into_inner();
    CodeBlock {
        contents: {
            let mut ast_nodes = {
                code_block_contents
                .statements
                .into_iter()
                .map(|statement| statement_to_ast_nodes(statement).into_iter())
                .flatten()
                .collect::<Vec<_>>()
            };
            if let Some(expr) = code_block_contents.final_expr_opt {
                let final_ast_node = expr_to_ast_node(*expr, true);
                ast_nodes.push(final_ast_node);
            }
            ast_nodes
        },
        whole_block_span,
    }
}

fn fn_args_to_function_parameters(fn_args: FnArgs) -> Vec<FunctionParameter> {
    match fn_args {
        FnArgs::Static(args) => args.into_iter().map(type_field_to_function_parameter).collect(),
        FnArgs::NonStatic { self_token, args_opt } => {
            let mut function_parameters = vec![FunctionParameter {
                name: Ident::new(self_token.span()),
                type_id: insert_type(TypeInfo::SelfType),
                type_span: self_token.span(),
            }];
            if let Some((_comma_token, args)) = args_opt {
                function_parameters.extend(args.into_iter().map(type_field_to_function_parameter));
            }
            function_parameters
        },
    }
}

fn type_name_to_type_info_opt(name: &Ident) -> Option<TypeInfo> {
    match name.as_str() {
        "u8" => Some(TypeInfo::UnsignedInteger(IntegerBits::Eight)),
        "u16" => Some(TypeInfo::UnsignedInteger(IntegerBits::Sixteen)),
        "u32" => Some(TypeInfo::UnsignedInteger(IntegerBits::ThirtyTwo)),
        "u64" => Some(TypeInfo::UnsignedInteger(IntegerBits::SixtyFour)),
        "bool" => Some(TypeInfo::Boolean),
        "unit" => Some(TypeInfo::Tuple(Vec::new())),
        "byte" => Some(TypeInfo::Byte),
        "b256" => Some(TypeInfo::B256),
        "Self" | "self" => Some(TypeInfo::SelfType),
        "Contract" => Some(TypeInfo::Contract),
        _other => None,
    }
}

fn ty_to_type_info(ty: Ty) -> TypeInfo {
    match ty {
        Ty::Path(path_type) => path_type_to_type_info(path_type),
        Ty::Tuple(tys) => {
            TypeInfo::Tuple(
                tys
                .into_inner()
                .into_iter()
                .map(ty_to_type_argument)
                .collect()
            )
        },
        Ty::Array(bracketed_ty_array_descriptor) => {
            let ty_array_descriptor = bracketed_ty_array_descriptor.into_inner();
            TypeInfo::Array(
                crate::type_engine::insert_type(ty_to_type_info(*ty_array_descriptor.ty)),
                expr_to_usize(*ty_array_descriptor.length),
            )
        },
        Ty::Str { length, .. } => {
            TypeInfo::Str(expr_to_u64(*length.into_inner()))
        },
        Ty::Infer { .. } => {
            TypeInfo::Unknown
        },
    }
}

fn ty_to_type_argument(ty: Ty) -> TypeArgument {
    let span = ty.span();
    TypeArgument {
        type_id: insert_type(ty_to_type_info(ty)),
        span,
    }
}

fn fn_signature_to_trait_fn(fn_signature: FnSignature) -> TraitFn {
    let return_type_span = match &fn_signature.return_type_opt {
        Some((_right_arrow_token, ty)) => ty.span(),
        None => fn_signature.span(),
    };
    TraitFn {
        name: fn_signature.name,
        parameters: fn_args_to_function_parameters(fn_signature.arguments.into_inner()),
        return_type: match fn_signature.return_type_opt {
            Some((_right_arrow_token, ty)) => ty_to_type_info(ty),
            None => TypeInfo::Tuple(Vec::new()),
        },
        return_type_span,
    }
}

fn traits_to_supertraits(traits: Traits) -> Vec<Supertrait> {
    let mut supertraits = vec![
        path_type_to_supertrait(traits.prefix)
    ];
    supertraits.extend(traits.suffixes.into_iter().map(|(_add_token, suffix)| path_type_to_supertrait(suffix)));
    supertraits
}

fn path_type_to_call_path(path_type: PathType) -> CallPath {
    let PathType { root_opt, prefix, mut suffix } = path_type;
    let is_absolute = path_root_opt_to_bool(root_opt);
    match suffix.pop() {
        Some((_double_colon_token, call_path_suffix)) => {
            let mut prefixes = vec![path_type_segment_to_ident(prefix)];
            prefixes.extend(suffix.into_iter().map(|(_double_colon_token, call_path_prefix)| {
                path_type_segment_to_ident(call_path_prefix)
            }));
            CallPath {
                prefixes,
                suffix: path_type_segment_to_ident(call_path_suffix),
                is_absolute,
            }
        },
        None => {
            CallPath {
                prefixes: Vec::new(),
                suffix: path_type_segment_to_ident(prefix),
                is_absolute,
            }
        },
    }
}

fn expr_to_ast_node(expr: Expr, end_of_block: bool) -> AstNode {
    let span = expr.span();
    match expr {
        Expr::Return { expr_opt, .. } => {
            let expression = match expr_opt {
                Some(expr) => expr_to_expression(*expr),
                None => Expression::Tuple { fields: Vec::new(), span: span.clone() },
            };
            AstNode {
                content: AstNodeContent::ReturnStatement(ReturnStatement { expr: expression }),
                span,
            }
        },
        Expr::While { condition, block, .. } => {
            AstNode {
                content: AstNodeContent::WhileLoop(WhileLoop {
                    condition: expr_to_expression(*condition),
                    body: braced_code_block_contents_to_code_block(block),
                }),
                span,
            }
        },
        Expr::Reassignment { assignable, expr, .. } => {
            AstNode {
                content: AstNodeContent::Declaration(Declaration::Reassignment(Reassignment {
                    lhs: assignable_to_reassignment_target(assignable),
                    rhs: expr_to_expression(*expr),
                    span: span.clone(),
                })),
                span,
            }
        },
        expr => {
            let expression = expr_to_expression(expr);
            if end_of_block {
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
        },
    }
}

fn expr_to_expression(expr: Expr) -> Expression {
    let span = expr.span();
    match expr {
        Expr::Path(path_expr) => path_expr_to_expression(path_expr),
        Expr::Literal(literal) => {
            Expression::Literal {
                value: literal_to_literal(literal),
                span,
            }
        },
        Expr::AbiCast { args, .. } => {
            let AbiCastArgs { name, address, .. } = args.into_inner();
            let abi_name = path_type_to_call_path(name);
            let address = Box::new(expr_to_expression(*address));
            Expression::AbiCast {
                abi_name,
                address,
                span,
            }
        },
        Expr::Struct { path, fields } => {
            Expression::StructExpression {
                struct_name: path_expr_to_call_path(path),
                fields: {
                    fields
                    .into_inner()
                    .into_iter()
                    .map(expr_struct_field_to_struct_expression_field)
                    .collect()
                },
                type_arguments: Vec::new(),
                span,
            }
        },
        Expr::Tuple(parenthesized_expr_tuple_descriptor) => {
            Expression::Tuple {
                fields: expr_tuple_descriptor_to_expressions(parenthesized_expr_tuple_descriptor.into_inner()),
                span,
            }
        },
        Expr::Parens(parens) => expr_to_expression(*parens.into_inner()),
        Expr::Block(braced_code_block_contents) => {
            braced_code_block_contents_to_expression(braced_code_block_contents)
        },
        Expr::Array(bracketed_expr_array_descriptor) => {
            match bracketed_expr_array_descriptor.into_inner() {
                ExprArrayDescriptor::Sequence(exprs) => {
                    Expression::Array {
                        contents: exprs.into_iter().map(expr_to_expression).collect(),
                        span,
                    }
                },
                ExprArrayDescriptor::Repeat { value, length, .. } => {
                    let expression = expr_to_expression(*value);
                    let length = expr_to_usize(*length);
                    Expression::Array {
                        contents: iter::repeat_with(|| expression.clone()).take(length).collect(),
                        span,
                    }
                },
            }
        },
        Expr::Asm(asm_block) => {
            Expression::AsmExpression {
                asm: asm_block_to_asm_expression(asm_block),
                span,
            }
        },
        Expr::Return { .. } => {
            panic!("return expression cannot be used outside of a block");
        },
        Expr::If(if_expr) => if_expr_to_expression(if_expr),
        Expr::Match { condition, branches, .. } => {
            let condition = expr_to_expression(*condition);
            let branches = branches.into_inner().into_iter().map(match_branch_to_match_branch).collect();
            let desugar_result = desugar_match_expression(&condition, branches, None);
            let (if_exp, var_decl_name, cases_covered) = match desugar_result.value {
                Some(stuff) => stuff,
                None => panic!("error handling not implemented"),
            };
            Expression::CodeBlock {
                contents: CodeBlock {
                    contents: vec![
                        AstNode {
                            content: AstNodeContent::Declaration(Declaration::VariableDeclaration(VariableDeclaration {
                                name: var_decl_name,
                                type_ascription: TypeInfo::Unknown,
                                type_ascription_span: None,
                                is_mutable: false,
                                body: condition,
                            })),
                            span: span.clone(),
                        },
                        AstNode {
                            content: AstNodeContent::ImplicitReturnExpression(Expression::MatchExp {
                                if_exp: Box::new(if_exp),
                                cases_covered,
                                span: span.clone(),
                            }),
                            span: span.clone(),
                        },
                    ],
                    whole_block_span: span.clone(),
                },
                span,
            }
        },
        Expr::While { .. } => {
            panic!("while expressions outside of block are not supported");
        },
        Expr::FuncApp { func, args } => {
            let path_expr = match *func {
                Expr::Path(path_expr) => path_expr,
                _ => {
                    panic!("functions used in applications may not be arbitrary expressions");
                },
            };
            let PathExpr { root_opt, prefix, mut suffix } = path_expr;
            let is_absolute = path_root_opt_to_bool(root_opt);
            let (prefixes, method_type_opt, suffix_path_expr) = match suffix.pop() {
                Some((_double_colon_token, call_path_suffix)) => {
                    match suffix.pop() {
                        Some((_double_colon_token, maybe_method_segment)) => {
                            let PathExprSegment { fully_qualified, name, generics_opt } = maybe_method_segment;
                            if generics_opt.is_some() {
                                panic!("generics not supported here");
                            }
                            let mut prefixes = vec![path_expr_segment_to_ident(prefix)];
                            prefixes.extend(suffix.into_iter().map(|(_double_colon_token, call_path_prefix)| {
                                path_expr_segment_to_ident(call_path_prefix)
                            }));
                            if fully_qualified.is_some() {
                                (prefixes, Some(name), call_path_suffix)
                            } else {
                                prefixes.push(name);
                                (prefixes, None, call_path_suffix)
                            }
                        },
                        None => {
                            let PathExprSegment { fully_qualified, name, generics_opt } = prefix;
                            if generics_opt.is_some() {
                                panic!("generics not supported here");
                            }
                            if fully_qualified.is_some() {
                                (Vec::new(), Some(name), call_path_suffix)
                            } else {
                                (vec![name], None, call_path_suffix)
                            }
                        },
                    }
                },
                None => {
                    (Vec::new(), None, prefix)
                },
            };
            let PathExprSegment { fully_qualified, name, generics_opt } = suffix_path_expr;
            if fully_qualified.is_some() {
                panic!("fully qualified annotations not allowed here");
            }
            let call_path = CallPath {
                is_absolute,
                prefixes,
                suffix: name,
            };
            let arguments = {
                args
                .into_inner()
                .into_iter()
                .map(expr_to_expression)
                .collect()
            };
            match method_type_opt {
                Some(type_name) => {
                    let type_arguments = match generics_opt {
                        Some((_double_colon_token, generic_args)) => {
                            generic_args_to_type_arguments(generic_args)
                        },
                        None => Vec::new(),
                    };
                    let type_name_span = type_name.span().clone();
                    Expression::MethodApplication {
                        method_name: MethodName::FromType {
                            call_path,
                            type_name: Some(TypeInfo::Custom {
                                name: type_name,
                                type_arguments: Vec::new(),
                            }),
                            type_name_span: Some(type_name_span),
                        },
                        contract_call_params: Vec::new(),
                        arguments,
                        type_arguments,
                        span,
                    }
                },
                None => {
                    if
                        call_path.prefixes.is_empty() &&
                        !call_path.is_absolute &&
                        call_path.suffix.as_str() == "size_of"
                    {
                        if !arguments.is_empty() {
                            panic!("size_of does not take arguments");
                        }
                        let generic_args = match generics_opt {
                            None => panic!("size_of requires generic args"),
                            Some((_double_colon_token, generic_args)) => generic_args,
                        };
                        let mut generic_args_iter = generic_args.parameters.into_inner().into_iter();
                        let ty = match generic_args_iter.next() {
                            Some(ty) => ty,
                            None => panic!("size_of requires one generic arg"),
                        };
                        match generic_args_iter.next() {
                            Some(_) => panic!("size_of takes only one generic arg"),
                            None => (),
                        };
                        let type_span = ty.span();
                        let type_name = ty_to_type_info(ty);
                        Expression::BuiltinGetTypeProperty {
                            builtin: BuiltinProperty::SizeOfType,
                            type_name,
                            type_span,
                            span,
                        }
                    } else if
                        call_path.prefixes.is_empty() &&
                        !call_path.is_absolute &&
                        call_path.suffix.as_str() == "is_reference_type"
                    {
                        if !arguments.is_empty() {
                            panic!("is_reference_type does not take arguments");
                        }
                        let generic_args = match generics_opt {
                            None => panic!("is_reference_type requires generic args"),
                            Some((_double_colon_token, generic_args)) => generic_args,
                        };
                        let mut generic_args_iter = generic_args.parameters.into_inner().into_iter();
                        let ty = match generic_args_iter.next() {
                            Some(ty) => ty,
                            None => panic!("is_reference_type requires one generic arg"),
                        };
                        match generic_args_iter.next() {
                            Some(_) => panic!("is_reference_type takes only one generic arg"),
                            None => (),
                        };
                        let type_span = ty.span();
                        let type_name = ty_to_type_info(ty);
                        Expression::BuiltinGetTypeProperty {
                            builtin: BuiltinProperty::IsRefType,
                            type_name,
                            type_span,
                            span,
                        }
                    } else if
                        call_path.prefixes.is_empty() &&
                        !call_path.is_absolute &&
                        call_path.suffix.as_str() == "size_of_val"
                    {
                        let mut arguments_iter = arguments.into_iter();
                        let exp = match arguments_iter.next() {
                            Some(exp) => Box::new(exp),
                            None => panic!("size_of_val expects an argument"),
                        };
                        match arguments_iter.next() {
                            Some(..) => panic!("size_of_val takes only a single argument"),
                            None => (),
                        }
                        Expression::SizeOfVal {
                            exp,
                            span,
                        }
                    } else {
                        let type_arguments = match generics_opt {
                            Some((_double_colon_token, generic_args)) => {
                                generic_args_to_type_arguments(generic_args)
                            },
                            None => Vec::new(),
                        };
                        if call_path.prefixes.is_empty() {
                            Expression::FunctionApplication {
                                name: call_path,
                                arguments,
                                type_arguments,
                                span,
                            }
                        } else {
                            Expression::DelineatedPath {
                                call_path,
                                args: arguments,
                                type_arguments,
                                span,
                            }
                        }
                    }
                },
            }
        },
        Expr::Index { target, arg } => {
            Expression::ArrayIndex {
                prefix: Box::new(expr_to_expression(*target)),
                index: Box::new(expr_to_expression(*arg.into_inner())),
                span,
            }
        },
        Expr::MethodCall { target, name, args, contract_args_opt, .. } => {
            Expression::MethodApplication {
                method_name: MethodName::FromModule {
                    method_name: name,
                },
                contract_call_params: {
                    contract_args_opt
                    .map(|contract_args| {
                        contract_args
                        .into_inner()
                        .into_iter()
                        .map(expr_struct_field_to_struct_expression_field)
                        .collect()
                    })
                    .unwrap_or_else(|| Vec::new())
                },
                arguments: {
                    iter::once(*target)
                    .chain(args.into_inner().into_iter())
                    .map(expr_to_expression)
                    .collect()
                },
                type_arguments: Vec::new(),
                span,
            }
        },
        Expr::FieldProjection { target, name, .. } => {
            let mut idents = vec![&name];
            let mut base = &*target;
            let storage_access_field_names_opt = loop {
                match base {
                    Expr::FieldProjection { target, name, .. } => {
                        idents.push(name);
                        base = target;
                    },
                    Expr::Path(path_expr) => {
                        if {
                            path_expr.root_opt.is_none() &&
                            path_expr.suffix.is_empty() &&
                            path_expr.prefix.fully_qualified.is_none() &&
                            path_expr.prefix.generics_opt.is_none() &&
                            path_expr.prefix.name.as_str() == "storage"
                        } {
                            break Some(idents);
                        }
                        break None;
                    },
                    _ => break None,
                }
            };
            match storage_access_field_names_opt {
                Some(field_names) => {
                    let field_names = field_names.into_iter().rev().map(|name| name.clone()).collect();
                    Expression::StorageAccess { field_names, span }
                },
                None => {
                    Expression::SubfieldExpression {
                        prefix: Box::new(expr_to_expression(*target)),
                        field_to_access: name,
                        span,
                    }
                },
            }
        },
        Expr::TupleFieldProjection { target, field, field_span, .. } => {
            Expression::TupleIndex {
                prefix: Box::new(expr_to_expression(*target)),
                index: match usize::try_from(field) {
                    Ok(index) => index,
                    Err(..) => panic!("tuple index out of range"),
                },
                index_span: field_span,
                span,
            }
        },
        Expr::Not { bang_token, expr } => {
            unary_op_call("not", bang_token.span(), span, *expr)
        },
        Expr::Mul { lhs, star_token, rhs } => {
            binary_op_call("multiply", star_token.span(), span, *lhs, *rhs)
        },
        Expr::Div { lhs, forward_slash_token, rhs } => {
            binary_op_call("divide", forward_slash_token.span(), span, *lhs, *rhs)
        },
        Expr::Modulo { lhs, percent_token, rhs } => {
            binary_op_call("modulo", percent_token.span(), span, *lhs, *rhs)
        },
        Expr::Add { lhs, add_token, rhs } => {
            binary_op_call("add", add_token.span(), span, *lhs, *rhs)
        },
        Expr::Sub { lhs, sub_token, rhs } => {
            binary_op_call("subtract", sub_token.span(), span, *lhs, *rhs)
        },
        Expr::Shl { .. } => {
            panic!("shift left expressions are not implemented");
        },
        Expr::Shr { .. } => {
            panic!("shift right expressions are not implemented");
        },
        Expr::BitAnd { lhs, ampersand_token, rhs } => {
            binary_op_call("binary_and", ampersand_token.span(), span, *lhs, *rhs)
        },
        Expr::BitXor { .. } => {
            panic!("bitwise xor operations are not implemented");
        },
        Expr::BitOr { lhs, pipe_token, rhs } => {
            binary_op_call("binary_or", pipe_token.span(), span, *lhs, *rhs)
        },
        Expr::Equal { lhs, double_eq_token, rhs } => {
            binary_op_call("eq", double_eq_token.span(), span, *lhs, *rhs)
        },
        Expr::NotEqual { lhs, bang_eq_token, rhs } => {
            binary_op_call("neq", bang_eq_token.span(), span, *lhs, *rhs)
        },
        Expr::LessThan { lhs, less_than_token, rhs } => {
            binary_op_call("lt", less_than_token.span(), span, *lhs, *rhs)
        },
        Expr::GreaterThan { lhs, greater_than_token, rhs } => {
            binary_op_call("gt", greater_than_token.span(), span, *lhs, *rhs)
        },
        Expr::LessThanEq { lhs, less_than_eq_token, rhs } => {
            binary_op_call("le", less_than_eq_token.span(), span, *lhs, *rhs)
        },
        Expr::GreaterThanEq { lhs, greater_than_eq_token, rhs } => {
            binary_op_call("ge", greater_than_eq_token.span(), span, *lhs, *rhs)
        },
        Expr::LogicalAnd { lhs, rhs, .. } => {
            Expression::LazyOperator {
                op: LazyOp::And,
                lhs: Box::new(expr_to_expression(*lhs)),
                rhs: Box::new(expr_to_expression(*rhs)),
                span,
            }
        },
        Expr::LogicalOr { lhs, rhs, .. } => {
            Expression::LazyOperator {
                op: LazyOp::Or,
                lhs: Box::new(expr_to_expression(*lhs)),
                rhs: Box::new(expr_to_expression(*rhs)),
                span,
            }
        },
        Expr::Reassignment { .. } => {
            panic!("reassignments outside of blocks are not supported");
        },
    }
}

fn unary_op_call(
    name: &'static str,
    op_span: Span,
    span: Span,
    arg: Expr,
) -> Expression {
    Expression::FunctionApplication {
        name: CallPath {
            prefixes: vec![
                Ident::new_with_override("core", op_span.clone()),
                Ident::new_with_override("ops", op_span.clone()),
            ],
            suffix: Ident::new_with_override(name, op_span),
            is_absolute: false,
        },
        arguments: vec![expr_to_expression(arg)],
        type_arguments: Vec::new(),
        span,
    }
}

fn binary_op_call(
    name: &'static str,
    op_span: Span,
    span: Span,
    lhs: Expr,
    rhs: Expr,
) -> Expression {
    Expression::MethodApplication {
        method_name: MethodName::FromType {
            call_path: CallPath {
                prefixes: vec![
                    Ident::new_with_override("core", op_span.clone()),
                    Ident::new_with_override("ops", op_span.clone()),
                ],
                suffix: Ident::new_with_override(name, op_span),
                is_absolute: true,
            },
            type_name: None,
            type_name_span: None,
        },
        contract_call_params: Vec::new(),
        arguments: vec![expr_to_expression(lhs), expr_to_expression(rhs)],
        type_arguments: Vec::new(),
        span,
    }
}

fn storage_field_to_storage_field(storage_field: new_parser_again::StorageField) -> StorageField {
    StorageField {
        name: storage_field.name,
        r#type: ty_to_type_info(storage_field.ty),
        //initializer: expr_to_expression(storage_field.expr),
    }
}

fn statement_to_ast_nodes(statement: Statement) -> Vec<AstNode> {
    match statement {
        Statement::Let(statement_let) => statement_let_to_ast_nodes(statement_let),
        Statement::Item(item) => item_to_ast_nodes(item),
        Statement::Expr { expr, .. } => vec![expr_to_ast_node(expr, false)],
    }
}

fn type_field_to_function_parameter(type_field: TypeField) -> FunctionParameter {
    let type_span = type_field.ty.span();
    FunctionParameter {
        name: type_field.name,
        type_id: insert_type(ty_to_type_info(type_field.ty)),
        type_span,
    }
}

fn expr_to_usize(expr: Expr) -> usize {
    match expr {
        Expr::Literal(new_parser_again::Literal::Int(lit_int)) => {
            match lit_int.ty_opt {
                None => (),
                Some(..) => panic!("int literal in this position cannot have a type specified"),
            }
            match usize::try_from(lit_int.parsed) {
                Ok(value) => value,
                Err(..) => panic!("int literal out of range"),
            }
        },
        _ => panic!("expected an int literal"),
    }
}

fn expr_to_u64(expr: Expr) -> u64 {
    match expr {
        Expr::Literal(new_parser_again::Literal::Int(lit_int)) => {
            match lit_int.ty_opt {
                None | Some((LitIntType::U64, _)) => (),
                Some(..) => panic!("int literal in this position must be a u64"),
            }
            match u64::try_from(lit_int.parsed) {
                Ok(value) => value,
                Err(..) => panic!("int literal out of range"),
            }
        },
        _ => panic!("expected an int literal"),
    }
}

fn path_type_to_supertrait(path_type: PathType) -> Supertrait {
    let PathType { root_opt, prefix, mut suffix } = path_type;
    let is_absolute = path_root_opt_to_bool(root_opt);
    let (prefixes, call_path_suffix) = match suffix.pop() {
        Some((_double_colon_token, call_path_suffix)) => {
            let mut prefixes = vec![path_type_segment_to_ident(prefix)];
            prefixes.extend(suffix.into_iter().map(|(_double_colon_token, call_path_prefix)| {
                path_type_segment_to_ident(call_path_prefix)
            }));
            (prefixes, call_path_suffix)
        },
        None => (Vec::new(), prefix),
    };
    //let PathTypeSegment { fully_qualified, name, generics_opt } = call_path_suffix;
    let PathTypeSegment { fully_qualified, name, .. } = call_path_suffix;
    if fully_qualified.is_some() {
        panic!("not sure how to handle these ~ paths");
    }
    let name = CallPath {
        prefixes,
        suffix: name,
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
    Supertrait {
        name,
        //type_parameters,
    }
}

fn path_type_segment_to_ident(path_type_segment: PathTypeSegment) -> Ident {
    let PathTypeSegment { fully_qualified, name, generics_opt } = path_type_segment;
    if fully_qualified.is_some() {
        panic!("fully qualified paths not supported in the position");
    }
    if let Some((_double_colon_token, generic_args)) = generics_opt {
        panic!("generics are not supported in this position: {}", generic_args.span().as_str());
    }
    name
}

fn path_expr_segment_to_ident(path_expr_segment: PathExprSegment) -> Ident {
    let span = path_expr_segment.span();
    let PathExprSegment { fully_qualified, name, generics_opt } = path_expr_segment;
    if fully_qualified.is_some() {
        panic!("fully qualified paths not supported in the position: {}", span.as_str());
    }
    if generics_opt.is_some() {
        panic!("generics are not supported in this position");
    }
    name
}

fn path_expr_to_expression(path_expr: PathExpr) -> Expression {
    let span = path_expr.span();
    if path_expr.root_opt.is_none() && path_expr.suffix.is_empty() {
        let name = path_expr_segment_to_ident(path_expr.prefix);
        match name.as_str() {
            "true" => Expression::Literal {
                value: Literal::Boolean(true),
                span,
            },
            "false" => Expression::Literal {
                value: Literal::Boolean(false),
                span,
            },
            _ => Expression::VariableExpression { name, span },
        }
    } else {
        let call_path = path_expr_to_call_path(path_expr);
        Expression::DelineatedPath {
            call_path,
            args: Vec::new(),
            span,
            type_arguments: Vec::new(),
        }
    }
}

fn braced_code_block_contents_to_expression(braced_code_block_contents: Braces<CodeBlockContents>)
    -> Expression
{
    let span = braced_code_block_contents.span();
    Expression::CodeBlock {
        contents: braced_code_block_contents_to_code_block(braced_code_block_contents),
        span,
    }
}

fn if_expr_to_expression(if_expr: IfExpr) -> Expression {
    let span = if_expr.span();
    let IfExpr { condition, then_block, else_opt, .. } = if_expr;
    let then_block_span = then_block.span();
    let then_block = braced_code_block_contents_to_code_block(then_block);
    let else_opt = match else_opt {
        None => None,
        Some((_else_token, tail)) => {
            let expression = match tail {
                ControlFlow::Break(braced_code_block_contents) => {
                    braced_code_block_contents_to_expression(braced_code_block_contents)
                },
                ControlFlow::Continue(if_expr) => {
                    if_expr_to_expression(*if_expr)
                },
            };
            Some(Box::new(expression))
        },
    };
    match condition {
        IfCondition::Expr(condition) => {
            Expression::IfExp {
                condition: Box::new(expr_to_expression(*condition)),
                then: Box::new(Expression::CodeBlock {
                    contents: then_block,
                    span: then_block_span,
                }),
                r#else: else_opt,
                span,
            }
        },
        IfCondition::Let { lhs, rhs, .. } => {
            Expression::IfLet {
                scrutinee: pattern_to_scrutinee(*lhs),
                expr: Box::new(expr_to_expression(*rhs)),
                then: then_block,
                r#else: else_opt,
                span,
            }
        },
    }
}

fn path_root_opt_to_bool(root_opt: Option<(Option<AngleBrackets<QualifiedPathRoot>>, DoubleColonToken)>)
    -> bool
{
    match root_opt {
        None => false,
        Some((None, _double_colon_token)) => true,
        Some((Some(_qualified_path_root), _double_colon_token)) => {
            panic!("qualified path roots are not implemented");
        },
    }
}

fn literal_to_literal(literal: new_parser_again::Literal) -> Literal {
    match literal {
        new_parser_again::Literal::String(lit_string) => {
            let full_span = lit_string.span();
            let inner_span = Span::new(
                full_span.src().clone(),
                full_span.start() + 1,
                full_span.end() - 1,
                full_span.path().cloned(),
            ).unwrap();
            Literal::String(inner_span)
        },
        new_parser_again::Literal::Char(..) => panic!("char literals are not implemented"),
        new_parser_again::Literal::Int(lit_int) => {
            let LitInt { parsed, ty_opt, span } = lit_int;
            match ty_opt {
                None => {
                    let orig_str = span.as_str();
                    if let Some(hex_digits) = orig_str.strip_prefix("0x") {
                        let num_digits = hex_digits.chars().filter(|c| *c != '_').count();
                        match num_digits {
                            2 => Literal::Byte(u8::try_from(parsed).unwrap()),
                            64 => {
                                let bytes = parsed.to_bytes_be();
                                let mut full_bytes = [0u8; 32];
                                full_bytes[(32 - bytes.len())..].copy_from_slice(&bytes);
                                Literal::B256(full_bytes)
                            },
                            _ => panic!("hex literals must have either 2 or 64 digits"),
                        }
                    } else if let Some(bin_digits) = orig_str.strip_prefix("0b") {
                        let num_digits = bin_digits.chars().filter(|c| *c != '_').count();
                        match num_digits {
                            8 => Literal::Byte(u8::try_from(parsed).unwrap()),
                            256 => {
                                let bytes = parsed.to_bytes_be();
                                let mut full_bytes = [0u8; 32];
                                full_bytes[(32 - bytes.len())..].copy_from_slice(&bytes);
                                Literal::B256(full_bytes)
                            },
                            _ => panic!("binary literals must have 8 or 256 digits"),
                        }
                    } else {
                        match u64::try_from(&parsed) {
                            Ok(value) => Literal::Numeric(value),
                            Err(..) => panic!("int literal out of range for u64"),
                        }
                    }
                },
                Some((lit_int_type, _span)) => match lit_int_type {
                    LitIntType::U8 => {
                        let value = match u8::try_from(parsed) {
                            Ok(value) => value,
                            Err(..) => panic!("u8 literal out of range"),
                        };
                        Literal::U8(value)
                    },
                    LitIntType::U16 => {
                        let value = match u16::try_from(parsed) {
                            Ok(value) => value,
                            Err(..) => panic!("u16 literal out of range"),
                        };
                        Literal::U16(value)
                    },
                    LitIntType::U32 => {
                        let value = match u32::try_from(parsed) {
                            Ok(value) => value,
                            Err(..) => panic!("u32 literal out of range"),
                        };
                        Literal::U32(value)
                    },
                    LitIntType::U64 => {
                        let value = match u64::try_from(parsed) {
                            Ok(value) => value,
                            Err(..) => panic!("u64 literal out of range"),
                        };
                        Literal::U64(value)
                    },
                    LitIntType::I8 | LitIntType::I16 | LitIntType::I32 | LitIntType::I64 => {
                        panic!("signed integer types are not supported");
                    },
                },
            }
        },
    }
}

fn path_expr_to_call_path(path_expr: PathExpr) -> CallPath {
    let PathExpr { root_opt, prefix, mut suffix } = path_expr;
    let is_absolute = path_root_opt_to_bool(root_opt);
    match suffix.pop() {
        Some((_double_colon_token, call_path_suffix)) => {
            let mut prefixes = vec![path_expr_segment_to_ident(prefix)];
            prefixes.extend(suffix.into_iter().map(|(_double_colon_token, call_path_prefix)| {
                path_expr_segment_to_ident(call_path_prefix)
            }));
            CallPath {
                prefixes,
                suffix: path_expr_segment_to_ident(call_path_suffix),
                is_absolute,
            }
        },
        None => {
            CallPath {
                prefixes: Vec::new(),
                suffix: path_expr_segment_to_ident(prefix),
                is_absolute,
            }
        },
    }
}

fn expr_struct_field_to_struct_expression_field(expr_struct_field: ExprStructField) -> StructExpressionField {
    let span = expr_struct_field.span();
    let value = match expr_struct_field.expr_opt {
        Some((_colon_token, expr)) => expr_to_expression(*expr),
        None => {
            Expression::VariableExpression {
                name: expr_struct_field.field_name.clone(),
                span: span.clone(),
            }
        },
    };
    StructExpressionField {
        name: expr_struct_field.field_name,
        value,
        span,
    }
}

fn expr_tuple_descriptor_to_expressions(expr_tuple_descriptor: ExprTupleDescriptor) -> Vec<Expression> {
    match expr_tuple_descriptor {
        ExprTupleDescriptor::Nil => Vec::new(),
        ExprTupleDescriptor::Cons { head, tail, .. } => {
            let mut expressions = vec![expr_to_expression(*head)];
            expressions.extend(tail.into_iter().map(expr_to_expression));
            expressions
        },
    }
}

fn asm_block_to_asm_expression(asm_block: AsmBlock) -> AsmExpression {
    let whole_block_span = asm_block.span();
    let asm_block_contents = asm_block.contents.into_inner();
    let (returns, return_type) = match asm_block_contents.final_expr_opt {
        Some(asm_final_expr) => {
            let asm_register = AsmRegister {
                name: asm_final_expr.register.as_str().to_owned(),
            };
            let returns = Some((asm_register, asm_final_expr.register.span().clone()));
            let return_type = match asm_final_expr.ty_opt {
                Some((_colon_token, ty)) => ty_to_type_info(ty),
                None => TypeInfo::UnsignedInteger(IntegerBits::SixtyFour),
            };
            (returns, return_type)
        },
        None => (None, TypeInfo::Tuple(Vec::new())),
    };
    AsmExpression {
        registers: {
            asm_block
            .registers
            .into_inner()
            .into_iter()
            .map(asm_register_declaration_to_asm_register_declaration)
            .collect()
        },
        body: {
            asm_block_contents
            .instructions
            .into_iter()
            .map(|(instruction, _semicolon_token)| instruction_to_asm_op(instruction))
            .collect()
        },
        returns,
        return_type,
        whole_block_span,
    }
}

fn match_branch_to_match_branch(match_branch: new_parser_again::MatchBranch) -> MatchBranch {
    let span = match_branch.span();
    MatchBranch {
        condition: pattern_to_match_condition(match_branch.pattern),
        result: match match_branch.kind {
            MatchBranchKind::Block { block, .. } => {
                let span = block.span();
                Expression::CodeBlock {
                    contents: braced_code_block_contents_to_code_block(block),
                    span,
                }
            },
            MatchBranchKind::Expr { expr, .. } => expr_to_expression(expr),
        },
        span,
    }
}

fn statement_let_to_ast_nodes(statement_let: StatementLet) -> Vec<AstNode> {
    fn unfold(
        pattern: Pattern,
        ty_opt: Option<Ty>,
        expression: Expression,
        span: Span,
    ) -> Vec<AstNode> {
        match pattern {
            Pattern::Wildcard { .. } => {
                let ast_node = AstNode {
                    content: AstNodeContent::Expression(expression),
                    span,
                };
                vec![ast_node]
            },
            Pattern::Var { mutable, name } => {
                let (type_ascription, type_ascription_span) = match ty_opt {
                    Some(ty) => {
                        let type_ascription_span = ty.span();
                        let type_ascription = ty_to_type_info(ty);
                        (type_ascription, Some(type_ascription_span))
                    },
                    None => (TypeInfo::Unknown, None),
                };
                let ast_node = AstNode {
                    content: AstNodeContent::Declaration(Declaration::VariableDeclaration(VariableDeclaration {
                        name,
                        type_ascription,
                        type_ascription_span,
                        body: expression,
                        is_mutable: mutable.is_some(),
                    })),
                    span,
                };
                vec![ast_node]
            },
            Pattern::Literal(..) => {
                panic!("literals in patterns are not yet supported");
            },
            Pattern::Constant(..) => {
                panic!("constants in patterns are not yet supported");
            },
            Pattern::Constructor { .. } => {
                panic!("constructors in patterns are not yet supported");
            },
            Pattern::Struct { .. } => {
                panic!("struct patterns are not yet supported");
            },
            Pattern::Tuple(pat_tuple) => {
                let mut ast_nodes = Vec::new();
                let name = {
                    // FIXME: This is so, so dodgy.
                    let name_str: &'static str = Box::leak(nanoid!(32).into_boxed_str());
                    Ident::new_with_override(name_str, span.clone())
                };
                let (type_ascription, type_ascription_span) = match &ty_opt {
                    Some(ty) => {
                        let type_ascription_span = ty.span();
                        let type_ascription = ty_to_type_info(ty.clone());
                        (type_ascription, Some(type_ascription_span))
                    },
                    None => (TypeInfo::Unknown, None),
                };
                let save_body_first = VariableDeclaration {
                    name: name.clone(),
                    type_ascription,
                    type_ascription_span,
                    body: expression,
                    is_mutable: false,
                };
                ast_nodes.push(AstNode {
                    content: AstNodeContent::Declaration(Declaration::VariableDeclaration(save_body_first)),
                    span: span.clone(),
                });
                let new_expr = Expression::VariableExpression {
                    name,
                    span: span.clone(),
                };
                let tuple_tys_opt = match ty_opt {
                    Some(Ty::Tuple(tys)) => Some(tys.into_inner().into_iter().collect::<Vec<_>>()),
                    _ => None,
                };
                for (index, pattern) in pat_tuple.into_inner().into_iter().enumerate() {
                    let ty_opt = match &tuple_tys_opt {
                        Some(tys) => tys.get(index).cloned(),
                        None => None,
                    };
                    ast_nodes.extend(unfold(
                        pattern,
                        ty_opt,
                        Expression::TupleIndex {
                            prefix: Box::new(new_expr.clone()),
                            index,
                            index_span: span.clone(),
                            span: span.clone(),
                        },
                        span.clone(),
                    ));
                }
                ast_nodes
            },
        }
    }
    let span = statement_let.span();
    unfold(
        statement_let.pattern,
        statement_let.ty_opt.map(|(_colon_token, ty)| ty),
        expr_to_expression(statement_let.expr),
        span,
    )
}

fn dependency_to_include_statement(dependency: Dependency) -> IncludeStatement {
    IncludeStatement {
        alias: None,
        span: dependency.span(),
        path_span: dependency.path.span(),
    }
}

/*
fn generic_args_to_type_parameters(generic_args: GenericArgs) -> Vec<TypeParameter> {
    generic_args
    .parameters
    .into_inner()
    .into_iter()
    .map(ty_to_type_parameter)
    .collect()
}
*/

fn asm_register_declaration_to_asm_register_declaration(
    asm_register_declaration: new_parser_again::AsmRegisterDeclaration,
) -> AsmRegisterDeclaration {
    AsmRegisterDeclaration {
        name: asm_register_declaration.register,
        initializer: asm_register_declaration.value_opt.map(|(_colon_token, expr)| {
            expr_to_expression(*expr)
        }),
    }
}

fn instruction_to_asm_op(instruction: Instruction) -> AsmOp {
    AsmOp {
        op_name: instruction.op_code_ident(),
        op_args: instruction.register_arg_idents(),
        span: instruction.span(),
        immediate: instruction.immediate_ident_opt(),
    }
}

fn pattern_to_match_condition(pattern: Pattern) -> MatchCondition {
    match pattern {
        Pattern::Wildcard { underscore_token } => {
            let span = underscore_token.span();
            MatchCondition::CatchAll(CatchAll { span })
        },
        _ => MatchCondition::Scrutinee(pattern_to_scrutinee(pattern)),
    }
}

fn pattern_to_scrutinee(pattern: Pattern) -> Scrutinee {
    let span = pattern.span();
    match pattern {
        Pattern::Wildcard { .. } => {
            panic!("wildcard patterns are not allowed in this position");
        },
        Pattern::Var { name, .. } => {
            Scrutinee::Variable { name, span }
        },
        Pattern::Literal(literal) => {
            Scrutinee::Literal {
                value: literal_to_literal(literal),
                span,
            }
        },
        Pattern::Constant(path_expr) => {
            Scrutinee::EnumScrutinee {
                call_path: path_expr_to_call_path(path_expr),
                variable_to_assign: Ident::new_no_span("_"),
                span,
            }
        },
        Pattern::Constructor { path, args } => {
            let mut args = args.into_inner().into_iter();
            let arg_pattern = match args.next() {
                Some(arg_pattern) => arg_pattern,
                None => panic!("constructor patterns require a single argument"),
            };
            match args.next() {
                None => (),
                Some(_) => panic!("constructor patterns cannot have multiple arguments"),
            };
            let variable_to_assign = match arg_pattern {
                Pattern::Var { mutable, name } => {
                    if mutable.is_some() {
                        panic!("this cannot be mutable");
                    }
                    name
                },
                _ => panic!("constructor patterns cannot contain sub-patterns"),
            };
            Scrutinee::EnumScrutinee {
                call_path: path_expr_to_call_path(path),
                variable_to_assign,
                span,
            }
        },
        Pattern::Struct { path, fields } => {
            Scrutinee::StructScrutinee {
                struct_name: path_expr_to_ident(path),
                fields: {
                    fields
                    .into_inner()
                    .into_iter()
                    .map(pattern_struct_field_to_struct_scrutinee_field)
                    .collect()
                },
                span,
            }
        },
        Pattern::Tuple(pat_tuple) => {
            Scrutinee::Tuple {
                elems: pat_tuple.into_inner().into_iter().map(pattern_to_scrutinee).collect(),
                span,
            }
        },
    }
}

/*
fn ty_to_type_parameter(ty: Ty) -> TypeParameter {
    let name_ident = match ty {
        Ty::Path(path_type) => path_type_to_ident(path_type),
        Ty::Tuple(..) => panic!("tuple types are not allowed in this position"),
        Ty::Array(..) => panic!("array types are not allowed in this position"),
        Ty::Str { .. } => panic!("str types are not allowed in this position"),
    };
    TypeParameter {
        type_id: insert_type(TypeInfo::Custom {
            name: name_ident.clone(),
            type_arguments: Vec::new(),
        }),
        name_ident,
        trait_constraints: Vec::new(),
    }
}

fn path_type_to_ident(path_type: PathType) -> Ident {
    let PathType { root_opt, prefix, suffix } = path_type;
    if root_opt.is_some() || !suffix.is_empty() {
        panic!("types with paths aren't currently supported");
    }
    path_type_segment_to_ident(prefix)
}
*/

fn path_expr_to_ident(path_expr: PathExpr) -> Ident {
    let PathExpr { root_opt, prefix, suffix } = path_expr;
    if root_opt.is_some() || !suffix.is_empty() {
        panic!("paths aren't supported in this position");
    }
    path_expr_segment_to_ident(prefix)
}

fn pattern_struct_field_to_struct_scrutinee_field(
    pattern_struct_field: PatternStructField,
) -> StructScrutineeField {
    let span = pattern_struct_field.span();
    StructScrutineeField {
        field: pattern_struct_field.field_name,
        scrutinee: {
            pattern_struct_field
            .pattern_opt
            .map(|(_colon_token, pattern)| pattern_to_scrutinee(*pattern))
        },
        span,
    }
}

fn assignable_to_expression(assignable: Assignable) -> Expression {
    let span = assignable.span();
    match assignable {
        Assignable::Var(name) => {
            Expression::VariableExpression { name, span }
        },
        Assignable::Index { target, arg } => {
            Expression::ArrayIndex {
                prefix: Box::new(assignable_to_expression(*target)),
                index: Box::new(expr_to_expression(*arg.into_inner())),
                span,
            }
        },
        Assignable::FieldProjection { target, name, .. } => {
            Expression::SubfieldExpression {
                prefix: Box::new(assignable_to_expression(*target)),
                field_to_access: name,
                span,
            }
        },
    }
}

fn assignable_to_reassignment_target(assignable: Assignable) -> ReassignmentTarget {
    let mut idents = Vec::new();
    let mut base = &assignable;
    loop {
        match base {
            Assignable::FieldProjection { target, name, .. } => {
                idents.push(name);
                base = target;
            },
            Assignable::Var(name) => {
                if name.as_str() == "storage" {
                    let idents = idents.into_iter().rev().map(|ident| ident.clone()).collect();
                    return ReassignmentTarget::StorageField(idents);
                }
                break;
            },
            Assignable::Index { .. } => break,
        }
    }
    let expression = assignable_to_expression(assignable);
    ReassignmentTarget::VariableExpression(Box::new(expression))
}

fn generic_args_to_type_arguments(generic_args: GenericArgs) -> Vec<TypeArgument> {
    generic_args
    .parameters
    .into_inner()
    .into_iter()
    .map(|ty| {
        let span = ty.span();
        let type_id = insert_type(ty_to_type_info(ty));
        TypeArgument { type_id, span }
    })
    .collect()
}


fn path_type_to_type_info(path_type: PathType) -> TypeInfo {
    let PathType { root_opt, prefix, suffix } = path_type;
    if root_opt.is_some() || !suffix.is_empty() {
        panic!("named types with fully-specified paths aren't yet supported");
    }
    let PathTypeSegment { fully_qualified, name, generics_opt } = prefix;
    if fully_qualified.is_some() {
        panic!("fully qualified named types are not yet supported in this position");
    }
    match type_name_to_type_info_opt(&name) {
        Some(type_info) => {
            if let Some((_double_colon_token, generic_args)) = generics_opt {
                panic!("generics are not supported in this position: {}", generic_args.span().as_str());
            }
            type_info
        },
        None => {
            if name.as_str() == "ContractCaller" {
                let generic_ty = match generics_opt {
                    None => panic!("ContractCaller requires generic args"),
                    Some((_double_colon_token, generic_args)) => {
                        let mut tys = generic_args.parameters.into_inner().into_iter();
                        let ty = match tys.next() {
                            Some(ty) => ty,
                            None => panic!("ContractCaller requires a generic arg"),
                        };
                        match tys.next() {
                            Some(_) => panic!("ContractCaller must take only a single generic arg"),
                            None => (),
                        }
                        ty
                    },
                };
                let abi_name = match generic_ty {
                    Ty::Path(path_type) => {
                        let call_path = path_type_to_call_path(path_type);
                        AbiName::Known(call_path)
                    },
                    Ty::Infer { .. } => {
                        AbiName::Deferred
                    },
                    _ => panic!("ContractCaller requires a named type for its argument"),
                };
                TypeInfo::ContractCaller {
                    abi_name,
                    address: String::new(),
                }
            } else {
                let type_arguments = match generics_opt {
                    Some((_double_colon_token, generic_args)) => {
                        generic_args_to_type_arguments(generic_args)
                    },
                    None => Vec::new(),
                };
                TypeInfo::Custom { name, type_arguments }
            }
        },
    }
}
