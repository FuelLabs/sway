use std::hash::{Hash, Hasher};

use sway_types::{Ident, Span};

use crate::{
    engine_threading::*,
    language::{ty::*, Visibility},
    transform,
    type_system::*,
};

#[derive(Clone, Debug)]
pub struct TyConstantDeclaration {
    pub name: Ident,
    pub value: TyExpression,
    pub visibility: Visibility,
    pub return_type: TypeId,
    pub is_configurable: bool,
    pub attributes: transform::AttributesMap,
    pub type_ascription_span: Option<Span>,
    pub span: Span,
}

impl EqWithEngines for TyConstantDeclaration {}
impl PartialEqWithEngines for TyConstantDeclaration {
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
        let type_engine = engines.te();
        self.name == other.name
            && self.value.eq(&other.value, engines)
            && self.visibility == other.visibility
            && type_engine
                .get(self.return_type)
                .eq(&type_engine.get(other.return_type), engines)
            && self.is_configurable == other.is_configurable
    }
}

impl HashWithEngines for TyConstantDeclaration {
    fn hash<H: Hasher>(&self, state: &mut H, engines: Engines<'_>) {
        let TyConstantDeclaration {
            name,
            value,
            visibility,
            return_type,
            is_configurable,
            // these fields are not hashed because they aren't relevant/a
            // reliable source of obj v. obj distinction
            attributes: _,
            type_ascription_span: _,
            span: _,
        } = self;
        let type_engine = engines.te();
        name.hash(state);
        value.hash(state, engines);
        visibility.hash(state);
        type_engine.get(*return_type).hash(state, engines);
        is_configurable.hash(state);
    }
}
