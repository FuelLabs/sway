use crate::priv_prelude::*;

pub struct Program {
    pub kind: ProgramKind,
    pub semicolon_token: SemicolonToken,
    pub dependencies: Vec<Dependency>,
    pub items: Vec<Item>,
}

pub enum ProgramKind {
    Script {
        script_token: ScriptToken,
    },
    Contract {
        contract_token: ContractToken,
    },
    Predicate {
        predicate_token: PredicateToken,
    },
    Library {
        library_token: LibraryToken,
        name: Ident,
    },
}

impl Parse for ProgramKind {
    fn parse(parser: &mut Parser) -> ParseResult<ProgramKind> {
        if let Some(script_token) = parser.take() {
            Ok(ProgramKind::Script { script_token })
        } else if let Some(contract_token) = parser.take() {
            Ok(ProgramKind::Contract { contract_token })
        } else if let Some(predicate_token) = parser.take() {
            Ok(ProgramKind::Predicate { predicate_token })
        } else if let Some(library_token) = parser.take() {
            let name = parser.parse()?;
            Ok(ProgramKind::Library {
                library_token,
                name,
            })
        } else {
            Err(parser.emit_error(
                "expected a program kind (script, contract, predicate, or library)"
            ))
        }
    }
}

impl ParseToEnd for Program {
    fn parse_to_end(mut parser: Parser) -> ParseResult<(Program, ParserConsumed)> {
        let kind = parser.parse()?;
        let semicolon_token = parser.parse()?;
        let mut dependencies = Vec::new();
        while let Some(..) = parser.peek::<DepToken>() {
            let dependency = parser.parse()?;
            dependencies.push(dependency);
        }
        let (items, consumed) = parser.parse_to_end()?; 
        let program = Program {
            kind,
            semicolon_token,
            dependencies,
            items,
        };
        Ok((program, consumed))
    }
}

