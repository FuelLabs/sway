use crate::{decl_engine::parsed_id::ParsedDeclId, transform};

use super::{FunctionDeclaration, Supertrait, TraitItem};

use sway_types::{ident::Ident, span::Span};

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
