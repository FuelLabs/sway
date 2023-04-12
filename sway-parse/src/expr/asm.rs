use crate::expr::op_code::parse_instruction;
use crate::{Parse, ParseResult, ParseToEnd, Parser, ParserConsumed};

use core::str::FromStr;
use num_bigint::BigUint;

use sway_ast::expr::asm::{
    AsmBlock, AsmBlockContents, AsmFinalExpr, AsmImmediate, AsmRegisterDeclaration,
};
use sway_error::parser_error::ParseErrorKind;
use sway_types::{Ident, Spanned};

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
        mut parser: Parser<'a, '_>,
    ) -> ParseResult<(AsmBlockContents, ParserConsumed<'a>)> {
        let mut instructions = Vec::new();
        let (final_expr_opt, consumed) = loop {
            if let Some(consumed) = parser.check_empty() {
                break (None, consumed);
            }

            // Parse the opcode directly instead of calling `parser.parse()` to avoid checking for
            // illegal identifiers such as keywords. opcode names should not be subject to those
            // checks because some opcodes, such as `mod`, are also Sway keywords.
            let ident = match parser.take::<Ident>() {
                Some(ident) => ident,
                None => return Err(parser.emit_error(ParseErrorKind::ExpectedIdent)),
            };

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
        let digits = ident
            .as_str()
            .strip_prefix('i')
            .ok_or_else(|| parser.emit_error(ParseErrorKind::MalformedAsmImmediate))?;
        let parsed = BigUint::from_str(digits)
            .ok()
            .ok_or_else(|| parser.emit_error(ParseErrorKind::MalformedAsmImmediate))?;
        Ok(AsmImmediate {
            span: ident.span(),
            parsed,
        })
    }
}
