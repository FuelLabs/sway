use dashmap::DashMap;
use sway_core::{
    language::parsed::{
        Declaration, EnumVariant, Expression, FunctionDeclaration, FunctionParameter,
        ReassignmentExpression, Scrutinee, StorageField, StructExpressionField, StructField,
        TraitFn,
    },
    semantic_analysis::ast_node::{
        expression::typed_expression::TyExpression, TyDeclaration, TyEnumVariant,
        TyFunctionDeclaration, TyFunctionParameter, TyReassignment, TyStorageField,
        TyStorageReassignDescriptor, TyStructField, TyTraitFn,
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
    pub parsed: AstToken,
    pub typed: Option<TypedAstToken>,
    pub type_def: Option<TypeDefinition>,
    pub kind: SymbolKind,
}

impl Token {
    pub fn from_parsed(token: AstToken, kind: SymbolKind) -> Self {
        Self {
            parsed: token,
            typed: None,
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
    TypedDeclaration(TyDeclaration),
    TypedExpression(TyExpression),
    TypedFunctionDeclaration(TyFunctionDeclaration),
    TypedFunctionParameter(TyFunctionParameter),
    TypedStructField(TyStructField),
    TypedEnumVariant(TyEnumVariant),
    TypedTraitFn(TyTraitFn),
    TypedStorageField(TyStorageField),
    TypeCheckedStorageReassignDescriptor(TyStorageReassignDescriptor),
    TypedReassignment(TyReassignment),
}

#[derive(Debug, Clone)]
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
