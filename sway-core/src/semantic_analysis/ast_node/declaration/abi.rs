use derivative::Derivative;
use sway_types::{Ident, Span};

use crate::{
    error::{err, ok},
    semantic_analysis::{Mode, TCOpts, TypeCheckArguments},
    type_engine::{insert_type, AbiName, TypeId},
    AbiDeclaration, CompileResult, FunctionDeclaration, Namespace, TypeInfo,
    TypedFunctionDeclaration,
};

use super::TypedTraitFn;

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

impl TypedAbiDeclaration {
    pub(crate) fn type_check(
        abi_decl: AbiDeclaration,
        namespace: &mut Namespace,
        self_type: TypeId,
    ) -> CompileResult<TypedAbiDeclaration> {
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
        let self_type_for_interface = insert_type(TypeInfo::SelfType);
        let mut new_interface_surface = vec![];
        for trait_fn in interface_surface.into_iter() {
            new_interface_surface.push(check!(
                TypedTraitFn::type_check(trait_fn, namespace, self_type_for_interface),
                continue,
                warnings,
                errors
            ));
        }

        // type check these for errors but don't actually use them yet -- the real
        // ones will be type checked with proper symbols when the ABI is implemented
        for method in methods.iter() {
            let opts = TCOpts {
                purity: method.purity,
            };
            check!(
                TypedFunctionDeclaration::type_check(TypeCheckArguments {
                    checkee: method.clone(),
                    namespace,
                    return_type_annotation: insert_type(TypeInfo::Unknown),
                    help_text: Default::default(),
                    self_type,
                    mode: Mode::NonAbi,
                    opts
                }),
                return err(warnings, errors),
                warnings,
                errors
            );
        }

        let decl = TypedAbiDeclaration {
            interface_surface: new_interface_surface,
            methods,
            name,
            span,
        };
        ok(decl, warnings, errors)
    }

    pub(crate) fn as_type(&self) -> TypeId {
        let ty = TypeInfo::ContractCaller {
            abi_name: AbiName::Known(self.name.clone().into()),
            address: None,
        };
        insert_type(ty)
    }
}
