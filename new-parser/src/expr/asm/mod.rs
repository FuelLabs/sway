use crate::priv_prelude::*;

mod opcode;
mod instruction;

pub use opcode::*;
pub use instruction::*;

#[derive(Clone, Debug)]
pub struct AsmBlock {
    pub asm_token: AsmToken,
    pub registers: Parens<Punctuated<AsmRegisterDeclaration, CommaToken>>,
    pub contents: Braces<AsmBlockContents>,
}

impl Spanned for AsmBlock {
    fn span(&self) -> Span {
        Span::join(self.asm_token.span(), self.contents.span())
    }
}

#[derive(Clone, Debug)]
pub struct AsmRegisterDeclaration {
    pub register: Ident,
    pub value_opt: Option<(ColonToken, Box<Expr>)>,
}

impl Spanned for AsmRegisterDeclaration {
    fn span(&self) -> Span {
        match &self.value_opt {
            Some((_, expr)) => Span::join(self.register.span(), expr.span()),
            None => self.register.span(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct AsmBlockContents {
    pub instructions: Vec<AsmInstruction>,
    pub final_expr_opt: Option<AsmExpr>,
}

#[derive(Clone, Debug)]
pub struct AsmExpr {
    pub register: Ident,
    pub ty_opt: Option<(ColonToken, Ty)>,
}

#[derive(Clone, Debug)]
pub struct AsmImmediate {
    pub span: Span,
    pub parsed: BigUint,
}

impl Spanned for AsmImmediate {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

pub fn asm_block() -> impl Parser<Output = AsmBlock> + Clone {
    asm_token()
    .then_optional_whitespace()
    .then(parens(padded(punctuated(asm_register_declaration(), padded(comma_token())))))
    .then_optional_whitespace()
    .then(braces(padded(asm_block_contents())))
    .map(|((asm_token, registers), contents)| {
        AsmBlock { asm_token, registers, contents }
    })
}

pub fn asm_register_declaration() -> impl Parser<Output = AsmRegisterDeclaration> + Clone {
    ident()
    .then_optional_whitespace()
    .then(
        colon_token()
        .then_optional_whitespace()
        .then(lazy(|| expr()).map(Box::new))
        .optional()
    )
    .map(|(register, value_opt)| {
        AsmRegisterDeclaration { register, value_opt }
    })
}

pub fn asm_block_contents() -> impl Parser<Output = AsmBlockContents> + Clone {
    asm_instruction()
    .then_optional_whitespace()
    .repeated()
    .then(asm_expr().optional())
    .map(|(instructions, final_expr_opt)| {
        AsmBlockContents { instructions, final_expr_opt }
    })
}

pub fn asm_expr() -> impl Parser<Output = AsmExpr> + Clone {
    ident()
    .then_optional_whitespace()
    .then(
        colon_token()
        .then_optional_whitespace()
        .then(ty())
        .optional()
    )
    .map(|(register, ty_opt)| {
        AsmExpr { register, ty_opt }
    })
}

pub fn asm_immediate() -> impl Parser<Output = AsmImmediate> + Clone {
    keyword("i")
    .then(digit(10))
    .then(digit(10).repeated())
    .map_with_span(|(((), digit), digits), span| {
        let mut parsed = BigUint::from(digit);
        for digit in digits {
            parsed *= 10u32;
            parsed += digit;
        }
        AsmImmediate { span, parsed }
    })
}

