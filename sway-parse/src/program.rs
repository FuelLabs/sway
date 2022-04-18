use crate::priv_prelude::*;

pub struct Program {
    pub kind: ProgramKind,
    pub semicolon_token: SemicolonToken,
    pub dependencies: Vec<Dependency>,
    pub items: Vec<Item>,
}

impl Program {
    pub fn span(&self) -> Span {
        let start = self.kind.span();
        let end = match self.items.last() {
            Some(item) => item.span(),
            None => match self.dependencies.last() {
                Some(dependency) => dependency.span(),
                None => self.semicolon_token.span(),
            },
        };
        Span::join(start, end)
    }
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

impl ProgramKind {
    fn span(&self) -> Span {
        match self {
            ProgramKind::Script { script_token } => script_token.span(),
            ProgramKind::Contract { contract_token } => contract_token.span(),
            ProgramKind::Predicate { predicate_token } => predicate_token.span(),
            ProgramKind::Library {
                library_token,
                name,
            } => Span::join(library_token.span(), name.span().clone()),
        }
    }
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
            Err(parser.emit_error(ParseErrorKind::ExpectedProgramKind))
        }
    }
}

impl ParseToEnd for Program {
    fn parse_to_end<'a, 'e>(
        mut parser: Parser<'a, 'e>,
    ) -> ParseResult<(Program, ParserConsumed<'a>)> {
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
