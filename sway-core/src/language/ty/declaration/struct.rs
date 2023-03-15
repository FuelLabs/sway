use std::{
    cmp::Ordering,
    hash::{Hash, Hasher},
};

use sway_error::error::CompileError;
use sway_types::{Ident, Named, Span, Spanned};

use crate::{
    engine_threading::*,
    error::*,
    language::{CallPath, Visibility},
    transform,
    type_system::*,
};

#[derive(Clone, Debug)]
pub struct TyStructDecl {
    pub call_path: CallPath,
    pub fields: Vec<TyStructField>,
    pub type_parameters: Vec<TypeParameter>,
    pub visibility: Visibility,
    pub span: Span,
    pub attributes: transform::AttributesMap,
}

impl Named for TyStructDecl {
    fn name(&self) -> &Ident {
        &self.call_path.suffix
    }
}

impl EqWithEngines for TyStructDecl {}
impl PartialEqWithEngines for TyStructDecl {
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
        self.call_path.suffix == other.call_path.suffix
            && self.fields.eq(&other.fields, engines)
            && self.type_parameters.eq(&other.type_parameters, engines)
            && self.visibility == other.visibility
    }
}

impl HashWithEngines for TyStructDecl {
    fn hash<H: Hasher>(&self, state: &mut H, engines: Engines<'_>) {
        let TyStructDecl {
            call_path,
            fields,
            type_parameters,
            visibility,
            // these fields are not hashed because they aren't relevant/a
            // reliable source of obj v. obj distinction
            span: _,
            attributes: _,
        } = self;
        call_path.suffix.hash(state);
        fields.hash(state, engines);
        type_parameters.hash(state, engines);
        visibility.hash(state);
    }
}

impl SubstTypes for TyStructDecl {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: Engines<'_>) {
        self.fields
            .iter_mut()
            .for_each(|x| x.subst(type_mapping, engines));
        self.type_parameters
            .iter_mut()
            .for_each(|x| x.subst(type_mapping, engines));
    }
}

impl Spanned for TyStructDecl {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl MonomorphizeHelper for TyStructDecl {
    fn type_parameters(&self) -> &[TypeParameter] {
        &self.type_parameters
    }

    fn name(&self) -> &Ident {
        &self.call_path.suffix
    }
}

impl TyStructDecl {
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
                    struct_name: self.call_path.suffix.clone(),
                    span: field_to_access.span(),
                });
                err(warnings, errors)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct TyStructField {
    pub name: Ident,
    pub span: Span,
    pub type_argument: TypeArgument,
    pub attributes: transform::AttributesMap,
}

impl HashWithEngines for TyStructField {
    fn hash<H: Hasher>(&self, state: &mut H, engines: Engines<'_>) {
        let TyStructField {
            name,
            type_argument,
            // these fields are not hashed because they aren't relevant/a
            // reliable source of obj v. obj distinction
            span: _,
            attributes: _,
        } = self;
        name.hash(state);
        type_argument.hash(state, engines);
    }
}

impl EqWithEngines for TyStructField {}
impl PartialEqWithEngines for TyStructField {
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
        self.name == other.name && self.type_argument.eq(&other.type_argument, engines)
    }
}

impl OrdWithEngines for TyStructField {
    fn cmp(&self, other: &Self, engines: Engines<'_>) -> Ordering {
        let TyStructField {
            name: ln,
            type_argument: lta,
            // these fields are not compared because they aren't relevant/a
            // reliable source of obj v. obj distinction
            span: _,
            attributes: _,
        } = self;
        let TyStructField {
            name: rn,
            type_argument: rta,
            // these fields are not compared because they aren't relevant/a
            // reliable source of obj v. obj distinction
            span: _,
            attributes: _,
        } = other;
        ln.cmp(rn).then_with(|| lta.cmp(rta, engines))
    }
}

impl SubstTypes for TyStructField {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: Engines<'_>) {
        self.type_argument.subst_inner(type_mapping, engines);
    }
}
