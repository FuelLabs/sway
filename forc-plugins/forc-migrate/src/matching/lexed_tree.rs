//! This module contains helper functions for matching elements within a mutable or immutable lexed tree.
//! Functions are grouped in two submodules, [self::matchers] and [self::matchers_mut]. Both modules
//! contain the same functions, that differ only in the mutability of their arguments and returned types.

use super::*;
use duplicate::duplicate_item;
use sway_ast::{
    attribute::Annotated, ItemImpl, ItemKind, ItemStorage, Module, StorageEntry, StorageField,
};
use sway_ast::{Literal, PathType};
use sway_core::language::{
    lexed::{LexedModule, LexedProgram},
    ty::TyImplSelfOrTrait,
};
use sway_types::Spanned;

// To avoid extensive code duplication, the `duplicate_item` macro is used.
// When adding new matchers, the proposed and simplest approach is the following:
//  - implement either a mutable or immutable version, as needed in the concrete new migration step.
//  - keep the new matcher function or trait implementation at first out of the `matchers/_mut` modules
//    and use it in the concrete migration step directly.
//  - once properly tested, move the function or trait implementation inside of the `__mod_name`
//    and perform the replacements of used identifiers. E.g., replace every occurrence of `iter` or
//    `iter_mut` with `__iter`.

// We need to specify `self` explicitly so that `duplicate_item` can produce
// both variants: `self: &'a Self` and `self: &'a mut Self`.
#[allow(clippy::needless_arbitrary_self_type)]
// We need to specify `'a` explicitly to be able to specify template implementation
// that will work both for immutable and mutable case.
#[allow(clippy::needless_lifetimes)]
#[duplicate_item(
    // Module name, `matchers` or `matchers_mut`.
    __mod_name
    // Traits to implement.
    __ElementsMatcher  __ElementsMatcherDeep  __LocateAnnotated   __LocateAsAnnotated
    // Trait methods.
    __match_elems      __match_elems_deep     __locate_annotated  __locate_as_annotated
    // Common implementation elements, e.g. functions like `iter().`
   __ref_type(type)    __ref_mut(value)    __ref(value)    __iter      __as_ref_mut  __any;

    [matchers]
    [LexedElementsMatcher] [LexedElementsMatcherDeep] [LexedLocateAnnotated] [LexedLocateAsAnnotated]
    [match_elems]          [match_elems_deep]         [locate_annotated]     [locate_as_annotated]
    [&'a type]        [value]             [&value]        [iter]      [as_ref]       [any];

    [matchers_mut]
    [LexedElementsMatcherMut] [LexedElementsMatcherDeepMut] [LexedLocateAnnotatedMut] [LexedLocateAsAnnotatedMut]
    [match_elems_mut]         [match_elems_deep_mut]        [locate_annotated_mut]    [locate_as_annotated_mut]
    [&'a mut type]    [ref mut value]     [&mut value]    [iter_mut]  [as_mut]       [any_mut];
)]
#[allow(dead_code)]
pub mod __mod_name {
    use super::*;

