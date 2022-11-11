use derivative::Derivative;
use sway_types::{Ident, Span};

use crate::{declaration_engine::DeclarationId, transform, type_system::*};

/// A [TyAbiDeclaration] contains the type-checked version of the parse tree's `AbiDeclaration`.
#[derive(Clone, Debug, Derivative)]
#[derivative(PartialEq, Eq)]
pub struct TyAbiDeclaration {
    /// The name of the abi trait (also known as a "contract trait")
    pub name: Ident,
    /// The methods a contract is required to implement in order opt in to this interface
    pub interface_surface: Vec<DeclarationId>,
    pub methods: Vec<DeclarationId>,
    #[derivative(PartialEq = "ignore")]
    #[derivative(Eq(bound = ""))]
    pub span: Span,
    pub attributes: transform::AttributesMap,
}

impl CreateTypeId for TyAbiDeclaration {
    fn create_type_id(&self, type_engine: &TypeEngine) -> TypeId {
        let ty = TypeInfo::ContractCaller {
            abi_name: AbiName::Known(self.name.clone().into()),
            address: None,
        };
        type_engine.insert_type(ty)
    }
}
