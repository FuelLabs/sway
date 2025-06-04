use crate::{
    ast_elements::type_parameter::ConstGenericExpr, decl_engine::{DeclEngineGet as _, DeclId, DeclRef, MaterializeConstGenerics}, has_changes, language::{parsed::ConstGenericDeclaration, ty::TyExpression, CallPath}, semantic_analysis::{TypeCheckAnalysis, TypeCheckAnalysisContext}, HasChanges, SubstTypes, TypeId
};
use serde::{Deserialize, Serialize};
use sway_error::handler::{ErrorEmitted, Handler};
use sway_types::{Ident, Named, Span, Spanned};

use super::TyDeclParsedType;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TyConstGenericDecl {
    pub name: Ident,
    pub return_type: TypeId,
    pub span: Span,
    pub value: Option<TyExpression>,
}

impl SubstTypes for TyConstGenericDecl {
    fn subst_inner(&mut self, ctx: &crate::SubstTypesContext) -> crate::HasChanges {
        has_changes!{
            self.return_type.subst_inner(ctx);
            self.value.subst_inner(ctx);
        }
    }
}

impl SubstTypes for DeclRef<DeclId<TyConstGenericDecl>> {
    fn subst_inner(&mut self, ctx: &crate::SubstTypesContext) -> crate::HasChanges {
        if let Some(new_id) = ctx.type_subst_map.as_ref().and_then(|map| map.const_generics_mapping.get(self.id())) {
            let decl = ctx.engines.de().get(new_id);
            *self = DeclRef::new(decl.name().clone(), new_id.clone(), decl.span.clone());
            HasChanges::Yes
        } else {
            HasChanges::No
        }
    }
}

impl SubstTypes for DeclId<TyConstGenericDecl> {
    fn subst_inner(&mut self, ctx: &crate::SubstTypesContext) -> crate::HasChanges {
        if let Some(new_id) = ctx.type_subst_map.as_ref().and_then(|map| map.const_generics_mapping.get(self)) {
            *self = new_id.clone();
            HasChanges::Yes
        } else {
            HasChanges::No
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
        if self.name.as_str() == name {
            assert!(self.value.is_none());
            self.value = Some(value.clone());
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

impl Named for TyConstGenericDecl {
    fn name(&self) -> &Ident {
        &self.name
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
