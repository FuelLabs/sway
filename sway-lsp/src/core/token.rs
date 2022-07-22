use dashmap::DashMap;
use sway_core::semantic_analysis::ast_node::{
    expression::typed_expression::TypedExpression, TypeCheckedStorageReassignDescriptor,
    TypedDeclaration, TypedEnumVariant, TypedFunctionDeclaration, TypedFunctionParameter,
    TypedReassignment, TypedStorageField, TypedStructField, TypedTraitFn,
};
use sway_core::{
    semantic_analysis::ast_node::{
        expression::typed_expression::TypedExpression, TypeCheckedStorageReassignDescriptor,
        TypedDeclaration, TypedEnumVariant, TypedFunctionDeclaration, TypedFunctionParameter,
        TypedReassignment, TypedStorageField, TypedStructField, TypedTraitFn,
    },
    type_engine::TypeId,
    Declaration, EnumVariant, Expression, FunctionDeclaration, FunctionParameter, Reassignment,
    StorageField, StructField, TraitFn,
};
use sway_types::{Ident, Span};

pub type TokenMap = DashMap<(Ident, Span), TokenType>;

#[derive(Debug, Clone)]
pub struct TokenType {
    pub parsed: AstToken,
    pub typed: Option<TypedAstToken>,
    pub type_id: Option<TypeId>,
}

impl TokenType {
    pub fn from_parsed(token: AstToken) -> Self {
        Self {
            parsed: token,
            typed: None,
            type_id: None,
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
