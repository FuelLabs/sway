use std::collections::HashMap;
use sway_core::semantic_analysis::ast_node::{
    expression::typed_expression::TypedExpression, TypeCheckedStorageReassignDescriptor,
    TypedDeclaration, TypedEnumVariant, TypedFunctionDeclaration, TypedFunctionParameter,
    TypedReassignment, TypedStorageField, TypedStructField, TypedTraitFn,
};
use sway_types::{Ident, Span};

pub type TokenMap = HashMap<(Ident, Span), TokenType>;

#[derive(Debug, Clone)]
pub enum TokenType {
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
