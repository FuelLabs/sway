use sway_core::{
    language::{
        parsed::{
            Declaration, EnumVariant, Expression, FunctionDeclaration, FunctionParameter,
            ReassignmentExpression, Scrutinee, StorageField, StructExpressionField, StructField,
            TraitFn,
        },
        ty,
    },
    type_system::{TypeId, TypeInfo},
    TypeEngine,
};
use sway_types::{Ident, Span, Spanned};
use tower_lsp::lsp_types::{Position, Range};

#[derive(Debug, Clone)]
pub enum AstToken {
    Declaration(Declaration),
    Expression(Expression),
    StructExpressionField(StructExpressionField),
    FunctionDeclaration(FunctionDeclaration),
    FunctionParameter(FunctionParameter),
    StructField(StructField),
    EnumVariant(EnumVariant),
    TraitFn(TraitFn),
    Reassignment(ReassignmentExpression),
    StorageField(StorageField),
    Scrutinee(Scrutinee),
}

#[derive(Debug, Clone)]
pub enum TypedAstToken {
    TypedDeclaration(ty::TyDeclaration),
    TypedExpression(ty::TyExpression),
    TypedFunctionDeclaration(ty::TyFunctionDeclaration),
    TypedFunctionParameter(ty::TyFunctionParameter),
    TypedStructField(ty::TyStructField),
    TypedEnumVariant(ty::TyEnumVariant),
    TypedTraitFn(ty::TyTraitFn),
    TypedStorageField(ty::TyStorageField),
    TypeCheckedStorageReassignDescriptor(ty::TyStorageReassignDescriptor),
    TypedReassignment(ty::TyReassignment),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymbolKind {
    Field,
    ValueParam,
    Function,
    Const,
    Struct,
    Trait,
    Enum,
    Variant,
    BoolLiteral,
    ByteLiteral,
    StringLiteral,
    NumericLiteral,
    Variable,
    BuiltinType,
    Module,
    TypeParameter,
    Unknown,
}

#[derive(Debug, Clone)]
pub enum TypeDefinition {
    TypeId(TypeId),
    Ident(Ident),
}

#[derive(Debug, Clone)]
pub struct Token {
    pub parsed: AstToken,
    pub typed: Option<TypedAstToken>,
    pub type_def: Option<TypeDefinition>,
    pub kind: SymbolKind,
}

impl Token {
    /// Create a new token with the given SymbolKind.
    /// This function is intended to be used during traversal of the
    /// `ParseProgram` AST.
    pub fn from_parsed(token: AstToken, kind: SymbolKind) -> Self {
        Self {
            parsed: token,
            typed: None,
            type_def: None,
            kind,
        }
    }

    /// Return the `Ident` of the declaration of the provided token.
    pub fn declared_token_ident(&self, type_engine: &TypeEngine) -> Option<Ident> {
        self.type_def.as_ref().and_then(|type_def| match type_def {
            TypeDefinition::TypeId(type_id) => ident_of_type_id(type_engine, type_id),
            TypeDefinition::Ident(ident) => Some(ident.clone()),
        })
    }

    /// Return the `Span` of the declaration of the provided token. This is useful for
    /// performaing == comparisons on spans. We need to do this instead of comparing
    /// the `Ident` because the `Ident` eq is only comparing the str name.
    pub fn declared_token_span(&self, type_engine: &TypeEngine) -> Option<Span> {
        self.type_def.as_ref().and_then(|type_def| match type_def {
            TypeDefinition::TypeId(type_id) => Some(ident_of_type_id(type_engine, type_id)?.span()),
            TypeDefinition::Ident(ident) => Some(ident.span()),
        })
    }
}

// Check if the given method is a `core::ops` application desugared from short-hand syntax like / + * - etc.
pub(crate) fn desugared_op(prefixes: &[Ident]) -> bool {
    let prefix0 = prefixes.get(0).map(|ident| ident.as_str());
    let prefix1 = prefixes.get(1).map(|ident| ident.as_str());
    if let (Some("core"), Some("ops")) = (prefix0, prefix1) {
        return true;
    }

    false
}

// We need to do this work around as the custom PartialEq for Ident impl
// only checks for the string, not the span.
pub(crate) fn to_ident_key(ident: &Ident) -> (Ident, Span) {
    (ident.clone(), ident.span())
}

/// Use the TypeId to look up the associated TypeInfo and return the Ident if one is found.
pub(crate) fn ident_of_type_id(type_engine: &TypeEngine, type_id: &TypeId) -> Option<Ident> {
    match type_engine.look_up_type_id(*type_id) {
        TypeInfo::UnknownGeneric { name, .. }
        | TypeInfo::Enum { name, .. }
        | TypeInfo::Struct { name, .. }
        | TypeInfo::Custom { name, .. } => Some(name),
        _ => None,
    }
}

pub(crate) fn type_info_to_symbol_kind(
    type_engine: &TypeEngine,
    type_info: &TypeInfo,
) -> SymbolKind {
    match type_info {
        TypeInfo::UnsignedInteger(..) | TypeInfo::Boolean | TypeInfo::Str(..) | TypeInfo::B256 => {
            SymbolKind::BuiltinType
        }
        TypeInfo::Numeric => SymbolKind::NumericLiteral,
        TypeInfo::Custom { .. } | TypeInfo::Struct { .. } => SymbolKind::Struct,
        TypeInfo::Enum { .. } => SymbolKind::Enum,
        TypeInfo::Array(elem_ty, ..) => {
            let type_info = type_engine.look_up_type_id(elem_ty.type_id);
            type_info_to_symbol_kind(type_engine, &type_info)
        }
        _ => SymbolKind::Unknown,
    }
}

/// Given a `Span`, convert into an `lsp_types::Range` and return.
pub(crate) fn get_range_from_span(span: &Span) -> Range {
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
