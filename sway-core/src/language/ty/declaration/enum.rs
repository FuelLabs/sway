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
pub struct TyEnumDeclaration {
    pub call_path: CallPath,
    pub type_parameters: Vec<TypeParameter>,
    pub attributes: transform::AttributesMap,
    pub variants: Vec<TyEnumVariant>,
    pub span: Span,
    pub visibility: Visibility,
}

impl Named for TyEnumDeclaration {
    fn name(&self) -> &Ident {
        &self.call_path.suffix
    }
}

impl EqWithEngines for TyEnumDeclaration {}
impl PartialEqWithEngines for TyEnumDeclaration {
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
        self.call_path.suffix == other.call_path.suffix
            && self.type_parameters.eq(&other.type_parameters, engines)
            && self.variants.eq(&other.variants, engines)
            && self.visibility == other.visibility
    }
}

impl HashWithEngines for TyEnumDeclaration {
    fn hash<H: Hasher>(&self, state: &mut H, engines: Engines<'_>) {
        let TyEnumDeclaration {
            call_path,
            type_parameters,
            variants,
            visibility,
            // these fields are not hashed because they aren't relevant/a
            // reliable source of obj v. obj distinction
            span: _,
            attributes: _,
        } = self;
        call_path.suffix.hash(state);
        variants.hash(state, engines);
        type_parameters.hash(state, engines);
        visibility.hash(state);
    }
}

impl SubstTypes for TyEnumDeclaration {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: Engines<'_>) {
        self.variants
            .iter_mut()
            .for_each(|x| x.subst(type_mapping, engines));
        self.type_parameters
            .iter_mut()
            .for_each(|x| x.subst(type_mapping, engines));
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
    fn create_type_id(&self, engines: Engines<'_>) -> TypeId {
        let type_engine = engines.te();
        let decl_engine = engines.de();
        type_engine.insert(
            decl_engine,
            TypeInfo::Enum {
                call_path: self.call_path.clone(),
                variant_types: self.variants.clone(),
                type_parameters: self.type_parameters.clone(),
            },
        )
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
        &self.call_path.suffix
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
    fn cmp(&self, other: &Self, type_engine: &TypeEngine) -> Ordering {
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
            .then_with(|| lta.cmp(rta, type_engine))
            .then_with(|| lt.cmp(rt))
    }
}

impl SubstTypes for TyEnumVariant {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: Engines<'_>) {
        self.type_argument.subst_inner(type_mapping, engines);
    }
}

impl ReplaceSelfType for TyEnumVariant {
    fn replace_self_type(&mut self, engines: Engines<'_>, self_type: TypeId) {
        self.type_argument.replace_self_type(engines, self_type);
    }
}
