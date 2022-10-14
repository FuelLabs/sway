use crate::AttributesMap;

use super::{FunctionDeclaration, TraitFn};

use sway_types::{ident::Ident, span::Span};

/// An `abi` declaration, which declares an interface for a contract
/// to implement or for a caller to use to call a contract.
#[derive(Debug, Clone)]
pub struct AbiDeclaration {
    /// The name of the abi trait (also known as a "contract trait")
    pub name: Ident,
    /// The methods a contract is required to implement in order opt in to this interface
    pub interface_surface: Vec<TraitFn>,
    /// The methods provided to a contract "for free" upon opting in to this interface
    pub methods: Vec<FunctionDeclaration>,
    pub(crate) span: Span,
    pub attributes: AttributesMap,
}
