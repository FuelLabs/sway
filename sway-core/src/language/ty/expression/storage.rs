use super::TyExpression;
use crate::{engine_threading::*, type_system::TypeId};
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use sway_macros::Visit;
use sway_types::{Ident, Span, Spanned};

/// Describes the full storage access including all the subfields
#[derive(Clone, Debug, Serialize, Deserialize, Visit)]
pub struct TyStorageAccess {
    #[visit(skip)]
    pub fields: Vec<TyStorageAccessDescriptor>,
    #[visit(skip)]
    pub storage_field_names: Vec<String>,
    #[visit(skip)]
    pub struct_field_names: Vec<String>,
    pub key_expression: Option<Box<TyExpression>>,
    #[visit(skip)]
    pub storage_keyword_span: Span,
}

impl EqWithEngines for TyStorageAccess {}
impl PartialEqWithEngines for TyStorageAccess {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        self.fields.len() == other.fields.len()
            && self.fields.eq(&other.fields, ctx)
            && self.storage_field_names.len() == other.storage_field_names.len()
            && self.storage_field_names.eq(&other.storage_field_names)
            && self.struct_field_names.len() == other.struct_field_names.len()
            && self.struct_field_names.eq(&other.struct_field_names)
            && self.key_expression.eq(&other.key_expression, ctx)
    }
}

impl HashWithEngines for TyStorageAccess {
    fn hash<H: Hasher>(&self, state: &mut H, engines: &Engines) {
        let TyStorageAccess {
            fields,
            storage_keyword_span,
            storage_field_names,
            struct_field_names,
            key_expression,
        } = self;
        fields.hash(state, engines);
        storage_field_names.hash(state);
        struct_field_names.hash(state);
        key_expression.hash(state, engines);
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
#[derive(Clone, Debug, Serialize, Deserialize)]
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
