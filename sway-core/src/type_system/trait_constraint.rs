use sway_types::Spanned;

use crate::language::CallPath;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub(crate) struct TraitConstraint {
    pub(crate) call_path: CallPath,
}

impl Spanned for TraitConstraint {
    fn span(&self) -> sway_types::Span {
        self.call_path.span()
    }
}
