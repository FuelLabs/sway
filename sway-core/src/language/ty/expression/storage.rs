use super::TyExpression;
use crate::{engine_threading::*, type_system::TypeId};
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use sway_types::{Ident, Span, Spanned};

/// Describes the full storage access including all the subfields.
/// E.g.: `storage::ns1::ns2.field1.field2` will be represented as
/// a `TyStorageAccess` with 2 fields in the `fields` vector: `field1` and `field2`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TyStorageAccess {
    /// The sequence of field accesses in the storage access expression.
    /// E.g., for `storage::ns1::ns2.field1.field2`, the fields are `field1` and `field2`.
    /// Note that the first field is always a field declared in the `storage` declaration,
    /// and the rest of the fields are struct fields accessed in the storage access expression.
    pub fields: Vec<TyStorageAccessDescriptor>,
    /// The full path to the first field in the storage access expression,
    /// including all the namespace segments and the first field itself,
    /// without the `storage` keyword.
    /// E.g., for `storage::ns1::ns2.field1.field2`, the storage field path is `["ns1", "ns2", "field1"]`.
    pub storage_field_path: Vec<String>,
    /// The field names in the struct fields access path.
    /// E.g., for `storage::ns1::ns2.field1.field2.field3`, the struct field names are `["field2", "field3"]`.
    pub struct_field_names: Vec<String>,
    /// The key in the `in` keyword expression, if specified for the
    /// first field in `fields`.
    pub key_expression: Option<Box<TyExpression>>,
    pub storage_keyword_span: Span,
}

impl EqWithEngines for TyStorageAccess {}
impl PartialEqWithEngines for TyStorageAccess {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        self.fields.len() == other.fields.len()
            && self.fields.eq(&other.fields, ctx)
            && self.storage_field_path.len() == other.storage_field_path.len()
            && self.storage_field_path.eq(&other.storage_field_path)
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
            storage_field_path: storage_field_names,
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
