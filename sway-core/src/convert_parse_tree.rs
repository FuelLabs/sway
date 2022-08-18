use std::collections::HashSet;

use crate::{
    type_system::{resolve_type, TraitConstraint, TypeArgument, TypeBinding, TypeParameter},
    WhileLoopExpression,
};

use {
    crate::{
        constants::{
            STORAGE_PURITY_ATTRIBUTE_NAME, STORAGE_PURITY_READ_NAME, STORAGE_PURITY_WRITE_NAME,
            VALID_ATTRIBUTE_NAMES,
        },
        error::{err, ok, CompileError, CompileResult, CompileWarning, Warning},
        type_system::{insert_type, AbiName, IntegerBits},
        AbiCastExpression, AbiDeclaration, ArrayIndexExpression, AsmExpression, AsmOp, AsmRegister,
        AsmRegisterDeclaration, AstNode, AstNodeContent, CallPath, CodeBlock, ConstantDeclaration,
        Declaration, DelineatedPathExpression, EnumDeclaration, EnumVariant, Expression,
        ExpressionKind, FunctionApplicationExpression, FunctionDeclaration, FunctionParameter,
        IfExpression, ImplSelf, ImplTrait, ImportType, IncludeStatement,
        IntrinsicFunctionExpression, LazyOp, LazyOperatorExpression, Literal, MatchBranch,
        MatchExpression, MethodApplicationExpression, MethodName, ParseTree, Purity, Reassignment,
        ReassignmentTarget, ReturnStatement, Scrutinee, StorageAccessExpression,
        StorageDeclaration, StorageField, StructDeclaration, StructExpression,
        StructExpressionField, StructField, StructScrutineeField, SubfieldExpression, Supertrait,
        TraitDeclaration, TraitFn, TreeType, TupleIndexExpression, TypeInfo, UseStatement,
        VariableDeclaration, Visibility,
    },
    std::{
        collections::HashMap,
        convert::TryFrom,
        iter,
        mem::MaybeUninit,
        ops::ControlFlow,
        sync::atomic::{AtomicUsize, Ordering},
    },
    sway_ast::{
        expr::{ReassignmentOp, ReassignmentOpVariant},
        ty::TyTupleDescriptor,
        AbiCastArgs, AngleBrackets, AsmBlock, Assignable, AttributeDecl, Braces, CodeBlockContents,
        CommaToken, Dependency, DoubleColonToken, Expr, ExprArrayDescriptor, ExprStructField,
        ExprTupleDescriptor, FnArg, FnArgs, FnSignature, GenericArgs, GenericParams, IfCondition,
        IfExpr, Instruction, Intrinsic, Item, ItemAbi, ItemConst, ItemEnum, ItemFn, ItemImpl,
        ItemKind, ItemStorage, ItemStruct, ItemTrait, ItemUse, LitInt, LitIntType, MatchBranchKind,
        Module, ModuleKind, Parens, PathExpr, PathExprSegment, PathType, PathTypeSegment, Pattern,
        PatternStructField, PubToken, Punctuated, QualifiedPathRoot, Statement, StatementLet,
        Traits, Ty, TypeField, UseTree, WhereClause,
    },
    sway_types::{Ident, Span, Spanned},
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

#[derive(Error, Debug, Clone, PartialEq, Eq, Hash)]
pub enum ConvertParseTreeError {
    #[error("pub use imports are not supported")]
    PubUseNotSupported { span: Span },
    #[error("return expressions are not allowed outside of blocks")]
    ReturnOutsideOfBlock { span: Span },
    #[error("functions used in applications may not be arbitrary expressions")]
    FunctionArbitraryExpression { span: Span },
    #[error("generics are not supported here")]
    GenericsNotSupportedHere { span: Span },
    #[error("fully qualified paths are not supported here")]
    FullyQualifiedPathsNotSupportedHere { span: Span },
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
    #[error("hex literals must have 1..16 or 64 digits")]
    HexLiteralLength { span: Span },
    #[error("binary literals must have either 1..64 or 256 digits")]
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
    #[error("ref variables are not supported")]
    RefVariablesNotSupported { span: Span },
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
    #[error("ref patterns not supported in this position")]
    RefPatternsNotSupportedHere { span: Span },
    #[error("constructor patterns require a single argument")]
    ConstructorPatternOneArg { span: Span },
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
    #[error("invalid argument for '{attribute}' attribute")]
    InvalidAttributeArgument { attribute: String, span: Span },
    #[error("cannot find type \"{ty_name}\" in this scope")]
    ConstrainedNonExistentType { ty_name: Ident, span: Span },
    #[error("__get_storage_key does not take arguments")]
    GetStorageKeyTooManyArgs { span: Span },
    #[error("recursive types are not supported")]
    RecursiveType { span: Span },
    #[error("enum variant \"{name}\" already declared")]
    DuplicateEnumVariant { name: Ident, span: Span },
    #[error("storage field \"{name}\" already declared")]
    DuplicateStorageField { name: Ident, span: Span },
    #[error("struct field \"{name}\" already declared")]
    DuplicateStructField { name: Ident, span: Span },
    #[error("identifier \"{name}\" bound more than once in this parameter list")]
    DuplicateParameterIdentifier { name: Ident, span: Span },
    #[error("self parameter is not allowed for a free function")]
    SelfParameterNotAllowedForFreeFn { span: Span },
}

impl Spanned for ConvertParseTreeError {
    fn span(&self) -> Span {
        match self {
            ConvertParseTreeError::PubUseNotSupported { span } => span.clone(),
            ConvertParseTreeError::ReturnOutsideOfBlock { span } => span.clone(),
            ConvertParseTreeError::FunctionArbitraryExpression { span } => span.clone(),
            ConvertParseTreeError::GenericsNotSupportedHere { span } => span.clone(),
            ConvertParseTreeError::FullyQualifiedPathsNotSupportedHere { span } => span.clone(),
            ConvertParseTreeError::TupleIndexOutOfRange { span } => span.clone(),
            ConvertParseTreeError::ShlNotImplemented { span } => span.clone(),
            ConvertParseTreeError::ShrNotImplemented { span } => span.clone(),
            ConvertParseTreeError::BitXorNotImplemented { span } => span.clone(),
            ConvertParseTreeError::ReassignmentOutsideOfBlock { span } => span.clone(),
            ConvertParseTreeError::IntTySuffixNotSupported { span } => span.clone(),
            ConvertParseTreeError::IntLiteralOutOfRange { span } => span.clone(),
            ConvertParseTreeError::IntLiteralExpected { span } => span.clone(),
            ConvertParseTreeError::FullyQualifiedTraitsNotSupported { span } => span.clone(),
            ConvertParseTreeError::QualifiedPathRootsNotImplemented { span } => span.clone(),
            ConvertParseTreeError::CharLiteralsNotImplemented { span } => span.clone(),
            ConvertParseTreeError::HexLiteralLength { span } => span.clone(),
            ConvertParseTreeError::BinaryLiteralLength { span } => span.clone(),
            ConvertParseTreeError::U8LiteralOutOfRange { span } => span.clone(),
            ConvertParseTreeError::U16LiteralOutOfRange { span } => span.clone(),
            ConvertParseTreeError::U32LiteralOutOfRange { span } => span.clone(),
            ConvertParseTreeError::U64LiteralOutOfRange { span } => span.clone(),
            ConvertParseTreeError::SignedIntegersNotSupported { span } => span.clone(),
            ConvertParseTreeError::RefVariablesNotSupported { span } => span.clone(),
            ConvertParseTreeError::LiteralPatternsNotSupportedHere { span } => span.clone(),
            ConvertParseTreeError::ConstantPatternsNotSupportedHere { span } => span.clone(),
            ConvertParseTreeError::ConstructorPatternsNotSupportedHere { span } => span.clone(),
            ConvertParseTreeError::StructPatternsNotSupportedHere { span } => span.clone(),
            ConvertParseTreeError::WildcardPatternsNotSupportedHere { span } => span.clone(),
            ConvertParseTreeError::TuplePatternsNotSupportedHere { span } => span.clone(),
            ConvertParseTreeError::RefPatternsNotSupportedHere { span } => span.clone(),
            ConvertParseTreeError::ConstructorPatternOneArg { span } => span.clone(),
            ConvertParseTreeError::ConstructorPatternSubPatterns { span } => span.clone(),
            ConvertParseTreeError::PathsNotSupportedHere { span } => span.clone(),
            ConvertParseTreeError::FullySpecifiedTypesNotSupported { span } => span.clone(),
            ConvertParseTreeError::ContractCallerOneGenericArg { span } => span.clone(),
            ConvertParseTreeError::ContractCallerNamedTypeGenericArg { span } => span.clone(),
            ConvertParseTreeError::InvalidAttributeArgument { span, .. } => span.clone(),
            ConvertParseTreeError::ConstrainedNonExistentType { span, .. } => span.clone(),
            ConvertParseTreeError::GetStorageKeyTooManyArgs { span, .. } => span.clone(),
            ConvertParseTreeError::RecursiveType { span } => span.clone(),
            ConvertParseTreeError::DuplicateEnumVariant { span, .. } => span.clone(),
            ConvertParseTreeError::DuplicateStorageField { span, .. } => span.clone(),
            ConvertParseTreeError::DuplicateStructField { span, .. } => span.clone(),
            ConvertParseTreeError::DuplicateParameterIdentifier { span, .. } => span.clone(),
            ConvertParseTreeError::SelfParameterNotAllowedForFreeFn { span, .. } => span.clone(),
        }
    }
}

pub fn convert_parse_tree(module: Module) -> CompileResult<(TreeType, ParseTree)> {
    let mut ec = ErrorContext {
        warnings: Vec::new(),
        errors: Vec::new(),
    };
    let tree_type = match module.kind {
        ModuleKind::Script { .. } => TreeType::Script,
        ModuleKind::Contract { .. } => TreeType::Contract,
        ModuleKind::Predicate { .. } => TreeType::Predicate,
        ModuleKind::Library { ref name, .. } => TreeType::Library { name: name.clone() },
    };
    let res = module_to_sway_parse_tree(&mut ec, module);
    let ErrorContext { warnings, errors } = ec;
    match res {
        Ok(parse_tree) => ok((tree_type, parse_tree), warnings, errors),
        Err(_error_emitted) => err(warnings, errors),
    }
}

pub fn module_to_sway_parse_tree(
    ec: &mut ErrorContext,
    module: Module,
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
            let ast_nodes = item_to_ast_nodes(ec, item)?;
            root_nodes.extend(ast_nodes);
        }
        root_nodes
    };
    Ok(ParseTree { span, root_nodes })
}

