use crate::language::Visibility;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VariableMutability {
    // private + mutable
    Mutable,
    // private + referenceable + mutable
    RefMutable,
    // private + immutable
    Immutable,
    // public + immutable
    ExportedConst,
    // public + mutable is invalid
}

impl Default for VariableMutability {
    fn default() -> Self {
        VariableMutability::Immutable
    }
}
impl VariableMutability {
    pub fn is_mutable(&self) -> bool {
        matches!(
            self,
            VariableMutability::Mutable | VariableMutability::RefMutable
        )
    }

    pub fn visibility(&self) -> Visibility {
        match self {
            VariableMutability::ExportedConst => Visibility::Public,
            _ => Visibility::Private,
        }
    }

    pub fn is_immutable(&self) -> bool {
        !self.is_mutable()
    }
}

pub fn convert_to_variable_immutability(
    is_reference: bool,
    is_mutable: bool,
) -> VariableMutability {
    if is_reference {
        VariableMutability::RefMutable
    } else if is_mutable {
        VariableMutability::Mutable
    } else {
        VariableMutability::Immutable
    }
}
