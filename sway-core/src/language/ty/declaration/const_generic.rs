use crate::{
    ast_elements::type_parameter::ConstGenericExpr, decl_engine::{parsed_id::ParsedDeclId, DeclEngineGet as _, DeclEngineReplace as _, DeclId, MaterializeConstGenerics}, language::{parsed::ConstGenericDeclaration, ty::TyExpression, CallPath}, semantic_analysis::{TypeCheckAnalysis, TypeCheckAnalysisContext}, SubstTypes, TypeId
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
    pub value: Option<ConstGenericExpr>,
}

impl SubstTypes for TyConstGenericDecl {
    fn subst_inner(&mut self, ctx: &crate::SubstTypesContext) -> crate::HasChanges {
        self.return_type.subst(ctx)
    }
}

impl MaterializeConstGenerics for DeclId<TyConstGenericDecl> {
    fn materialize_const_generics(
        &mut self,
        engines: &crate::Engines,
        handler: &Handler,
        name: &str,
        value: &TyExpression,
    ) -> Result<(), ErrorEmitted> {
        let decl = engines.de().get(self);
        if decl.name().as_str() == name {
            match decl.value.as_ref() {
                Some(expr) => {
                    eprintln!("{:?} {:?} {:?}", self, expr, value);
                    assert!(expr.as_literal_val().unwrap() as u64 == value.extract_literal_value().unwrap().cast_value_to_u64().unwrap());
                }
                None => {
                    let mut new_decl = (&*decl).clone();
                    new_decl.value = Some(ConstGenericExpr::from_ty_expression(handler, value)?);

                    engines.de().replace(*self, new_decl);
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
