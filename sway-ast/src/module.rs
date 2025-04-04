use sway_types::SourceId;

use crate::priv_prelude::*;

#[derive(Clone, Debug, Serialize)]
pub struct Module {
    pub kind: ModuleKind,
    pub semicolon_token: SemicolonToken,
    pub items: Vec<Item>,
}

impl Module {
    pub fn submodules(&self) -> impl Iterator<Item = &Submodule> {
        self.items.iter().filter_map(|i| {
            if let ItemKind::Submodule(submod) = &i.value {
                Some(submod)
            } else {
                None
            }
        })
    }

    pub fn source_id(&self) -> Option<SourceId> {
        self.kind.span().source_id().copied()
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
        Span::join(start, &end)
    }
}

#[derive(Clone, Debug, Serialize)]
pub enum ModuleKind {
    Script { script_token: ScriptToken },
    Contract { contract_token: ContractToken },
    Predicate { predicate_token: PredicateToken },
    Library { library_token: LibraryToken },
}

impl ModuleKind {
    /// [ModuleKind]'s friendly name string used for various reportings.
    pub fn friendly_name(&self) -> &'static str {
        use ModuleKind::*;
        match self {
            Script { .. } => "module kind (script)",
            Contract { .. } => "module kind (contract)",
            Predicate { .. } => "module kind (predicate)",
            Library { .. } => "module kind (library)",
        }
    }
}

impl Spanned for ModuleKind {
    fn span(&self) -> Span {
        match self {
            Self::Script { script_token } => script_token.span(),
            Self::Contract { contract_token } => contract_token.span(),
            Self::Predicate { predicate_token } => predicate_token.span(),
            Self::Library { library_token } => library_token.span(),
        }
    }
}