    impl __ElementsMatcher<ItemFn> for Module {
        fn __match_elems<'a, F>(
            self: __ref_type([Self]),
            predicate: F,
        ) -> impl Iterator<Item = __ref_type([ItemFn])>
        where
            F: Fn(&__ref_type([ItemFn])) -> bool + Clone + 'a,
            ItemFn: 'a,
        {
            self.items
                .__iter()
                .map(|annotated| __ref([annotated.value]))
                .filter_map(|decl| match decl {
                    sway_ast::ItemKind::Fn(module_fn) => Some(module_fn),
                    _ => None,
                })
                .filter(predicate)
        }
    }

    impl __ElementsMatcher<ItemStorage> for LexedProgram {
        fn __match_elems<'a, F>(
            self: __ref_type([Self]),
            predicate: F,
        ) -> impl Iterator<Item = __ref_type([ItemStorage])>
        where
            F: Fn(&__ref_type([ItemStorage])) -> bool + Clone + 'a,
            ItemStorage: 'a,
        {
            // Storage can be declared only in the root module of a contract.
            self.root.__match_elems(predicate)
        }
    }

    impl __ElementsMatcher<ItemStorage> for LexedModule {
        fn __match_elems<'a, F>(
            self: __ref_type([Self]),
            predicate: F,
        ) -> impl Iterator<Item = __ref_type([ItemStorage])>
        where
            F: Fn(&__ref_type([ItemStorage])) -> bool + Clone + 'a,
            ItemStorage: 'a,
        {
            self.tree
                .value
                .items
                .__iter()
                .map(|annotated_item| __ref([annotated_item.value]))
                .filter_map(move |decl| match decl {
                    ItemKind::Storage(__ref_mut([item_storage])) => {
                        if predicate(&item_storage) {
                            Some(item_storage)
                        } else {
                            None
                        }
                    }
                    _ => None,
                })
        }
    }

    impl __ElementsMatcher<StorageField> for ItemStorage {
        fn __match_elems<'a, F>(
            self: __ref_type([Self]),
            predicate: F,
        ) -> impl Iterator<Item = __ref_type([StorageField])>
        where
            F: Fn(&__ref_type([StorageField])) -> bool + Clone + 'a,
            StorageField: 'a,
        {
            self.entries
                .inner
                .__iter()
                .map(|annotated_item| __ref([annotated_item.value]))
                .filter_map(move |storage_entry| {
                    storage_entry
                        .field
                        .__as_ref_mut()
                        .filter(|sf| predicate(sf))
                })
        }
    }

    impl __ElementsMatcherDeep<StorageField> for ItemStorage {
        fn __match_elems_deep<'a, F>(
            self: __ref_type([Self]),
            predicate: F,
        ) -> Vec<__ref_type([StorageField])>
        where
            F: Fn(&__ref_type([StorageField])) -> bool + Clone + 'a,
            StorageField: 'a,
        {
            fn recursively_collect_storage_fields_in_storage_entry<'a, P>(
                result: &mut Vec<__ref_type([StorageField])>,
                predicate: P,
                storage_entry: __ref_type([StorageEntry]),
            ) where
                P: Fn(&__ref_type([StorageField])) -> bool + Clone + 'a,
            {
                if let Some(sf) = __ref([storage_entry.field]) {
                    if predicate(&sf) {
                        result.push(sf)
                    }
                }

                if let Some(namespace) = __ref([storage_entry.namespace]) {
                    namespace
                        .inner
                        .__iter()
                        .map(|annotated_item| __ref([annotated_item.value]))
                        .for_each(|storage_entry| {
                            recursively_collect_storage_fields_in_storage_entry(
                                result,
                                predicate.clone(),
                                storage_entry.__as_ref_mut(),
                            )
                        });
                }
            }

            let mut result = vec![];
            self.entries
                .inner
                .__iter()
                .map(|annotated_item| __ref([annotated_item.value]))
                .for_each(|storage_entry| {
                    recursively_collect_storage_fields_in_storage_entry(
                        &mut result,
                        predicate.clone(),
                        storage_entry,
                    )
                });

            result
        }
    }

    impl __ElementsMatcher<Annotated<ItemKind>> for Module {
        fn __match_elems<'a, F>(
            self: __ref_type([Self]),
            predicate: F,
        ) -> impl Iterator<Item = __ref_type([Annotated<ItemKind>])>
        where
            F: Fn(&__ref_type([Annotated<ItemKind>])) -> bool + Clone + 'a,
            Annotated<ItemKind>: 'a,
        {
            self.items.__iter().filter(predicate)
        }
    }

    impl __ElementsMatcher<PathType> for ItemImpl {
        fn __match_elems<'a, F>(
            self: __ref_type([Self]),
            predicate: F,
        ) -> impl Iterator<Item = __ref_type([PathType])>
        where
            F: Fn(&__ref_type([PathType])) -> bool + Clone + 'a,
            PathType: 'a,
        {
            self.where_clause_opt
                .__iter()
                .flat_map(|where_clause| where_clause.bounds.__iter())
                .flat_map(move |bound| bound.bounds.__iter().filter(predicate.clone()))
        }
    }

    impl __LocateAnnotated<TyImplSelfOrTrait, ItemImpl> for Module {
        fn __locate_annotated<'a>(
            self: __ref_type([Self]),
            ty_element: &TyImplSelfOrTrait,
        ) -> Option<(__ref_type([Vec<AttributeDecl>]), __ref_type([ItemImpl]))> {
            self.items
                .__iter()
                .filter_map(|annotated| match __ref([annotated.value]) {
                    ItemKind::Impl(item_impl) => Some((__ref([annotated.attributes]), item_impl)),
                    _ => None,
                })
                .find(|(_attributes, item_impl)| item_impl.span() == ty_element.span)
        }
    }

    impl __LocateAsAnnotated<TyImplSelfOrTrait, ItemKind> for Module {
        fn __locate_as_annotated<'a>(
            self: __ref_type([Self]),
            ty_element: &TyImplSelfOrTrait,
        ) -> Option<__ref_type([Annotated<ItemKind>])> {
            self.items
                .__iter()
                .find(|annotated| match &annotated.value {
                    ItemKind::Impl(item_impl) => item_impl.span() == ty_element.span,
                    _ => false,
                })
        }
    }

    use sway_ast::{
        attribute::{Attribute, AttributeArg},
        AttributeDecl, CommaToken, Parens, Punctuated,
    };

    pub(crate) fn storage_decl<'a, P>(parent: __ref_type([P])) -> Option<__ref_type([ItemStorage])>
    where
        P: __ElementsMatcher<ItemStorage>,
    {
        parent.__match_elems(__any).next()
    }

    pub(crate) fn storage_fields<'a, P, F>(
        parent: __ref_type([P]),
        predicate: F,
    ) -> impl Iterator<Item = __ref_type([StorageField])>
    where
        F: Fn(&__ref_type([StorageField])) -> bool + Clone + 'a,
        P: __ElementsMatcher<StorageField>,
    {
        parent.__match_elems(predicate)
    }

    pub(crate) fn storage_fields_deep<'a, S, F>(
        scope: __ref_type([S]),
        predicate: F,
    ) -> Vec<__ref_type([StorageField])>
    where
        F: Fn(&__ref_type([StorageField])) -> bool + Clone + 'a,
        S: __ElementsMatcherDeep<StorageField>,
    {
        scope.__match_elems_deep(predicate)
    }

    pub(crate) fn attributes<'a, F>(
        attributes: __ref_type([[AttributeDecl]]),
        predicate: F,
    ) -> impl Iterator<Item = __ref_type([Attribute])>
    where
        F: Fn(&__ref_type([Attribute])) -> bool + Clone + 'a,
    {
        attributes
            .__iter()
            .flat_map(|attr| attr.attribute.inner.__iter())
            .filter(predicate)
    }

    /// Returns all `cfg` attributes found in `attributes`.
    pub(crate) fn cfg_attributes<'a>(
        attributes: __ref_type([[AttributeDecl]]),
    ) -> impl Iterator<Item = __ref_type([Attribute])> {
        attributes
            .__iter()
            .flat_map(|attr| attr.attribute.inner.__iter())
            .filter(|attr| attr.is_cfg())
    }

    /// Returns all `cfg` attributes that act as only attribute within
    /// an [AttributeDecl] and have exactly one argument.
    ///
    /// E.g.:
    /// - `#[cfg(experimental_feature = true)]` will be returned,
    /// - `#[cfg(experimental_feature = true, experimental_other_feature = false)]` will not,
    /// - `#[test, cfg(experimental_feature = true)]` will also not be returned.
    pub(crate) fn cfg_attributes_standalone_single_arg<'a>(
        attributes: __ref_type([[AttributeDecl]]),
    ) -> impl Iterator<Item = __ref_type([Attribute])> {
        attributes
            .__iter()
            .filter(|attr| attr.attribute.inner.iter().count() == 1)
            .flat_map(|attr| attr.attribute.inner.__iter())
            .filter(|attr| attr.is_cfg())
            .filter(|attr| {
                attr.args
                    .as_ref()
                    .is_some_and(|args| args.inner.iter().count() == 1)
            })
    }

    /// Returns the first [AttributeArg] of the first occurrence of a `cfg` attribute within `attributes`,
    /// that satisfies the `predicate`.
    pub(crate) fn cfg_attribute_arg<'a, F>(
        attributes: __ref_type([[AttributeDecl]]),
        predicate: F,
    ) -> Option<__ref_type([AttributeArg])>
    where
        F: Fn(&__ref_type([AttributeArg])) -> bool + Clone + 'a,
    {
        for cfg_attribute in cfg_attributes(attributes) {
            match cfg_attribute.args.__as_ref_mut() {
                Some(args) => match attribute_arg(args, predicate.clone()) {
                    Some(arg) => return Some(arg),
                    None => continue,
                },
                None => continue,
            }
        }

        None
    }

    /// Returns the first `cfg` [Attribute] that act as only attribute within
    /// an [AttributeDecl] and have exactly one argument that satisfies the `predicate`.
    pub(crate) fn cfg_attribute_standalone_single_arg<'a, N, F>(
        attributes: __ref_type([[AttributeDecl]]),
        arg_name: &N,
        arg_val_predicate: F,
    ) -> Option<__ref_type([Attribute])>
    where
        N: AsRef<str> + ?Sized,
        F: Fn(&&Option<Literal>) -> bool + Clone,
    {
        for cfg_attribute in cfg_attributes_standalone_single_arg(attributes) {
            // We for sure have a `cfg` attribute with exactly one argument.
            // Thus, the unwraps are safe.
            let arg = cfg_attribute
                .args
                .as_ref()
                .unwrap()
                .inner
                .iter()
                .next()
                .unwrap();
            if arg.name.as_str() == arg_name.as_ref() && arg_val_predicate(&&arg.value) {
                return Some(cfg_attribute);
            }
        }

        None
    }

    /// Returns the first attribute in `attributes` that satisfies the `predicate`.
    pub(crate) fn attribute<'a, F>(
        attributes: __ref_type([[AttributeDecl]]),
        predicate: F,
    ) -> Option<__ref_type([Attribute])>
    where
        F: Fn(&__ref_type([Attribute])) -> bool + Clone + 'a,
    {
        attributes
            .__iter()
            .flat_map(|attr| attr.attribute.inner.__iter())
            .find(predicate)
    }

    pub(crate) fn attribute_args<'a, F>(
        attribute_args: __ref_type([Parens<Punctuated<AttributeArg, CommaToken>>]),
        predicate: F,
    ) -> impl Iterator<Item = __ref_type([AttributeArg])>
    where
        F: Fn(&__ref_type([AttributeArg])) -> bool + Clone + 'a,
    {
        attribute_args.inner.__iter().filter(predicate)
    }

    pub(crate) fn attribute_arg<'a, F>(
        attribute_args: __ref_type([Parens<Punctuated<AttributeArg, CommaToken>>]),
        predicate: F,
    ) -> Option<__ref_type([AttributeArg])>
    where
        F: Fn(&__ref_type([AttributeArg])) -> bool + Clone + 'a,
    {
        attribute_args.inner.__iter().find(predicate)
    }

    pub(crate) fn impl_self_or_trait_decls_annotated<'a, P>(
        parent: __ref_type([P]),
    ) -> impl Iterator<Item = __ref_type([Annotated<ItemKind>])>
    where
        P: __ElementsMatcher<Annotated<ItemKind>>,
    {
        parent.__match_elems(|annotated| matches!(annotated.value, ItemKind::Impl(_)))
    }

    /// Returns all trait constraints for all constrained generic
    /// arguments in the `parent`, that satisfy the `predicate`.
    /// The result is flattened and cumulative. This means that
    /// all the trait constraints will be collected from all
    /// the constraint arguments, even if there are duplicates.
    ///
    /// E.g., for this `where` clause and no predicate:
    /// ```ignore
    /// where A: Eq + AbiEncode + SomeTrait,
    ///       B: Eq + SomeTrait,
    /// ```
    /// The returned trait constraints will be:
    /// ```ignore
    /// Eq, AbiEncode, SomeTrait, Eq, SomeTrait
    /// ```
    pub(crate) fn trait_constraints<'a, P, F>(
        parent: __ref_type([P]),
        predicate: F,
    ) -> impl Iterator<Item = __ref_type([PathType])>
    where
        F: Fn(&__ref_type([PathType])) -> bool + Clone + 'a,
        P: __ElementsMatcher<PathType>,
    {
        parent.__match_elems(predicate)
    }

    pub(crate) fn functions<'a, P, F>(
        parent: __ref_type([P]),
        predicate: F,
    ) -> impl Iterator<Item = __ref_type([ItemFn])>
    where
        F: Fn(&__ref_type([ItemFn])) -> bool + Clone + 'a,
        P: __ElementsMatcher<ItemFn>,
    {
        parent.__match_elems(predicate)
    }
}

