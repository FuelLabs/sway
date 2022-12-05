use std::hash::{Hash, Hasher};

use sway_error::error::CompileError;
use sway_types::{Ident, Span, Spanned};

use crate::{engine_threading::*, error::*, language::Visibility, transform, type_system::*};

#[derive(Clone, Debug)]
pub struct TyStructDeclaration {
    pub name: Ident,
    pub fields: Vec<TyStructField>,
    pub type_parameters: Vec<TypeParameter>,
    pub visibility: Visibility,
    pub span: Span,
    pub attributes: transform::AttributesMap,
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl EqWithEngines for TyStructDeclaration {}
impl PartialEqWithEngines for TyStructDeclaration {
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
        self.name == other.name
            && self.fields.eq(&other.fields, engines)
            && self.type_parameters.eq(&other.type_parameters, engines)
            && self.visibility == other.visibility
    }
}

impl CopyTypes for TyStructDeclaration {
    fn copy_types_inner(&mut self, type_mapping: &TypeMapping, engines: Engines<'_>) {
        self.fields
            .iter_mut()
            .for_each(|x| x.copy_types(type_mapping, engines));
        self.type_parameters
            .iter_mut()
            .for_each(|x| x.copy_types(type_mapping, engines));
    }
}

impl ReplaceSelfType for TyStructDeclaration {
    fn replace_self_type(&mut self, engines: Engines<'_>, self_type: TypeId) {
        self.fields
            .iter_mut()
            .for_each(|x| x.replace_self_type(engines, self_type));
        self.type_parameters
            .iter_mut()
            .for_each(|x| x.replace_self_type(engines, self_type));
    }
}

impl CreateTypeId for TyStructDeclaration {
    fn create_type_id(&self, type_engine: &TypeEngine) -> TypeId {
        type_engine.insert_type(TypeInfo::Struct {
            name: self.name.clone(),
            fields: self.fields.clone(),
            type_parameters: self.type_parameters.clone(),
        })
    }
}

impl Spanned for TyStructDeclaration {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl MonomorphizeHelper for TyStructDeclaration {
    fn type_parameters(&self) -> &[TypeParameter] {
        &self.type_parameters
    }

    fn name(&self) -> &Ident {
        &self.name
    }
}

impl TyStructDeclaration {
    pub(crate) fn expect_field(&self, field_to_access: &Ident) -> CompileResult<&TyStructField> {
        let warnings = vec![];
        let mut errors = vec![];
        match self
            .fields
            .iter()
            .find(|TyStructField { name, .. }| name.as_str() == field_to_access.as_str())
        {
            Some(field) => ok(field, warnings, errors),
            None => {
                errors.push(CompileError::FieldNotFound {
                    available_fields: self
                        .fields
                        .iter()
                        .map(|TyStructField { name, .. }| name.to_string())
                        .collect::<Vec<_>>()
                        .join("\n"),
                    field_name: field_to_access.clone(),
                    struct_name: self.name.clone(),
                });
                err(warnings, errors)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct TyStructField {
    pub name: Ident,
    pub type_id: TypeId,
    pub initial_type_id: TypeId,
    pub span: Span,
    pub type_span: Span,
    pub attributes: transform::AttributesMap,
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl HashWithEngines for TyStructField {
    fn hash<H: Hasher>(&self, state: &mut H, type_engine: &TypeEngine) {
        self.name.hash(state);
        type_engine
            .look_up_type_id(self.type_id)
            .hash(state, type_engine);
    }
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl EqWithEngines for TyStructField {}
impl PartialEqWithEngines for TyStructField {
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
        let type_engine = engines.te();
        self.name == other.name
            && type_engine
                .look_up_type_id(self.type_id)
                .eq(&type_engine.look_up_type_id(other.type_id), engines)
    }
}

impl CopyTypes for TyStructField {
    fn copy_types_inner(&mut self, type_mapping: &TypeMapping, engines: Engines<'_>) {
        self.type_id.copy_types(type_mapping, engines);
    }
}

impl ReplaceSelfType for TyStructField {
    fn replace_self_type(&mut self, engines: Engines<'_>, self_type: TypeId) {
        self.type_id.replace_self_type(engines, self_type);
    }
}
