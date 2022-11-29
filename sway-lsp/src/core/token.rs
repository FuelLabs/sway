use crate::utils;
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
    TypeEngine,
};
use sway_types::{Ident, Span, Spanned};

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
            TypeDefinition::TypeId(type_id) => utils::token::ident_of_type_id(type_engine, type_id),
            TypeDefinition::Ident(ident) => Some(ident.clone()),
        })
    }

    /// Return the `Span` of the declaration of the provided token. This is useful for
    /// performaing == comparisons on spans. We need to do this instead of comparing
    /// the `Ident` because the `Ident` eq is only comparing the str name.
    pub fn declared_token_span(&self, type_engine: &TypeEngine) -> Option<Span> {
        self.type_def.as_ref().and_then(|type_def| match type_def {
            TypeDefinition::TypeId(type_id) => {
                Some(utils::token::ident_of_type_id(type_engine, type_id)?.span())
            }
            TypeDefinition::Ident(ident) => Some(ident.span()),
        })
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
