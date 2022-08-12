use derivative::Derivative;
use sway_types::{Ident, Span};

use crate::{
    error::{err, ok},
    semantic_analysis::{
        ast_node::{type_check_interface_surface, type_check_trait_methods},
        TypeCheckContext,
    },
    type_system::{AbiName, TypeEngine, TypeId},
    AbiDeclaration, CompileResult, FunctionDeclaration, TypeInfo,
};

use super::{CreateTypeId, TypedTraitFn};

/// A `TypedAbiDeclaration` contains the type-checked version of the parse tree's `AbiDeclaration`.
#[derive(Clone, Debug, Derivative)]
#[derivative(PartialEq, Eq)]
pub struct TypedAbiDeclaration {
    /// The name of the abi trait (also known as a "contract trait")
    pub name: Ident,
    /// The methods a contract is required to implement in order opt in to this interface
    pub interface_surface: Vec<TypedTraitFn>,
    /// The methods provided to a contract "for free" upon opting in to this interface
    // NOTE: It may be important in the future to include this component
    #[derivative(PartialEq = "ignore")]
    #[derivative(Eq(bound = ""))]
    pub(crate) methods: Vec<FunctionDeclaration>,
    #[derivative(PartialEq = "ignore")]
    #[derivative(Eq(bound = ""))]
    pub(crate) span: Span,
}

impl CreateTypeId for TypedAbiDeclaration {
    fn create_type_id(&self, type_engine: &TypeEngine) -> TypeId {
        let ty = TypeInfo::ContractCaller {
            abi_name: AbiName::Known(self.name.clone().into()),
            address: None,
        };
        type_engine.insert_type(ty)
    }
}

impl TypedAbiDeclaration {
    pub(crate) fn type_check(
        ctx: TypeCheckContext,
        type_engine: &TypeEngine,
        abi_decl: AbiDeclaration,
    ) -> CompileResult<Self> {
        let mut warnings = vec![];
        let mut errors = vec![];

        let AbiDeclaration {
            name,
            interface_surface,
            methods,
            span,
        } = abi_decl;

        // type check the interface surface and methods
        // We don't want the user to waste resources by contract calling
        // themselves, and we don't want to do more work in the compiler,
        // so we don't support the case of calling a contract's own interface
        // from itself. This is by design.
        let interface_surface = check!(
            type_check_interface_surface(type_engine, interface_surface, ctx.namespace),
            return err(warnings, errors),
            warnings,
            errors
        );
        // type check these for errors but don't actually use them yet -- the real
        // ones will be type checked with proper symbols when the ABI is implemented
        let _methods = check!(
            type_check_trait_methods(ctx, type_engine, methods.clone()),
            vec![],
            warnings,
            errors
        );
        let abi_decl = TypedAbiDeclaration {
            interface_surface,
            methods,
            name,
            span,
        };
        ok(abi_decl, warnings, errors)
    }
}
