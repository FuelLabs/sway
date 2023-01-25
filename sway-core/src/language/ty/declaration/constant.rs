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
            && self.attributes == other.attributes
            && self.span == other.span
    }
}
