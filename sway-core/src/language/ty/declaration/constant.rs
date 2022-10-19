use sway_types::Ident;

use crate::{
    language::{ty::*, Visibility},
    transform,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TyConstantDeclaration {
    pub name: Ident,
    pub value: TyExpression,
    pub visibility: Visibility,
    pub attributes: transform::AttributesMap,
}
