use crate::{
    decl_engine::{DeclMapping, MaterializeConstGenerics, ReplaceDecls},
    engine_threading::*,
    has_changes,
    language::{parsed::ConstantDeclaration, ty::*, CallPath, Visibility},
    semantic_analysis::TypeCheckContext,
    transform,
    type_system::*,
};
use serde::{Deserialize, Serialize};
use std::{
    fmt,
    hash::{Hash, Hasher},
};
use sway_error::handler::{ErrorEmitted, Handler};
use sway_types::{Ident, Named, Span, Spanned};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TyConstantDecl {
    pub call_path: CallPath,
    pub value: Option<TyExpression>,
    pub visibility: Visibility,
    pub attributes: transform::Attributes,
    pub return_type: TypeId,
    pub type_ascription: GenericTypeArgument,
    pub span: Span,
}

impl TyDeclParsedType for TyConstantDecl {
    type ParsedType = ConstantDeclaration;
}

impl DebugWithEngines for TyConstantDecl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, _engines: &Engines) -> fmt::Result {
        write!(f, "{}", self.call_path)
    }
}

impl EqWithEngines for TyConstantDecl {}
impl PartialEqWithEngines for TyConstantDecl {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        let type_engine = ctx.engines().te();
        self.call_path == other.call_path
            && self.value.eq(&other.value, ctx)
            && self.visibility == other.visibility
            && self.type_ascription.eq(&other.type_ascription, ctx)
            && type_engine
                .get(self.return_type)
                .eq(&type_engine.get(other.return_type), ctx)
    }
}

impl HashWithEngines for TyConstantDecl {
    fn hash<H: Hasher>(&self, state: &mut H, engines: &Engines) {
        let type_engine = engines.te();
        let TyConstantDecl {
            call_path,
            value,
            visibility,
            return_type,
            type_ascription,
            // these fields are not hashed because they aren't relevant/a
            // reliable source of obj v. obj distinction
            attributes: _,
            span: _,
        } = self;
        call_path.hash(state);
        value.hash(state, engines);
        visibility.hash(state);
        type_engine.get(*return_type).hash(state, engines);
        type_ascription.hash(state, engines);
    }
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

impl IsConcrete for TyConstantDecl {
    fn is_concrete(&self, engines: &Engines) -> bool {
        self.return_type
            .is_concrete(engines, TreatNumericAs::Concrete)
    }
}

impl SubstTypes for TyConstantDecl {
    fn subst_inner(&mut self, ctx: &SubstTypesContext) -> HasChanges {
        has_changes! {
            self.return_type.subst(ctx);
            self.type_ascription.subst(ctx);
            self.value.subst(ctx);
        }
    }
}

impl ReplaceDecls for TyConstantDecl {
    fn replace_decls_inner(
        &mut self,
        decl_mapping: &DeclMapping,
        handler: &Handler,
        ctx: &mut TypeCheckContext,
    ) -> Result<bool, ErrorEmitted> {
        if let Some(expr) = &mut self.value {
            expr.replace_decls(decl_mapping, handler, ctx)
        } else {
            Ok(false)
        }
    }
}

impl MaterializeConstGenerics for TyConstantDecl {
    fn materialize_const_generics(
        &mut self,
        engines: &Engines,
        handler: &Handler,
        name: &str,
        value: &TyExpression,
    ) -> Result<(), ErrorEmitted> {
        if let Some(v) = self.value.as_mut() {
            v.materialize_const_generics(engines, handler, name, value)?;
        }
        self.return_type
            .materialize_const_generics(engines, handler, name, value)?;
        self.type_ascription
            .type_id
            .materialize_const_generics(engines, handler, name, value)?;
        Ok(())
    }
}
