use std::collections::BTreeMap;

use sway_error::warning::{CompileWarning, Warning};
use sway_types::{style::is_upper_camel_case, Ident, Spanned};

use crate::{
    declaration_engine::*,
    error::*,
    language::{parsed::*, ty, CallPath},
    semantic_analysis::{declaration::insert_supertraits_into_namespace, Mode, TypeCheckContext},
    type_system::*,
};

type MethodMap = BTreeMap<Ident, DeclarationId>;

impl ty::TyTraitDeclaration {
    pub(crate) fn type_check(
        ctx: TypeCheckContext,
        trait_decl: TraitDeclaration,
    ) -> CompileResult<Self> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        let TraitDeclaration {
            name,
            type_parameters,
            attributes,
            interface_surface,
            methods,
            supertraits,
            visibility,
            span,
        } = trait_decl;

        if !is_upper_camel_case(name.as_str()) {
            warnings.push(CompileWarning {
                span: name.span(),
                warning_content: Warning::NonClassCaseTraitName { name: name.clone() },
            })
        }

        // A temporary namespace for checking within the trait's scope.
        let self_type = insert_type(TypeInfo::SelfType);
        let mut trait_namespace = ctx.namespace.clone();
        let mut ctx = ctx.scoped(&mut trait_namespace).with_self_type(self_type);

        // type check the type parameters, which will insert them into the namespace
        let mut new_type_parameters = vec![];
        for type_parameter in type_parameters.into_iter() {
            new_type_parameters.push(check!(
                TypeParameter::type_check(ctx.by_ref(), type_parameter),
                return err(warnings, errors),
                warnings,
                errors
            ));
        }

        // Recursively make the interface surfaces and methods of the
        // supertraits available to this trait.
        check!(
            insert_supertraits_into_namespace(ctx.by_ref(), self_type, &supertraits),
            return err(warnings, errors),
            warnings,
            errors
        );

        // type check the interface surface
        let mut new_interface_surface = vec![];
        let mut dummy_interface_surface = vec![];
        for method in interface_surface.into_iter() {
            let method = check!(
                ty::TyTraitFn::type_check(ctx.by_ref(), method),
                return err(warnings, errors),
                warnings,
                errors
            );
            let decl_id = de_insert_trait_fn(method.clone());
            new_interface_surface.push(decl_id.clone());
            dummy_interface_surface
                .push(de_insert_function(method.to_dummy_func(Mode::NonAbi)).with_parent(decl_id));
        }

        // insert placeholder functions representing the interface surface
        // to allow methods to use those functions
        check!(
            ctx.namespace.insert_trait_implementation(
                CallPath {
                    prefixes: vec![],
                    suffix: name.clone(),
                    is_absolute: false,
                },
                new_type_parameters.iter().map(|x| x.into()).collect(),
                self_type,
                &dummy_interface_surface,
                &span,
                false
            ),
            return err(warnings, errors),
            warnings,
            errors
        );

        // type check the methods
        let mut new_methods = vec![];
        for method in methods.into_iter() {
            let method = check!(
                ty::TyFunctionDeclaration::type_check(ctx.by_ref(), method.clone(), true, false),
                ty::TyFunctionDeclaration::error(method),
                warnings,
                errors
            );
            new_methods.push(de_insert_function(method));
        }

