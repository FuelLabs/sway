use std::hash::{Hash, Hasher};

use sway_types::{Ident, Named, Span, Spanned};

use crate::{engine_threading::*, language::Visibility, transform, type_system::*};

#[derive(Clone, Debug)]
pub struct TyTypeAliasDeclaration {
    pub name: Ident,
    pub attributes: transform::AttributesMap,
    pub ty: TypeArgument,
    pub visibility: Visibility,
    pub span: Span,
}

impl Named for TyTypeAliasDeclaration {
    fn name(&self) -> &Ident {
        &self.name
    }
}

impl EqWithEngines for TyTypeAliasDeclaration {}
impl PartialEqWithEngines for TyTypeAliasDeclaration {
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
        self.name == other.name
            && self.ty.eq(&other.ty, engines)
            && self.visibility == other.visibility
    }
}

impl HashWithEngines for TyTypeAliasDeclaration {
    fn hash<H: Hasher>(&self, state: &mut H, engines: Engines<'_>) {
        let TyTypeAliasDeclaration {
            name,
            ty,
            visibility,
            // these fields are not hashed because they aren't relevant/a
            // reliable source of obj v. obj distinction
            span: _,
            attributes: _,
        } = self;
        name.hash(state);
        ty.hash(state, engines);
        visibility.hash(state);
    }
}

impl SubstTypes for TyTypeAliasDeclaration {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: Engines<'_>) {
        self.ty.subst(type_mapping, engines);
    }
}

impl ReplaceSelfType for TyTypeAliasDeclaration {
    fn replace_self_type(&mut self, engines: Engines<'_>, self_type: TypeId) {
        self.ty.replace_self_type(engines, self_type);
    }
}

impl CreateTypeId for TyTypeAliasDeclaration {
    fn create_type_id(&self, engines: Engines<'_>) -> TypeId {
        let type_engine = engines.te();
        let decl_engine = engines.de();
        type_engine.insert(
            decl_engine,
            TypeInfo::Alias {
                name: self.name.clone(),
                ty: self.ty.clone(),
            },
        )
    }
}

impl Spanned for TyTypeAliasDeclaration {
    fn span(&self) -> Span {
        self.span.clone()
    }
}