#[allow(dead_code)]
pub mod predicates {
    pub mod lexed_storage_field {
        use super::super::*;

        pub(crate) fn with_in_keyword(storage_field: &&StorageField) -> bool {
            storage_field.key_expr.is_some()
        }

        pub(crate) fn without_in_keyword(storage_field: &&mut StorageField) -> bool {
            storage_field.key_expr.is_none()
        }
    }

    pub mod item_impl {
        use super::super::*;

        pub(crate) fn implements_trait<'a, N: AsRef<str> + ?Sized>(
            trait_name: &'a N,
        ) -> impl Fn(&&'a ItemImpl) -> bool {
            move |item_impl: &&ItemImpl| {
                if let Some((path, _for_token)) = &item_impl.trait_opt {
                    path.last_segment().name.as_str() == trait_name.as_ref()
                } else {
                    false
                }
            }
        }
    }

    pub mod literal {
        use sway_ast::literal::{LitBool, LitBoolType};

        use super::super::*;

        pub(crate) fn is_bool_true(literal: &Literal) -> bool {
            matches!(
                literal,
                Literal::Bool(LitBool {
                    kind: LitBoolType::True,
                    ..
                })
            )
        }

        pub(crate) fn is_bool_false(literal: &Literal) -> bool {
            matches!(
                literal,
                Literal::Bool(LitBool {
                    kind: LitBoolType::False,
                    ..
                })
            )
        }
    }
}
