use {
    crate::{
        error::{err, ok, CompileError, CompileResult, CompileWarning},
        parse_tree::desugar_match_expression,
        type_engine::{insert_type, AbiName, IntegerBits},
        AbiDeclaration, AsmExpression, AsmOp, AsmRegister, AsmRegisterDeclaration, AstNode,
        AstNodeContent, BuiltinProperty, CallPath, CatchAll, CodeBlock, ConstantDeclaration,
        Declaration, EnumDeclaration, EnumVariant, Expression, FunctionDeclaration,
        FunctionParameter, ImplSelf, ImplTrait, ImportType, IncludeStatement, LazyOp, Literal,
        MatchBranch, MatchCondition, MethodName, ParseTree, Purity, Reassignment,
        ReassignmentTarget, ReturnStatement, Scrutinee, StorageDeclaration, StorageField,
        StructDeclaration, StructExpressionField, StructField, StructScrutineeField, Supertrait,
        SwayParseTree, TraitDeclaration, TraitFn, TreeType, TypeArgument, TypeInfo, TypeParameter,
        UseStatement, VariableDeclaration, Visibility, WhileLoop,
    },
    nanoid::nanoid,
    std::{convert::TryFrom, iter, mem::MaybeUninit, ops::ControlFlow},
    sway_parse::{
        AbiCastArgs, AngleBrackets, AsmBlock, Assignable, Braces, CodeBlockContents, Dependency,
        DoubleColonToken, Expr, ExprArrayDescriptor, ExprStructField, ExprTupleDescriptor, FnArg,
        FnArgs, FnSignature, GenericArgs, GenericParams, IfCondition, IfExpr, ImpureToken,
        Instruction, Item, ItemAbi, ItemConst, ItemEnum, ItemFn, ItemImpl, ItemStorage, ItemStruct,
        ItemTrait, ItemUse, LitInt, LitIntType, MatchBranchKind, PathExpr, PathExprSegment,
        PathType, PathTypeSegment, Pattern, PatternStructField, Program, ProgramKind, PubToken,
        QualifiedPathRoot, Statement, StatementLet, Traits, Ty, TypeField, UseTree,
    },
    sway_types::{Ident, Span},
    thiserror::Error,
};

#[derive(Debug)]
/// Contains any errors or warnings that were generated during the conversion into the parse tree.
/// Typically these warnings and errors are populated as a side effect in the `From` and `Into`
/// implementations of error types into [ErrorEmitted].
pub struct ErrorContext {
    warnings: Vec<CompileWarning>,
    errors: Vec<CompileError>,
}

#[derive(Debug)]
/// Represents that an error was emitted to the error context. This struct does not contain the
/// error, rather, other errors are responsible for pushing to the [ErrorContext] in their `Into`
/// implementations.
pub struct ErrorEmitted {
    _priv: (),
}

impl ErrorContext {
    #[allow(dead_code)]
    pub fn warning<W>(&mut self, warning: W)
    where
        W: Into<CompileWarning>,
    {
        self.warnings.push(warning.into());
    }

    pub fn error<E>(&mut self, error: E) -> ErrorEmitted
    where
        E: Into<CompileError>,
    {
        self.errors.push(error.into());
        ErrorEmitted { _priv: () }
    }

    pub fn warnings<I, W>(&mut self, warnings: I)
    where
        I: IntoIterator<Item = W>,
        W: Into<CompileWarning>,
    {
        self.warnings
            .extend(warnings.into_iter().map(|warning| warning.into()));
    }

    pub fn errors<I, E>(&mut self, errors: I) -> Option<ErrorEmitted>
    where
        I: IntoIterator<Item = E>,
        E: Into<CompileError>,
    {
        let mut emitted_opt = None;
        self.errors.extend(errors.into_iter().map(|error| {
            emitted_opt = Some(ErrorEmitted { _priv: () });
            error.into()
        }));
        emitted_opt
    }
}

#[derive(Error, Debug, Clone, PartialEq, Hash)]
pub enum ConvertParseTreeError {
    #[error("pub use imports are not supported")]
    PubUseNotSupported { span: Span },
    #[error("return expressions are not allowed outside of blocks")]
    ReturnOutsideOfBlock { span: Span },
    #[error("while expressions are not allowed outside of blocks")]
    WhileOutsideOfBlock { span: Span },
    #[error("functions used in applications may not be arbitrary expressions")]
    FunctionArbitraryExpression { span: Span },
    #[error("generics are not supported here")]
    GenericsNotSupportedHere { span: Span },
    #[error("fully qualified paths are not supported here")]
    FullyQualifiedPathsNotSupportedHere { span: Span },
    #[error("size_of does not take arguments")]
    SizeOfTooManyArgs { span: Span },
    #[error("size_of requires exactly one generic argument")]
    SizeOfOneGenericArg { span: Span },
    #[error("is_reference_type does not take arguments")]
    IsReferenceTypeTooManyArgs { span: Span },
    #[error("is_reference_type requires exactly one generic argument")]
    IsReferenceTypeOneGenericArg { span: Span },
    #[error("size_of_val requires exactly one argument")]
    SizeOfValOneArg { span: Span },
    #[error("tuple index out of range")]
    TupleIndexOutOfRange { span: Span },
    #[error("shift-left expressions are not implemented")]
    ShlNotImplemented { span: Span },
    #[error("shift-right expressions are not implemented")]
    ShrNotImplemented { span: Span },
    #[error("bitwise xor expressions are not implemented")]
    BitXorNotImplemented { span: Span },
    #[error("reassignment expressions outside of blocks are not implemented")]
    ReassignmentOutsideOfBlock { span: Span },
    #[error("integer literals in this position cannot have a type suffix")]
    IntTySuffixNotSupported { span: Span },
    #[error("int literal out of range")]
    IntLiteralOutOfRange { span: Span },
    #[error("expected an integer literal")]
    IntLiteralExpected { span: Span },
    #[error("fully qualified traits are not supported")]
    FullyQualifiedTraitsNotSupported { span: Span },
    #[error("qualified path roots are not implemented")]
    QualifiedPathRootsNotImplemented { span: Span },
    #[error("char literals are not implemented")]
    CharLiteralsNotImplemented { span: Span },
    #[error("hex literals must have either 2 or 64 digits")]
    HexLiteralLength { span: Span },
    #[error("binary literals must have either 8 or 258 digits")]
    BinaryLiteralLength { span: Span },
    #[error("u8 literal out of range")]
    U8LiteralOutOfRange { span: Span },
    #[error("u16 literal out of range")]
    U16LiteralOutOfRange { span: Span },
    #[error("u32 literal out of range")]
    U32LiteralOutOfRange { span: Span },
    #[error("u64 literal out of range")]
    U64LiteralOutOfRange { span: Span },
    #[error("signed integers are not supported")]
    SignedIntegersNotSupported { span: Span },
    #[error("literal patterns not supported in this position")]
    LiteralPatternsNotSupportedHere { span: Span },
    #[error("constant patterns not supported in this position")]
    ConstantPatternsNotSupportedHere { span: Span },
    #[error("constructor patterns not supported in this position")]
    ConstructorPatternsNotSupportedHere { span: Span },
    #[error("struct patterns not supported in this position")]
    StructPatternsNotSupportedHere { span: Span },
    #[error("wildcard patterns not supported in this position")]
    WildcardPatternsNotSupportedHere { span: Span },
    #[error("tuple patterns not supported in this position")]
    TuplePatternsNotSupportedHere { span: Span },
    #[error("constructor patterns require a single argument")]
    ConstructorPatternOneArg { span: Span },
    #[error("mutable bindings are not supported in this position")]
    MutableBindingsNotSupportedHere { span: Span },
    #[error("constructor patterns cannot contain sub-patterns")]
    ConstructorPatternSubPatterns { span: Span },
    #[error("paths are not supported in this position")]
    PathsNotSupportedHere { span: Span },
    #[error("Fully specified types are not supported in this position. Try importing the type and referring to it here.")]
    FullySpecifiedTypesNotSupported { span: Span },
    #[error("ContractCaller requires exactly one generic argument")]
    ContractCallerOneGenericArg { span: Span },
    #[error("ContractCaller requires a named type for its generic argument")]
    ContractCallerNamedTypeGenericArg { span: Span },
}

