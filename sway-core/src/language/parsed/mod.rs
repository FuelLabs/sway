//! Contains all the code related to parsing Sway source code.
mod code_block;
pub mod declaration;
mod expression;
mod include_statement;
mod module;
mod program;
mod use_statement;

pub use code_block::*;
pub use declaration::*;
pub use expression::*;
pub use include_statement::IncludeStatement;
pub use module::{ModuleEvaluationOrder, ParseModule, ParseSubmodule};
pub use program::{ParseProgram, TreeType};
use sway_error::handler::ErrorEmitted;
use sway_types::span::Span;
pub use use_statement::{ImportType, UseStatement};

use crate::Engines;

/// Represents some exportable information that results from compiling some
/// Sway source code.
#[derive(Debug, Clone)]
pub struct ParseTree {
    /// The untyped AST nodes that constitute this tree's root nodes.
    pub root_nodes: Vec<AstNode>,
    /// The [Span] of the entire tree.
    pub span: Span,
}

/// A single [AstNode] represents a node in the parse tree. Note that [AstNode]
/// is a recursive type and can contain other [AstNode], thus populating the tree.
#[derive(Debug, Clone, deepsize::DeepSizeOf)]
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
    /// Any type of declaration, of which there are quite a few. See [Declaration] for more details
    /// on the possible variants.
    Declaration(Declaration),
    /// Any type of expression, of which there are quite a few. See [Expression] for more details.
    Expression(Expression),
    /// A statement of the form `mod foo::bar;` which imports/includes another source file.
    IncludeStatement(IncludeStatement),
    /// A malformed statement.
    ///
    /// Used for parser recovery when we cannot form a more specific node.
    /// The list of `Span`s are for consumption by the LSP and are,
    /// when joined, the same as that stored in `statement.span`.
    Error(Box<[Span]>, ErrorEmitted),
}

impl deepsize::DeepSizeOf for AstNodeContent {
    fn deep_size_of_children(&self, _context: &mut deepsize::Context) -> usize {
        0
    }
}

impl ParseTree {
    /// Excludes all test functions from the parse tree.
    pub(crate) fn exclude_tests(&mut self, engines: &Engines) {
        self.root_nodes.retain(|node| !node.is_test(engines));
    }
}

impl AstNode {
    /// Checks if this `AstNode` is a test.
    pub(crate) fn is_test(&self, engines: &Engines) -> bool {
        if let AstNodeContent::Declaration(decl) = &self.content {
            decl.is_test(engines)
        } else {
            false
        }
    }
}
