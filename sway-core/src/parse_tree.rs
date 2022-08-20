//! Contains all the code related to parsing Sway source code.
mod call_path;
mod code_block;
pub mod declaration;
mod expression;
mod include_statement;
mod literal;
mod module;
mod program;
mod return_statement;
mod use_statement;
mod visibility;

pub use call_path::*;
pub use code_block::*;
pub use declaration::*;
pub use expression::*;
pub(crate) use include_statement::IncludeStatement;
pub use literal::Literal;
pub use module::{DepName, ParseModule, ParseSubmodule};
pub use program::{ParseProgram, TreeType};
pub use return_statement::*;
use sway_types::span::Span;
pub use use_statement::{ImportType, UseStatement};
pub use visibility::Visibility;

/// Represents some exportable information that results from compiling some
/// Sway source code.
#[derive(Debug)]
pub struct ParseTree {
    /// The untyped AST nodes that constitute this tree's root nodes.
    pub root_nodes: Vec<AstNode>,
    /// The [Span] of the entire tree.
    pub span: Span,
}

/// A single [AstNode] represents a node in the parse tree. Note that [AstNode]
/// is a recursive type and can contain other [AstNode], thus populating the tree.
#[derive(Debug, Clone)]
pub struct AstNode {
    /// The content of this ast node, which could be any control flow structure or other
    /// basic organizational component.
    pub content: AstNodeContent,
    /// The [Span] representing this entire [AstNode].
    pub span: Span,
}

/// Represents the various structures that constitute a Sway program.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum AstNodeContent {
    /// A statement of the form `use foo::bar;` or `use ::foo::bar;`
    UseStatement(UseStatement),
    /// A statement of the form `return foo;`
    ReturnStatement(ReturnStatement),
    /// Any type of declaration, of which there are quite a few. See [Declaration] for more details
    /// on the possible variants.
    Declaration(Declaration),
    /// Any type of expression, of which there are quite a few. See [Expression] for more details.
    Expression(Expression),
    /// An implicit return expression is different from a [AstNodeContent::ReturnStatement] because
    /// it is not a control flow item. Therefore it is a different variant.
    ///
    /// An implicit return expression is an [Expression] at the end of a code block which has no
    /// semicolon, denoting that it is the [Expression] to be returned from that block.
    ImplicitReturnExpression(Expression),
    /// A statement of the form `dep foo::bar;` which imports/includes another source file.
    IncludeStatement(IncludeStatement),
}
