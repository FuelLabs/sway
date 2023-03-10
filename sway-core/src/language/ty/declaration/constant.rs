use std::hash::{Hash, Hasher};

use sway_types::{Ident, Named, Span, Spanned};

use crate::{
    engine_threading::*,
    language::{ty::*, CallPath, Visibility},
    transform,
    type_system::*,
};

#[derive(Clone, Debug)]
pub struct TyConstantDeclaration {
    pub call_path: CallPath,
    pub value: Option<TyExpression>,
    pub visibility: Visibility,
    pub is_configurable: bool,
    pub attributes: transform::AttributesMap,
    pub type_ascription: TypeArgument,
    pub span: Span,
}

impl EqWithEngines for TyConstantDeclaration {}
impl PartialEqWithEngines for TyConstantDeclaration {
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
        self.call_path == other.call_path
            && self.value.eq(&other.value, engines)
            && self.visibility == other.visibility
            && self.type_ascription.eq(&other.type_ascription, engines)
            && self.is_configurable == other.is_configurable
    }
}

impl HashWithEngines for TyConstantDeclaration {
    fn hash<H: Hasher>(&self, state: &mut H, engines: Engines<'_>) {
        let TyConstantDeclaration {
            call_path,
            value,
            visibility,
            type_ascription,
            is_configurable,
            // these fields are not hashed because they aren't relevant/a
            // reliable source of obj v. obj distinction
            attributes: _,
            span: _,
        } = self;
        call_path.hash(state);
        value.hash(state, engines);
        visibility.hash(state);
        type_ascription.hash(state, engines);
        is_configurable.hash(state);
    }
}

impl Named for TyConstantDeclaration {
    fn name(&self) -> &Ident {
        &self.call_path.suffix
    }
}

impl Spanned for TyConstantDeclaration {
    fn span(&self) -> Span {
        self.span.clone()
    }
}
