use crate::{engine_threading::*, language::parsed, transform, type_system::*};

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

impl CreateTypeId for TyAbiDecl {
    fn create_type_id(&self, engines: Engines<'_>) -> TypeId {
        let type_engine = engines.te();
        let decl_engine = engines.de();
        let ty = TypeInfo::ContractCaller {
            abi_name: AbiName::Known(self.name.clone().into()),
            address: None,
        };
        type_engine.insert(decl_engine, ty)
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
