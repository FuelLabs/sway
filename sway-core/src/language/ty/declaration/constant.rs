use sway_types::{Ident, Named, Span, Spanned};

use crate::{
    language::{ty::*, CallPath, Visibility},
    transform,
    type_system::*,
};

#[derive(Clone, Debug)]
pub struct TyConstantDecl {
    pub call_path: CallPath,
    pub value: Option<TyExpression>,
    pub visibility: Visibility,
    pub is_configurable: bool,
    pub attributes: transform::AttributesMap,
    pub return_type: TypeId,
    pub type_ascription: TypeArgument,
    pub span: Span,
    pub implementing_type: Option<TyDecl>,
}

impl Named for TyConstantDecl {
    fn name(&self) -> &Ident {
        &self.call_path.suffix
    }
}

impl Spanned for TyConstantDecl {
    fn span(&self) -> Span {
        self.span.clone()
    }
}
