use crate::priv_prelude::*;

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

pub fn code_block() -> impl Parser<Output = CodeBlock> + Clone {
    braces(padded(code_block_contents()))
    .map(|contents| {
        CodeBlock { contents }
    })
}

pub fn code_block_contents() -> impl Parser<Output = CodeBlockContents> + Clone {
    statement()
    .then_optional_whitespace()
    .repeated()
    .then(lazy(|| expr()).map(Box::new).optional())
    .map(|(statements, final_expr_opt)| {
        CodeBlockContents { statements, final_expr_opt }
    })
}

