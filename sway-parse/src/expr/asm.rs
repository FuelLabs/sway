use crate::priv_prelude::*;

#[derive(Clone, Debug)]
pub struct AsmBlock {
    pub asm_token: AsmToken,
    pub registers: Parens<Punctuated<AsmRegisterDeclaration, CommaToken>>,
    pub contents: Braces<AsmBlockContents>,
}

#[derive(Clone, Debug)]
pub struct AsmRegisterDeclaration {
    pub register: Ident,
    pub value_opt: Option<(ColonToken, Box<Expr>)>,
}

#[derive(Clone, Debug)]
pub struct AsmBlockContents {
    pub instructions: Vec<(Instruction, SemicolonToken)>,
    pub final_expr_opt: Option<AsmFinalExpr>,
}

#[derive(Clone, Debug)]
pub struct AsmFinalExpr {
    pub register: Ident,
    pub ty_opt: Option<(ColonToken, Ty)>,
}

#[derive(Clone, Debug)]
pub struct AsmImmediate {
    pub span: Span,
    pub parsed: BigUint,
}

impl AsmImmediate {
    pub fn span(&self) -> Span {
        self.span.clone()
    }
}

impl AsmBlock {
    pub fn span(&self) -> Span {
        Span::join(self.asm_token.span(), self.contents.span())
    }
}

impl Parse for AsmBlock {
    fn parse(parser: &mut Parser) -> ParseResult<AsmBlock> {
        let asm_token = parser.parse()?;
        let registers = parser.parse()?;
        let contents = parser.parse()?;
        Ok(AsmBlock {
            asm_token,
            registers,
            contents,
        })
    }
}

impl Parse for AsmRegisterDeclaration {
    fn parse(parser: &mut Parser) -> ParseResult<AsmRegisterDeclaration> {
        let register = parser.parse()?;
        let value_opt = match parser.take() {
            Some(colon_token) => {
                let value = parser.parse()?;
                Some((colon_token, value))
            }
            None => None,
        };
        Ok(AsmRegisterDeclaration {
            register,
            value_opt,
        })
    }
}

impl ParseToEnd for AsmBlockContents {
    fn parse_to_end<'a, 'e>(
        mut parser: Parser<'a, 'e>,
    ) -> ParseResult<(AsmBlockContents, ParserConsumed<'a>)> {
        let mut instructions = Vec::new();
        let (final_expr_opt, consumed) = loop {
            if let Some(consumed) = parser.check_empty() {
                break (None, consumed);
            }
            let ident = parser.parse()?;
            if let Some(consumed) = parser.check_empty() {
                let final_expr = AsmFinalExpr {
                    register: ident,
                    ty_opt: None,
                };
                break (Some(final_expr), consumed);
            }
            if let Some(colon_token) = parser.take() {
                let ty = parser.parse()?;
                let consumed = match parser.check_empty() {
                    Some(consumed) => consumed,
                    None => {
                        return Err(
                            parser.emit_error(ParseErrorKind::UnexpectedTokenAfterAsmReturnType)
                        );
                    }
                };
                let final_expr = AsmFinalExpr {
                    register: ident,
                    ty_opt: Some((colon_token, ty)),
                };
                break (Some(final_expr), consumed);
            }
            let instruction = parse_instruction(ident, &mut parser)?;
            let semicolon_token = parser.parse()?;
            instructions.push((instruction, semicolon_token));
        };
        let contents = AsmBlockContents {
            instructions,
            final_expr_opt,
        };
        Ok((contents, consumed))
    }
}

impl Parse for AsmImmediate {
    fn parse(parser: &mut Parser) -> ParseResult<AsmImmediate> {
        let ident = parser.parse::<Ident>()?;
        let digits = match ident.as_str().strip_prefix('i') {
            Some(digits) => digits,
            None => return Err(parser.emit_error(ParseErrorKind::MalformedAsmImmediate)),
        };
        let parsed = match BigUint::from_str(digits).ok() {
            Some(parsed) => parsed,
            None => return Err(parser.emit_error(ParseErrorKind::MalformedAsmImmediate)),
        };
        Ok(AsmImmediate {
            span: ident.span().clone(),
            parsed,
        })
    }
}
