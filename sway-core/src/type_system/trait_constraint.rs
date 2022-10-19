use crate::language::CallPath;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct TraitConstraint {
    pub call_path: CallPath,
}
