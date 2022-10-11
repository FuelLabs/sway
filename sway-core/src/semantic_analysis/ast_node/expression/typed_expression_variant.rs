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
