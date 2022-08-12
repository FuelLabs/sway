use crate::{semantic_analysis::*, type_system::*, Ident, Visibility};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VariableMutability {
    // private + mutable
    Mutable,
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
        matches!(self, VariableMutability::Mutable)
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

impl From<bool> for VariableMutability {
    fn from(o: bool) -> Self {
        if o {
            VariableMutability::Mutable
        } else {
            VariableMutability::Immutable
        }
    }
}
// as a bool, true means mutable
impl From<VariableMutability> for bool {
    fn from(o: VariableMutability) -> bool {
        o.is_mutable()
    }
}
#[derive(Clone, Debug, Eq)]
pub struct TypedVariableDeclaration {
    pub name: Ident,
    pub body: TypedExpression,
    pub is_mutable: VariableMutability,
    pub type_ascription: TypeId,
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl PartialEq for TypedVariableDeclaration {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.body == other.body
            && self.is_mutable == other.is_mutable
            && look_up_type_id(self.type_ascription) == look_up_type_id(other.type_ascription)
    }
}

impl CopyTypes for TypedVariableDeclaration {
    fn copy_types(&mut self, type_engine: &TypeEngine, type_mapping: &TypeMapping) {
        self.type_ascription
            .update_type(type_engine, type_mapping, &self.body.span);
        self.body.copy_types(type_engine, type_mapping)
    }
}
