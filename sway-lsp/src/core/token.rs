use lsp_types::{Position, Range};
use sway_ast::Intrinsic;
use sway_core::{
    language::{
        parsed::{
            AbiCastExpression, AmbiguousPathExpression, Declaration, DelineatedPathExpression,
            EnumVariant, Expression, FunctionApplicationExpression, FunctionParameter,
            MethodApplicationExpression, Scrutinee, StorageField, StructExpression,
            StructExpressionField, StructField, StructScrutineeField, Supertrait, TraitFn,
            UseStatement,
        },
        ty,
    },
    transform::Attribute,
    type_system::{TypeId, TypeInfo, TypeParameter},
    Engines, TraitConstraint, TypeArgument, TypeEngine,
};
use sway_types::{Ident, Span, Spanned};

/// The `AstToken` holds the types produced by the [sway_core::language::parsed::ParseProgram].
/// These tokens have not been type-checked.
/// See this issue https://github.com/FuelLabs/sway/issues/2257 for more information about why they are
/// useful to the language server.
#[derive(Debug, Clone)]
pub enum AstToken {
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
    IncludeStatement,
    Intrinsic(Intrinsic),
    Keyword(Ident),
    LibrarySpan(Span),
    MethodApplicationExpression(MethodApplicationExpression),
    Scrutinee(Scrutinee),
    StorageField(StorageField),
    StructExpression(StructExpression),
    StructExpressionField(StructExpressionField),
    StructField(StructField),
    StructScrutineeField(StructScrutineeField),
    Supertrait(Supertrait),
    TraitConstraint(TraitConstraint),
    TraitFn(TraitFn),
    TypeArgument(TypeArgument),
    TypeParameter(TypeParameter),
    UseStatement(UseStatement),
}

/// The `TypedAstToken` holds the types produced by the [sway_core::language::ty::TyProgram].
#[derive(Debug, Clone)]
pub enum TypedAstToken {
    TypedTypeAliasDeclaration(ty::TyTypeAliasDecl),
    TypedDeclaration(ty::TyDecl),
    TypedExpression(ty::TyExpression),
    TypedScrutinee(ty::TyScrutinee),
    TyStructScrutineeField(ty::TyStructScrutineeField),
    TypedConstantDeclaration(ty::TyConstantDecl),
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
    TypedArgument(TypeArgument),
    TypedParameter(TypeParameter),
    TypedTraitConstraint(TraitConstraint),
    TypedIncludeStatement,
    TypedUseStatement(ty::TyUseStatement),
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

/// The `Token` type is created during traversal of the parsed and typed AST's of a program.
/// It holds the parsed and typed data structures produced by the sway compiler.
/// It also holds the type definition & semantic type of the token if they could be inferred
/// during traversal of the AST's.
#[derive(Debug, Clone)]
pub struct Token {
    pub parsed: AstToken,
    pub typed: Option<TypedAstToken>,
    pub type_def: Option<TypeDefinition>,
    pub kind: SymbolKind,
}

impl Token {
    /// Create a new token with the given [SymbolKind].
    /// This function is intended to be used during traversal of the
    /// [sway_core::language::parsed::ParseProgram] AST.
    pub fn from_parsed(token: AstToken, kind: SymbolKind) -> Self {
        Self {
            parsed: token,
            typed: None,
            type_def: None,
            kind,
        }
    }

    /// Return the [Ident] of the declaration of the provided token.
    pub fn declared_token_ident(&self, engines: &Engines) -> Option<Ident> {
        self.type_def.as_ref().and_then(|type_def| match type_def {
            TypeDefinition::TypeId(type_id) => ident_of_type_id(engines, type_id),
            TypeDefinition::Ident(ident) => Some(ident.clone()),
        })
    }

    /// Return the [Span] of the declaration of the provided token. This is useful for
    /// performaing == comparisons on spans. We need to do this instead of comparing
    /// the [Ident] because the [PartialEq] implementation is only comparing the name.
    pub fn declared_token_span(&self, engines: &Engines) -> Option<Span> {
        self.type_def.as_ref().and_then(|type_def| match type_def {
            TypeDefinition::TypeId(type_id) => Some(ident_of_type_id(engines, type_id)?.span()),
            TypeDefinition::Ident(ident) => Some(ident.span()),
        })
    }
}

/// Check if the given method is a [core::ops] application desugared from short-hand syntax like / + * - etc.
pub fn desugared_op(prefixes: &[Ident]) -> bool {
    let prefix0 = prefixes.get(0).map(|ident| ident.as_str());
    let prefix1 = prefixes.get(1).map(|ident| ident.as_str());
    if let (Some("core"), Some("ops")) = (prefix0, prefix1) {
        return true;
    }
    false
}

/// We need to do this work around as the custom [PartialEq] for [Ident] impl
/// only checks for the string, not the [Span].
pub fn to_ident_key(ident: &Ident) -> (Ident, Span) {
    (ident.clone(), ident.span())
}

/// Use the [TypeId] to look up the associated [TypeInfo] and return the [Ident] if one is found.
pub fn ident_of_type_id(engines: &Engines, type_id: &TypeId) -> Option<Ident> {
    match engines.te().get(*type_id) {
        TypeInfo::UnknownGeneric { name, .. } => Some(name),
        TypeInfo::Enum(decl_ref) => Some(engines.de().get_enum(&decl_ref).call_path.suffix),
        TypeInfo::Struct(decl_ref) => Some(engines.de().get_struct(&decl_ref).call_path.suffix),
        TypeInfo::Custom { call_path, .. } => Some(call_path.suffix),
        _ => None,
    }
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
        TypeInfo::Numeric | TypeInfo::Str(..) => SymbolKind::NumericLiteral,
        TypeInfo::Custom { .. } | TypeInfo::Struct { .. } | TypeInfo::Contract => {
            SymbolKind::Struct
        }
        TypeInfo::Enum { .. } => SymbolKind::Enum,
        TypeInfo::Array(elem_ty, ..) => {
            let type_info = type_engine.get(elem_ty.type_id);
            type_info_to_symbol_kind(type_engine, &type_info, Some(&elem_ty.span()))
        }
        TypeInfo::SelfType => SymbolKind::SelfTypeKeyword,
        _ => SymbolKind::Unknown,
    }
}

/// Given a [Span], convert into a [Range] and return.
pub fn get_range_from_span(span: &Span) -> Range {
    let start = span.start_pos().line_col();
    let end = span.end_pos().line_col();

    let start_line = start.0 as u32 - 1;
    let start_character = start.1 as u32 - 1;

    let end_line = end.0 as u32 - 1;
    let end_character = end.1 as u32 - 1;

    Range {
        start: Position::new(start_line, start_character),
        end: Position::new(end_line, end_character),
    }
}

pub fn get_range_from_span_opt(span: &Span) -> Range {
    let start = span.start_pos_opt().line_col();
    let end = span.end_pos_opt().line_col();

    // let start_line = start.0 as u32 - 1;
    // let start_character = start.1 as u32 - 1;

    // let end_line = end.0 as u32 - 1;
    // let end_character = end.1 as u32 - 1;

    Range {
        start: Position::new(start.0 as u32 - 1, start.1 as u32 - 1),
        end: Position::new(end.0 as u32 - 1, end.1 as u32 - 1),
    }
}
