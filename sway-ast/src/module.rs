use crate::priv_prelude::*;

pub struct Module {
    pub kind: ModuleKind,
    pub semicolon_token: SemicolonToken,
    pub items: Vec<Item>,
}

impl Module {
    pub fn dependencies(&self) -> impl Iterator<Item = &Dependency> {
        self.items.iter().filter_map(|i| {
            if let ItemKind::Dependency(dep) = &i.value {
                Some(dep)
            } else {
                None
            }
        })
    }
}

impl Spanned for Module {
    fn span(&self) -> Span {
        let start = self.kind.span();
        let end = if let Some(item) = self.items.last() {
            item.span()
        } else {
            self.semicolon_token.span()
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
