use sway_types::Ident;

use crate::{
    language::{ty::*, Visibility},
    AttributesMap,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TyConstantDeclaration {
    pub name: Ident,
    pub value: TyExpression,
    pub(crate) visibility: Visibility,
    pub attributes: AttributesMap,
}
