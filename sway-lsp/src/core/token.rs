use dashmap::DashMap;
use sway_core::{
    language::{
        parsed::{
            Declaration, EnumVariant, Expression, FunctionDeclaration, FunctionParameter,
            ReassignmentExpression, Scrutinee, StorageField, StructExpressionField, StructField,
            TraitFn,
        },
        ty,
    },
    type_system::TypeId,
};
use sway_types::{Ident, Span};

pub type TokenMap = DashMap<(Ident, Span), Token>;

#[derive(Debug, Clone)]
pub enum TypeDefinition {
    TypeId(TypeId),
    Ident(Ident),
}

#[derive(Debug, Clone)]
pub struct Token {
    pub parsed: Option<AstToken>,
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
            parsed: Some(token),
            typed: None,
            type_def: None,
            kind,
        }
    }

    /// Create a new token with the given SymbolKind.
    /// This function is only intended to be used when collecting
    /// tokens from dependencies.
    pub fn from_typed(token: TypedAstToken, kind: SymbolKind) -> Self {
        Self {
            parsed: None,
            typed: Some(token),
            type_def: None,
            kind,
        }
    }
}

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
