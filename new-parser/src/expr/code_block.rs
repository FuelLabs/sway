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

pub fn code_block() -> impl Parser<Output = CodeBlock> + Clone {
    braces(padded(code_block_contents()))
    .map(|braces: Braces<WithSpan<_>>| {
        let contents = braces.map(|contents_with_span| contents_with_span.parsed);
        CodeBlock { contents }
    })
}

pub fn code_block_contents() -> impl Parser<Output = WithSpan<CodeBlockContents>> + Clone {
    statement()
    .then_optional_whitespace()
    .repeated()
    .then(lazy(|| expr()).optional())
    .map(|(statements_with_span, final_expr_res): (WithSpan<Vec<Statement>>, Result<Expr, Span>)| {
        let span = Span::join(statements_with_span.span(), final_expr_res.span());
        let parsed = CodeBlockContents {
            statements: statements_with_span.parsed,
            final_expr_opt: final_expr_res.ok().map(Box::new),
        };
        WithSpan { parsed, span }
    })
}

