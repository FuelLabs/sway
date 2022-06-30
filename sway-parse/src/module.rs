use crate::priv_prelude::*;

pub struct Module {
    pub kind: ModuleKind,
    pub semicolon_token: SemicolonToken,
    pub dependencies: Vec<Dependency>,
    pub items: Vec<Item>,
}

impl Spanned for Module {
    fn span(&self) -> Span {
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

pub enum ModuleKind {
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

impl Spanned for ModuleKind {
    fn span(&self) -> Span {
        match self {
            Self::Script { script_token } => script_token.span(),
            Self::Contract { contract_token } => contract_token.span(),
            Self::Predicate { predicate_token } => predicate_token.span(),
            Self::Library {
                library_token,
                name,
            } => Span::join(library_token.span(), name.span()),
        }
    }
}

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
        let kind = parser.parse()?;
        let semicolon_token = parser.parse()?;
        let mut dependencies = Vec::new();
        while let Some(..) = parser.peek::<DepToken>() {
            let dependency = parser.parse()?;
            dependencies.push(dependency);
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
