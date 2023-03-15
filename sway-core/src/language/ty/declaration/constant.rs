use std::hash::{Hash, Hasher};

use sway_types::{Ident, Named, Span, Spanned};

use crate::{
    decl_engine::{DeclMapping, ReplaceDecls},
    engine_threading::*,
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

impl EqWithEngines for TyConstantDecl {}
impl PartialEqWithEngines for TyConstantDecl {
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
        let type_engine = engines.te();
        self.call_path == other.call_path
            && self.value.eq(&other.value, engines)
            && self.visibility == other.visibility
            && self.type_ascription.eq(&other.type_ascription, engines)
            && self.is_configurable == other.is_configurable
            && type_engine
                .get(self.return_type)
                .eq(&type_engine.get(other.return_type), engines)
            && match (&self.implementing_type, &other.implementing_type) {
                (Some(self_), Some(other)) => self_.eq(other, engines),
                _ => false,
            }
    }
}

impl HashWithEngines for TyConstantDecl {
    fn hash<H: Hasher>(&self, state: &mut H, engines: Engines<'_>) {
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
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: Engines<'_>) {
        self.return_type.subst(type_mapping, engines);
        self.type_ascription.subst(type_mapping, engines);
        if let Some(expr) = &mut self.value {
            expr.subst(type_mapping, engines);
        }
    }
}

impl ReplaceDecls for TyConstantDecl {
    fn replace_decls_inner(&mut self, decl_mapping: &DeclMapping, engines: Engines<'_>) {
        if let Some(expr) = &mut self.value {
            expr.replace_decls(decl_mapping, engines);
        }
    }
}