fn item_to_ast_nodes(ec: &mut ErrorContext, item: Item) -> Result<Vec<AstNode>, ErrorEmitted> {
    let attributes = item_attrs_to_map(ec, &item.attribute_list)?;

    let span = item.span();
    let contents = match item.value {
        ItemKind::Use(item_use) => {
            let use_statements = item_use_to_use_statements(ec, item_use)?;
            use_statements
                .into_iter()
                .map(AstNodeContent::UseStatement)
                .collect()
        }
        ItemKind::Struct(item_struct) => {
            let struct_declaration = item_struct_to_struct_declaration(ec, item_struct)?;
            vec![AstNodeContent::Declaration(Declaration::StructDeclaration(
                struct_declaration,
            ))]
        }
        ItemKind::Enum(item_enum) => {
            let enum_declaration = item_enum_to_enum_declaration(ec, item_enum)?;
            vec![AstNodeContent::Declaration(Declaration::EnumDeclaration(
                enum_declaration,
            ))]
        }
        ItemKind::Fn(item_fn) => {
            let function_declaration = item_fn_to_function_declaration(ec, item_fn, &attributes)?;
            for param in &function_declaration.parameters {
                if let Ok(ty) = resolve_type(param.type_id, &param.type_span) {
                    if matches!(ty, TypeInfo::SelfType) {
                        let error = ConvertParseTreeError::SelfParameterNotAllowedForFreeFn {
                            span: param.type_span.clone(),
                        };
                        return Err(ec.error(error));
                    }
                }
            }
            vec![AstNodeContent::Declaration(
                Declaration::FunctionDeclaration(function_declaration),
            )]
        }
        ItemKind::Trait(item_trait) => {
            let trait_declaration = item_trait_to_trait_declaration(ec, item_trait)?;
            vec![AstNodeContent::Declaration(Declaration::TraitDeclaration(
                trait_declaration,
            ))]
        }
        ItemKind::Impl(item_impl) => {
            let declaration = item_impl_to_declaration(ec, item_impl)?;
            vec![AstNodeContent::Declaration(declaration)]
        }
        ItemKind::Abi(item_abi) => {
            let abi_declaration = item_abi_to_abi_declaration(ec, item_abi)?;
            vec![AstNodeContent::Declaration(Declaration::AbiDeclaration(
                abi_declaration,
            ))]
        }
        ItemKind::Const(item_const) => {
            let constant_declaration = item_const_to_constant_declaration(ec, item_const)?;
            vec![AstNodeContent::Declaration(
                Declaration::ConstantDeclaration(constant_declaration),
            )]
        }
        ItemKind::Storage(item_storage) => {
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

// Each item may have a list of attributes, each with a name (the key to the hashmap) and a list of
// zero or more args.  Attributes may be specified more than once in which case we use the union of
// their args.
//
// E.g.,
//
//   #[foo(bar)]
//   #[foo(baz, xyzzy)]
//
// is essentially equivalent to
//
//   #[foo(bar, baz, xyzzy)]
//
// but no uniquing is done so
//
//   #[foo(bar)]
//   #[foo(bar)]
//
// is
//
//   #[foo(bar, bar)]

type AttributesMap<'a> = HashMap<&'a str, Vec<&'a Ident>>;

fn item_attrs_to_map<'a>(
    ec: &mut ErrorContext,
    attribute_list: &'a [AttributeDecl],
) -> Result<AttributesMap<'a>, ErrorEmitted> {
    let mut attrs_map = AttributesMap::new();
    for attr_decl in attribute_list {
        let attr = attr_decl.attribute.get();
        let name = attr.name.as_str();
        if !VALID_ATTRIBUTE_NAMES.contains(&name) {
            ec.warning(CompileWarning {
                span: attr_decl.span().clone(),
                warning_content: Warning::UnrecognizedAttribute {
                    attrib_name: attr.name.clone(),
                },
            })
        }
        let mut args = attr
            .args
            .as_ref()
            .map(|parens| parens.get().into_iter().collect())
            .unwrap_or_else(Vec::new);
        match attrs_map.get_mut(name) {
            Some(old_args) => {
                old_args.append(&mut args);
            }
            None => {
                attrs_map.insert(name, args);
            }
        }
    }
    Ok(attrs_map)
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
    let mut errors = Vec::new();
    let span = item_struct.span();
    let fields = item_struct
        .fields
        .into_inner()
        .into_iter()
        .map(|type_field| type_field_to_struct_field(ec, type_field.value))
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

    if let Some(errors) = ec.errors(errors) {
        return Err(errors);
    }

    let struct_declaration = StructDeclaration {
        name: item_struct.name,
        fields,
        type_parameters: generic_params_opt_to_type_parameters(
            ec,
            item_struct.generics,
            item_struct.where_clause_opt,
        )?,
        visibility: pub_token_opt_to_visibility(item_struct.visibility),
        span,
    };
    Ok(struct_declaration)
}

fn item_enum_to_enum_declaration(
    ec: &mut ErrorContext,
    item_enum: ItemEnum,
) -> Result<EnumDeclaration, ErrorEmitted> {
    let mut errors = Vec::new();
    let span = item_enum.span();
    let variants = item_enum
        .fields
        .into_inner()
        .into_iter()
        .enumerate()
        .map(|(tag, type_field)| type_field_to_enum_variant(ec, type_field.value, tag))
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

    if let Some(errors) = ec.errors(errors) {
        return Err(errors);
    }

    let enum_declaration = EnumDeclaration {
        name: item_enum.name,
        type_parameters: generic_params_opt_to_type_parameters(
            ec,
            item_enum.generics,
            item_enum.where_clause_opt,
        )?,
        variants,
        span,
        visibility: pub_token_opt_to_visibility(item_enum.visibility),
    };
    Ok(enum_declaration)
}

fn item_fn_to_function_declaration(
    ec: &mut ErrorContext,
    item_fn: ItemFn,
    attributes: &AttributesMap,
) -> Result<FunctionDeclaration, ErrorEmitted> {
    let span = item_fn.span();
    let return_type_span = match &item_fn.fn_signature.return_type_opt {
        Some((_right_arrow_token, ty)) => ty.span(),
        None => item_fn.fn_signature.span(),
    };
    Ok(FunctionDeclaration {
        purity: get_attributed_purity(ec, attributes)?,
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
        type_parameters: generic_params_opt_to_type_parameters(
            ec,
            item_fn.fn_signature.generics,
            item_fn.fn_signature.where_clause_opt,
        )?,
        return_type_span,
    })
}

fn get_attributed_purity(
    ec: &mut ErrorContext,
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
    match attributes.get(STORAGE_PURITY_ATTRIBUTE_NAME) {
        Some(args) if !args.is_empty() => {
            for arg in args {
                match arg.as_str() {
                    STORAGE_PURITY_READ_NAME => add_impurity(Purity::Reads, Purity::Writes),
                    STORAGE_PURITY_WRITE_NAME => add_impurity(Purity::Writes, Purity::Reads),
                    _otherwise => {
                        return Err(ec.error(ConvertParseTreeError::InvalidAttributeArgument {
                            attribute: "storage".to_owned(),
                            span: arg.span(),
                        }));
                    }
                }
            }
            Ok(purity)
        }
        _otherwise => Ok(Purity::Pure),
    }
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
            .map(|(fn_signature, _semicolon_token)| {
                let attributes = item_attrs_to_map(ec, &fn_signature.attribute_list)?;
                fn_signature_to_trait_fn(ec, fn_signature.value, &attributes)
            })
            .collect::<Result<_, _>>()?
    };
    let methods = match item_trait.trait_defs_opt {
        None => Vec::new(),
        Some(trait_defs) => trait_defs
            .into_inner()
            .into_iter()
            .map(|item_fn| {
                let attributes = item_attrs_to_map(ec, &item_fn.attribute_list)?;
                item_fn_to_function_declaration(ec, item_fn.value, &attributes)
            })
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
    let type_implementing_for_span = item_impl.ty.span();
    let type_implementing_for = ty_to_type_info(ec, item_impl.ty)?;
    let functions = {
        item_impl
            .contents
            .into_inner()
            .into_iter()
            .map(|item| {
                let attributes = item_attrs_to_map(ec, &item.attribute_list)?;
                item_fn_to_function_declaration(ec, item.value, &attributes)
            })
            .collect::<Result<_, _>>()?
    };

    let type_parameters = generic_params_opt_to_type_parameters(
        ec,
        item_impl.generic_params_opt,
        item_impl.where_clause_opt,
    )?;

    match item_impl.trait_opt {
        Some((path_type, _for_token)) => {
            let impl_trait = ImplTrait {
                trait_name: path_type_to_call_path(ec, path_type)?,
                type_implementing_for,
                type_implementing_for_span,
                type_parameters,
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
                .map(|(fn_signature, _semicolon_token)| {
                    let attributes = item_attrs_to_map(ec, &fn_signature.attribute_list)?;
                    fn_signature_to_trait_fn(ec, fn_signature.value, &attributes)
                })
                .collect::<Result<_, _>>()?
        },
        methods: match item_abi.abi_defs_opt {
            None => Vec::new(),
            Some(abi_defs) => abi_defs
                .into_inner()
                .into_iter()
                .map(|item_fn| {
                    let attributes = item_attrs_to_map(ec, &item_fn.attribute_list)?;
                    item_fn_to_function_declaration(ec, item_fn.value, &attributes)
                })
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
        visibility: pub_token_opt_to_visibility(item_const.visibility),
    })
}

fn item_storage_to_storage_declaration(
    ec: &mut ErrorContext,
    item_storage: ItemStorage,
) -> Result<StorageDeclaration, ErrorEmitted> {
    let mut errors = Vec::new();
    let span = item_storage.span();
    let fields: Vec<StorageField> = item_storage
        .fields
        .into_inner()
        .into_iter()
        .map(|storage_field| storage_field_to_storage_field(ec, storage_field))
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

    if let Some(errors) = ec.errors(errors) {
        return Err(errors);
    }

    let storage_declaration = StorageDeclaration { span, fields };
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
        type_info: ty_to_type_info(ec, type_field.ty)?,
        span,
        type_span,
    };
    Ok(struct_field)
}

fn generic_params_opt_to_type_parameters(
    ec: &mut ErrorContext,
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

        param_to_edit
            .trait_constraints
            .extend(
                traits_to_call_paths(ec, bounds)?
                    .iter()
                    .map(|call_path| TraitConstraint {
                        call_path: call_path.clone(),
                    }),
            );
    }
    if let Some(errors) = ec.errors(errors) {
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
    ec: &mut ErrorContext,
    type_field: TypeField,
    tag: usize,
) -> Result<EnumVariant, ErrorEmitted> {
    let span = type_field.span();
    let enum_variant = EnumVariant {
        name: type_field.name,
        type_info: ty_to_type_info(ec, type_field.ty)?,
        tag,
        span,
    };
    Ok(enum_variant)
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
            let final_ast_node = expr_to_ast_node(ec, *expr, false)?;
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
            ref_self,
            mutable_self,
            args_opt,
        } => {
            let mut function_parameters = vec![FunctionParameter {
                name: Ident::new(self_token.span()),
                is_reference: ref_self.is_some(),
                is_mutable: mutable_self.is_some(),
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

    let mut unique_params = HashSet::<Ident>::default();
    for fn_param in &function_parameters {
        let already_used = !unique_params.insert(fn_param.name.clone());
        if already_used {
            return Err(
                ec.error(ConvertParseTreeError::DuplicateParameterIdentifier {
                    name: fn_param.name.clone(),
                    span: fn_param.name.span(),
                }),
            );
        }
    }

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
        Ty::Tuple(parenthesized_ty_tuple_descriptor) => {
            TypeInfo::Tuple(ty_tuple_descriptor_to_type_arguments(
                ec,
                parenthesized_ty_tuple_descriptor.into_inner(),
            )?)
        }
        Ty::Array(bracketed_ty_array_descriptor) => {
            let ty_array_descriptor = bracketed_ty_array_descriptor.into_inner();
            let initial_elem_ty = insert_type(ty_to_type_info(ec, *ty_array_descriptor.ty)?);
            TypeInfo::Array(
                initial_elem_ty,
                expr_to_usize(ec, *ty_array_descriptor.length)?,
                initial_elem_ty,
            )
        }
        Ty::Str { length, .. } => TypeInfo::Str(expr_to_u64(ec, *length.into_inner())?),
        Ty::Infer { .. } => TypeInfo::Unknown,
    };
    Ok(type_info)
}

fn ty_to_type_argument(ec: &mut ErrorContext, ty: Ty) -> Result<TypeArgument, ErrorEmitted> {
    let span = ty.span();
    let initial_type_id = insert_type(ty_to_type_info(ec, ty)?);
    let type_argument = TypeArgument {
        type_id: initial_type_id,
        initial_type_id,
        span,
    };
    Ok(type_argument)
}

fn fn_signature_to_trait_fn(
    ec: &mut ErrorContext,
    fn_signature: FnSignature,
    attributes: &AttributesMap,
) -> Result<TraitFn, ErrorEmitted> {
    let return_type_span = match &fn_signature.return_type_opt {
        Some((_right_arrow_token, ty)) => ty.span(),
        None => fn_signature.span(),
    };
    let trait_fn = TraitFn {
        name: fn_signature.name,
        purity: get_attributed_purity(ec, attributes)?,
        parameters: fn_args_to_function_parameters(ec, fn_signature.arguments.into_inner())?,
        return_type: match fn_signature.return_type_opt {
            Some((_right_arrow_token, ty)) => ty_to_type_info(ec, ty)?,
            None => TypeInfo::Tuple(Vec::new()),
        },
        return_type_span,
    };
    Ok(trait_fn)
}

fn traits_to_call_paths(
    ec: &mut ErrorContext,
    traits: Traits,
) -> Result<Vec<CallPath>, ErrorEmitted> {
    let mut call_paths = vec![path_type_to_call_path(ec, traits.prefix)?];
    for (_add_token, suffix) in traits.suffixes {
        let supertrait = path_type_to_call_path(ec, suffix)?;
        call_paths.push(supertrait);
    }
    Ok(call_paths)
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
    is_statement: bool,
) -> Result<AstNode, ErrorEmitted> {
    let span = expr.span();
    let ast_node = match expr {
        Expr::Return { expr_opt, .. } => {
            let expression = match expr_opt {
                Some(expr) => expr_to_expression(ec, *expr)?,
                None => Expression {
                    kind: ExpressionKind::Tuple(Vec::new()),
                    span: span.clone(),
                },
            };
            AstNode {
                content: AstNodeContent::ReturnStatement(ReturnStatement { expr: expression }),
                span,
            }
        }
        Expr::Reassignment {
            assignable,
            expr,
            reassignment_op:
                ReassignmentOp {
                    variant: op_variant,
                    span: op_span,
                },
        } => match op_variant {
            ReassignmentOpVariant::Equals => AstNode {
                content: AstNodeContent::Declaration(Declaration::Reassignment(Reassignment {
                    lhs: assignable_to_reassignment_target(ec, assignable)?,
                    rhs: expr_to_expression(ec, *expr)?,
                    span: span.clone(),
                })),
                span,
            },
            op_variant => {
                let lhs = assignable_to_reassignment_target(ec, assignable.clone())?;
                let rhs = binary_op_call(
                    op_variant.core_name(),
                    op_span,
                    span.clone(),
                    assignable_to_expression(ec, assignable)?,
                    expr_to_expression(ec, *expr)?,
                )?;
                let content =
                    AstNodeContent::Declaration(Declaration::Reassignment(Reassignment {
                        lhs,
                        rhs,
                        span: span.clone(),
                    }));
                AstNode { content, span }
            }
        },
        expr => {
            let expression = expr_to_expression(ec, expr)?;
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
        }
    };
    Ok(ast_node)
}

fn abi_cast_args_to_abi_cast_expression(
    ec: &mut ErrorContext,
    args: Parens<AbiCastArgs>,
) -> Result<Box<AbiCastExpression>, ErrorEmitted> {
    let AbiCastArgs { name, address, .. } = args.into_inner();
    let abi_name = path_type_to_call_path(ec, name)?;
    let address = Box::new(expr_to_expression(ec, *address)?);
    Ok(Box::new(AbiCastExpression { abi_name, address }))
}

fn struct_path_and_fields_to_struct_expression(
    ec: &mut ErrorContext,
    path: PathExpr,
    fields: Braces<Punctuated<ExprStructField, CommaToken>>,
) -> Result<Box<StructExpression>, ErrorEmitted> {
    let call_path_binding = path_expr_to_call_path_binding(ec, path)?;
    let fields = {
        fields
            .into_inner()
            .into_iter()
            .map(|expr_struct_field| {
                expr_struct_field_to_struct_expression_field(ec, expr_struct_field)
            })
            .collect::<Result<_, _>>()?
    };
    Ok(Box::new(StructExpression {
        call_path_binding,
        fields,
    }))
}

fn method_call_fields_to_method_application_expression(
    ec: &mut ErrorContext,
    target: Box<Expr>,
    name: Ident,
    contract_args_opt: Option<Braces<Punctuated<ExprStructField, CommaToken>>>,
    args: Parens<Punctuated<Expr, CommaToken>>,
) -> Result<Box<MethodApplicationExpression>, ErrorEmitted> {
    let method_name_binding = TypeBinding {
        inner: MethodName::FromModule {
            method_name: name.clone(),
        },
        type_arguments: vec![],
        span: name.span(),
    };
    let contract_call_params = match contract_args_opt {
        None => Vec::new(),
        Some(contract_args) => contract_args
            .into_inner()
            .into_iter()
            .map(|expr_struct_field| {
                expr_struct_field_to_struct_expression_field(ec, expr_struct_field)
            })
            .collect::<Result<_, _>>()?,
    };
    let arguments = iter::once(*target)
        .chain(args.into_inner().into_iter())
        .map(|expr| expr_to_expression(ec, expr))
        .collect::<Result<_, _>>()?;
    Ok(Box::new(MethodApplicationExpression {
        method_name_binding,
        contract_call_params,
        arguments,
    }))
}

fn expr_func_app_to_expression_kind(
    ec: &mut ErrorContext,
    func: Box<Expr>,
    args: Parens<Punctuated<Expr, CommaToken>>,
) -> Result<ExpressionKind, ErrorEmitted> {
    let span = Span::join(func.span(), args.span());
    let path_expr = match *func {
        Expr::Path(path_expr) => path_expr,
        _ => {
            let error = ConvertParseTreeError::FunctionArbitraryExpression { span: func.span() };
            return Err(ec.error(error));
        }
    };
    let PathExpr {
        root_opt,
        prefix,
        mut suffix,
    } = path_expr;
    let is_absolute = path_root_opt_to_bool(ec, root_opt)?;
    let (
        prefixes,
        method_type_opt,
        parent_type_arguments,
        parent_type_arguments_span,
        suffix_path_expr,
    ) = match suffix.pop() {
        Some((_double_colon_token, call_path_suffix)) => match suffix.pop() {
            Some((_double_colon_token, maybe_method_segment)) => {
                let PathExprSegment {
                    fully_qualified,
                    name,
                    generics_opt,
                } = maybe_method_segment;
                let (parent_type_arguments, parent_type_arguments_span) = match generics_opt {
                    Some((_double_colon_token, generic_args)) => (
                        generic_args_to_type_arguments(ec, generic_args.clone())?,
                        Some(generic_args.span()),
                    ),
                    None => (Vec::new(), None),
                };
                let mut prefixes = vec![path_expr_segment_to_ident(ec, prefix)?];
                for (_double_colon_token, call_path_prefix) in suffix {
                    let ident = path_expr_segment_to_ident(ec, call_path_prefix)?;
                    prefixes.push(ident);
                }
                if fully_qualified.is_some() {
                    (
                        prefixes,
                        Some(name),
                        parent_type_arguments,
                        parent_type_arguments_span,
                        call_path_suffix,
                    )
                } else {
                    prefixes.push(name);
                    (
                        prefixes,
                        None,
                        parent_type_arguments,
                        parent_type_arguments_span,
                        call_path_suffix,
                    )
                }
            }
            None => {
                let PathExprSegment {
                    fully_qualified,
                    name,
                    generics_opt,
                } = prefix;
                let (parent_type_arguments, parent_type_arguments_span) = match generics_opt {
                    Some((_double_colon_token, generic_args)) => (
                        generic_args_to_type_arguments(ec, generic_args.clone())?,
                        Some(generic_args.span()),
                    ),
                    None => (Vec::new(), None),
                };
                if fully_qualified.is_some() {
                    (
                        Vec::new(),
                        Some(name),
                        parent_type_arguments,
                        parent_type_arguments_span,
                        call_path_suffix,
                    )
                } else {
                    (
                        vec![name],
                        None,
                        parent_type_arguments,
                        parent_type_arguments_span,
                        call_path_suffix,
                    )
                }
            }
        },
        None => (Vec::new(), None, vec![], None, prefix),
    };
    let PathExprSegment {
        fully_qualified,
        name: method_name,
        generics_opt,
    } = suffix_path_expr;
    if let Some(tilde_token) = fully_qualified {
        let error = ConvertParseTreeError::FullyQualifiedPathsNotSupportedHere {
            span: tilde_token.span(),
        };
        return Err(ec.error(error));
    }
    let arguments = {
        args.into_inner()
            .into_iter()
            .map(|expr| expr_to_expression(ec, expr))
            .collect::<Result<_, _>>()?
    };
    let expression_kind = match method_type_opt {
        Some(type_name) => {
            let type_info_span = type_name.span();
            let type_info = match type_name_to_type_info_opt(&type_name) {
                Some(type_info) => type_info,
                None => TypeInfo::Custom {
                    name: type_name,
                    type_arguments: None,
                },
            };
            let call_path_binding = TypeBinding {
                inner: CallPath {
                    prefixes,
                    suffix: (type_info, type_info_span.clone()),
                    is_absolute,
                },
                type_arguments: parent_type_arguments,
                span: parent_type_arguments_span
                    .map(|parent_type_arguments_span| {
                        Span::join(type_info_span.clone(), parent_type_arguments_span)
                    })
                    .unwrap_or_else(|| type_info_span.clone()),
            };
            let (method_type_arguments, method_type_arguments_span) = match generics_opt {
                Some((_double_colon_token, generic_args)) => (
                    generic_args_to_type_arguments(ec, generic_args.clone())?,
                    Some(generic_args.span()),
                ),
                None => (Vec::new(), None),
            };
            let method_name_binding = TypeBinding {
                inner: MethodName::FromType {
                    call_path_binding,
                    method_name: method_name.clone(),
                },
                type_arguments: method_type_arguments,
                span: method_type_arguments_span
                    .map(|method_type_arguments_span| {
                        Span::join(method_name.span(), method_type_arguments_span)
                    })
                    .unwrap_or_else(|| method_name.span()),
            };
            ExpressionKind::MethodApplication(Box::new(MethodApplicationExpression {
                method_name_binding,
                contract_call_params: Vec::new(),
                arguments,
            }))
        }
        None => {
            if !parent_type_arguments.is_empty() {
                let error = ConvertParseTreeError::GenericsNotSupportedHere {
                    span: parent_type_arguments_span.unwrap(),
                };
                return Err(ec.error(error));
            }
            let (type_arguments, type_arguments_span) = match generics_opt {
                Some((_double_colon_token, generic_args)) => (
                    generic_args_to_type_arguments(ec, generic_args.clone())?,
                    Some(generic_args.span()),
                ),
                None => (Vec::new(), None),
            };
            match Intrinsic::try_from_str(method_name.as_str()) {
                Some(intrinsic) if prefixes.is_empty() && !is_absolute => {
                    ExpressionKind::IntrinsicFunction(IntrinsicFunctionExpression {
                        kind_binding: TypeBinding {
                            inner: intrinsic,
                            type_arguments,
                            span: type_arguments_span
                                .map(|type_arguments_span| {
                                    Span::join(span.clone(), type_arguments_span)
                                })
                                .unwrap_or_else(|| span.clone()),
                        },
                        arguments,
                    })
                }
                _ => {
                    let call_path = CallPath {
                        prefixes,
                        suffix: method_name,
                        is_absolute,
                    };
                    let call_path_binding = TypeBinding {
                        inner: call_path.clone(),
                        type_arguments,
                        span: call_path.span(), // TODO: change this span so that it includes the type arguments
                    };
                    if call_path.prefixes.is_empty() {
                        ExpressionKind::FunctionApplication(Box::new(
                            FunctionApplicationExpression {
                                call_path_binding,
                                arguments,
                            },
                        ))
                    } else {
                        ExpressionKind::DelineatedPath(Box::new(DelineatedPathExpression {
                            call_path_binding,
                            args: arguments,
                        }))
                    }
                }
            }
        }
    };
    Ok(expression_kind)
}

fn expr_to_expression(ec: &mut ErrorContext, expr: Expr) -> Result<Expression, ErrorEmitted> {
    let span = expr.span();
    let expression = match expr {
        Expr::Path(path_expr) => path_expr_to_expression(ec, path_expr)?,
        Expr::Literal(literal) => Expression {
            kind: ExpressionKind::Literal(literal_to_literal(ec, literal)?),
            span,
        },
        Expr::AbiCast { args, .. } => {
            let abi_cast_expression = abi_cast_args_to_abi_cast_expression(ec, args)?;
            Expression {
                kind: ExpressionKind::AbiCast(abi_cast_expression),
                span,
            }
        }
        Expr::Struct { path, fields } => {
            let struct_expression = struct_path_and_fields_to_struct_expression(ec, path, fields)?;
            Expression {
                kind: ExpressionKind::Struct(struct_expression),
                span,
            }
        }
        Expr::Tuple(parenthesized_expr_tuple_descriptor) => {
            let fields = expr_tuple_descriptor_to_expressions(
                ec,
                parenthesized_expr_tuple_descriptor.into_inner(),
            )?;
            Expression {
                kind: ExpressionKind::Tuple(fields),
                span,
            }
        }
        Expr::Parens(parens) => expr_to_expression(ec, *parens.into_inner())?,
        Expr::Block(braced_code_block_contents) => {
            braced_code_block_contents_to_expression(ec, braced_code_block_contents)?
        }
        Expr::Array(bracketed_expr_array_descriptor) => {
            match bracketed_expr_array_descriptor.into_inner() {
                ExprArrayDescriptor::Sequence(exprs) => {
                    let contents = exprs
                        .into_iter()
                        .map(|expr| expr_to_expression(ec, expr))
                        .collect::<Result<_, _>>()?;
                    Expression {
                        kind: ExpressionKind::Array(contents),
                        span,
                    }
                }
                ExprArrayDescriptor::Repeat { value, length, .. } => {
                    let expression = expr_to_expression(ec, *value)?;
                    let length = expr_to_usize(ec, *length)?;
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
            let asm_expression = asm_block_to_asm_expression(ec, asm_block)?;
            Expression {
                kind: ExpressionKind::Asm(asm_expression),
                span,
            }
        }
        Expr::Return { return_token, .. } => {
            let error = ConvertParseTreeError::ReturnOutsideOfBlock {
                span: return_token.span(),
            };
            return Err(ec.error(error));
        }
        Expr::If(if_expr) => if_expr_to_expression(ec, if_expr)?,
        Expr::Match {
            value, branches, ..
        } => {
            let value = expr_to_expression(ec, *value)?;
            let var_decl_span = value.span();

            // Generate a deterministic name for the variable returned by the match expression.
            // Because the parser is single threaded, the name generated below will be stable.
            static COUNTER: AtomicUsize = AtomicUsize::new(0);
            let match_return_var_name = format!(
                "{}{}",
                crate::constants::MATCH_RETURN_VAR_NAME_PREFIX,
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
                    .map(|match_branch| match_branch_to_match_branch(ec, match_branch))
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
                condition: Box::new(expr_to_expression(ec, *condition)?),
                body: braced_code_block_contents_to_code_block(ec, block)?,
            }),
            span,
        },
        Expr::FuncApp { func, args } => {
            let kind = expr_func_app_to_expression_kind(ec, func, args)?;
            Expression { kind, span }
        }
        Expr::Index { target, arg } => Expression {
            kind: ExpressionKind::ArrayIndex(ArrayIndexExpression {
                prefix: Box::new(expr_to_expression(ec, *target)?),
                index: Box::new(expr_to_expression(ec, *arg.into_inner())?),
            }),
            span,
        },
        Expr::MethodCall {
            target,
            name,
            args,
            contract_args_opt,
            ..
        } => {
            let method_application_expression =
                method_call_fields_to_method_application_expression(
                    ec,
                    target,
                    name,
                    contract_args_opt,
                    args,
                )?;
            Expression {
                kind: ExpressionKind::MethodApplication(method_application_expression),
                span,
            }
        }
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
                    Expression {
                        kind: ExpressionKind::StorageAccess(StorageAccessExpression {
                            field_names,
                        }),
                        span,
                    }
                }
                None => Expression {
                    kind: ExpressionKind::Subfield(SubfieldExpression {
                        prefix: Box::new(expr_to_expression(ec, *target)?),
                        field_to_access: name,
                    }),
                    span,
                },
            }
        }
        Expr::TupleFieldProjection {
            target,
            field,
            field_span,
            ..
        } => Expression {
            kind: ExpressionKind::TupleIndex(TupleIndexExpression {
                prefix: Box::new(expr_to_expression(ec, *target)?),
                index: match usize::try_from(field) {
                    Ok(index) => index,
                    Err(..) => {
                        let error =
                            ConvertParseTreeError::TupleIndexOutOfRange { span: field_span };
                        return Err(ec.error(error));
                    }
                },
                index_span: field_span,
            }),
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
        } => {
            let lhs = expr_to_expression(ec, *lhs)?;
            let rhs = expr_to_expression(ec, *rhs)?;
            binary_op_call("multiply", star_token.span(), span, lhs, rhs)?
        }
        Expr::Div {
            lhs,
            forward_slash_token,
            rhs,
        } => {
            let lhs = expr_to_expression(ec, *lhs)?;
            let rhs = expr_to_expression(ec, *rhs)?;
            binary_op_call("divide", forward_slash_token.span(), span, lhs, rhs)?
        }
        Expr::Modulo {
            lhs,
            percent_token,
            rhs,
        } => {
            let lhs = expr_to_expression(ec, *lhs)?;
            let rhs = expr_to_expression(ec, *rhs)?;
            binary_op_call("modulo", percent_token.span(), span, lhs, rhs)?
        }
        Expr::Add {
            lhs,
            add_token,
            rhs,
        } => {
            let lhs = expr_to_expression(ec, *lhs)?;
            let rhs = expr_to_expression(ec, *rhs)?;
            binary_op_call("add", add_token.span(), span, lhs, rhs)?
        }
        Expr::Sub {
            lhs,
            sub_token,
            rhs,
        } => {
            let lhs = expr_to_expression(ec, *lhs)?;
            let rhs = expr_to_expression(ec, *rhs)?;
            binary_op_call("subtract", sub_token.span(), span, lhs, rhs)?
        }
        Expr::Shl {
            lhs,
            shl_token,
            rhs,
        } => {
            let lhs = expr_to_expression(ec, *lhs)?;
            let rhs = expr_to_expression(ec, *rhs)?;
            binary_op_call("lsh", shl_token.span(), span, lhs, rhs)?
        }
        Expr::Shr {
            lhs,
            shr_token,
            rhs,
        } => {
            let lhs = expr_to_expression(ec, *lhs)?;
            let rhs = expr_to_expression(ec, *rhs)?;
            binary_op_call("rsh", shr_token.span(), span, lhs, rhs)?
        }
        Expr::BitAnd {
            lhs,
            ampersand_token,
            rhs,
        } => {
            let lhs = expr_to_expression(ec, *lhs)?;
            let rhs = expr_to_expression(ec, *rhs)?;
            binary_op_call("binary_and", ampersand_token.span(), span, lhs, rhs)?
        }
        Expr::BitXor {
            lhs,
            caret_token,
            rhs,
        } => {
            let lhs = expr_to_expression(ec, *lhs)?;
            let rhs = expr_to_expression(ec, *rhs)?;
            binary_op_call("binary_xor", caret_token.span(), span, lhs, rhs)?
        }
        Expr::BitOr {
            lhs,
            pipe_token,
            rhs,
        } => {
            let lhs = expr_to_expression(ec, *lhs)?;
            let rhs = expr_to_expression(ec, *rhs)?;
            binary_op_call("binary_or", pipe_token.span(), span, lhs, rhs)?
        }
        Expr::Equal {
            lhs,
            double_eq_token,
            rhs,
        } => {
            let lhs = expr_to_expression(ec, *lhs)?;
            let rhs = expr_to_expression(ec, *rhs)?;
            binary_op_call("eq", double_eq_token.span(), span, lhs, rhs)?
        }
        Expr::NotEqual {
            lhs,
            bang_eq_token,
            rhs,
        } => {
            let lhs = expr_to_expression(ec, *lhs)?;
            let rhs = expr_to_expression(ec, *rhs)?;
            binary_op_call("neq", bang_eq_token.span(), span, lhs, rhs)?
        }
        Expr::LessThan {
            lhs,
            less_than_token,
            rhs,
        } => {
            let lhs = expr_to_expression(ec, *lhs)?;
            let rhs = expr_to_expression(ec, *rhs)?;
            binary_op_call("lt", less_than_token.span(), span, lhs, rhs)?
        }
        Expr::GreaterThan {
            lhs,
            greater_than_token,
            rhs,
        } => {
            let lhs = expr_to_expression(ec, *lhs)?;
            let rhs = expr_to_expression(ec, *rhs)?;
            binary_op_call("gt", greater_than_token.span(), span, lhs, rhs)?
        }
        Expr::LessThanEq {
            lhs,
            less_than_eq_token,
            rhs,
        } => {
            let lhs = expr_to_expression(ec, *lhs)?;
            let rhs = expr_to_expression(ec, *rhs)?;
            binary_op_call("le", less_than_eq_token.span(), span, lhs, rhs)?
        }
        Expr::GreaterThanEq {
            lhs,
            greater_than_eq_token,
            rhs,
        } => {
            let lhs = expr_to_expression(ec, *lhs)?;
            let rhs = expr_to_expression(ec, *rhs)?;
            binary_op_call("ge", greater_than_eq_token.span(), span, lhs, rhs)?
        }
        Expr::LogicalAnd { lhs, rhs, .. } => Expression {
            kind: ExpressionKind::LazyOperator(LazyOperatorExpression {
                op: LazyOp::And,
                lhs: Box::new(expr_to_expression(ec, *lhs)?),
                rhs: Box::new(expr_to_expression(ec, *rhs)?),
            }),
            span,
        },
        Expr::LogicalOr { lhs, rhs, .. } => Expression {
            kind: ExpressionKind::LazyOperator(LazyOperatorExpression {
                op: LazyOp::Or,
                lhs: Box::new(expr_to_expression(ec, *lhs)?),
                rhs: Box::new(expr_to_expression(ec, *rhs)?),
            }),
            span,
        },
        Expr::Reassignment { .. } => {
            let error = ConvertParseTreeError::ReassignmentOutsideOfBlock { span };
            return Err(ec.error(error));
        }
        Expr::Break { .. } => Expression {
            kind: ExpressionKind::CodeBlock(CodeBlock {
                contents: vec![AstNode {
                    content: AstNodeContent::Declaration(Declaration::Break { span: span.clone() }),
                    span: span.clone(),
                }],
                whole_block_span: span.clone(),
            }),
            span,
        },
        Expr::Continue { .. } => Expression {
            kind: ExpressionKind::CodeBlock(CodeBlock {
                contents: vec![AstNode {
                    content: AstNodeContent::Declaration(Declaration::Continue {
                        span: span.clone(),
                    }),
                    span: span.clone(),
                }],
                whole_block_span: span.clone(),
            }),
            span,
        },
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
    let call_path_binding = TypeBinding {
        inner: CallPath {
            prefixes: vec![
                Ident::new_with_override("core", op_span.clone()),
                Ident::new_with_override("ops", op_span.clone()),
            ],
            suffix: Ident::new_with_override(name, op_span.clone()),
            is_absolute: false,
        },
        type_arguments: vec![],
        span: op_span,
    };
    Ok(Expression {
        kind: ExpressionKind::FunctionApplication(Box::new(FunctionApplicationExpression {
            call_path_binding,
            arguments: vec![expr_to_expression(ec, arg)?],
        })),
        span,
    })
}

fn binary_op_call(
    name: &'static str,
    op_span: Span,
    span: Span,
    lhs: Expression,
    rhs: Expression,
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
            arguments: vec![lhs, rhs],
        })),
        span,
    })
}

