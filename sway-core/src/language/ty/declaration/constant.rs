use sway_types::{Ident, Span};

use crate::{
    language::{ty::*, Visibility},
    transform,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TyConstantDeclaration {
    pub name: Ident,
    pub value: TyExpression,
    pub(crate) visibility: Visibility,
    pub attributes: transform::AttributesMap,
    pub span: Span,
}
