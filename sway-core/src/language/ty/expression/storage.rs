use sway_types::{state::StateIndex, Ident, Span, Spanned};

use crate::type_system::TypeId;

/// Describes the full storage access including all the subfields
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TyStorageAccess {
    pub fields: Vec<TyStorageAccessDescriptor>,
    pub(crate) ix: StateIndex,
}

impl Spanned for TyStorageAccess {
    fn span(&self) -> Span {
        self.fields
            .iter()
            .fold(self.fields[0].span.clone(), |acc, field| {
                Span::join(acc, field.span.clone())
            })
    }
}

impl TyStorageAccess {
    pub fn storage_field_name(&self) -> Ident {
        self.fields[0].name.clone()
    }
}

/// Describes a single subfield access in the sequence when accessing a subfield within storage.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TyStorageAccessDescriptor {
    pub name: Ident,
    pub(crate) type_id: TypeId,
    pub(crate) span: Span,
}