fn storage_field_to_storage_field(
    ec: &mut ErrorContext,
    storage_field: sway_ast::StorageField,
) -> Result<StorageField, ErrorEmitted> {
    let storage_field = StorageField {
        name: storage_field.name,
        type_info: ty_to_type_info(ec, storage_field.ty)?,
        initializer: expr_to_expression(ec, storage_field.initializer)?,
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
        Statement::Expr { expr, .. } => vec![expr_to_ast_node(ec, expr, true)?],
    };
    Ok(ast_nodes)
}

fn fn_arg_to_function_parameter(
    ec: &mut ErrorContext,
    fn_arg: FnArg,
) -> Result<FunctionParameter, ErrorEmitted> {
    let type_span = fn_arg.ty.span();
    let pat_span = fn_arg.pattern.span();
    let (reference, mutable, name) = match fn_arg.pattern {
        Pattern::Wildcard { .. } => {
            let error = ConvertParseTreeError::WildcardPatternsNotSupportedHere { span: pat_span };
            return Err(ec.error(error));
        }
        Pattern::Var {
            reference,
            mutable,
            name,
        } => (reference, mutable, name),
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
        is_reference: reference.is_some(),
        is_mutable: mutable.is_some(),
        type_id: insert_type(ty_to_type_info(ec, fn_arg.ty)?),
        type_span,
    };
    Ok(function_parameter)
}

