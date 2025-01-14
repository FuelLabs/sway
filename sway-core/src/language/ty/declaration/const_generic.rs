use crate::{
    language::{parsed::ConstGenericDeclaration, CallPath},
    semantic_analysis::{TypeCheckAnalysis, TypeCheckAnalysisContext},
    TypeId,
};
use serde::{Deserialize, Serialize};
use sway_error::handler::{ErrorEmitted, Handler};
use sway_types::{BaseIdent, Ident, Named, Span, Spanned};

use super::TyDeclParsedType;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TyConstGenericDecl {
    pub call_path: CallPath,
    pub return_type: TypeId,
    pub span: Span,
}

impl TypeCheckAnalysis for TyConstGenericDecl {
    fn type_check_analyze(
        &self,
        handler: &Handler,
        ctx: &mut TypeCheckAnalysisContext,
    ) -> Result<(), ErrorEmitted> {
        Ok(())
    }
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
