use crate::{
    parse_tree::{Expression, Visibility},
    type_system::TypeInfo,
};

use sway_types::ident::Ident;

#[derive(Debug, Clone)]
pub struct ConstantDeclaration {
    pub name: Ident,
    pub type_ascription: TypeInfo,
    pub value: Expression,
    pub visibility: Visibility,
}
