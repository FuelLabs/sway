use crate::{
    language::{parsed::Expression, Visibility},
    transform,
    type_system::TypeInfo,
};
use sway_types::{Ident, Span};

#[derive(Debug, Clone)]
pub struct ConstantDeclaration {
    pub name: Ident,
    pub attributes: transform::AttributesMap,
    pub type_ascription: TypeInfo,
    pub type_ascription_span: Option<Span>,
    pub value: Expression,
    pub visibility: Visibility,
}