fn expr_to_usize(ec: &mut ErrorContext, expr: Expr) -> Result<usize, ErrorEmitted> {
    let span = expr.span();
    let value = match expr {
        Expr::Literal(sway_ast::Literal::Int(lit_int)) => {
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
        Expr::Literal(sway_ast::Literal::Int(lit_int)) => {
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
        Expression {
            kind: ExpressionKind::Variable(name),
            span,
        }
    } else {
        let call_path = path_expr_to_call_path(ec, path_expr)?;
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
    ec: &mut ErrorContext,
    braced_code_block_contents: Braces<CodeBlockContents>,
) -> Result<Expression, ErrorEmitted> {
    let span = braced_code_block_contents.span();
    let code_block = braced_code_block_contents_to_code_block(ec, braced_code_block_contents)?;
    Ok(Expression {
        kind: ExpressionKind::CodeBlock(code_block),
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
    let then_block = Expression {
        kind: ExpressionKind::CodeBlock(braced_code_block_contents_to_code_block(ec, then_block)?),
        span: then_block_span.clone(),
    };
    let else_block = match else_opt {
        None => None,
        Some((_else_token, tail)) => {
            let expression = match tail {
                ControlFlow::Break(braced_code_block_contents) => {
                    braced_code_block_contents_to_expression(ec, braced_code_block_contents)?
                }
                ControlFlow::Continue(if_expr) => if_expr_to_expression(ec, *if_expr)?,
            };
            Some(expression)
        }
    };
    let expression = match condition {
        IfCondition::Expr(condition) => Expression {
            kind: ExpressionKind::If(IfExpression {
                condition: Box::new(expr_to_expression(ec, *condition)?),
                then: Box::new(then_block),
                r#else: else_block.map(Box::new),
            }),
            span,
        },
        IfCondition::Let { lhs, rhs, .. } => {
            let scrutinee = pattern_to_scrutinee(ec, *lhs)?;
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
                    value: Box::new(expr_to_expression(ec, *rhs)?),
                    branches,
                }),
                span,
            }
        }
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
            return Err(ec.error(error));
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
                                return Err(ec.error(error));
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
fn path_expr_to_call_path_binding(
    ec: &mut ErrorContext,
    path_expr: PathExpr,
) -> Result<TypeBinding<CallPath<(TypeInfo, Span)>>, ErrorEmitted> {
    let PathExpr {
        root_opt,
        prefix,
        mut suffix,
    } = path_expr;
    let is_absolute = path_root_opt_to_bool(ec, root_opt)?;
    let (prefixes, type_info, type_info_span, type_arguments) = match suffix.pop() {
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
            let type_info_span = suffix.span();
            let type_info = type_name_to_type_info_opt(&suffix).unwrap_or(TypeInfo::Custom {
                name: suffix,
                type_arguments: None,
            });
            (prefixes, type_info, type_info_span, ty_args)
        }
        None => {
            let (suffix, ty_args) = path_expr_segment_to_ident_or_type_argument(ec, prefix)?;
            let type_info_span = suffix.span();
            let type_info = match type_name_to_type_info_opt(&suffix) {
                Some(type_info) => type_info,
                None => TypeInfo::Custom {
                    name: suffix,
                    type_arguments: None,
                },
            };
            (vec![], type_info, type_info_span, ty_args)
        }
    };
    Ok(TypeBinding {
        inner: CallPath {
            prefixes,
            suffix: (type_info, type_info_span.clone()),
            is_absolute,
        },
        type_arguments,
        span: type_info_span, // TODO: change this span so that it includes the type arguments
    })
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
    Ok(Box::new(AsmExpression {
        registers,
        body,
        returns,
        return_type,
        whole_block_span,
    }))
}

fn match_branch_to_match_branch(
    ec: &mut ErrorContext,
    match_branch: sway_ast::MatchBranch,
) -> Result<MatchBranch, ErrorEmitted> {
    let span = match_branch.span();
    Ok(MatchBranch {
        scrutinee: pattern_to_scrutinee(ec, match_branch.pattern)?,
        result: match match_branch.kind {
            MatchBranchKind::Block { block, .. } => {
                let span = block.span();
                Expression {
                    kind: ExpressionKind::CodeBlock(braced_code_block_contents_to_code_block(
                        ec, block,
                    )?),
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
                    return Err(ec.error(error));
                }
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
            Pattern::Struct { fields, .. } => {
                let mut ast_nodes = Vec::new();

                // Generate a deterministic name for the destructured struct
                // Because the parser is single threaded, the name generated below will be stable.
                static COUNTER: AtomicUsize = AtomicUsize::new(0);
                let destructured_name = format!(
                    "{}{}",
                    crate::constants::DESTRUCTURE_PREFIX,
                    COUNTER.load(Ordering::SeqCst)
                );
                COUNTER.fetch_add(1, Ordering::SeqCst);
                let destructure_name = Ident::new_with_override(
                    Box::leak(destructured_name.into_boxed_str()),
                    span.clone(),
                );

                // Parse the type ascription and the type ascription span.
                // In the event that the user did not provide a type ascription,
                // it is set to TypeInfo::Unknown and the span to None.
                let (type_ascription, type_ascription_span) = match &ty_opt {
                    Some(ty) => {
                        let type_ascription_span = ty.span();
                        let type_ascription = ty_to_type_info(ec, ty.clone())?;
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
                        ec,
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
                let tuple_name = format!(
                    "{}{}",
                    crate::constants::TUPLE_NAME_PREFIX,
                    COUNTER.load(Ordering::SeqCst)
                );
                COUNTER.fetch_add(1, Ordering::SeqCst);
                let tuple_name =
                    Ident::new_with_override(Box::leak(tuple_name.into_boxed_str()), span.clone());

                // Parse the type ascription and the type ascription span.
                // In the event that the user did not provide a type ascription,
                // it is set to TypeInfo::Unknown and the span to None.
                let (type_ascription, type_ascription_span) = match &ty_opt {
                    Some(ty) => {
                        let type_ascription_span = ty.span();
                        let type_ascription = ty_to_type_info(ec, ty.clone())?;
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
                        ec,
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
    let initial_expression = expr_to_expression(ec, statement_let.expr)?;
    unfold(
        ec,
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
    asm_register_declaration: sway_ast::AsmRegisterDeclaration,
) -> Result<AsmRegisterDeclaration, ErrorEmitted> {
    Ok(AsmRegisterDeclaration {
        name: asm_register_declaration.register,
        initializer: asm_register_declaration
            .value_opt
            .map(|(_colon_token, expr)| expr_to_expression(ec, *expr))
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

fn pattern_to_scrutinee(
    ec: &mut ErrorContext,
    pattern: Pattern,
) -> Result<Scrutinee, ErrorEmitted> {
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
                return Err(ec.error(error));
            }
            Scrutinee::Variable { name, span }
        }
        Pattern::Literal(literal) => Scrutinee::Literal {
            value: literal_to_literal(ec, literal)?,
            span,
        },
        Pattern::Constant(path_expr) => {
            let call_path = path_expr_to_call_path(ec, path_expr)?;
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
                    return Err(ec.error(error));
                }
            };
            Scrutinee::EnumScrutinee {
                call_path: path_expr_to_call_path(ec, path)?,
                value: Box::new(pattern_to_scrutinee(ec, value)?),
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

            if let Some(errors) = ec.errors(errors) {
                return Err(errors);
            }

            let scrutinee_fields = fields
                .into_iter()
                .map(|field| pattern_struct_field_to_struct_scrutinee_field(ec, field))
                .collect::<Result<_, _>>()?;

            Scrutinee::StructScrutinee {
                struct_name: path_expr_to_ident(ec, path)?,
                fields: { scrutinee_fields },
                span,
            }
        }
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
            let unknown_type = insert_type(TypeInfo::Unknown);
            return Ok(TypeParameter {
                type_id: unknown_type,
                initial_type_id: unknown_type,
                name_ident: underscore_token.into(),
                trait_constraints: Default::default(),
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
                    .map(|(_colon_token, pattern)| pattern_to_scrutinee(ec, *pattern))
                    .transpose()?,
                span,
            };
            Ok(struct_scrutinee_field)
        }
    }
}

fn assignable_to_expression(
    ec: &mut ErrorContext,
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
                prefix: Box::new(assignable_to_expression(ec, *target)?),
                index: Box::new(expr_to_expression(ec, *arg.into_inner())?),
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
                        prefix: Box::new(assignable_to_expression(ec, *target)?),
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
                    return Err(ec.error(error));
                }
            };
            Expression {
                kind: ExpressionKind::TupleIndex(TupleIndexExpression {
                    prefix: Box::new(assignable_to_expression(ec, *target)?),
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
            Assignable::TupleFieldProjection { .. } => break,
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
            Ok(TypeArgument {
                type_id,
                initial_type_id: type_id,
                span,
            })
        })
        .collect()
}

fn ty_tuple_descriptor_to_type_arguments(
    ec: &mut ErrorContext,
    ty_tuple_descriptor: TyTupleDescriptor,
) -> Result<Vec<TypeArgument>, ErrorEmitted> {
    let type_arguments = match ty_tuple_descriptor {
        TyTupleDescriptor::Nil => vec![],
        TyTupleDescriptor::Cons { head, tail, .. } => {
            let mut type_arguments = vec![ty_to_type_argument(ec, *head)?];
            for ty in tail.into_iter() {
                type_arguments.push(ty_to_type_argument(ec, ty)?);
            }
            type_arguments
        }
    };
    Ok(type_arguments)
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
                    address: None,
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
