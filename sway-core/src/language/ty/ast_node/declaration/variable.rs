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

#[derive(Clone, Debug, Eq)]
pub struct TyVariableDeclaration {
    pub name: Ident,
    pub body: TyExpression,
    pub mutability: VariableMutability,
    pub type_ascription: TypeId,
    pub type_ascription_span: Option<Span>,
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl PartialEq for TyVariableDeclaration {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.body == other.body
            && self.mutability == other.mutability
            && look_up_type_id(self.type_ascription) == look_up_type_id(other.type_ascription)
    }
}

impl CopyTypes for TyVariableDeclaration {
    fn copy_types(&mut self, type_mapping: &TypeMapping) {
        self.type_ascription.copy_types(type_mapping);
        self.body.copy_types(type_mapping)
    }
}