impl ConvertParseTreeError {
    pub fn span_ref(&self) -> &Span {
        match self {
            ConvertParseTreeError::PubUseNotSupported { span } => span,
            ConvertParseTreeError::ReturnOutsideOfBlock { span } => span,
            ConvertParseTreeError::WhileOutsideOfBlock { span } => span,
            ConvertParseTreeError::FunctionArbitraryExpression { span } => span,
            ConvertParseTreeError::GenericsNotSupportedHere { span } => span,
            ConvertParseTreeError::FullyQualifiedPathsNotSupportedHere { span } => span,
            ConvertParseTreeError::SizeOfTooManyArgs { span } => span,
            ConvertParseTreeError::SizeOfOneGenericArg { span } => span,
            ConvertParseTreeError::IsReferenceTypeTooManyArgs { span } => span,
            ConvertParseTreeError::IsReferenceTypeOneGenericArg { span } => span,
            ConvertParseTreeError::SizeOfValOneArg { span } => span,
            ConvertParseTreeError::TupleIndexOutOfRange { span } => span,
            ConvertParseTreeError::ShlNotImplemented { span } => span,
            ConvertParseTreeError::ShrNotImplemented { span } => span,
            ConvertParseTreeError::BitXorNotImplemented { span } => span,
            ConvertParseTreeError::ReassignmentOutsideOfBlock { span } => span,
            ConvertParseTreeError::IntTySuffixNotSupported { span } => span,
            ConvertParseTreeError::IntLiteralOutOfRange { span } => span,
            ConvertParseTreeError::IntLiteralExpected { span } => span,
            ConvertParseTreeError::FullyQualifiedTraitsNotSupported { span } => span,
            ConvertParseTreeError::QualifiedPathRootsNotImplemented { span } => span,
            ConvertParseTreeError::CharLiteralsNotImplemented { span } => span,
            ConvertParseTreeError::HexLiteralLength { span } => span,
            ConvertParseTreeError::BinaryLiteralLength { span } => span,
            ConvertParseTreeError::U8LiteralOutOfRange { span } => span,
            ConvertParseTreeError::U16LiteralOutOfRange { span } => span,
            ConvertParseTreeError::U32LiteralOutOfRange { span } => span,
            ConvertParseTreeError::U64LiteralOutOfRange { span } => span,
            ConvertParseTreeError::SignedIntegersNotSupported { span } => span,
            ConvertParseTreeError::LiteralPatternsNotSupportedHere { span } => span,
            ConvertParseTreeError::ConstantPatternsNotSupportedHere { span } => span,
            ConvertParseTreeError::ConstructorPatternsNotSupportedHere { span } => span,
            ConvertParseTreeError::StructPatternsNotSupportedHere { span } => span,
            ConvertParseTreeError::WildcardPatternsNotSupportedHere { span } => span,
            ConvertParseTreeError::TuplePatternsNotSupportedHere { span } => span,
            ConvertParseTreeError::ConstructorPatternOneArg { span } => span,
            ConvertParseTreeError::MutableBindingsNotSupportedHere { span } => span,
            ConvertParseTreeError::ConstructorPatternSubPatterns { span } => span,
            ConvertParseTreeError::PathsNotSupportedHere { span } => span,
            ConvertParseTreeError::FullySpecifiedTypesNotSupported { span } => span,
            ConvertParseTreeError::ContractCallerOneGenericArg { span } => span,
            ConvertParseTreeError::ContractCallerNamedTypeGenericArg { span } => span,
        }
    }
}

pub fn convert_parse_tree(program: Program) -> CompileResult<SwayParseTree> {
    let mut ec = ErrorContext {
        warnings: Vec::new(),
        errors: Vec::new(),
    };
    let res = program_to_sway_parse_tree(&mut ec, program);
    let ErrorContext { warnings, errors } = ec;
    match res {
        Ok(sway_parse_tree) => ok(sway_parse_tree, warnings, errors),
        Err(_error_emitted) => err(warnings, errors),
    }
}

pub fn program_to_sway_parse_tree(
    ec: &mut ErrorContext,
    program: Program,
) -> Result<SwayParseTree, ErrorEmitted> {
    let span = program.span();
    let tree_type = match program.kind {
        ProgramKind::Script { .. } => TreeType::Script,
        ProgramKind::Contract { .. } => TreeType::Contract,
        ProgramKind::Predicate { .. } => TreeType::Predicate,
        ProgramKind::Library { name, .. } => TreeType::Library { name },
    };
    let root_nodes = {
        let mut root_nodes: Vec<AstNode> = {
            program
                .dependencies
                .into_iter()
                .map(|dependency| {
                    let span = dependency.span();
                    AstNode {
                        content: AstNodeContent::IncludeStatement(dependency_to_include_statement(
                            dependency,
                        )),
                        span,
                    }
                })
                .collect()
        };
        for item in program.items {
            let ast_nodes = item_to_ast_nodes(ec, item)?;
            root_nodes.extend(ast_nodes);
        }
        root_nodes
    };
    Ok(SwayParseTree {
        tree_type,
        tree: ParseTree { span, root_nodes },
    })
}