        let typed_trait_decl = ty::TyTraitDeclaration {
            name,
            type_parameters: new_type_parameters,
            interface_surface: new_interface_surface,
            methods: new_methods,
            supertraits,
            visibility,
            attributes,
            span,
        };
        ok(typed_trait_decl, warnings, errors)
    }

    /// Retrieves the interface surface and implemented methods for this trait.
    pub(crate) fn retrieve_interface_surface_and_implemented_methods_for_type(
        &self,
        ctx: TypeCheckContext,
        type_id: TypeId,
        call_path: &CallPath,
    ) -> CompileResult<(MethodMap, MethodMap)> {
        let mut warnings = vec![];
        let mut errors = vec![];

        let mut interface_surface_method_ids: MethodMap = BTreeMap::new();
        let mut impld_method_ids: MethodMap = BTreeMap::new();

        let ty::TyTraitDeclaration {
            interface_surface,
            name,
            ..
        } = self;

        // Retrieve the interface surface for this trait.
        for decl_id in interface_surface.iter() {
            let method = check!(
                CompileResult::from(de_get_trait_fn(decl_id.clone(), &call_path.span())),
                return err(warnings, errors),
                warnings,
                errors
            );
            interface_surface_method_ids.insert(method.name, decl_id.clone());
        }

        // Retrieve the implemented methods for this type.
        for decl_id in ctx
            .namespace
            .get_methods_for_type_and_trait_name(type_id, call_path)
            .into_iter()
        {
            let method = check!(
                CompileResult::from(de_get_function(decl_id.clone(), &name.span())),
                return err(warnings, errors),
                warnings,
                errors
            );
            impld_method_ids.insert(method.name, decl_id);
        }

        ok(
            (interface_surface_method_ids, impld_method_ids),
            warnings,
            errors,
        )
    }

    /// Retrieves the interface surface, methods, and implemented methods for
    /// this trait.
    pub(crate) fn retrieve_interface_surface_and_methods_and_implemented_methods_for_type(
        &self,
        ctx: TypeCheckContext,
        type_id: TypeId,
        call_path: &CallPath,
        type_arguments: &[TypeArgument],
    ) -> CompileResult<(MethodMap, MethodMap, MethodMap)> {
        let mut warnings = vec![];
        let mut errors = vec![];

        let mut interface_surface_method_ids: MethodMap = BTreeMap::new();
        let mut method_ids: MethodMap = BTreeMap::new();
        let mut impld_method_ids: MethodMap = BTreeMap::new();

        let ty::TyTraitDeclaration {
            interface_surface,
            methods,
            type_parameters,
            ..
        } = self;

        // Retrieve the interface surface for this trait.
        for decl_id in interface_surface.iter() {
            let method = check!(
                CompileResult::from(de_get_trait_fn(decl_id.clone(), &call_path.span())),
                return err(warnings, errors),
                warnings,
                errors
            );
            interface_surface_method_ids.insert(method.name, decl_id.clone());
        }

        // Retrieve the trait methods for this trait.
        for decl_id in methods.iter() {
            let method = check!(
                CompileResult::from(de_get_function(decl_id.clone(), &call_path.span())),
                return err(warnings, errors),
                warnings,
                errors
            );
            method_ids.insert(method.name, decl_id.clone());
        }

        // Retrieve the implemented methods for this type.
        let type_mapping = TypeMapping::from_type_parameters_and_type_arguments(
            type_parameters
                .iter()
                .map(|type_param| type_param.type_id)
                .collect(),
            type_arguments
                .iter()
                .map(|type_arg| type_arg.type_id)
                .collect(),
        );
        for decl_id in ctx
            .namespace
            .get_methods_for_type_and_trait_name(type_id, call_path)
            .into_iter()
        {
            let mut method = check!(
                CompileResult::from(de_get_function(decl_id.clone(), &call_path.span())),
                return err(warnings, errors),
                warnings,
                errors
            );
            method.copy_types(&type_mapping);
            impld_method_ids.insert(
                method.name.clone(),
                de_insert_function(method).with_parent(decl_id),
            );
        }

        ok(
            (interface_surface_method_ids, method_ids, impld_method_ids),
            warnings,
            errors,
        )
    }

    pub(crate) fn insert_interface_surface_and_methods_into_namespace(
        &self,
        ctx: TypeCheckContext,
        trait_name: &CallPath,
        type_arguments: &[TypeArgument],
        type_id: TypeId,
    ) -> CompileResult<()> {
        let mut warnings = vec![];
        let mut errors = vec![];

        let ty::TyTraitDeclaration {
            interface_surface,
            methods,
            type_parameters,
            ..
        } = self;

        let mut all_methods = vec![];

        // Retrieve the trait methods for this trait. Transform them into the
        // correct typing for this impl block by using the type parameters from
        // the original trait declaration and the given type arguments.
        let type_mapping = TypeMapping::from_type_parameters_and_type_arguments(
            type_parameters
                .iter()
                .map(|type_param| type_param.type_id)
                .collect(),
            type_arguments
                .iter()
                .map(|type_arg| type_arg.type_id)
                .collect(),
        );
        for decl_id in interface_surface.iter() {
            let mut method = check!(
                CompileResult::from(de_get_trait_fn(decl_id.clone(), &trait_name.span())),
                continue,
                warnings,
                errors
            );
            method.replace_self_type(type_id);
            method.copy_types(&type_mapping);
            all_methods.push(
                de_insert_function(method.to_dummy_func(Mode::NonAbi)).with_parent(decl_id.clone()),
            );
        }
        for decl_id in methods.iter() {
            let mut method = check!(
                CompileResult::from(de_get_function(decl_id.clone(), &trait_name.span())),
                continue,
                warnings,
                errors
            );
            method.replace_self_type(type_id);
            method.copy_types(&type_mapping);
            all_methods.push(de_insert_function(method).with_parent(decl_id.clone()));
        }

        // Insert the methods of the trait into the namespace.
        // Specifically do not check for conflicting definitions because
        // this is just a temporary namespace for type checking and
        // these are not actual impl blocks.
        ctx.namespace.insert_trait_implementation(
            trait_name.clone(),
            type_arguments.to_vec(),
            type_id,
            &all_methods,
            &trait_name.span(),
            false,
        );

        if errors.is_empty() {
            ok((), warnings, errors)
        } else {
            err(warnings, errors)
        }
    }
}
