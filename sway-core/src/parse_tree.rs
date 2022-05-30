//! Contains all the code related to parsing Sway source code.
mod call_path;
mod code_block;
pub mod declaration;
mod expression;
pub mod ident;
mod include_statement;
pub mod literal;
mod return_statement;
mod use_statement;
mod visibility;
mod while_loop;

pub use call_path::*;
pub use code_block::*;
pub use declaration::*;
pub use expression::*;
pub(crate) use include_statement::IncludeStatement;
pub use literal::Literal;
pub use return_statement::*;
use sway_types::{ident::Ident, span::Span};
pub use use_statement::{ImportType, UseStatement};
pub use visibility::Visibility;
pub use while_loop::WhileLoop;

/// A parsed, but not yet type-checked, Sway program.
///
/// Includes all modules in the form of a `ParseModule` tree accessed via the `root`.
#[derive(Debug)]
pub struct ParseProgram {
    pub kind: TreeType,
    pub root: ParseModule,
}

/// A module and its submodules in the form of a tree.
#[derive(Debug)]
pub struct ParseModule {
    /// The content of this module in the form of a `ParseTree`.
    pub tree: ParseTree,
    /// Submodules introduced within this module using the `dep` syntax in order of declaration.
    pub submodules: Vec<(DepName, ParseSubmodule)>,
}

/// The name used within a module to refer to one of its submodules.
///
/// If an alias was given to the `dep`, this will be the alias. If not, this is the submodule's
/// library name.
pub type DepName = Ident;

/// A library module that was declared as a `dep` of another module.
///
/// Only submodules are guaranteed to be a `library` and have a `library_name`.
#[derive(Debug)]
pub struct ParseSubmodule {
    /// The name of a submodule, parsed from the `library` declaration within the module itself.
    pub library_name: Ident,
    pub module: ParseModule,
}

/// A Sway program can be either a contract, script, predicate, or a library.
///
/// All submodules declared with `dep` should be `Library`s.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TreeType {
    Predicate,
    Script,
    Contract,
    Library { name: Ident },
}

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
    /// A control flow element which loops continually until some boolean expression evaluates as
    /// `false`.
    WhileLoop(WhileLoop),
    /// A statement of the form `dep foo::bar;` which imports/includes another source file.
    IncludeStatement(IncludeStatement),
}
