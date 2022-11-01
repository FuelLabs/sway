use crate::{Parse, ParseResult, ParseToEnd, Parser, ParserConsumed, Peeker};

use sway_ast::keywords::DepToken;
use sway_ast::token::DocComment;
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

        // Parses multiple Dependency
        loop {
            // Return error if there is any DocComment before a Dependency
            let mut doc_comment: Option<DocComment> = None;
            let mut token_trees = parser.token_trees();
            while let Some((doc, tokens)) = Peeker::with::<DocComment>(token_trees) {
                token_trees = tokens;
                doc_comment = Some(doc);
            }
            if let Some(doc) = doc_comment {
                if let Some((_, _)) = Peeker::with::<DepToken>(token_trees) {
                    return Err(parser
                        .emit_error_with_span(ParseErrorKind::CannotDocCommentDepToken, doc.span));
                }
            }

            if let Some(dep) = parser.guarded_parse::<DepToken, _>()? {
                dependencies.push(dep);
            } else {
                break;
            }
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
