use crate::{language::ty, type_system::*};

use sway_types::{state::StateIndex, Ident, Span, Spanned};

#[derive(Clone, Debug)]
pub struct ContractCallParams {
    pub(crate) func_selector: [u8; 4],
    pub(crate) contract_address: Box<ty::TyExpression>,
}

/// Describes the full storage access including all the subfields
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeCheckedStorageAccess {
    pub fields: Vec<TypeCheckedStorageAccessDescriptor>,
    pub(crate) ix: StateIndex,
}

impl Spanned for TypeCheckedStorageAccess {
    fn span(&self) -> Span {
        self.fields
            .iter()
            .fold(self.fields[0].span.clone(), |acc, field| {
                Span::join(acc, field.span.clone())
            })
    }
}

impl TypeCheckedStorageAccess {
    pub fn storage_field_name(&self) -> Ident {
        self.fields[0].name.clone()
    }
}

/// Describes a single subfield access in the sequence when accessing a subfield within storage.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeCheckedStorageAccessDescriptor {
    pub name: Ident,
    pub(crate) type_id: TypeId,
    pub(crate) span: Span,
}

#[derive(Clone, Debug)]
pub struct TyAsmRegisterDeclaration {
    pub(crate) initializer: Option<ty::TyExpression>,
    pub(crate) name: Ident,
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl PartialEq for TyAsmRegisterDeclaration {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && if let (Some(l), Some(r)) = (self.initializer.clone(), other.initializer.clone()) {
                l == r
            } else {
                true
            }
    }
}

impl CopyTypes for TyAsmRegisterDeclaration {
    fn copy_types(&mut self, type_mapping: &TypeMapping) {
        if let Some(ref mut initializer) = self.initializer {
            initializer.copy_types(type_mapping)
        }
    }
}
