use crate::language::Visibility;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum VariableMutability {
    // mutable
    Mutable,
    // referenceable + mutable
    RefMutable,
    // immutable
    Immutable,
}

impl Default for VariableMutability {
    fn default() -> Self {
        VariableMutability::Immutable
    }
}

impl VariableMutability {
    pub fn new_from_ref_mut(is_reference: bool, is_mutable: bool) -> VariableMutability {
        if is_reference {
            VariableMutability::RefMutable
        } else if is_mutable {
            VariableMutability::Mutable
        } else {
            VariableMutability::Immutable
        }
    }

    pub fn is_mutable(&self) -> bool {
        matches!(
            self,
            VariableMutability::Mutable | VariableMutability::RefMutable
        )
    }

    pub fn visibility(&self) -> Visibility {
        Visibility::Private
    }

    pub fn is_immutable(&self) -> bool {
        !self.is_mutable()
    }
}
