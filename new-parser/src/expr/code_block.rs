pub use crate::priv_prelude::*;

pub struct CodeBlock {
    contents: Braces<CodeBlockContents>,
}

pub struct CodeBlockContents {
    pub statements: Vec<Statement>,
    pub final_expr_opt: Option<Box<Expr>>,
}

impl Spanned for CodeBlock {
    fn span(&self) -> Span {
        self.contents.span()
    }
}

pub fn code_block() -> impl Parser<char, CodeBlock, Error = Cheap<char, Span>> + Clone {
    braces(padded(code_block_contents()))
    .map(|contents| CodeBlock { contents })
}

pub fn code_block_contents() -> impl Parser<char, CodeBlockContents, Error = Cheap<char, Span>> + Clone {
    statement()
    .then_optional_whitespace()
    .repeated()
    .then(expr().or_not())
    .map(|(statements, final_expr_opt)| {
        CodeBlockContents {
            statements,
            final_expr_opt: final_expr_opt.map(Box::new),
        }
    })
}

