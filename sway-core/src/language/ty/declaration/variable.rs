use crate::{
    ast_elements::type_argument::GenericTypeArgument,
    engine_threading::*,
    language::{parsed::VariableDeclaration, ty::*},
    type_system::*,
};
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use sway_types::{Ident, Named, Spanned};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TyVariableDecl {
    pub name: Ident,
    pub body: TyExpression,
    pub mutability: VariableMutability,
    pub return_type: TypeId,
    pub type_ascription: GenericTypeArgument,
}

impl TyDeclParsedType for TyVariableDecl {
    type ParsedType = VariableDeclaration;
}

impl Named for TyVariableDecl {
    fn name(&self) -> &sway_types::BaseIdent {
        &self.name
    }
}

impl Spanned for TyVariableDecl {
    fn span(&self) -> sway_types::Span {
        self.name.span()
    }
}

impl EqWithEngines for TyVariableDecl {}
impl PartialEqWithEngines for TyVariableDecl {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        let type_engine = ctx.engines().te();
        self.name == other.name
            && self.body.eq(&other.body, ctx)
            && self.mutability == other.mutability
            && type_engine
                .get(self.return_type)
                .eq(&type_engine.get(other.return_type), ctx)
            && self.type_ascription.eq(&other.type_ascription, ctx)
    }
}

impl HashWithEngines for TyVariableDecl {
    fn hash<H: Hasher>(&self, state: &mut H, engines: &Engines) {
        let TyVariableDecl {
            name,
            body,
            mutability,
            return_type,
            type_ascription,
        } = self;
        let type_engine = engines.te();
        name.hash(state);
        body.hash(state, engines);
        type_engine.get(*return_type).hash(state, engines);
        type_ascription.hash(state, engines);
        mutability.hash(state);
    }
}

// impl SubstTypes for TyVariableDecl {
//     fn subst_inner(&mut self, ctx: &SubstTypesContext) -> HasChanges {
//         self.return_type.subst(ctx);
//         self.type_ascription.subst(ctx);
//         self.body.subst(ctx)
//     }
// }
