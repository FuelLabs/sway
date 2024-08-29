use std::hash::{Hash, Hasher};

use sway_types::{Ident, Named, Span, Spanned};

use crate::{
    engine_threading::*,
    language::{parsed::TypeAliasDeclaration, CallPath, Visibility},
    transform,
    type_system::*,
};

use super::TyDeclParsedType;

#[derive(Clone, Debug)]
pub struct TyTypeAliasDecl {
    pub name: Ident,
    pub call_path: CallPath,
    pub attributes: transform::AttributesMap,
    pub ty: TypeArgument,
    pub visibility: Visibility,
    pub span: Span,
}

impl TyDeclParsedType for TyTypeAliasDecl {
    type ParsedType = TypeAliasDeclaration;
}

impl Named for TyTypeAliasDecl {
    fn name(&self) -> &Ident {
        &self.name
    }
}

impl EqWithEngines for TyTypeAliasDecl {}
impl PartialEqWithEngines for TyTypeAliasDecl {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        self.name == other.name && self.ty.eq(&other.ty, ctx) && self.visibility == other.visibility
    }
}

impl HashWithEngines for TyTypeAliasDecl {
    fn hash<H: Hasher>(&self, state: &mut H, engines: &Engines) {
        let TyTypeAliasDecl {
            name,
            ty,
            visibility,
            // these fields are not hashed because they aren't relevant/a
            // reliable source of obj v. obj distinction
            call_path: _,
            span: _,
            attributes: _,
        } = self;
        name.hash(state);
        ty.hash(state, engines);
        visibility.hash(state);
    }
}

impl SubstTypes for TyTypeAliasDecl {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, ctx: &SubstTypesContext) -> HasChanges {
        self.ty.subst(type_mapping, ctx)
    }
}

impl CreateTypeId for TyTypeAliasDecl {
    fn create_type_id(&self, engines: &Engines) -> TypeId {
        let type_engine = engines.te();
        type_engine.insert(
            engines,
            TypeInfo::Alias {
                name: self.name.clone(),
                ty: self.ty.clone(),
            },
            self.name.span().source_id(),
        )
    }
}

impl Spanned for TyTypeAliasDecl {
    fn span(&self) -> Span {
        self.span.clone()
    }
}
