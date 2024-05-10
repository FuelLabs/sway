use std::hash::{Hash, Hasher};

use sway_types::{state::StateIndex, Ident, Span, Spanned};

use crate::{engine_threading::*, type_system::TypeId};

/// Describes the full storage access including all the subfields
#[derive(Clone, Debug)]
pub struct TyStorageAccess {
    pub fields: Vec<TyStorageAccessDescriptor>,
    pub(crate) namespace: Option<Ident>,
    pub(crate) ix: StateIndex,
    pub storage_keyword_span: Span,
}

impl EqWithEngines for TyStorageAccess {}
impl PartialEqWithEngines for TyStorageAccess {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        self.ix == other.ix
            && self.fields.len() == other.fields.len()
            && self.fields.eq(&other.fields, ctx)
    }
}

impl HashWithEngines for TyStorageAccess {
    fn hash<H: Hasher>(&self, state: &mut H, engines: &Engines) {
        let TyStorageAccess {
            fields,
            ix,
            namespace,
            storage_keyword_span,
        } = self;
        fields.hash(state, engines);
        ix.hash(state);
        namespace.hash(state);
        storage_keyword_span.hash(state);
    }
}

impl Spanned for TyStorageAccess {
    fn span(&self) -> Span {
        // TODO: Use Span::join_all().
        self.fields
            .iter()
            .fold(self.fields[0].span.clone(), |acc, field| {
                Span::join(acc, &field.span)
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
    pub type_id: TypeId,
    pub(crate) span: Span,
}

impl EqWithEngines for TyStorageAccessDescriptor {}
impl PartialEqWithEngines for TyStorageAccessDescriptor {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        let type_engine = ctx.engines().te();
        self.name == other.name
            && type_engine
                .get(self.type_id)
                .eq(&type_engine.get(other.type_id), ctx)
    }
}

impl HashWithEngines for TyStorageAccessDescriptor {
    fn hash<H: Hasher>(&self, state: &mut H, engines: &Engines) {
        let TyStorageAccessDescriptor {
            name,
            type_id,
            // these fields are not hashed because they aren't relevant/a
            // reliable source of obj v. obj distinction
            span: _,
        } = self;
        let type_engine = engines.te();
        name.hash(state);
        type_engine.get(*type_id).hash(state, engines);
    }
}
