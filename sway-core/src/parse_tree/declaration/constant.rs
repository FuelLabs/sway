use crate::{
    parse_tree::{Expression, Visibility},
    type_system::TypeInfo,
};

use sway_types::{Ident, Span};

#[derive(Debug, Clone)]
pub struct ConstantDeclaration {
    pub name: Ident,
    pub type_ascription: TypeInfo,
    pub type_ascription_span: Option<Span>,
    pub value: Expression,
    pub visibility: Visibility,
}
