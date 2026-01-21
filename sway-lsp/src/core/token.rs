use lsp_types::{Position, Range};
use std::path::PathBuf;
use sway_ast::Intrinsic;
use sway_core::{
    decl_engine::parsed_id::ParsedDeclId,
    language::{
        parsed::{
            AbiCastExpression, AmbiguousPathExpression, Declaration, DelineatedPathExpression,
            EnumVariant, Expression, FunctionApplicationExpression, FunctionParameter,
            MethodApplicationExpression, ModStatement, Scrutinee, StorageField, StorageNamespace,
            StructExpression, StructExpressionField, StructField, StructScrutineeField, Supertrait,
            TraitFn, UseStatement,
        },
        ty,
    },
    transform::Attribute,
    type_system::{TypeId, TypeInfo, TypeParameter},
    Engines, GenericTypeArgument, TraitConstraint, TypeEngine,
};
use sway_types::{Ident, ProgramId, SourceEngine, SourceId, Span, Spanned};

/// The `ParsedAstToken` holds the types produced by the [sway_core::language::parsed::ParseProgram].
/// These tokens have not been type-checked.
/// See this issue https://github.com/FuelLabs/sway/issues/2257 for more information about why they are
/// useful to the language server.
#[derive(Debug, Clone)]
pub enum ParsedAstToken {
    AbiCastExpression(AbiCastExpression),
    AmbiguousPathExpression(AmbiguousPathExpression),
    Attribute(Attribute),
    Declaration(Declaration),
    DelineatedPathExpression(DelineatedPathExpression),
    EnumVariant(EnumVariant),
    ErrorRecovery(Span),
    Expression(Expression),
    FunctionApplicationExpression(FunctionApplicationExpression),
    FunctionParameter(FunctionParameter),
    Ident(Ident),
    ModuleName,
    ModStatement(ModStatement),
    Intrinsic(Intrinsic),
    Keyword(Ident),
    LibrarySpan(Span),
    MethodApplicationExpression(MethodApplicationExpression),
    Scrutinee(Scrutinee),
    StorageField(StorageField),
    StorageNamespace(StorageNamespace),
    StructExpression(StructExpression),
    StructExpressionField(StructExpressionField),
    StructField(StructField),
    StructScrutineeField(StructScrutineeField),
    Supertrait(Supertrait),
    TraitConstraint(TraitConstraint),
    TraitFn(ParsedDeclId<TraitFn>),
    TypeArgument(GenericTypeArgument),
    TypeParameter(TypeParameter),
    UseStatement(UseStatement),
}

/// The `TypedAstToken` holds the types produced by the [sway_core::language::ty::TyProgram].
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum TypedAstToken {
    TypedTypeAliasDeclaration(ty::TyTypeAliasDecl),
    TypedDeclaration(ty::TyDecl),
    TypedExpression(ty::TyExpression),
    TypedScrutinee(ty::TyScrutinee),
    TyStructScrutineeField(ty::TyStructScrutineeField),
    TypedConstantDeclaration(ty::TyConstantDecl),
    TypedConfigurableDeclaration(ty::TyConfigurableDecl),
    TypedConstGenericDeclaration(ty::TyConstGenericDecl),
    TypedTraitTypeDeclaration(ty::TyTraitType),
    TypedFunctionDeclaration(ty::TyFunctionDecl),
    TypedFunctionParameter(ty::TyFunctionParameter),
    TypedStructField(ty::TyStructField),
    TypedEnumVariant(ty::TyEnumVariant),
    TypedTraitFn(ty::TyTraitFn),
    TypedSupertrait(Supertrait),
    TypedStorageField(ty::TyStorageField),
    TypedStorageAccess(ty::TyStorageAccess),
    TypedStorageAccessDescriptor(ty::TyStorageAccessDescriptor),
    TypedReassignment(ty::TyReassignment),
    TypedArgument(GenericTypeArgument),
    TypedParameter(TypeParameter),
    TypedTraitConstraint(TraitConstraint),
    TypedModuleName,
    TypedModStatement(ty::TyModStatement),
    TypedUseStatement(ty::TyUseStatement),
    TypedStatement(ty::TyStatement),
    Ident(Ident),
}

