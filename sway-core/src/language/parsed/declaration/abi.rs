use crate::{
    decl_engine::parsed_id::ParsedDeclId,
    engine_threading::{EqWithEngines, PartialEqWithEngines, PartialEqWithEnginesContext},
    transform,
};

use super::{FunctionDeclaration, Supertrait, TraitItem};

use sway_types::{ident::Ident, span::Span, Named, Spanned};

/// An `abi` declaration, which declares an interface for a contract
/// to implement or for a caller to use to call a contract.
#[derive(Debug, Clone)]
pub struct AbiDeclaration {
    /// The name of the abi trait (also known as a "contract trait")
    pub name: Ident,
    /// The methods a contract is required to implement in order opt in to this interface
    pub interface_surface: Vec<TraitItem>,
    pub supertraits: Vec<Supertrait>,
    /// The methods provided to a contract "for free" upon opting in to this interface
    pub methods: Vec<ParsedDeclId<FunctionDeclaration>>,
    pub(crate) span: Span,
    pub attributes: transform::AttributesMap,
}

impl EqWithEngines for AbiDeclaration {}
impl PartialEqWithEngines for AbiDeclaration {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        self.name == other.name
            && self.interface_surface.eq(&other.interface_surface, ctx)
            && self.supertraits.eq(&other.supertraits, ctx)
            && PartialEqWithEngines::eq(&self.methods, &other.methods, ctx)
            && self.span == other.span
            && self.attributes == other.attributes
    }
}

impl Named for AbiDeclaration {
    fn name(&self) -> &sway_types::BaseIdent {
        &self.name
    }
}

impl Spanned for AbiDeclaration {
    fn span(&self) -> sway_types::Span {
        self.span.clone()
    }
}
