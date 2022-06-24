use std::collections::HashMap;
use sway_core::semantic_analysis::ast_node::{
    expression::typed_expression::TypedExpression, TypeCheckedStorageReassignDescriptor,
    TypedDeclaration, TypedEnumVariant, TypedFunctionDeclaration, TypedFunctionParameter,
    TypedReassignment, TypedStorageField, TypedStructField, TypedTraitFn,
};
use sway_types::{Ident, Span};

use sway_core::{
    Declaration, EnumVariant, Expression, FunctionDeclaration, FunctionParameter, Reassignment,
    StorageField, StructField, TraitFn,
};

pub type TokenMap = HashMap<(Ident, Span), TokenType>;

//#[derive(Debug, Clone)]
// pub enum TokenType {
//     Token(AstToken),
//     TypedToken(TypedAstToken),
// }

#[derive(Debug, Clone)]
pub struct TokenType {
    pub parsed: AstToken,
    pub typed: Option<TypedAstToken>,
}

impl TokenType {
    pub fn from_parsed(token: AstToken) -> Self {
        Self {
            parsed: token,
            typed: None,
        }
    }

    // an iterator that just returns tokens
    // that have typed(Some)
    //pub fn get_typed_tokens() ->
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