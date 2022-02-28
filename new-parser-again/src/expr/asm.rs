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
    pub instructions: Vec<AsmInstruction>,
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

#[derive(Clone, Debug)]
pub struct AsmInstruction {
    pub op_code: OpCode,
    pub args: Vec<AsmArg>,
    pub semicolon_token: SemicolonToken,
}

#[derive(Clone, Debug)]
pub enum AsmArg {
    Register(Ident),
    Immediate(AsmImmediate),
}

impl Parse for AsmBlock {
    fn parse(parser: &mut Parser) -> ParseResult<AsmBlock> {
        let asm_token = parser.parse()?;
        let registers = parser.parse()?;
        let contents = parser.parse()?;
        Ok(AsmBlock { asm_token, registers, contents })
    }
}

impl Parse for AsmRegisterDeclaration {
    fn parse(parser: &mut Parser) -> ParseResult<AsmRegisterDeclaration> {
        let register = parser.parse()?;
        let value_opt = match parser.take() {
            Some(colon_token) => {
                let value = parser.parse()?;
                Some((colon_token, value))
            },
            None => None,
        };
        Ok(AsmRegisterDeclaration { register, value_opt })
    }
}

impl ParseToEnd for AsmBlockContents {
    fn parse_to_end<'a>(mut parser: Parser<'a>) -> ParseResult<(AsmBlockContents, ParserConsumed<'a>)> {
        let mut instructions = Vec::new();
        let (final_expr_opt, consumed) = loop {
            if let Some(consumed) = parser.check_empty() {
                break (None, consumed);
            }
            if let Some(op_code) = parser.take() {
                let mut args = Vec::new();
                let semicolon_token = loop {
                    if let Some(semicolon_token) = parser.take() {
                        break semicolon_token;
                    }
                    let arg = parser.parse()?;
                    args.push(arg);
                };
                let instruction = AsmInstruction { op_code, args, semicolon_token };
                instructions.push(instruction);
            } else {
                let register = parser.parse()?;
                let ty_opt = match parser.take() {
                    Some(colon_token) => {
                        let ty = parser.parse()?;
                        Some((colon_token, ty))
                    },
                    None => None,
                };
                let consumed = match parser.check_empty() {
                    Some(consumed) => consumed,
                    None => {
                        // TODO: notify user that it could be a mis-typed opcode being interpreted as a
                        // register name.
                        return Err(parser.emit_error("unexpected tokens after final expression in asm block"));
                    },
                };
                let final_expr = AsmFinalExpr { register, ty_opt };
                break (Some(final_expr), consumed);
            }
        };
        let contents = AsmBlockContents { instructions, final_expr_opt };
        Ok((contents, consumed))
    }
}

impl Parse for AsmArg {
    fn parse(parser: &mut Parser) -> ParseResult<AsmArg> {
        let ident: Ident = parser.parse()?;
        if let Some(maybe_digits) = ident.as_str().strip_prefix("i") {
            if let Some(parsed) = BigUint::from_str(maybe_digits).ok() {
                let immediate = AsmImmediate {
                    span: ident.span(),
                    parsed,
                };
                return Ok(AsmArg::Immediate(immediate));
            }
        }
        Ok(AsmArg::Register(ident))
    }
}

/*
impl ParseToEnd for AsmFinalExpr {
    fn parse_to_end<'a>(parser: Parser<'a>) -> ParseResult<(AsmFinalExpr, ParserConsumed<'a>)> {
        let register = parser.parse()?;
        let ty_opt = match parser.take() {
            Some(colon_token) => {
                let ty = parser.parse()?;
                Some((colon_token, ty))
            },
            None => None,
        };
        let consumed = match parser.check_empty() {
            Some(consumed) => consumed,
            None => {
                return Err(parser.emit_error("unexpected tokens after final expression in asm block"));
            },
        };
        let final_expr = AsmFinalExpr { register, ty_opt };
        Ok((final_expr, consumed))
    }
}

impl Parse for AsmInstruction {
    fn parse(parser: &mut Parser) -> ParseResult<AsmInstruction> {
        let op_code = parser.parse()?;
        let mut args = Vec::new();
        let semicolon_token = loop {
            if let Some(semicolon_token) = parser.take() {
                break semicolon_token;
            }
            let arg = parser.parse()?;
            args.push(arg);
        };
        Ok(AsmInstruction { op_code, args, semicolon_token })
    }
}
*/

