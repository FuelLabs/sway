use std::collections::HashMap;
use sway_types::{Ident, Span};
use sway_core::semantic_analysis::ast_node::{TypedDeclaration, TypedFunctionDeclaration, TypedFunctionParameter, TypedStructField, 
    TypedEnumVariant, TypedTraitFn, TypedStorageField, TypeCheckedStorageReassignDescriptor, ReassignmentLhs,
    expression::{
        typed_expression::TypedExpression,
    },
};

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
    ReassignmentLhs(ReassignmentLhs),
}