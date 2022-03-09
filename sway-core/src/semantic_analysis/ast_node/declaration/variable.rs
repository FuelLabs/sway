use crate::semantic_analysis::TypedExpression;
use crate::type_engine::*;
use crate::Ident;
use crate::Visibility;
use crate::{type_engine::TypeId, TypeParameter};

#[derive(Clone, Debug)]
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
#[derive(Clone, Debug)]
pub struct TypedVariableDeclaration {
    pub(crate) name: Ident,
    pub(crate) body: TypedExpression,
    pub(crate) is_mutable: VariableMutability,
    pub(crate) type_ascription: TypeId,
    pub(crate) const_decl_origin: bool,
}

impl TypedVariableDeclaration {
    pub(crate) fn copy_types(&mut self, type_mapping: &[(TypeParameter, TypeId)]) {
        self.type_ascription =
            match look_up_type_id(self.type_ascription).matches_type_parameter(type_mapping) {
                Some(matching_id) => insert_type(TypeInfo::Ref(matching_id)),
                None => insert_type(look_up_type_id_raw(self.type_ascription)),
            };
        self.body.copy_types(type_mapping)
    }
}
