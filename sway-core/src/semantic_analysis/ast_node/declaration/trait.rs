use derivative::Derivative;
use sway_types::{Ident, Span, Spanned};

use crate::{
    error::{err, ok},
    semantic_analysis::{
        ast_node::{handle_supertraits, type_check_trait_methods},
        Mode, TypedCodeBlock,
    },
    style::is_upper_camel_case,
    type_engine::{insert_type, CopyTypes, TypeId, TypeMapping},
    CallPath, CompileResult, FunctionDeclaration, Namespace, Purity, Supertrait, TraitDeclaration,
    TraitFn, TypeInfo, TypedFunctionDeclaration, Visibility,
};

use super::{EnforceTypeArguments, TypedFunctionParameter};

#[derive(Clone, Debug, Derivative)]
#[derivative(PartialEq, Eq)]
pub struct TypedTraitDeclaration {
    pub name: Ident,
    pub interface_surface: Vec<TypedTraitFn>,
    // NOTE: deriving partialeq and hash on this element may be important in the
    // future, but I am not sure. For now, adding this would 2x the amount of
    // work, so I am just going to exclude it
    #[derivative(PartialEq = "ignore")]
    #[derivative(Eq(bound = ""))]
    pub(crate) methods: Vec<FunctionDeclaration>,
    pub(crate) supertraits: Vec<Supertrait>,
    pub(crate) visibility: Visibility,
}

impl CopyTypes for TypedTraitDeclaration {
    fn copy_types(&mut self, type_mapping: &TypeMapping) {
        self.interface_surface
            .iter_mut()
            .for_each(|x| x.copy_types(type_mapping));
        // we don't have to type check the methods because it hasn't been type checked yet
    }
}

impl TypedTraitDeclaration {
    pub(crate) fn type_check(
        trait_decl: TraitDeclaration,
        namespace: &mut Namespace,
    ) -> CompileResult<TypedTraitDeclaration> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        is_upper_camel_case(&trait_decl.name).ok(&mut warnings, &mut errors);

        // A temporary namespace for checking within the trait's scope.
        let mut namespace = namespace.clone();

        // type check the interface surface
        let self_type_for_interface = insert_type(TypeInfo::SelfType);
        let mut new_interface_surface = vec![];
        for trait_fn in trait_decl.interface_surface.into_iter() {
            new_interface_surface.push(check!(
                TypedTraitFn::type_check(trait_fn, &mut namespace, self_type_for_interface),
                continue,
                warnings,
                errors
            ));
        }

        // Recursively handle supertraits: make their interfaces and methods available to this trait
        check!(
            handle_supertraits(&trait_decl.supertraits, &mut namespace),
            return err(warnings, errors),
            warnings,
            errors
        );

        // insert placeholder functions representing the interface surface
        // to allow methods to use those functions
        namespace.insert_trait_implementation(
            CallPath {
                prefixes: vec![],
                suffix: trait_decl.name.clone(),
                is_absolute: false,
            },
            TypeInfo::SelfType,
            new_interface_surface
                .iter()
                .map(|x| x.to_dummy_func(Mode::NonAbi))
                .collect(),
        );

        // check the methods for errors but throw them away and use vanilla [FunctionDeclaration]s
        let _methods = check!(
            type_check_trait_methods(
                trait_decl.methods.clone(),
                &mut namespace,
                insert_type(TypeInfo::SelfType),
            ),
            vec![],
            warnings,
            errors
        );

        let typed_trait_decl = TypedTraitDeclaration {
            name: trait_decl.name.clone(),
            interface_surface: new_interface_surface,
            methods: trait_decl.methods.to_vec(),
            supertraits: trait_decl.supertraits.to_vec(),
            visibility: trait_decl.visibility,
        };
        ok(typed_trait_decl, warnings, errors)
    }
}

#[derive(Clone, Debug, Derivative)]
#[derivative(PartialEq, Eq)]
pub struct TypedTraitFn {
    pub name: Ident,
    pub(crate) purity: Purity,
    pub(crate) parameters: Vec<TypedFunctionParameter>,
    pub return_type: TypeId,
    #[derivative(PartialEq = "ignore")]
    #[derivative(Eq(bound = ""))]
    pub(crate) return_type_span: Span,
}

impl CopyTypes for TypedTraitFn {
    fn copy_types(&mut self, type_mapping: &TypeMapping) {
        self.return_type
            .update_type(type_mapping, &self.return_type_span);
    }
}

impl TypedTraitFn {
    pub(crate) fn type_check(
        trait_fn: TraitFn,
        namespace: &mut Namespace,
        self_type: TypeId,
    ) -> CompileResult<TypedTraitFn> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let mut new_parameters = vec![];
        for parameter in trait_fn.parameters.into_iter() {
            new_parameters.push(check!(
                TypedFunctionParameter::type_check(
                    parameter,
                    namespace,
                    self_type,
                    EnforceTypeArguments::Yes
                ),
                continue,
                warnings,
                errors
            ));
        }
        let return_type = check!(
            namespace.resolve_type_with_self(
                trait_fn.return_type,
                self_type,
                &trait_fn.return_type_span,
                EnforceTypeArguments::Yes
            ),
            insert_type(TypeInfo::ErrorRecovery),
            warnings,
            errors,
        );
        let trait_fn = TypedTraitFn {
            name: trait_fn.name,
            purity: trait_fn.purity,
            return_type_span: trait_fn.return_type_span.clone(),
            parameters: new_parameters,
            return_type,
        };
        ok(trait_fn, warnings, errors)
    }

    /// This function is used in trait declarations to insert "placeholder" functions
    /// in the methods. This allows the methods to use functions declared in the
    /// interface surface.
    pub(crate) fn to_dummy_func(&self, mode: Mode) -> TypedFunctionDeclaration {
        TypedFunctionDeclaration {
            purity: self.purity,
            name: self.name.clone(),
            body: TypedCodeBlock { contents: vec![] },
            parameters: self.parameters.clone(),
            span: self.name.span(),
            return_type: self.return_type,
            return_type_span: self.return_type_span.clone(),
            visibility: Visibility::Public,
            type_parameters: vec![],
            is_contract_call: mode == Mode::ImplAbiFn,
        }
    }
}
