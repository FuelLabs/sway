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
    pub attributes: transform::AttributesMap,
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
                .look_up_type_id(self.return_type)
                .eq(&type_engine.look_up_type_id(other.return_type), engines)
            && self.attributes == other.attributes
            && self.span == other.span
    }
}