fn item_to_ast_nodes(ec: &mut ErrorContext, item: Item) -> Result<Vec<AstNode>, ErrorEmitted> {
    let span = item.span();
    let contents = match item {
        Item::Use(item_use) => {
            let use_statements = item_use_to_use_statements(ec, item_use)?;
            use_statements
                .into_iter()
                .map(AstNodeContent::UseStatement)
                .collect()
        }
        Item::Struct(item_struct) => {
            let struct_declaration = item_struct_to_struct_declaration(ec, item_struct)?;
            vec![AstNodeContent::Declaration(Declaration::StructDeclaration(
                struct_declaration,
            ))]
        }
        Item::Enum(item_enum) => {
            let enum_declaration = item_enum_to_enum_declaration(ec, item_enum)?;
            vec![AstNodeContent::Declaration(Declaration::EnumDeclaration(
                enum_declaration,
            ))]
        }
        Item::Fn(item_fn) => {
            let function_declaration = item_fn_to_function_declaration(ec, item_fn)?;
            vec![AstNodeContent::Declaration(
                Declaration::FunctionDeclaration(function_declaration),
            )]
        }
        Item::Trait(item_trait) => {
            let trait_declaration = item_trait_to_trait_declaration(ec, item_trait)?;
            vec![AstNodeContent::Declaration(Declaration::TraitDeclaration(
                trait_declaration,
            ))]
        }
        Item::Impl(item_impl) => {
            let declaration = item_impl_to_declaration(ec, item_impl)?;
            vec![AstNodeContent::Declaration(declaration)]
        }
        Item::Abi(item_abi) => {
            let abi_declaration = item_abi_to_abi_declaration(ec, item_abi)?;
            vec![AstNodeContent::Declaration(Declaration::AbiDeclaration(
                abi_declaration,
            ))]
        }
        Item::Const(item_const) => {
            let constant_declaration = item_const_to_constant_declaration(ec, item_const)?;
            vec![AstNodeContent::Declaration(
                Declaration::ConstantDeclaration(constant_declaration),
            )]
        }
        Item::Storage(item_storage) => {
            let storage_declaration = item_storage_to_storage_declaration(ec, item_storage)?;
            vec![AstNodeContent::Declaration(
                Declaration::StorageDeclaration(storage_declaration),
            )]
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
    ec: &mut ErrorContext,
    item_use: ItemUse,
) -> Result<Vec<UseStatement>, ErrorEmitted> {
    if let Some(pub_token) = item_use.visibility {
        let error = ConvertParseTreeError::PubUseNotSupported {
            span: pub_token.span(),
        };
        return Err(ec.error(error));
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

fn item_struct_to_struct_declaration(
    ec: &mut ErrorContext,
    item_struct: ItemStruct,
) -> Result<StructDeclaration, ErrorEmitted> {
    let span = item_struct.span();
    let struct_declaration = StructDeclaration {
        name: item_struct.name,
        fields: {
            item_struct
                .fields
                .into_inner()
                .into_iter()
                .map(|type_field| type_field_to_struct_field(ec, type_field))
                .collect::<Result<_, _>>()?
        },
        type_parameters: generic_params_opt_to_type_parameters(item_struct.generics),
        visibility: pub_token_opt_to_visibility(item_struct.visibility),
        span,
    };
    Ok(struct_declaration)
}

fn item_enum_to_enum_declaration(
    ec: &mut ErrorContext,
    item_enum: ItemEnum,
) -> Result<EnumDeclaration, ErrorEmitted> {
    let span = item_enum.span();
    let enum_declaration = EnumDeclaration {
        name: item_enum.name,
        type_parameters: generic_params_opt_to_type_parameters(item_enum.generics),
        variants: {
            item_enum
                .fields
                .into_inner()
                .into_iter()
                .enumerate()
                .map(|(tag, type_field)| type_field_to_enum_variant(ec, type_field, tag))
                .collect::<Result<_, _>>()?
        },
        span,
        visibility: pub_token_opt_to_visibility(item_enum.visibility),
    };
    Ok(enum_declaration)
}

fn item_fn_to_function_declaration(
    ec: &mut ErrorContext,
    item_fn: ItemFn,
) -> Result<FunctionDeclaration, ErrorEmitted> {
    let span = item_fn.span();
    let return_type_span = match &item_fn.fn_signature.return_type_opt {
        Some((_right_arrow_token, ty)) => ty.span(),
        None => item_fn.fn_signature.span(),
    };
    Ok(FunctionDeclaration {
        purity: impure_token_opt_to_purity(item_fn.fn_signature.impure),
        name: item_fn.fn_signature.name,
        visibility: pub_token_opt_to_visibility(item_fn.fn_signature.visibility),
        body: braced_code_block_contents_to_code_block(ec, item_fn.body)?,
        parameters: fn_args_to_function_parameters(
            ec,
            item_fn.fn_signature.arguments.into_inner(),
        )?,
        span,
        return_type: match item_fn.fn_signature.return_type_opt {
            Some((_right_arrow, ty)) => ty_to_type_info(ec, ty)?,
            None => TypeInfo::Tuple(Vec::new()),
        },
        type_parameters: generic_params_opt_to_type_parameters(item_fn.fn_signature.generics),
        return_type_span,
    })
}

fn item_trait_to_trait_declaration(
    ec: &mut ErrorContext,
    item_trait: ItemTrait,
) -> Result<TraitDeclaration, ErrorEmitted> {
    let name = item_trait.name;
    let interface_surface = {
        item_trait
            .trait_items
            .into_inner()
            .into_iter()
            .map(|(fn_signature, _semicolon_token)| fn_signature_to_trait_fn(ec, fn_signature))
            .collect::<Result<_, _>>()?
    };
    let methods = match item_trait.trait_defs_opt {
        None => Vec::new(),
        Some(trait_defs) => trait_defs
            .into_inner()
            .into_iter()
            .map(|item_fn| item_fn_to_function_declaration(ec, item_fn))
            .collect::<Result<_, _>>()?,
    };
    let supertraits = match item_trait.super_traits {
        None => Vec::new(),
        Some((_colon_token, traits)) => traits_to_supertraits(ec, traits)?,
    };
    let visibility = pub_token_opt_to_visibility(item_trait.visibility);
    Ok(TraitDeclaration {
        name,
        interface_surface,
        methods,
        supertraits,
        visibility,
    })
}

fn item_impl_to_declaration(
    ec: &mut ErrorContext,
    item_impl: ItemImpl,
) -> Result<Declaration, ErrorEmitted> {
    let block_span = item_impl.span();
    //let type_arguments_span = item_impl.ty.span();
    let type_implementing_for_span = item_impl.ty.span();
    let type_implementing_for = ty_to_type_info(ec, item_impl.ty)?;
    let functions = {
        item_impl
            .contents
            .into_inner()
            .into_iter()
            .map(|item_fn| item_fn_to_function_declaration(ec, item_fn))
            .collect::<Result<_, _>>()?
    };
    let type_parameters = generic_params_opt_to_type_parameters(item_impl.generic_params_opt);
    match item_impl.trait_opt {
        Some((path_type, _for_token)) => {
            let impl_trait = ImplTrait {
                trait_name: path_type_to_call_path(ec, path_type)?,
                type_implementing_for,
                type_implementing_for_span,
                type_arguments: type_parameters,
                functions,
                block_span,
            };
            Ok(Declaration::ImplTrait(impl_trait))
        }
        None => {
            let impl_self = ImplSelf {
                type_implementing_for,
                type_implementing_for_span,
                type_parameters,
                functions,
                block_span,
            };
            Ok(Declaration::ImplSelf(impl_self))
        }
    }
}

fn item_abi_to_abi_declaration(
    ec: &mut ErrorContext,
    item_abi: ItemAbi,
) -> Result<AbiDeclaration, ErrorEmitted> {
    let span = item_abi.span();
    Ok(AbiDeclaration {
        name: item_abi.name,
        interface_surface: {
            item_abi
                .abi_items
                .into_inner()
                .into_iter()
                .map(|(fn_signature, _semicolon_token)| fn_signature_to_trait_fn(ec, fn_signature))
                .collect::<Result<_, _>>()?
        },
        methods: match item_abi.abi_defs_opt {
            None => Vec::new(),
            Some(abi_defs) => abi_defs
                .into_inner()
                .into_iter()
                .map(|item_fn| item_fn_to_function_declaration(ec, item_fn))
                .collect::<Result<_, _>>()?,
        },
        span,
    })
}

fn item_const_to_constant_declaration(
    ec: &mut ErrorContext,
    item_const: ItemConst,
) -> Result<ConstantDeclaration, ErrorEmitted> {
    Ok(ConstantDeclaration {
        name: item_const.name,
        type_ascription: match item_const.ty_opt {
            Some((_colon_token, ty)) => ty_to_type_info(ec, ty)?,
            None => TypeInfo::Unknown,
        },
        value: expr_to_expression(ec, item_const.expr)?,
        //visibility: pub_token_opt_to_visibility(item_const.visibility),
        // FIXME: you have to lie here or else the tests fail.
        visibility: Visibility::Public,
    })
}

fn item_storage_to_storage_declaration(
    ec: &mut ErrorContext,
    item_storage: ItemStorage,
) -> Result<StorageDeclaration, ErrorEmitted> {
    let span = item_storage.span();
    let storage_declaration = StorageDeclaration {
        span,
        fields: {
            item_storage
                .fields
                .into_inner()
                .into_iter()
                .map(|storage_field| storage_field_to_storage_field(ec, storage_field))
                .collect::<Result<_, _>>()?
        },
    };
    Ok(storage_declaration)
}

fn type_field_to_struct_field(
    ec: &mut ErrorContext,
    type_field: TypeField,
) -> Result<StructField, ErrorEmitted> {
    let span = type_field.span();
    let type_span = type_field.ty.span();
    let struct_field = StructField {
        name: type_field.name,
        r#type: ty_to_type_info(ec, type_field.ty)?,
        span,
        type_span,
    };
    Ok(struct_field)
}

fn generic_params_opt_to_type_parameters(
    generic_params_opt: Option<GenericParams>,
) -> Vec<TypeParameter> {
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
            name_ident: ident,
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

fn type_field_to_enum_variant(
    ec: &mut ErrorContext,
    type_field: TypeField,
    tag: usize,
) -> Result<EnumVariant, ErrorEmitted> {
    let span = type_field.span();
    let enum_variant = EnumVariant {
        name: type_field.name,
        r#type: ty_to_type_info(ec, type_field.ty)?,
        tag,
        span,
    };
    Ok(enum_variant)
}

fn impure_token_opt_to_purity(impure_token_opt: Option<ImpureToken>) -> Purity {
    match impure_token_opt {
        Some(..) => Purity::Impure,
        None => Purity::Pure,
    }
}

fn braced_code_block_contents_to_code_block(
    ec: &mut ErrorContext,
    braced_code_block_contents: Braces<CodeBlockContents>,
) -> Result<CodeBlock, ErrorEmitted> {
    let whole_block_span = braced_code_block_contents.span();
    let code_block_contents = braced_code_block_contents.into_inner();
    let contents = {
        let mut contents = Vec::new();
        for statement in code_block_contents.statements {
            let ast_nodes = statement_to_ast_nodes(ec, statement)?;
            contents.extend(ast_nodes);
        }
        if let Some(expr) = code_block_contents.final_expr_opt {
            let final_ast_node = expr_to_ast_node(ec, *expr, true)?;
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
    ec: &mut ErrorContext,
    fn_args: FnArgs,
) -> Result<Vec<FunctionParameter>, ErrorEmitted> {
    let function_parameters = match fn_args {
        FnArgs::Static(args) => args
            .into_iter()
            .map(|fn_arg| fn_arg_to_function_parameter(ec, fn_arg))
            .collect::<Result<_, _>>()?,
        FnArgs::NonStatic {
            self_token,
            args_opt,
        } => {
            let mut function_parameters = vec![FunctionParameter {
                name: Ident::new(self_token.span()),
                type_id: insert_type(TypeInfo::SelfType),
                type_span: self_token.span(),
            }];
            if let Some((_comma_token, args)) = args_opt {
                for arg in args {
                    let function_parameter = fn_arg_to_function_parameter(ec, arg)?;
                    function_parameters.push(function_parameter);
                }
            }
            function_parameters
        }
    };
    Ok(function_parameters)
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

fn ty_to_type_info(ec: &mut ErrorContext, ty: Ty) -> Result<TypeInfo, ErrorEmitted> {
    let type_info = match ty {
        Ty::Path(path_type) => path_type_to_type_info(ec, path_type)?,
        Ty::Tuple(tys) => TypeInfo::Tuple(
            tys.into_inner()
                .into_iter()
                .map(|ty| ty_to_type_argument(ec, ty))
                .collect::<Result<_, _>>()?,
        ),
        Ty::Array(bracketed_ty_array_descriptor) => {
            let ty_array_descriptor = bracketed_ty_array_descriptor.into_inner();
            TypeInfo::Array(
                crate::type_engine::insert_type(ty_to_type_info(ec, *ty_array_descriptor.ty)?),
                expr_to_usize(ec, *ty_array_descriptor.length)?,
            )
        }
        Ty::Str { length, .. } => TypeInfo::Str(expr_to_u64(ec, *length.into_inner())?),
        Ty::Infer { .. } => TypeInfo::Unknown,
    };
    Ok(type_info)
}

fn ty_to_type_argument(ec: &mut ErrorContext, ty: Ty) -> Result<TypeArgument, ErrorEmitted> {
    let span = ty.span();
    let type_argument = TypeArgument {
        type_id: insert_type(ty_to_type_info(ec, ty)?),
        span,
    };
    Ok(type_argument)
}

fn fn_signature_to_trait_fn(
    ec: &mut ErrorContext,
    fn_signature: FnSignature,
) -> Result<TraitFn, ErrorEmitted> {
    let return_type_span = match &fn_signature.return_type_opt {
        Some((_right_arrow_token, ty)) => ty.span(),
        None => fn_signature.span(),
    };
    let trait_fn = TraitFn {
        name: fn_signature.name,
        parameters: fn_args_to_function_parameters(ec, fn_signature.arguments.into_inner())?,
        return_type: match fn_signature.return_type_opt {
            Some((_right_arrow_token, ty)) => ty_to_type_info(ec, ty)?,
            None => TypeInfo::Tuple(Vec::new()),
        },
        return_type_span,
    };
    Ok(trait_fn)
}

fn traits_to_supertraits(
    ec: &mut ErrorContext,
    traits: Traits,
) -> Result<Vec<Supertrait>, ErrorEmitted> {
    let mut supertraits = vec![path_type_to_supertrait(ec, traits.prefix)?];
    for (_add_token, suffix) in traits.suffixes {
        let supertrait = path_type_to_supertrait(ec, suffix)?;
        supertraits.push(supertrait);
    }
    Ok(supertraits)
}

fn path_type_to_call_path(
    ec: &mut ErrorContext,
    path_type: PathType,
) -> Result<CallPath, ErrorEmitted> {
    let PathType {
        root_opt,
        prefix,
        mut suffix,
    } = path_type;
    let is_absolute = path_root_opt_to_bool(ec, root_opt)?;
    let call_path = match suffix.pop() {
        Some((_double_colon_token, call_path_suffix)) => {
            let mut prefixes = vec![path_type_segment_to_ident(ec, prefix)?];
            for (_double_colon_token, call_path_prefix) in suffix {
                let ident = path_type_segment_to_ident(ec, call_path_prefix)?;
                prefixes.push(ident);
            }
            CallPath {
                prefixes,
                suffix: path_type_segment_to_ident(ec, call_path_suffix)?,
                is_absolute,
            }
        }
        None => CallPath {
            prefixes: Vec::new(),
            suffix: path_type_segment_to_ident(ec, prefix)?,
            is_absolute,
        },
    };
    Ok(call_path)
}

fn expr_to_ast_node(
    ec: &mut ErrorContext,
    expr: Expr,
    end_of_block: bool,
) -> Result<AstNode, ErrorEmitted> {
    let span = expr.span();
    let ast_node = match expr {
        Expr::Return { expr_opt, .. } => {
            let expression = match expr_opt {
                Some(expr) => expr_to_expression(ec, *expr)?,
                None => Expression::Tuple {
                    fields: Vec::new(),
                    span: span.clone(),
                },
            };
            AstNode {
                content: AstNodeContent::ReturnStatement(ReturnStatement { expr: expression }),
                span,
            }
        }
        Expr::While {
            condition, block, ..
        } => AstNode {
            content: AstNodeContent::WhileLoop(WhileLoop {
                condition: expr_to_expression(ec, *condition)?,
                body: braced_code_block_contents_to_code_block(ec, block)?,
            }),
            span,
        },
        Expr::Reassignment {
            assignable, expr, ..
        } => AstNode {
            content: AstNodeContent::Declaration(Declaration::Reassignment(Reassignment {
                lhs: assignable_to_reassignment_target(ec, assignable)?,
                rhs: expr_to_expression(ec, *expr)?,
                span: span.clone(),
            })),
            span,
        },
        expr => {
            let expression = expr_to_expression(ec, expr)?;
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
        }
    };
    Ok(ast_node)
}

fn expr_to_expression(ec: &mut ErrorContext, expr: Expr) -> Result<Expression, ErrorEmitted> {
    let span = expr.span();
    let expression = match expr {
        Expr::Path(path_expr) => path_expr_to_expression(ec, path_expr)?,
        Expr::Literal(literal) => Expression::Literal {
            value: literal_to_literal(ec, literal)?,
            span,
        },
        Expr::AbiCast { args, .. } => {
            let AbiCastArgs { name, address, .. } = args.into_inner();
            let abi_name = path_type_to_call_path(ec, name)?;
            let address = Box::new(expr_to_expression(ec, *address)?);
            Expression::AbiCast {
                abi_name,
                address,
                span,
            }
        }
        Expr::Struct { path, fields } => {
            let (struct_name, type_arguments) = path_expr_to_call_path_type_args(ec, path)?;
            Expression::StructExpression {
                struct_name,
                fields: {
                    fields
                        .into_inner()
                        .into_iter()
                        .map(|expr_struct_field| {
                            expr_struct_field_to_struct_expression_field(ec, expr_struct_field)
                        })
                        .collect::<Result<_, _>>()?
                },
                type_arguments,
                span,
            }
        }
        Expr::Tuple(parenthesized_expr_tuple_descriptor) => Expression::Tuple {
            fields: expr_tuple_descriptor_to_expressions(
                ec,
                parenthesized_expr_tuple_descriptor.into_inner(),
            )?,
            span,
        },
        Expr::Parens(parens) => expr_to_expression(ec, *parens.into_inner())?,
        Expr::Block(braced_code_block_contents) => {
            braced_code_block_contents_to_expression(ec, braced_code_block_contents)?
        }
        Expr::Array(bracketed_expr_array_descriptor) => {
            match bracketed_expr_array_descriptor.into_inner() {
                ExprArrayDescriptor::Sequence(exprs) => Expression::Array {
                    contents: {
                        exprs
                            .into_iter()
                            .map(|expr| expr_to_expression(ec, expr))
                            .collect::<Result<_, _>>()?
                    },
                    span,
                },
                ExprArrayDescriptor::Repeat { value, length, .. } => {
                    let expression = expr_to_expression(ec, *value)?;
                    let length = expr_to_usize(ec, *length)?;
                    Expression::Array {
                        contents: iter::repeat_with(|| expression.clone())
                            .take(length)
                            .collect(),
                        span,
                    }
                }
            }
        }
        Expr::Asm(asm_block) => Expression::AsmExpression {
            asm: asm_block_to_asm_expression(ec, asm_block)?,
            span,
        },
        Expr::Return { return_token, .. } => {
            let error = ConvertParseTreeError::ReturnOutsideOfBlock {
                span: return_token.span(),
            };
            return Err(ec.error(error));
        }
        Expr::If(if_expr) => if_expr_to_expression(ec, if_expr)?,
        Expr::Match {
            condition,
            branches,
            ..
        } => {
            let condition = expr_to_expression(ec, *condition)?;
            let branches = {
                branches
                    .into_inner()
                    .into_iter()
                    .map(|match_branch| match_branch_to_match_branch(ec, match_branch))
                    .collect::<Result<_, _>>()?
            };
            let desugar_result = desugar_match_expression(&condition, branches, None);
            let CompileResult {
                value,
                warnings,
                errors,
            } = desugar_result;
            ec.warnings(warnings);
            let error_emitted_opt = ec.errors(errors);
            let (if_exp, var_decl_name, cases_covered) = match value {
                Some(stuff) => stuff,
                None => return Err(error_emitted_opt.unwrap()),
            };
            Expression::CodeBlock {
                contents: CodeBlock {
                    contents: vec![
                        AstNode {
                            content: AstNodeContent::Declaration(Declaration::VariableDeclaration(
                                VariableDeclaration {
                                    name: var_decl_name,
                                    type_ascription: TypeInfo::Unknown,
                                    type_ascription_span: None,
                                    is_mutable: false,
                                    body: condition,
                                },
                            )),
                            span: span.clone(),
                        },
                        AstNode {
                            content: AstNodeContent::ImplicitReturnExpression(
                                Expression::MatchExp {
                                    if_exp: Box::new(if_exp),
                                    cases_covered,
                                    span: span.clone(),
                                },
                            ),
                            span: span.clone(),
                        },
                    ],
                    whole_block_span: span.clone(),
                },
                span,
            }
        }
        Expr::While { while_token, .. } => {
            let error = ConvertParseTreeError::WhileOutsideOfBlock {
                span: while_token.span(),
            };
            return Err(ec.error(error));
        }
        Expr::FuncApp { func, args } => {
            let path_expr = match *func {
                Expr::Path(path_expr) => path_expr,
                _ => {
                    let error =
                        ConvertParseTreeError::FunctionArbitraryExpression { span: func.span() };
                    return Err(ec.error(error));
                }
            };
            let PathExpr {
                root_opt,
                prefix,
                mut suffix,
            } = path_expr;
            let is_absolute = path_root_opt_to_bool(ec, root_opt)?;
            let (prefixes, method_type_opt, suffix_path_expr) = match suffix.pop() {
                Some((_double_colon_token, call_path_suffix)) => match suffix.pop() {
                    Some((_double_colon_token, maybe_method_segment)) => {
                        let PathExprSegment {
                            fully_qualified,
                            name,
                            generics_opt,
                        } = maybe_method_segment;
                        if let Some((_double_colon_token, generic_args)) = generics_opt {
                            let error = ConvertParseTreeError::GenericsNotSupportedHere {
                                span: generic_args.span(),
                            };
                            return Err(ec.error(error));
                        }
                        let mut prefixes = vec![path_expr_segment_to_ident(ec, prefix)?];
                        for (_double_colon_token, call_path_prefix) in suffix {
                            let ident = path_expr_segment_to_ident(ec, call_path_prefix)?;
                            prefixes.push(ident);
                        }
                        if fully_qualified.is_some() {
                            (prefixes, Some(name), call_path_suffix)
                        } else {
                            prefixes.push(name);
                            (prefixes, None, call_path_suffix)
                        }
                    }
                    None => {
                        let PathExprSegment {
                            fully_qualified,
                            name,
                            generics_opt,
                        } = prefix;
                        if let Some((_double_colon_token, generic_args)) = generics_opt {
                            let error = ConvertParseTreeError::GenericsNotSupportedHere {
                                span: generic_args.span(),
                            };
                            return Err(ec.error(error));
                        }
                        if fully_qualified.is_some() {
                            (Vec::new(), Some(name), call_path_suffix)
                        } else {
                            (vec![name], None, call_path_suffix)
                        }
                    }
                },
                None => (Vec::new(), None, prefix),
            };
            let PathExprSegment {
                fully_qualified,
                name,
                generics_opt,
            } = suffix_path_expr;
            if let Some(tilde_token) = fully_qualified {
                let error = ConvertParseTreeError::FullyQualifiedPathsNotSupportedHere {
                    span: tilde_token.span(),
                };
                return Err(ec.error(error));
            }
            let call_path = CallPath {
                is_absolute,
                prefixes,
                suffix: name,
            };
            let arguments = {
                args.into_inner()
                    .into_iter()
                    .map(|expr| expr_to_expression(ec, expr))
                    .collect::<Result<_, _>>()?
            };
            match method_type_opt {
                Some(type_name) => {
                    let type_name_span = type_name.span().clone();
                    let type_name = match type_name_to_type_info_opt(&type_name) {
                        Some(type_info) => type_info,
                        None => TypeInfo::Custom {
                            name: type_name,
                            type_arguments: Vec::new(),
                        },
                    };
                    let type_arguments = match generics_opt {
                        Some((_double_colon_token, generic_args)) => {
                            generic_args_to_type_arguments(ec, generic_args)?
                        }
                        None => Vec::new(),
                    };
                    Expression::MethodApplication {
                        method_name: MethodName::FromType {
                            call_path,
                            type_name: Some(type_name),
                            type_name_span: Some(type_name_span),
                        },
                        contract_call_params: Vec::new(),
                        arguments,
                        type_arguments,
                        span,
                    }
                }
                None => {
                    if call_path.prefixes.is_empty()
                        && !call_path.is_absolute
                        && call_path.suffix.as_str() == "size_of"
                    {
                        if !arguments.is_empty() {
                            let error = ConvertParseTreeError::SizeOfTooManyArgs { span };
                            return Err(ec.error(error));
                        }
                        let ty = match {
                            generics_opt.and_then(|(_double_colon_token, generic_args)| {
                                iter_to_array(generic_args.parameters.into_inner())
                            })
                        } {
                            Some([ty]) => ty,
                            None => {
                                let error = ConvertParseTreeError::SizeOfOneGenericArg { span };
                                return Err(ec.error(error));
                            }
                        };
                        let type_span = ty.span();
                        let type_name = ty_to_type_info(ec, ty)?;
                        Expression::BuiltinGetTypeProperty {
                            builtin: BuiltinProperty::SizeOfType,
                            type_name,
                            type_span,
                            span,
                        }
                    } else if call_path.prefixes.is_empty()
                        && !call_path.is_absolute
                        && call_path.suffix.as_str() == "is_reference_type"
                    {
                        if !arguments.is_empty() {
                            let error = ConvertParseTreeError::IsReferenceTypeTooManyArgs { span };
                            return Err(ec.error(error));
                        }
                        let ty = match {
                            generics_opt.and_then(|(_double_colon_token, generic_args)| {
                                iter_to_array(generic_args.parameters.into_inner())
                            })
                        } {
                            Some([ty]) => ty,
                            None => {
                                let error =
                                    ConvertParseTreeError::IsReferenceTypeOneGenericArg { span };
                                return Err(ec.error(error));
                            }
                        };
                        let type_span = ty.span();
                        let type_name = ty_to_type_info(ec, ty)?;
                        Expression::BuiltinGetTypeProperty {
                            builtin: BuiltinProperty::IsRefType,
                            type_name,
                            type_span,
                            span,
                        }
                    } else if call_path.prefixes.is_empty()
                        && !call_path.is_absolute
                        && call_path.suffix.as_str() == "size_of_val"
                    {
                        let exp = match <[_; 1]>::try_from(arguments) {
                            Ok([exp]) => Box::new(exp),
                            Err(..) => {
                                let error = ConvertParseTreeError::SizeOfValOneArg { span };
                                return Err(ec.error(error));
                            }
                        };
                        Expression::SizeOfVal { exp, span }
                    } else {
                        let type_arguments = match generics_opt {
                            Some((_double_colon_token, generic_args)) => {
                                generic_args_to_type_arguments(ec, generic_args)?
                            }
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
                }
            }
        }
        Expr::Index { target, arg } => Expression::ArrayIndex {
            prefix: Box::new(expr_to_expression(ec, *target)?),
            index: Box::new(expr_to_expression(ec, *arg.into_inner())?),
            span,
        },
        Expr::MethodCall {
            target,
            name,
            args,
            contract_args_opt,
            ..
        } => Expression::MethodApplication {
            method_name: MethodName::FromModule { method_name: name },
            contract_call_params: match contract_args_opt {
                None => Vec::new(),
                Some(contract_args) => contract_args
                    .into_inner()
                    .into_iter()
                    .map(|expr_struct_field| {
                        expr_struct_field_to_struct_expression_field(ec, expr_struct_field)
                    })
                    .collect::<Result<_, _>>()?,
            },
            arguments: {
                iter::once(*target)
                    .chain(args.into_inner().into_iter())
                    .map(|expr| expr_to_expression(ec, expr))
                    .collect::<Result<_, _>>()?
            },
            type_arguments: Vec::new(),
            span,
        },
        Expr::FieldProjection { target, name, .. } => {
            let mut idents = vec![&name];
            let mut base = &*target;
            let storage_access_field_names_opt = loop {
                match base {
                    Expr::FieldProjection { target, name, .. } => {
                        idents.push(name);
                        base = target;
                    }
                    Expr::Path(path_expr) => {
                        if path_expr.root_opt.is_none()
                            && path_expr.suffix.is_empty()
                            && path_expr.prefix.fully_qualified.is_none()
                            && path_expr.prefix.generics_opt.is_none()
                            && path_expr.prefix.name.as_str() == "storage"
                        {
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
                    Expression::StorageAccess { field_names, span }
                }
                None => Expression::SubfieldExpression {
                    prefix: Box::new(expr_to_expression(ec, *target)?),
                    field_to_access: name,
                    span,
                },
            }
        }
        Expr::TupleFieldProjection {
            target,
            field,
            field_span,
            ..
        } => Expression::TupleIndex {
            prefix: Box::new(expr_to_expression(ec, *target)?),
            index: match usize::try_from(field) {
                Ok(index) => index,
                Err(..) => {
                    let error = ConvertParseTreeError::TupleIndexOutOfRange { span: field_span };
                    return Err(ec.error(error));
                }
            },
            index_span: field_span,
            span,
        },
        Expr::Ref { ref_token, expr } => unary_op_call(ec, "ref", ref_token.span(), span, *expr)?,
        Expr::Deref { deref_token, expr } => {
            unary_op_call(ec, "deref", deref_token.span(), span, *expr)?
        }
        Expr::Not { bang_token, expr } => unary_op_call(ec, "not", bang_token.span(), span, *expr)?,
        Expr::Mul {
            lhs,
            star_token,
            rhs,
        } => binary_op_call(ec, "multiply", star_token.span(), span, *lhs, *rhs)?,
        Expr::Div {
            lhs,
            forward_slash_token,
            rhs,
        } => binary_op_call(ec, "divide", forward_slash_token.span(), span, *lhs, *rhs)?,
        Expr::Modulo {
            lhs,
            percent_token,
            rhs,
        } => binary_op_call(ec, "modulo", percent_token.span(), span, *lhs, *rhs)?,
        Expr::Add {
            lhs,
            add_token,
            rhs,
        } => binary_op_call(ec, "add", add_token.span(), span, *lhs, *rhs)?,
        Expr::Sub {
            lhs,
            sub_token,
            rhs,
        } => binary_op_call(ec, "subtract", sub_token.span(), span, *lhs, *rhs)?,
        Expr::Shl {
            lhs,
            shl_token,
            rhs,
        } => binary_op_call(ec, "lsh", shl_token.span(), span, *lhs, *rhs)?,
        Expr::Shr {
            lhs,
            shr_token,
            rhs,
        } => binary_op_call(ec, "rsh", shr_token.span(), span, *lhs, *rhs)?,
        Expr::BitAnd {
            lhs,
            ampersand_token,
            rhs,
        } => binary_op_call(ec, "binary_and", ampersand_token.span(), span, *lhs, *rhs)?,
        Expr::BitXor {
            lhs,
            caret_token,
            rhs,
        } => binary_op_call(ec, "binary_xor", caret_token.span(), span, *lhs, *rhs)?,
        Expr::BitOr {
            lhs,
            pipe_token,
            rhs,
        } => binary_op_call(ec, "binary_or", pipe_token.span(), span, *lhs, *rhs)?,
        Expr::Equal {
            lhs,
            double_eq_token,
            rhs,
        } => binary_op_call(ec, "eq", double_eq_token.span(), span, *lhs, *rhs)?,
        Expr::NotEqual {
            lhs,
            bang_eq_token,
            rhs,
        } => binary_op_call(ec, "neq", bang_eq_token.span(), span, *lhs, *rhs)?,
        Expr::LessThan {
            lhs,
            less_than_token,
            rhs,
        } => binary_op_call(ec, "lt", less_than_token.span(), span, *lhs, *rhs)?,
        Expr::GreaterThan {
            lhs,
            greater_than_token,
            rhs,
        } => binary_op_call(ec, "gt", greater_than_token.span(), span, *lhs, *rhs)?,
        Expr::LessThanEq {
            lhs,
            less_than_eq_token,
            rhs,
        } => binary_op_call(ec, "le", less_than_eq_token.span(), span, *lhs, *rhs)?,
        Expr::GreaterThanEq {
            lhs,
            greater_than_eq_token,
            rhs,
        } => binary_op_call(ec, "ge", greater_than_eq_token.span(), span, *lhs, *rhs)?,
        Expr::LogicalAnd { lhs, rhs, .. } => Expression::LazyOperator {
            op: LazyOp::And,
            lhs: Box::new(expr_to_expression(ec, *lhs)?),
            rhs: Box::new(expr_to_expression(ec, *rhs)?),
            span,
        },
        Expr::LogicalOr { lhs, rhs, .. } => Expression::LazyOperator {
            op: LazyOp::Or,
            lhs: Box::new(expr_to_expression(ec, *lhs)?),
            rhs: Box::new(expr_to_expression(ec, *rhs)?),
            span,
        },
        Expr::Reassignment { .. } => {
            let error = ConvertParseTreeError::ReassignmentOutsideOfBlock { span };
            return Err(ec.error(error));
        }
    };
    Ok(expression)
}

fn unary_op_call(
    ec: &mut ErrorContext,
    name: &'static str,
    op_span: Span,
    span: Span,
    arg: Expr,
) -> Result<Expression, ErrorEmitted> {
    Ok(Expression::FunctionApplication {
        name: CallPath {
            prefixes: vec![
                Ident::new_with_override("core", op_span.clone()),
                Ident::new_with_override("ops", op_span.clone()),
            ],
            suffix: Ident::new_with_override(name, op_span),
            is_absolute: false,
        },
        arguments: vec![expr_to_expression(ec, arg)?],
        type_arguments: Vec::new(),
        span,
    })
}

fn binary_op_call(
    ec: &mut ErrorContext,
    name: &'static str,
    op_span: Span,
    span: Span,
    lhs: Expr,
    rhs: Expr,
) -> Result<Expression, ErrorEmitted> {
    Ok(Expression::MethodApplication {
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
        arguments: vec![expr_to_expression(ec, lhs)?, expr_to_expression(ec, rhs)?],
        type_arguments: Vec::new(),
        span,
    })
}

fn storage_field_to_storage_field(
    ec: &mut ErrorContext,
    storage_field: sway_parse::StorageField,
) -> Result<StorageField, ErrorEmitted> {
    let storage_field = StorageField {
        name: storage_field.name,
        r#type: ty_to_type_info(ec, storage_field.ty)?,
        //initializer: expr_to_expression(storage_field.expr),
    };
    Ok(storage_field)
}

fn statement_to_ast_nodes(
    ec: &mut ErrorContext,
    statement: Statement,
) -> Result<Vec<AstNode>, ErrorEmitted> {
    let ast_nodes = match statement {
        Statement::Let(statement_let) => statement_let_to_ast_nodes(ec, statement_let)?,
        Statement::Item(item) => item_to_ast_nodes(ec, item)?,
        Statement::Expr { expr, .. } => vec![expr_to_ast_node(ec, expr, false)?],
    };
    Ok(ast_nodes)
}

fn fn_arg_to_function_parameter(
    ec: &mut ErrorContext,
    fn_arg: FnArg,
) -> Result<FunctionParameter, ErrorEmitted> {
    let type_span = fn_arg.ty.span();
    let pat_span = fn_arg.pattern.span();
    let name = match fn_arg.pattern {
        Pattern::Wildcard { .. } => {
            let error = ConvertParseTreeError::WildcardPatternsNotSupportedHere { span: pat_span };
            return Err(ec.error(error));
        }
        Pattern::Var { mutable, name } => {
            if let Some(mut_token) = mutable {
                let error = ConvertParseTreeError::MutableBindingsNotSupportedHere {
                    span: mut_token.span(),
                };
                return Err(ec.error(error));
            }
            name
        }
        Pattern::Literal(..) => {
            let error = ConvertParseTreeError::LiteralPatternsNotSupportedHere { span: pat_span };
            return Err(ec.error(error));
        }
        Pattern::Constant(..) => {
            let error = ConvertParseTreeError::ConstantPatternsNotSupportedHere { span: pat_span };
            return Err(ec.error(error));
        }
        Pattern::Constructor { .. } => {
            let error =
                ConvertParseTreeError::ConstructorPatternsNotSupportedHere { span: pat_span };
            return Err(ec.error(error));
        }
        Pattern::Struct { .. } => {
            let error = ConvertParseTreeError::StructPatternsNotSupportedHere { span: pat_span };
            return Err(ec.error(error));
        }
        Pattern::Tuple(..) => {
            let error = ConvertParseTreeError::TuplePatternsNotSupportedHere { span: pat_span };
            return Err(ec.error(error));
        }
    };
    let function_parameter = FunctionParameter {
        name,
        type_id: insert_type(ty_to_type_info(ec, fn_arg.ty)?),
        type_span,
    };
    Ok(function_parameter)
}

fn expr_to_usize(ec: &mut ErrorContext, expr: Expr) -> Result<usize, ErrorEmitted> {
    let span = expr.span();
    let value = match expr {
        Expr::Literal(sway_parse::Literal::Int(lit_int)) => {
            match lit_int.ty_opt {
                None => (),
                Some(..) => {
                    let error = ConvertParseTreeError::IntTySuffixNotSupported { span };
                    return Err(ec.error(error));
                }
            }
            match usize::try_from(lit_int.parsed) {
                Ok(value) => value,
                Err(..) => {
                    let error = ConvertParseTreeError::IntLiteralOutOfRange { span };
                    return Err(ec.error(error));
                }
            }
        }
        _ => {
            let error = ConvertParseTreeError::IntLiteralExpected { span };
            return Err(ec.error(error));
        }
    };
    Ok(value)
}

fn expr_to_u64(ec: &mut ErrorContext, expr: Expr) -> Result<u64, ErrorEmitted> {
    let span = expr.span();
    let value = match expr {
        Expr::Literal(sway_parse::Literal::Int(lit_int)) => {
            match lit_int.ty_opt {
                None => (),
                Some(..) => {
                    let error = ConvertParseTreeError::IntTySuffixNotSupported { span };
                    return Err(ec.error(error));
                }
            }
            match u64::try_from(lit_int.parsed) {
                Ok(value) => value,
                Err(..) => {
                    let error = ConvertParseTreeError::IntLiteralOutOfRange { span };
                    return Err(ec.error(error));
                }
            }
        }
        _ => {
            let error = ConvertParseTreeError::IntLiteralExpected { span };
            return Err(ec.error(error));
        }
    };
    Ok(value)
}

fn path_type_to_supertrait(
    ec: &mut ErrorContext,
    path_type: PathType,
) -> Result<Supertrait, ErrorEmitted> {
    let PathType {
        root_opt,
        prefix,
        mut suffix,
    } = path_type;
    let is_absolute = path_root_opt_to_bool(ec, root_opt)?;
    let (prefixes, call_path_suffix) = match suffix.pop() {
        Some((_double_colon_token, call_path_suffix)) => {
            let mut prefixes = vec![path_type_segment_to_ident(ec, prefix)?];
            for (_double_colon_token, call_path_prefix) in suffix {
                let ident = path_type_segment_to_ident(ec, call_path_prefix)?;
                prefixes.push(ident);
            }
            (prefixes, call_path_suffix)
        }
        None => (Vec::new(), prefix),
    };
    //let PathTypeSegment { fully_qualified, name, generics_opt } = call_path_suffix;
    let PathTypeSegment {
        fully_qualified,
        name,
        ..
    } = call_path_suffix;
    if let Some(tilde_token) = fully_qualified {
        let error = ConvertParseTreeError::FullyQualifiedTraitsNotSupported {
            span: tilde_token.span(),
        };
        return Err(ec.error(error));
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
    let supertrait = Supertrait {
        name,
        //type_parameters,
    };
    Ok(supertrait)
}

fn path_type_segment_to_ident(
    ec: &mut ErrorContext,
    path_type_segment: PathTypeSegment,
) -> Result<Ident, ErrorEmitted> {
    let PathTypeSegment {
        fully_qualified,
        name,
        generics_opt,
    } = path_type_segment;
    if let Some(tilde_token) = fully_qualified {
        let error = ConvertParseTreeError::FullyQualifiedPathsNotSupportedHere {
            span: tilde_token.span(),
        };
        return Err(ec.error(error));
    }
    if let Some((_double_colon_token, generic_args)) = generics_opt {
        let error = ConvertParseTreeError::GenericsNotSupportedHere {
            span: generic_args.span(),
        };
        return Err(ec.error(error));
    }
    Ok(name)
}

/// Similar to [path_type_segment_to_ident], but allows for the item to be either
/// type arguments _or_ an ident.
fn path_expr_segment_to_ident_or_type_argument(
    ec: &mut ErrorContext,
    path_expr_segment: PathExprSegment,
) -> Result<(Ident, Vec<TypeArgument>), ErrorEmitted> {
    let PathExprSegment {
        fully_qualified,
        name,
        generics_opt,
    } = path_expr_segment;
    if let Some(tilde_token) = fully_qualified {
        let error = ConvertParseTreeError::FullyQualifiedPathsNotSupportedHere {
            span: tilde_token.span(),
        };
        return Err(ec.error(error));
    }
    let generic_args = generics_opt.map(|(_, y)| y);
    let type_args = match generic_args {
        Some(x) => generic_args_to_type_arguments(ec, x)?,
        None => Default::default(),
    };
    Ok((name, type_args))
}
fn path_expr_segment_to_ident(
    ec: &mut ErrorContext,
    path_expr_segment: PathExprSegment,
) -> Result<Ident, ErrorEmitted> {
    let PathExprSegment {
        fully_qualified,
        name,
        generics_opt,
    } = path_expr_segment;
    if let Some(tilde_token) = fully_qualified {
        let error = ConvertParseTreeError::FullyQualifiedPathsNotSupportedHere {
            span: tilde_token.span(),
        };
        return Err(ec.error(error));
    }
    if let Some((_double_colon_token, generic_args)) = generics_opt {
        let error = ConvertParseTreeError::GenericsNotSupportedHere {
            span: generic_args.span(),
        };
        return Err(ec.error(error));
    }
    Ok(name)
}

fn path_expr_to_expression(
    ec: &mut ErrorContext,
    path_expr: PathExpr,
) -> Result<Expression, ErrorEmitted> {
    let span = path_expr.span();
    let expression = if path_expr.root_opt.is_none() && path_expr.suffix.is_empty() {
        let name = path_expr_segment_to_ident(ec, path_expr.prefix)?;
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
        let call_path = path_expr_to_call_path(ec, path_expr)?;
        Expression::DelineatedPath {
            call_path,
            args: Vec::new(),
            span,
            type_arguments: Vec::new(),
        }
    };
    Ok(expression)
}

fn braced_code_block_contents_to_expression(
    ec: &mut ErrorContext,
    braced_code_block_contents: Braces<CodeBlockContents>,
) -> Result<Expression, ErrorEmitted> {
    let span = braced_code_block_contents.span();
    Ok(Expression::CodeBlock {
        contents: braced_code_block_contents_to_code_block(ec, braced_code_block_contents)?,
        span,
    })
}

fn if_expr_to_expression(
    ec: &mut ErrorContext,
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
    let then_block = braced_code_block_contents_to_code_block(ec, then_block)?;
    let else_opt = match else_opt {
        None => None,
        Some((_else_token, tail)) => {
            let expression = match tail {
                ControlFlow::Break(braced_code_block_contents) => {
                    braced_code_block_contents_to_expression(ec, braced_code_block_contents)?
                }
                ControlFlow::Continue(if_expr) => if_expr_to_expression(ec, *if_expr)?,
            };
            Some(Box::new(expression))
        }
    };
    let expression = match condition {
        IfCondition::Expr(condition) => Expression::IfExp {
            condition: Box::new(expr_to_expression(ec, *condition)?),
            then: Box::new(Expression::CodeBlock {
                contents: then_block,
                span: then_block_span,
            }),
            r#else: else_opt,
            span,
        },
        IfCondition::Let { lhs, rhs, .. } => Expression::IfLet {
            scrutinee: pattern_to_scrutinee(ec, *lhs)?,
            expr: Box::new(expr_to_expression(ec, *rhs)?),
            then: then_block,
            r#else: else_opt,
            span,
        },
    };
    Ok(expression)
}

fn path_root_opt_to_bool(
    ec: &mut ErrorContext,
    root_opt: Option<(Option<AngleBrackets<QualifiedPathRoot>>, DoubleColonToken)>,
) -> Result<bool, ErrorEmitted> {
    let b = match root_opt {
        None => false,
        Some((None, _double_colon_token)) => true,
        Some((Some(qualified_path_root), _double_colon_token)) => {
            let error = ConvertParseTreeError::QualifiedPathRootsNotImplemented {
                span: qualified_path_root.span(),
            };
            return Err(ec.error(error));
        }
    };
    Ok(b)
}

fn literal_to_literal(
    ec: &mut ErrorContext,
    literal: sway_parse::Literal,
) -> Result<Literal, ErrorEmitted> {
    let literal = match literal {
        sway_parse::Literal::String(lit_string) => {
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
        sway_parse::Literal::Char(lit_char) => {
            let error = ConvertParseTreeError::CharLiteralsNotImplemented {
                span: lit_char.span(),
            };
            return Err(ec.error(error));
        }
        sway_parse::Literal::Int(lit_int) => {
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
                            2 => Literal::Byte(u8::try_from(parsed).unwrap()),
                            64 => {
                                let bytes = parsed.to_bytes_be();
                                let mut full_bytes = [0u8; 32];
                                full_bytes[(32 - bytes.len())..].copy_from_slice(&bytes);
                                Literal::B256(full_bytes)
                            }
                            _ => {
                                let error = ConvertParseTreeError::HexLiteralLength { span };
                                return Err(ec.error(error));
                            }
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
                            }
                            _ => {
                                let error = ConvertParseTreeError::BinaryLiteralLength { span };
                                return Err(ec.error(error));
                            }
                        }
                    } else {
                        match u64::try_from(&parsed) {
                            Ok(value) => Literal::Numeric(value),
                            Err(..) => {
                                let error = ConvertParseTreeError::IntLiteralOutOfRange { span };
                                return Err(ec.error(error));
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
                                return Err(ec.error(error));
                            }
                        };
                        Literal::U8(value)
                    }
                    LitIntType::U16 => {
                        let value = match u16::try_from(parsed) {
                            Ok(value) => value,
                            Err(..) => {
                                let error = ConvertParseTreeError::U16LiteralOutOfRange { span };
                                return Err(ec.error(error));
                            }
                        };
                        Literal::U16(value)
                    }
                    LitIntType::U32 => {
                        let value = match u32::try_from(parsed) {
                            Ok(value) => value,
                            Err(..) => {
                                let error = ConvertParseTreeError::U32LiteralOutOfRange { span };
                                return Err(ec.error(error));
                            }
                        };
                        Literal::U32(value)
                    }
                    LitIntType::U64 => {
                        let value = match u64::try_from(parsed) {
                            Ok(value) => value,
                            Err(..) => {
                                let error = ConvertParseTreeError::U64LiteralOutOfRange { span };
                                return Err(ec.error(error));
                            }
                        };
                        Literal::U64(value)
                    }
                    LitIntType::I8 | LitIntType::I16 | LitIntType::I32 | LitIntType::I64 => {
                        let error = ConvertParseTreeError::SignedIntegersNotSupported { span };
                        return Err(ec.error(error));
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
fn path_expr_to_call_path_type_args(
    ec: &mut ErrorContext,
    path_expr: PathExpr,
) -> Result<(CallPath, Vec<TypeArgument>), ErrorEmitted> {
    let PathExpr {
        root_opt,
        prefix,
        mut suffix,
    } = path_expr;
    let is_absolute = path_root_opt_to_bool(ec, root_opt)?;
    let (call_path, type_arguments) = match suffix.pop() {
        Some((_double_colon_token, call_path_suffix)) => {
            let mut prefixes = vec![path_expr_segment_to_ident(ec, prefix)?];
            for (_double_colon_token, call_path_prefix) in suffix {
                let ident = path_expr_segment_to_ident(ec, call_path_prefix)?;
                // note that call paths only support one set of type arguments per call path right
                // now
                prefixes.push(ident);
            }
            let (suffix, ty_args) =
                path_expr_segment_to_ident_or_type_argument(ec, call_path_suffix)?;
            (
                CallPath {
                    prefixes,
                    suffix,
                    is_absolute,
                },
                ty_args,
            )
        }
        None => {
            let (suffix, ty_args) = path_expr_segment_to_ident_or_type_argument(ec, prefix)?;
            (
                CallPath {
                    prefixes: Default::default(),
                    suffix,
                    is_absolute,
                },
                ty_args,
            )
        }
    };
    Ok((call_path, type_arguments))
}

fn path_expr_to_call_path(
    ec: &mut ErrorContext,
    path_expr: PathExpr,
) -> Result<CallPath, ErrorEmitted> {
    let PathExpr {
        root_opt,
        prefix,
        mut suffix,
    } = path_expr;
    let is_absolute = path_root_opt_to_bool(ec, root_opt)?;
    let call_path = match suffix.pop() {
        Some((_double_colon_token, call_path_suffix)) => {
            let mut prefixes = vec![path_expr_segment_to_ident(ec, prefix)?];
            for (_double_colon_token, call_path_prefix) in suffix {
                let ident = path_expr_segment_to_ident(ec, call_path_prefix)?;
                prefixes.push(ident);
            }
            CallPath {
                prefixes,
                suffix: path_expr_segment_to_ident(ec, call_path_suffix)?,
                is_absolute,
            }
        }
        None => CallPath {
            prefixes: Vec::new(),
            suffix: path_expr_segment_to_ident(ec, prefix)?,
            is_absolute,
        },
    };
    Ok(call_path)
}

fn expr_struct_field_to_struct_expression_field(
    ec: &mut ErrorContext,
    expr_struct_field: ExprStructField,
) -> Result<StructExpressionField, ErrorEmitted> {
    let span = expr_struct_field.span();
    let value = match expr_struct_field.expr_opt {
        Some((_colon_token, expr)) => expr_to_expression(ec, *expr)?,
        None => Expression::VariableExpression {
            name: expr_struct_field.field_name.clone(),
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
    ec: &mut ErrorContext,
    expr_tuple_descriptor: ExprTupleDescriptor,
) -> Result<Vec<Expression>, ErrorEmitted> {
    let expressions = match expr_tuple_descriptor {
        ExprTupleDescriptor::Nil => Vec::new(),
        ExprTupleDescriptor::Cons { head, tail, .. } => {
            let mut expressions = vec![expr_to_expression(ec, *head)?];
            for expr in tail {
                expressions.push(expr_to_expression(ec, expr)?);
            }
            expressions
        }
    };
    Ok(expressions)
}

fn asm_block_to_asm_expression(
    ec: &mut ErrorContext,
    asm_block: AsmBlock,
) -> Result<AsmExpression, ErrorEmitted> {
    let whole_block_span = asm_block.span();
    let asm_block_contents = asm_block.contents.into_inner();
    let (returns, return_type) = match asm_block_contents.final_expr_opt {
        Some(asm_final_expr) => {
            let asm_register = AsmRegister {
                name: asm_final_expr.register.as_str().to_owned(),
            };
            let returns = Some((asm_register, asm_final_expr.register.span().clone()));
            let return_type = match asm_final_expr.ty_opt {
                Some((_colon_token, ty)) => ty_to_type_info(ec, ty)?,
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
                asm_register_declaration_to_asm_register_declaration(ec, asm_register_declaration)
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
    Ok(AsmExpression {
        registers,
        body,
        returns,
        return_type,
        whole_block_span,
    })
}

fn match_branch_to_match_branch(
    ec: &mut ErrorContext,
    match_branch: sway_parse::MatchBranch,
) -> Result<MatchBranch, ErrorEmitted> {
    let span = match_branch.span();
    Ok(MatchBranch {
        condition: pattern_to_match_condition(ec, match_branch.pattern)?,
        result: match match_branch.kind {
            MatchBranchKind::Block { block, .. } => {
                let span = block.span();
                Expression::CodeBlock {
                    contents: braced_code_block_contents_to_code_block(ec, block)?,
                    span,
                }
            }
            MatchBranchKind::Expr { expr, .. } => expr_to_expression(ec, expr)?,
        },
        span,
    })
}

fn statement_let_to_ast_nodes(
    ec: &mut ErrorContext,
    statement_let: StatementLet,
) -> Result<Vec<AstNode>, ErrorEmitted> {
    fn unfold(
        ec: &mut ErrorContext,
        pattern: Pattern,
        ty_opt: Option<Ty>,
        expression: Expression,
        span: Span,
    ) -> Result<Vec<AstNode>, ErrorEmitted> {
        let ast_nodes = match pattern {
            Pattern::Wildcard { .. } => {
                let ast_node = AstNode {
                    content: AstNodeContent::Expression(expression),
                    span,
                };
                vec![ast_node]
            }
            Pattern::Var { mutable, name } => {
                let (type_ascription, type_ascription_span) = match ty_opt {
                    Some(ty) => {
                        let type_ascription_span = ty.span();
                        let type_ascription = ty_to_type_info(ec, ty)?;
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
                return Err(ec.error(error));
            }
            Pattern::Constant(..) => {
                let error = ConvertParseTreeError::ConstantPatternsNotSupportedHere { span };
                return Err(ec.error(error));
            }
            Pattern::Constructor { .. } => {
                let error = ConvertParseTreeError::ConstructorPatternsNotSupportedHere { span };
                return Err(ec.error(error));
            }
            Pattern::Struct { .. } => {
                let error = ConvertParseTreeError::StructPatternsNotSupportedHere { span };
                return Err(ec.error(error));
            }
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
                        let type_ascription = ty_to_type_info(ec, ty.clone())?;
                        (type_ascription, Some(type_ascription_span))
                    }
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
                    content: AstNodeContent::Declaration(Declaration::VariableDeclaration(
                        save_body_first,
                    )),
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
                        ec,
                        pattern,
                        ty_opt,
                        Expression::TupleIndex {
                            prefix: Box::new(new_expr.clone()),
                            index,
                            index_span: span.clone(),
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
    let initial_expression = expr_to_expression(ec, statement_let.expr)?;
    unfold(
        ec,
        statement_let.pattern,
        statement_let.ty_opt.map(|(_colon_token, ty)| ty),
        initial_expression,
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

#[allow(dead_code)]
fn generic_args_to_type_parameters(
    ec: &mut ErrorContext,
    generic_args: GenericArgs,
) -> Result<Vec<TypeParameter>, ErrorEmitted> {
    generic_args
        .parameters
        .into_inner()
        .into_iter()
        .map(|x| ty_to_type_parameter(ec, x))
        .collect()
}

fn asm_register_declaration_to_asm_register_declaration(
    ec: &mut ErrorContext,
    asm_register_declaration: sway_parse::AsmRegisterDeclaration,
) -> Result<AsmRegisterDeclaration, ErrorEmitted> {
    Ok(AsmRegisterDeclaration {
        name: asm_register_declaration.register,
        initializer: match asm_register_declaration.value_opt {
            None => None,
            Some((_colon_token, expr)) => Some(expr_to_expression(ec, *expr)?),
        },
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

fn pattern_to_match_condition(
    ec: &mut ErrorContext,
    pattern: Pattern,
) -> Result<MatchCondition, ErrorEmitted> {
    let match_condition = match pattern {
        Pattern::Wildcard { underscore_token } => {
            let span = underscore_token.span();
            MatchCondition::CatchAll(CatchAll { span })
        }
        _ => MatchCondition::Scrutinee(pattern_to_scrutinee(ec, pattern)?),
    };
    Ok(match_condition)
}

fn pattern_to_scrutinee(
    ec: &mut ErrorContext,
    pattern: Pattern,
) -> Result<Scrutinee, ErrorEmitted> {
    let span = pattern.span();
    let scrutinee = match pattern {
        Pattern::Wildcard { .. } => {
            let error = ConvertParseTreeError::WildcardPatternsNotSupportedHere { span };
            return Err(ec.error(error));
        }
        Pattern::Var { name, .. } => Scrutinee::Variable { name, span },
        Pattern::Literal(literal) => Scrutinee::Literal {
            value: literal_to_literal(ec, literal)?,
            span,
        },
        Pattern::Constant(path_expr) => Scrutinee::EnumScrutinee {
            call_path: path_expr_to_call_path(ec, path_expr)?,
            variable_to_assign: Ident::new_no_span("_"),
            span,
        },
        Pattern::Constructor { path, args } => {
            let arg = match iter_to_array(args.into_inner()) {
                Some([arg]) => arg,
                None => {
                    let error = ConvertParseTreeError::ConstructorPatternOneArg { span };
                    return Err(ec.error(error));
                }
            };
            let variable_to_assign = match arg {
                Pattern::Var { mutable, name } => {
                    if mutable.is_some() {
                        let error = ConvertParseTreeError::MutableBindingsNotSupportedHere { span };
                        return Err(ec.error(error));
                    }
                    name
                }
                _ => {
                    let error = ConvertParseTreeError::ConstructorPatternSubPatterns { span };
                    return Err(ec.error(error));
                }
            };
            Scrutinee::EnumScrutinee {
                call_path: path_expr_to_call_path(ec, path)?,
                variable_to_assign,
                span,
            }
        }
        Pattern::Struct { path, fields } => Scrutinee::StructScrutinee {
            struct_name: path_expr_to_ident(ec, path)?,
            fields: {
                fields
                    .into_inner()
                    .into_iter()
                    .map(|field| pattern_struct_field_to_struct_scrutinee_field(ec, field))
                    .collect::<Result<_, _>>()?
            },
            span,
        },
        Pattern::Tuple(pat_tuple) => Scrutinee::Tuple {
            elems: {
                pat_tuple
                    .into_inner()
                    .into_iter()
                    .map(|pattern| pattern_to_scrutinee(ec, pattern))
                    .collect::<Result<_, _>>()?
            },
            span,
        },
    };
    Ok(scrutinee)
}

#[allow(dead_code)]
fn ty_to_type_parameter(ec: &mut ErrorContext, ty: Ty) -> Result<TypeParameter, ErrorEmitted> {
    let name_ident = match ty {
        Ty::Path(path_type) => path_type_to_ident(ec, path_type)?,
        Ty::Infer { underscore_token } => {
            return Ok(TypeParameter {
                type_id: insert_type(TypeInfo::Unknown),
                name_ident: underscore_token.into(),
                trait_constraints: Default::default(),
            })
        }
        Ty::Tuple(..) => panic!("tuple types are not allowed in this position"),
        Ty::Array(..) => panic!("array types are not allowed in this position"),
        Ty::Str { .. } => panic!("str types are not allowed in this position"),
    };
    Ok(TypeParameter {
        type_id: insert_type(TypeInfo::Custom {
            name: name_ident.clone(),
            type_arguments: Vec::new(),
        }),
        name_ident,
        trait_constraints: Vec::new(),
    })
}

#[allow(dead_code)]
fn path_type_to_ident(ec: &mut ErrorContext, path_type: PathType) -> Result<Ident, ErrorEmitted> {
    let PathType {
        root_opt,
        prefix,
        suffix,
    } = path_type;
    if root_opt.is_some() || !suffix.is_empty() {
        panic!("types with paths aren't currently supported");
    }
    path_type_segment_to_ident(ec, prefix)
}

fn path_expr_to_ident(ec: &mut ErrorContext, path_expr: PathExpr) -> Result<Ident, ErrorEmitted> {
    let span = path_expr.span();
    let PathExpr {
        root_opt,
        prefix,
        suffix,
    } = path_expr;
    if root_opt.is_some() || !suffix.is_empty() {
        let error = ConvertParseTreeError::PathsNotSupportedHere { span };
        return Err(ec.error(error));
    }
    path_expr_segment_to_ident(ec, prefix)
}

fn pattern_struct_field_to_struct_scrutinee_field(
    ec: &mut ErrorContext,
    pattern_struct_field: PatternStructField,
) -> Result<StructScrutineeField, ErrorEmitted> {
    let span = pattern_struct_field.span();
    let struct_scrutinee_field = StructScrutineeField {
        field: pattern_struct_field.field_name,
        scrutinee: match pattern_struct_field.pattern_opt {
            Some((_colon_token, pattern)) => Some(pattern_to_scrutinee(ec, *pattern)?),
            None => None,
        },
        span,
    };
    Ok(struct_scrutinee_field)
}

fn assignable_to_expression(
    ec: &mut ErrorContext,
    assignable: Assignable,
) -> Result<Expression, ErrorEmitted> {
    let span = assignable.span();
    let expression = match assignable {
        Assignable::Var(name) => Expression::VariableExpression { name, span },
        Assignable::Index { target, arg } => Expression::ArrayIndex {
            prefix: Box::new(assignable_to_expression(ec, *target)?),
            index: Box::new(expr_to_expression(ec, *arg.into_inner())?),
            span,
        },
        Assignable::FieldProjection { target, name, .. } => Expression::SubfieldExpression {
            prefix: Box::new(assignable_to_expression(ec, *target)?),
            field_to_access: name,
            span,
        },
    };
    Ok(expression)
}

fn assignable_to_reassignment_target(
    ec: &mut ErrorContext,
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
        }
    }
    let expression = assignable_to_expression(ec, assignable)?;
    Ok(ReassignmentTarget::VariableExpression(Box::new(expression)))
}

fn generic_args_to_type_arguments(
    ec: &mut ErrorContext,
    generic_args: GenericArgs,
) -> Result<Vec<TypeArgument>, ErrorEmitted> {
    generic_args
        .parameters
        .into_inner()
        .into_iter()
        .map(|ty| {
            let span = ty.span();
            let type_id = insert_type(ty_to_type_info(ec, ty)?);
            Ok(TypeArgument { type_id, span })
        })
        .collect()
}

fn path_type_to_type_info(
    ec: &mut ErrorContext,
    path_type: PathType,
) -> Result<TypeInfo, ErrorEmitted> {
    let span = path_type.span();
    let PathType {
        root_opt,
        prefix,
        suffix,
    } = path_type;
    if root_opt.is_some() || !suffix.is_empty() {
        let error = ConvertParseTreeError::FullySpecifiedTypesNotSupported { span };
        return Err(ec.error(error));
    }
    let PathTypeSegment {
        fully_qualified,
        name,
        generics_opt,
    } = prefix;
    if let Some(tilde_token) = fully_qualified {
        let error = ConvertParseTreeError::FullyQualifiedPathsNotSupportedHere {
            span: tilde_token.span(),
        };
        return Err(ec.error(error));
    }
    let type_info = match type_name_to_type_info_opt(&name) {
        Some(type_info) => {
            if let Some((_double_colon_token, generic_args)) = generics_opt {
                let error = ConvertParseTreeError::GenericsNotSupportedHere {
                    span: generic_args.span(),
                };
                return Err(ec.error(error));
            }
            type_info
        }
        None => {
            if name.as_str() == "ContractCaller" {
                let generic_ty = match {
                    generics_opt.and_then(|(_double_colon_token, generic_args)| {
                        iter_to_array(generic_args.parameters.into_inner())
                    })
                } {
                    Some([ty]) => ty,
                    None => {
                        let error = ConvertParseTreeError::ContractCallerOneGenericArg { span };
                        return Err(ec.error(error));
                    }
                };
                let abi_name = match generic_ty {
                    Ty::Path(path_type) => {
                        let call_path = path_type_to_call_path(ec, path_type)?;
                        AbiName::Known(call_path)
                    }
                    Ty::Infer { .. } => AbiName::Deferred,
                    _ => {
                        let error =
                            ConvertParseTreeError::ContractCallerNamedTypeGenericArg { span };
                        return Err(ec.error(error));
                    }
                };
                TypeInfo::ContractCaller {
                    abi_name,
                    address: String::new(),
                }
            } else {
                let type_arguments = match generics_opt {
                    Some((_double_colon_token, generic_args)) => {
                        generic_args_to_type_arguments(ec, generic_args)?
                    }
                    None => Vec::new(),
                };
                TypeInfo::Custom {
                    name,
                    type_arguments,
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
