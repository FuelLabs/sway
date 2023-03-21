use crate::expr::op_code::parse_instruction;
use crate::{Parse, ParseResult, ParseToEnd, Parser, ParserConsumed};

use core::str::FromStr;
use num_bigint::BigUint;

use sway_ast::expr::asm::{
    AsmBlock, AsmBlockContents, AsmFinalExpr, AsmImmediate, AsmRegisterDeclaration,
};
use sway_ast::keywords::CloseCurlyBraceToken;
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

impl Parse for AsmBlockContents {
    fn parse(mut parser: &mut Parser) -> ParseResult<AsmBlockContents> {
        let mut instructions = Vec::new();
        let final_expr_opt = loop {
            if parser.peek::<CloseCurlyBraceToken>().is_some() {
                break None;
            }
            let ident = parser.parse()?;
            if parser.peek::<CloseCurlyBraceToken>().is_some() {
                break Some(AsmFinalExpr {
                    register: ident,
                    ty_opt: None,
                });
            }
            if let Some(colon_token) = parser.take() {
                let ty = parser.parse()?;
                match parser.peek::<CloseCurlyBraceToken>() {
                    Some(_) => {
                        break Some(AsmFinalExpr {
                            register: ident,
                            ty_opt: Some((colon_token, ty)),
                        })
                    }
                    None => {
                        return Err(
                            parser.emit_error(ParseErrorKind::UnexpectedTokenAfterAsmReturnType)
                        );
                    }
                }
            }
            let instruction = parse_instruction(ident, &mut parser)?;
            let semicolon_token = parser.parse()?;
            instructions.push((instruction, semicolon_token));
        };
        Ok(AsmBlockContents {
            instructions,
            final_expr_opt,
        })
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