/// These variants are used to represent the semantic type of the [Token].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymbolKind {
    /// Emitted for the boolean literals `true` and `false`.
    BoolLiteral,
    /// Emitted for builtin types like `u32`, and `str`.
    BuiltinType,
    /// Emitted for byte literals.
    ByteLiteral,
    /// Emitted for constants.
    Const,
    /// Emitted for derive helper attributes.
    DeriveHelper,
    /// Emitted for enums.
    Enum,
    /// Emitted for struct fields.
    Field,
    /// Emitted for free-standing & associated functions.
    Function,
    /// Emitted for compiler intrinsics.
    Intrinsic,
    /// Emitted for keywords.
    Keyword,
    /// Emitted for modules.
    Module,
    /// Emitted for numeric literals.
    NumericLiteral,
    /// Emitted for keywords.
    ProgramTypeKeyword,
    /// Emitted for the self function parameter and self path-specifier.
    SelfKeyword,
    /// Emitted for the Self type parameter.
    SelfTypeKeyword,
    /// Emitted for string literals.
    StringLiteral,
    /// Emitted for structs.
    Struct,
    /// Emitted for traits.
    Trait,
    /// Emitted for associated types.
    TraitType,
    /// Emitted for type aliases.
    TypeAlias,
    /// Emitted for type parameters.
    TypeParameter,
    /// Emitted for generic tokens that have no mapping.
    Unknown,
    /// Emitted for non-self function parameters.
    ValueParam,
    /// Emitted for enum variants.
    Variant,
    /// Emitted for locals.
    Variable,
}

#[derive(Debug, Clone)]
pub enum TypeDefinition {
    TypeId(TypeId),
    Ident(Ident),
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum TokenAstNode {
    Parsed(ParsedAstToken),
    Typed(TypedAstToken),
}

/// The `Token` type is created during traversal of the parsed and typed AST's of a program.
/// It holds the parsed and typed data structures produced by the sway compiler.
/// It also holds the type definition & semantic type of the token if they could be inferred
/// during traversal of the AST's.
#[derive(Debug, Clone)]
pub struct Token {
    pub ast_node: TokenAstNode,
    pub type_def: Option<TypeDefinition>,
    pub kind: SymbolKind,
}

impl Token {
    /// Create a new token with the given [SymbolKind].
    /// This function is intended to be used during traversal of the
    /// [sway_core::language::parsed::ParseProgram] AST.
    pub fn from_parsed(token: ParsedAstToken, kind: SymbolKind) -> Self {
        Self {
            ast_node: TokenAstNode::Parsed(token),
            type_def: None,
            kind,
        }
    }

    /// Get the `AstToken`, if this is a parsed token.
    pub fn as_parsed(&self) -> Option<&ParsedAstToken> {
        match &self.ast_node {
            TokenAstNode::Parsed(token) => Some(token),
            _ => None,
        }
    }

    /// Get the `TypedAstToken`, if this is a typed token.
    pub fn as_typed(&self) -> Option<&TypedAstToken> {
        match &self.ast_node {
            TokenAstNode::Typed(token) => Some(token),
            _ => None,
        }
    }

    /// Return the [TokenIdent] of the declaration of the provided token.
    pub fn declared_token_ident(&self, engines: &Engines) -> Option<TokenIdent> {
        self.type_def.as_ref().and_then(|type_def| match type_def {
            TypeDefinition::TypeId(type_id) => ident_of_type_id(engines, type_id),
            TypeDefinition::Ident(ident) => Some(TokenIdent::new(ident, engines.se())),
        })
    }
}

/// A more convenient [Ident] type for use in the language server.
///
/// This type is used as the key in the [TokenMap]. It's constructed during AST traversal
/// where we compute the [Range] of the token and the convert [SourceId]'s to [PathBuf]'s.
/// Although this introduces a small amount of overhead while traversing, precomputing this
/// greatly speeds up performance in all other areas of the language server.
///
/// [TokenMap]: crate::core::token_map::TokenMap
/// [SourceId]: sway_types::SourceId
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct TokenIdent {
    pub name: String,
    pub range: Range,
    pub path: Option<PathBuf>,
    pub source_id: Option<SourceId>,
    pub is_raw_ident: bool,
}

impl TokenIdent {
    pub fn new(ident: &Ident, se: &SourceEngine) -> Self {
        let source_id = ident.span().source_id().copied();
        let path = source_id.as_ref().map(|source_id| se.get_path(source_id));
        Self {
            name: ident.span().str(),
            range: get_range_from_span(&ident.span()),
            path,
            source_id,
            is_raw_ident: ident.is_raw_ident(),
        }
    }

