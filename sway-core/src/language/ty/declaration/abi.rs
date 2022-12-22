use sway_types::{Ident, Span};

use crate::{declaration_engine::DeclarationId, engine_threading::*, transform, type_system::*};

/// A [TyAbiDeclaration] contains the type-checked version of the parse tree's `AbiDeclaration`.
#[derive(Clone, Debug)]
pub struct TyAbiDeclaration {
    /// The name of the abi trait (also known as a "contract trait")
    pub name: Ident,
    /// The methods a contract is required to implement in order opt in to this interface
    pub interface_surface: Vec<DeclarationId>,
    pub methods: Vec<DeclarationId>,
    pub span: Span,
    pub attributes: transform::AttributesMap,
}

impl EqWithEngines for TyAbiDeclaration {}
impl PartialEqWithEngines for TyAbiDeclaration {
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
        self.name == other.name
        && self.interface_surface.eq(&other.interface_surface, engines)
        && self.methods.eq(&other.methods, engines)
        // span ignored
        && self.attributes == other.attributes
    }
}

impl CreateTypeId for TyAbiDeclaration {
    fn create_type_id(&self, engines: Engines<'_>) -> TypeId {
        let type_engine = engines.te();
        let declaration_engine = engines.de();
        let ty = TypeInfo::ContractCaller {
            abi_name: AbiName::Known(self.name.clone().into()),
            address: None,
        };
        type_engine.insert_type(declaration_engine, ty)
    }
}
