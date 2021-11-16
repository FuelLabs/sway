
#[derive(Debug)]
pub struct ParseTree {
    /// In a typical programming language, you might have a single root node for your syntax tree.
    /// In this language however, we want to expose multiple public functions at the root
    /// level so the tree is multi-root.
    pub root_nodes: Vec<AstNode>,
}

#[derive(Debug, Clone)]
pub struct AstNode {
    pub content: AstNodeContent,
    pub span: crate::Span,
}

#[derive(Debug, Clone)]
pub enum AstNodeContent {
    /*
    UseStatement(UseStatement),
    ReturnStatement(ReturnStatement),
    Declaration(Declaration),
    Expression(Expression),
    ImplicitReturnExpression(Expression),
    WhileLoop(WhileLoop),
    IncludeStatement(IncludeStatement),
    */
}

