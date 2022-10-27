use sway_types::Spanned;

use crate::{language::CallPath, TypeArgument};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub(crate) struct TraitConstraint {
    pub(crate) trait_name: CallPath,
    pub(crate) type_arguments: Vec<TypeArgument>,
}

impl Spanned for TraitConstraint {
    fn span(&self) -> sway_types::Span {
        self.trait_name.span()
    }
}
