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
pub struct TyEnumDecl {
    pub call_path: CallPath,
    pub type_parameters: Vec<TypeParameter>,
    pub attributes: transform::AttributesMap,
    pub variants: Vec<TyEnumVariant>,
    pub span: Span,
    pub visibility: Visibility,
}

impl Named for TyEnumDecl {
    fn name(&self) -> &Ident {
        &self.call_path.suffix
    }
}

impl Spanned for TyEnumDecl {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl TyEnumDecl {
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
                    enum_name: self.call_path.suffix.clone(),
                    variant_name: variant_name.clone(),
                    span: variant_name.span(),
                });
                err(warnings, errors)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct TyEnumVariant {
    pub name: Ident,
    pub type_argument: TypeArgument,
    pub(crate) tag: usize,
    pub span: Span,
    pub attributes: transform::AttributesMap,
}

impl HashWithEngines for TyEnumVariant {
    fn hash<H: Hasher>(&self, state: &mut H, engines: Engines<'_>) {
        self.name.hash(state);
        self.type_argument.hash(state, engines);
        self.tag.hash(state);
    }
}

impl EqWithEngines for TyEnumVariant {}
impl PartialEqWithEngines for TyEnumVariant {
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
        self.name == other.name
            && self.type_argument.eq(&other.type_argument, engines)
            && self.tag == other.tag
    }
}

impl OrdWithEngines for TyEnumVariant {
    fn cmp(&self, other: &Self, engines: Engines<'_>) -> Ordering {
        let TyEnumVariant {
            name: ln,
            type_argument: lta,
            tag: lt,
            // these fields are not compared because they aren't relevant/a
            // reliable source of obj v. obj distinction
            span: _,
            attributes: _,
        } = self;
        let TyEnumVariant {
            name: rn,
            type_argument: rta,
            tag: rt,
            // these fields are not compared because they aren't relevant/a
            // reliable source of obj v. obj distinction
            span: _,
            attributes: _,
        } = other;
        ln.cmp(rn)
            .then_with(|| lta.cmp(rta, engines))
            .then_with(|| lt.cmp(rt))
    }
}

impl SubstTypes for TyEnumVariant {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: Engines<'_>) {
        self.type_argument.subst_inner(type_mapping, engines);
    }
}
