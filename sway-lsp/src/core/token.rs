use dashmap::DashMap;
use sway_core::{
    semantic_analysis::ast_node::{
        expression::typed_expression::TypedExpression, TypeCheckedStorageReassignDescriptor,
        TypedDeclaration, TypedEnumVariant, TypedFunctionDeclaration, TypedFunctionParameter,
        TypedReassignment, TypedStorageField, TypedStructField, TypedTraitFn,
    },
    type_system::TypeId,
    Declaration, EnumVariant, Expression, FunctionDeclaration, FunctionParameter, Reassignment,
    StorageField, StructField, TraitFn,
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
}

impl Token {
    pub fn from_parsed(token: AstToken) -> Self {
        Self {
            parsed: token,
            typed: None,
            type_def: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum AstToken {
    Declaration(Declaration),
    Expression(Expression),
    FunctionDeclaration(FunctionDeclaration),
    FunctionParameter(FunctionParameter),
    StructField(StructField),
    EnumVariant(EnumVariant),
    TraitFn(TraitFn),
    Reassignment(Reassignment),
    StorageField(StorageField),
}

#[derive(Debug, Clone)]
pub enum TypedAstToken {
    TypedDeclaration(TypedDeclaration),
    TypedExpression(TypedExpression),
    TypedFunctionDeclaration(TypedFunctionDeclaration),
    TypedFunctionParameter(TypedFunctionParameter),
    TypedStructField(TypedStructField),
    TypedEnumVariant(TypedEnumVariant),
    TypedTraitFn(TypedTraitFn),
    TypedStorageField(TypedStorageField),
    TypeCheckedStorageReassignDescriptor(TypeCheckedStorageReassignDescriptor),
    TypedReassignment(TypedReassignment),
}
