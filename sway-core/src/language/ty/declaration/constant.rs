use sway_types::{Ident, Named, Span, Spanned};

use crate::{
    language::{ty::*, CallPath, Visibility},
    transform,
    type_system::*,
};

#[derive(Clone, Debug)]
pub struct TyConstantDeclaration {
    pub call_path: CallPath,
    pub value: Option<TyExpression>,
    pub visibility: Visibility,
    pub is_configurable: bool,
    pub attributes: transform::AttributesMap,
    pub type_ascription: TypeArgument,
    pub span: Span,
}

impl Named for TyConstantDeclaration {
    fn name(&self) -> &Ident {
        &self.call_path.suffix
    }
}

impl Spanned for TyConstantDeclaration {
    fn span(&self) -> Span {
        self.span.clone()
    }
}
