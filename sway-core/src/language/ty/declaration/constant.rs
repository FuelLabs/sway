use std::{
    fmt,
    hash::{Hash, Hasher},
};

use sway_error::handler::{ErrorEmitted, Handler};
use sway_types::{Ident, Named, Span, Spanned};

use crate::{
    decl_engine::{DeclMapping, ReplaceDecls},
    engine_threading::*,
    has_changes,
    language::{ty::*, CallPath, Visibility},
    semantic_analysis::TypeCheckContext,
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
            && self.is_configurable == other.is_configurable
            && type_engine
                .get(self.return_type)
                .eq(&type_engine.get(other.return_type), ctx)
            && match (&self.implementing_type, &other.implementing_type) {
                (Some(self_), Some(other)) => self_.eq(other, ctx),
                _ => false,
            }
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
            is_configurable,
            implementing_type,
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
        is_configurable.hash(state);
        if let Some(implementing_type) = implementing_type {
            (*implementing_type).hash(state, engines);
        }
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

impl SubstTypes for TyConstantDecl {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: &Engines) -> HasChanges {
        has_changes! {
            self.return_type.subst(type_mapping, engines);
            self.type_ascription.subst(type_mapping, engines);
            self.value.subst(type_mapping, engines);
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
