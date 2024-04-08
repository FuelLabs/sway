use std::{
    cmp::Ordering,
    hash::{Hash, Hasher},
};

use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
};
use sway_types::{Ident, Named, Span, Spanned};

use crate::{
    engine_threading::*,
    has_changes,
    language::{CallPath, Visibility},
    semantic_analysis::type_check_context::MonomorphizeHelper,
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

impl EqWithEngines for TyEnumDecl {}
impl PartialEqWithEngines for TyEnumDecl {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        self.call_path == other.call_path
            && self.type_parameters.eq(&other.type_parameters, ctx)
            && self.variants.eq(&other.variants, ctx)
            && self.visibility == other.visibility
    }
}

impl HashWithEngines for TyEnumDecl {
    fn hash<H: Hasher>(&self, state: &mut H, engines: &Engines) {
        let TyEnumDecl {
            call_path,
            type_parameters,
            variants,
            visibility,
            // these fields are not hashed because they aren't relevant/a
            // reliable source of obj v. obj distinction
            span: _,
            attributes: _,
        } = self;
        call_path.hash(state);
        variants.hash(state, engines);
        type_parameters.hash(state, engines);
        visibility.hash(state);
    }
}

impl SubstTypes for TyEnumDecl {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: &Engines) -> HasChanges {
        has_changes! {
            self.variants.subst(type_mapping, engines);
            self.type_parameters.subst(type_mapping, engines);
        }
    }
}

impl Spanned for TyEnumDecl {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl MonomorphizeHelper for TyEnumDecl {
    fn type_parameters(&self) -> &[TypeParameter] {
        &self.type_parameters
    }

    fn name(&self) -> &Ident {
        &self.call_path.suffix
    }

    fn has_self_type_param(&self) -> bool {
        false
    }
}

impl TyEnumDecl {
    pub(crate) fn expect_variant_from_name(
        &self,
        handler: &Handler,
        variant_name: &Ident,
    ) -> Result<&TyEnumVariant, ErrorEmitted> {
        match self
            .variants
            .iter()
            .find(|x| x.name.as_str() == variant_name.as_str())
        {
            Some(variant) => Ok(variant),
            None => Err(handler.emit_err(CompileError::UnknownEnumVariant {
                enum_name: self.call_path.suffix.clone(),
                variant_name: variant_name.clone(),
                span: variant_name.span(),
            })),
        }
    }
}

impl Spanned for TyEnumVariant {
    fn span(&self) -> Span {
        self.span.clone()
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
    fn hash<H: Hasher>(&self, state: &mut H, engines: &Engines) {
        self.name.hash(state);
        self.type_argument.hash(state, engines);
        self.tag.hash(state);
    }
}

impl EqWithEngines for TyEnumVariant {}
impl PartialEqWithEngines for TyEnumVariant {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        self.name == other.name
            && self.type_argument.eq(&other.type_argument, ctx)
            && self.tag == other.tag
    }
}

impl OrdWithEngines for TyEnumVariant {
    fn cmp(&self, other: &Self, ctx: &OrdWithEnginesContext) -> Ordering {
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
            .then_with(|| lta.cmp(rta, ctx))
            .then_with(|| lt.cmp(rt))
    }
}

impl SubstTypes for TyEnumVariant {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: &Engines) -> HasChanges {
        self.type_argument.subst_inner(type_mapping, engines)
    }
}
