use crate::{
    language::{parsed::ConstGenericDeclaration, CallPath},
    TypeId,
};
use serde::{Deserialize, Serialize};
use sway_types::{BaseIdent, Ident, Named, Span, Spanned};

use super::TyDeclParsedType;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TyConstGenericDecl {
    pub call_path: CallPath,
    pub return_type: TypeId,
    pub span: Span,
}

impl TyConstGenericDecl {
    pub fn name(&self) -> &BaseIdent {
        &self.call_path.suffix
    }
}

impl Named for TyConstGenericDecl {
    fn name(&self) -> &Ident {
        &self.call_path.suffix
    }
}

impl Spanned for TyConstGenericDecl {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl TyDeclParsedType for TyConstGenericDecl {
    type ParsedType = ConstGenericDeclaration;
}
