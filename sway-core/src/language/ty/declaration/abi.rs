use crate::{engine_threading::*, language::parsed, transform, type_system::*};
use std::hash::{Hash, Hasher};

use sway_types::{Ident, Named, Span, Spanned};

use super::{TyTraitInterfaceItem, TyTraitItem};

/// A [TyAbiDecl] contains the type-checked version of the parse tree's
/// `AbiDeclaration`.
#[derive(Clone, Debug)]
pub struct TyAbiDecl {
    /// The name of the abi trait (also known as a "contract trait")
    pub name: Ident,
    /// The methods a contract is required to implement in order opt in to this interface
    pub interface_surface: Vec<TyTraitInterfaceItem>,
    pub supertraits: Vec<parsed::Supertrait>,
    pub items: Vec<TyTraitItem>,
    pub span: Span,
    pub attributes: transform::AttributesMap,
}

impl EqWithEngines for TyAbiDecl {}
impl PartialEqWithEngines for TyAbiDecl {
    fn eq(&self, other: &Self, engines: &Engines) -> bool {
        let TyAbiDecl {
            name: ln,
            interface_surface: lis,
            supertraits: ls,
            items: li,
            // these fields are not compared because they aren't relevant/a
            // reliable source of obj v. obj distinction
            attributes: _,
            span: _,
        } = self;
        let TyAbiDecl {
            name: rn,
            interface_surface: ris,
            supertraits: rs,
            items: ri,
            // these fields are not compared because they aren't relevant/a
            // reliable source of obj v. obj distinction
            attributes: _,
            span: _,
        } = other;
        ln == rn && lis.eq(ris, engines) && li.eq(ri, engines) && ls.eq(rs, engines)
    }
}

impl HashWithEngines for TyAbiDecl {
    fn hash<H: Hasher>(&self, state: &mut H, engines: &Engines) {
        let TyAbiDecl {
            name,
            interface_surface,
            items,
            supertraits,
            // these fields are not hashed because they aren't relevant/a
            // reliable source of obj v. obj distinction
            attributes: _,
            span: _,
        } = self;
        name.hash(state);
        interface_surface.hash(state, engines);
        items.hash(state, engines);
        supertraits.hash(state, engines);
    }
}

impl CreateTypeId for TyAbiDecl {
    fn create_type_id(&self, engines: &Engines) -> TypeId {
        let type_engine = engines.te();
        let ty = TypeInfo::ContractCaller {
            abi_name: AbiName::Known(self.name.clone().into()),
            address: None,
        };
        type_engine.insert(engines, ty)
    }
}

impl Spanned for TyAbiDecl {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl Named for TyAbiDecl {
    fn name(&self) -> &Ident {
        &self.name
    }
}
