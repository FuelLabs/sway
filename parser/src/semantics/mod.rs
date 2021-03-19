use crate::error::*;
use crate::parse_tree::*;
use crate::types::{IntegerBits, TypeInfo};
use crate::{AstNode, AstNodeContent, CodeBlock, ParseTree, ReturnStatement, TraitFn};
use either::Either;
use pest::Span;
use std::collections::HashMap;

mod ast_node;
mod namespace;
mod syntax_tree;
pub(crate) use ast_node::{
    TypedAstNode, TypedAstNodeContent, TypedExpression, TypedVariableDeclaration,
};
pub use ast_node::{TypedDeclaration, TypedFunctionDeclaration};
pub(crate) use namespace::Namespace;
pub(crate) use syntax_tree::{TreeType, TypedParseTree};

const ERROR_RECOVERY_DECLARATION: TypedDeclaration = TypedDeclaration::ErrorRecovery;
