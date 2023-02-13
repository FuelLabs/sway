use crate::{
    language::{parsed::Expression, Visibility},
    transform, TypeArgument,
};
use sway_types::{Ident, Span};

#[derive(Debug, Clone)]
pub struct ConstantDeclaration {
    pub name: Ident,
    pub attributes: transform::AttributesMap,
    pub type_ascription: TypeArgument,
    pub value: Expression,
    pub visibility: Visibility,
    pub is_configurable: bool,
    pub span: Span,
}
