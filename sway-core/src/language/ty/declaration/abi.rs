use crate::{decl_engine::DeclRef, engine_threading::*, transform, type_system::*};
use std::hash::{Hash, Hasher};
use sway_types::{Ident, Span, Spanned};

/// A [TyAbiDeclaration] contains the type-checked version of the parse tree's `AbiDeclaration`.
#[derive(Clone, Debug)]
pub struct TyAbiDeclaration {
    /// The name of the abi trait (also known as a "contract trait")
    pub name: Ident,
    /// The methods a contract is required to implement in order opt in to this interface
    pub interface_surface: Vec<DeclRef>,
    pub methods: Vec<DeclRef>,
    pub span: Span,
    pub attributes: transform::AttributesMap,
}

impl EqWithEngines for TyAbiDeclaration {}
impl PartialEqWithEngines for TyAbiDeclaration {
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
        self.name == other.name
            && self.interface_surface.eq(&other.interface_surface, engines)
            && self.methods.eq(&other.methods, engines)
    }
}

impl HashWithEngines for TyAbiDeclaration {
    fn hash<H: Hasher>(&self, state: &mut H, engines: Engines<'_>) {
        let TyAbiDeclaration {
            name,
            interface_surface,
            methods,
            // these fields are not hashed because they aren't relevant/a
            // reliable source of obj v. obj distinction
            attributes: _,
            span: _,
        } = self;
        name.hash(state);
        interface_surface.hash(state, engines);
        methods.hash(state, engines);
    }
}

impl CreateTypeId for TyAbiDeclaration {
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

impl Spanned for TyAbiDeclaration {
    fn span(&self) -> Span {
        self.span.clone()
    }
}
