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
