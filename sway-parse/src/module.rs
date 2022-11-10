use crate::{Parse, ParseResult, ParseToEnd, Parser, ParserConsumed};

use sway_ast::keywords::DepToken;
use sway_ast::{Module, ModuleKind};
use sway_error::parser_error::ParseErrorKind;

impl Parse for ModuleKind {
    fn parse(parser: &mut Parser) -> ParseResult<Self> {
        if let Some(script_token) = parser.take() {
            Ok(Self::Script { script_token })
        } else if let Some(contract_token) = parser.take() {
            Ok(Self::Contract { contract_token })
        } else if let Some(predicate_token) = parser.take() {
            Ok(Self::Predicate { predicate_token })
        } else if let Some(library_token) = parser.take() {
            let name = parser.parse()?;
            Ok(Self::Library {
                library_token,
                name,
            })
        } else {
            Err(parser.emit_error(ParseErrorKind::ExpectedModuleKind))
        }
    }
}

impl ParseToEnd for Module {
    fn parse_to_end<'a, 'e>(mut parser: Parser<'a, 'e>) -> ParseResult<(Self, ParserConsumed<'a>)> {
        let (kind, semicolon_token) = parser.parse()?;
        let mut dependencies = Vec::new();
        while let Some(dep) = parser.guarded_parse::<DepToken, _>()? {
            dependencies.push(dep);
        }
        let (items, consumed) = parser.parse_to_end()?;
        let module = Self {
            kind,
            semicolon_token,
            dependencies,
            items,
        };
        Ok((module, consumed))
    }
}