    pub fn is_raw_ident(&self) -> bool {
        self.is_raw_ident
    }

    pub fn program_id(&self) -> Option<ProgramId> {
        self.source_id.map(|source_id| source_id.program_id())
    }
}

impl std::hash::Hash for TokenIdent {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.range.start.line.hash(state);
        self.range.start.character.hash(state);
        self.range.end.line.hash(state);
        self.range.end.character.hash(state);
        self.path.hash(state);
        self.is_raw_ident.hash(state);
    }
}

/// Check if the given method is a [`core::ops`] application desugared from short-hand syntax like / + * - etc.
pub fn desugared_op(prefixes: &[Ident]) -> bool {
    let prefix0 = prefixes.first().map(sway_types::BaseIdent::as_str);
    let prefix1 = prefixes.get(1).map(sway_types::BaseIdent::as_str);
    if let (Some("core"), Some("ops")) = (prefix0, prefix1) {
        return true;
    }
    false
}

/// Use the [TypeId] to look up the associated [TypeInfo] and return the [TokenIdent] if one is found.
pub fn ident_of_type_id(engines: &Engines, type_id: &TypeId) -> Option<TokenIdent> {
    let ident = match &*engines.te().get(*type_id) {
        TypeInfo::UnknownGeneric { name, .. } | TypeInfo::Alias { name, .. } => name.clone(),
        TypeInfo::Enum(decl_ref) => engines.de().get_enum(decl_ref).call_path.suffix.clone(),
        TypeInfo::Struct(decl_ref) => engines.de().get_struct(decl_ref).call_path.suffix.clone(),
        TypeInfo::Custom {
            qualified_call_path,
            ..
        } => qualified_call_path.call_path.suffix.clone(),
        _ => return None,
    };
    Some(TokenIdent::new(&ident, engines.se()))
}

/// Intended to be used during traversal of the [sway_core::language::parsed::ParseProgram] AST.
/// We can then use the [TypeInfo] to infer the semantic type of the token before type-checking.
pub fn type_info_to_symbol_kind(
    type_engine: &TypeEngine,
    type_info: &TypeInfo,
    type_span: Option<&Span>,
) -> SymbolKind {
    // This is necessary because the type engine resolves `Self` & `self` to the type it refers to.
    // We want to keep the semantics of these keywords.
    if let Some(type_span) = type_span {
        if type_span.as_str() == "Self" {
            return SymbolKind::SelfTypeKeyword;
        } else if type_span.as_str() == "self" {
            return SymbolKind::SelfKeyword;
        }
    }

    match type_info {
        TypeInfo::UnsignedInteger(..) | TypeInfo::Boolean | TypeInfo::B256 => {
            SymbolKind::BuiltinType
        }
        TypeInfo::Numeric | TypeInfo::StringArray(..) => SymbolKind::NumericLiteral,
        TypeInfo::Custom { .. } | TypeInfo::Struct { .. } | TypeInfo::Contract => {
            SymbolKind::Struct
        }
        TypeInfo::Enum { .. } => SymbolKind::Enum,
        TypeInfo::Array(elem_ty, ..) => {
            let type_info = type_engine.get(elem_ty.type_id);
            type_info_to_symbol_kind(type_engine, &type_info, Some(&elem_ty.span()))
        }
        TypeInfo::Slice(elem_ty) => {
            let type_info = type_engine.get(elem_ty.type_id);
            type_info_to_symbol_kind(type_engine, &type_info, Some(&elem_ty.span()))
        }
        _ => SymbolKind::Unknown,
    }
}

/// Given a [Span], convert into a [Range] and return.
pub fn get_range_from_span(span: &Span) -> Range {
    let start = span.start_line_col_one_index();
    let end = span.end_line_col_one_index();
    Range {
        start: Position::new(start.line as u32 - 1, start.col as u32 - 1),
        end: Position::new(end.line as u32 - 1, end.col as u32 - 1),
    }
}
