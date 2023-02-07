use std::hash::{Hash, Hasher};

use sway_types::{state::StateIndex, Ident, Span, Spanned};

use crate::{engine_threading::*, type_system::TypeId};

/// Describes the full storage access including all the subfields
#[derive(Clone, Debug)]
pub struct TyStorageAccess {
    pub fields: Vec<TyStorageAccessDescriptor>,
    pub(crate) ix: StateIndex,
}

impl EqWithEngines for TyStorageAccess {}
impl PartialEqWithEngines for TyStorageAccess {
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
        self.ix == other.ix
            && self.fields.len() == other.fields.len()
            && self.fields.eq(&other.fields, engines)
    }
}

impl HashWithEngines for TyStorageAccess {
    fn hash<H: Hasher>(&self, state: &mut H, engines: Engines<'_>) {
        self.fields.hash(state, engines);
        self.ix.hash(state);
    }
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
#[derive(Clone, Debug)]
pub struct TyStorageAccessDescriptor {
    pub name: Ident,
    pub(crate) type_id: TypeId,
    pub(crate) span: Span,
}

impl EqWithEngines for TyStorageAccessDescriptor {}
impl PartialEqWithEngines for TyStorageAccessDescriptor {
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
        let type_engine = engines.te();
        self.name == other.name
            && type_engine
                .get(self.type_id)
                .eq(&type_engine.get(other.type_id), engines)
    }
}

impl HashWithEngines for TyStorageAccessDescriptor {
    fn hash<H: Hasher>(&self, state: &mut H, engines: Engines<'_>) {
        let type_engine = engines.te();
        self.name.hash(state);
        type_engine.get(self.type_id).hash(state, engines);
    }
}
