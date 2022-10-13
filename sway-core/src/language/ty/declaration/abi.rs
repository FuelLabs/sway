use derivative::Derivative;
use sway_types::{Ident, Span};

use crate::{
    language::{parsed, ty::*},
    type_system::*,
};

/// A [TyAbiDeclaration] contains the type-checked version of the parse tree's `AbiDeclaration`.
#[derive(Clone, Debug, Derivative)]
#[derivative(PartialEq, Eq)]
pub struct TyAbiDeclaration {
    /// The name of the abi trait (also known as a "contract trait")
    pub name: Ident,
    /// The methods a contract is required to implement in order opt in to this interface
    pub interface_surface: Vec<TyTraitFn>,
    /// The methods provided to a contract "for free" upon opting in to this interface
    // NOTE: It may be important in the future to include this component
    #[derivative(PartialEq = "ignore")]
    #[derivative(Eq(bound = ""))]
    pub methods: Vec<parsed::FunctionDeclaration>,
    #[derivative(PartialEq = "ignore")]
    #[derivative(Eq(bound = ""))]
    pub(crate) span: Span,
}

impl CreateTypeId for TyAbiDeclaration {
    fn create_type_id(&self) -> TypeId {
        let ty = TypeInfo::ContractCaller {
            abi_name: AbiName::Known(self.name.clone().into()),
            address: None,
        };
        insert_type(ty)
    }
}
