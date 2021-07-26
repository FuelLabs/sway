use super::TraitFn;
use super::{FunctionDeclaration, FunctionParameter, Visibility};
use crate::parser::Rule;
use crate::{error::*, Ident};
use pest::iterators::Pair;
use pest::Span;

/// An `abi` declaration, which declares an interface for a contract
/// to implement or for a caller to use to call a contract.
#[derive(Debug, Clone)]
pub struct AbiDeclaration<'sc> {
    /// If the abi declaration is `Visibility::Public`, then other contracts, scripts, etc can
    /// import this type to call it.
    pub(crate) visibility: Visibility,
    /// The name of the abi trait (also known as a "contract trait")
    pub(crate) trait_name: Ident<'sc>,
    /// The methods a contract is required to implement in order opt in to this interface
    pub(crate) interface_surface: Vec<TraitFn<'sc>>,
    /// The methods provided to a contract "for free" upon opting in to this interface
    pub(crate) methods: Vec<FunctionDeclaration<'sc>>,
}

impl<'sc> AbiDeclaration<'sc> {
    pub(crate) fn parse_from_pair(pair: Pair<'sc, Rule>) -> CompileResult<'sc, Self> {
        todo!()
    }
}
