use sway_types::Ident;

use crate::language::{ty::TyExpression, Visibility};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TyConstantDeclaration {
    pub name: Ident,
    pub value: TyExpression,
    pub(crate) visibility: Visibility,
}
