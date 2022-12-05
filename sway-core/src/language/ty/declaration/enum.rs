use std::hash::{Hash, Hasher};

use sway_error::error::CompileError;
use sway_types::{Ident, Span, Spanned};

use crate::{engine_threading::*, error::*, language::Visibility, transform, type_system::*};

#[derive(Clone, Debug)]
pub struct TyEnumDeclaration {
    pub name: Ident,
    pub type_parameters: Vec<TypeParameter>,
    pub attributes: transform::AttributesMap,
    pub variants: Vec<TyEnumVariant>,
    pub span: Span,
    pub visibility: Visibility,
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl EqWithEngines for TyEnumDeclaration {}
impl PartialEqWithEngines for TyEnumDeclaration {
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
        self.name == other.name
            && self.type_parameters.eq(&other.type_parameters, engines)
            && self.variants.eq(&other.variants, engines)
            && self.visibility == other.visibility
    }
}

impl CopyTypes for TyEnumDeclaration {
    fn copy_types_inner(&mut self, type_mapping: &TypeMapping, engines: Engines<'_>) {
        self.variants
            .iter_mut()
            .for_each(|x| x.copy_types(type_mapping, engines));
        self.type_parameters
            .iter_mut()
            .for_each(|x| x.copy_types(type_mapping, engines));
    }
}

impl ReplaceSelfType for TyEnumDeclaration {
    fn replace_self_type(&mut self, engines: Engines<'_>, self_type: TypeId) {
        self.variants
            .iter_mut()
            .for_each(|x| x.replace_self_type(engines, self_type));
        self.type_parameters
            .iter_mut()
            .for_each(|x| x.replace_self_type(engines, self_type));
    }
}

impl CreateTypeId for TyEnumDeclaration {
    fn create_type_id(&self, type_engine: &TypeEngine) -> TypeId {
        type_engine.insert_type(TypeInfo::Enum {
            name: self.name.clone(),
            variant_types: self.variants.clone(),
            type_parameters: self.type_parameters.clone(),
        })
    }
}

impl Spanned for TyEnumDeclaration {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl MonomorphizeHelper for TyEnumDeclaration {
    fn type_parameters(&self) -> &[TypeParameter] {
        &self.type_parameters
    }

    fn name(&self) -> &Ident {
        &self.name
    }
}

impl TyEnumDeclaration {
    pub(crate) fn expect_variant_from_name(
        &self,
        variant_name: &Ident,
    ) -> CompileResult<&TyEnumVariant> {
        let warnings = vec![];
        let mut errors = vec![];
        match self
            .variants
            .iter()
            .find(|x| x.name.as_str() == variant_name.as_str())
        {
            Some(variant) => ok(variant, warnings, errors),
            None => {
                errors.push(CompileError::UnknownEnumVariant {
                    enum_name: self.name.clone(),
                    variant_name: variant_name.clone(),
                    span: self.span.clone(),
                });
                err(warnings, errors)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct TyEnumVariant {
    pub name: Ident,
    pub type_id: TypeId,
    pub initial_type_id: TypeId,
    pub type_span: Span,
    pub(crate) tag: usize,
    pub span: Span,
    pub attributes: transform::AttributesMap,
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl HashWithEngines for TyEnumVariant {
    fn hash<H: Hasher>(&self, state: &mut H, type_engine: &TypeEngine) {
        self.name.hash(state);
        type_engine
            .look_up_type_id(self.type_id)
            .hash(state, type_engine);
        self.tag.hash(state);
    }
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl EqWithEngines for TyEnumVariant {}
impl PartialEqWithEngines for TyEnumVariant {
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
        let type_engine = engines.te();
        self.name == other.name
            && type_engine
                .look_up_type_id(self.type_id)
                .eq(&type_engine.look_up_type_id(other.type_id), engines)
            && self.tag == other.tag
    }
}

impl CopyTypes for TyEnumVariant {
    fn copy_types_inner(&mut self, type_mapping: &TypeMapping, engines: Engines<'_>) {
        self.type_id.copy_types(type_mapping, engines);
    }
}

impl ReplaceSelfType for TyEnumVariant {
    fn replace_self_type(&mut self, engines: Engines<'_>, self_type: TypeId) {
        self.type_id.replace_self_type(engines, self_type);
    }
}
