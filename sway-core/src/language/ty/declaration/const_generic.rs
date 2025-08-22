use crate::{
    decl_engine::MaterializeConstGenerics,
    has_changes,
    language::{parsed::ConstGenericDeclaration, ty::TyExpression, CallPath},
    semantic_analysis::{TypeCheckAnalysis, TypeCheckAnalysisContext},
    HasChanges, SubstTypes, TypeId,
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
    pub value: Option<TyExpression>,
}

impl SubstTypes for TyConstGenericDecl {
    fn subst_inner(&mut self, ctx: &crate::SubstTypesContext) -> crate::HasChanges {
        has_changes! {
            self.return_type.subst(ctx);
            if let Some(v) = ctx.get_renamed_const_generic(&self.call_path.suffix) {
                self.call_path.suffix = v.clone();
                HasChanges::Yes
            } else {
                HasChanges::No
            };
        }
    }
}

impl MaterializeConstGenerics for TyConstGenericDecl {
    fn materialize_const_generics(
        &mut self,
        _engines: &crate::Engines,
        _handler: &Handler,
        name: &str,
        value: &TyExpression,
    ) -> Result<(), ErrorEmitted> {
        if self.call_path.suffix.as_str() == name {
            match self.value.as_ref() {
                Some(v) => {
                    assert!(
                        v.extract_literal_value()
                            .unwrap()
                            .cast_value_to_u64()
                            .unwrap()
                            == value
                                .extract_literal_value()
                                .unwrap()
                                .cast_value_to_u64()
                                .unwrap(),
                        "{:?} {:?}", v, value
                    );
                }
                None => {
                    self.value = Some(value.clone());
                }
            }
        }
        Ok(())
    }
}

impl TypeCheckAnalysis for TyConstGenericDecl {
    fn type_check_analyze(
        &self,
        _handler: &Handler,
        _ctx: &mut TypeCheckAnalysisContext,
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
