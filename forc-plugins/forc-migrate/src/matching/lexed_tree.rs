//! This module contains helper functions for matching elements within a lexed program.

use super::{any_mut, LexedElementsMatcher, LexedElementsMatcherDeep};
use sway_ast::{ItemKind, ItemStorage, StorageEntry, StorageField};
use sway_core::language::lexed::{LexedModule, LexedProgram};

impl LexedElementsMatcher<ItemStorage> for LexedProgram {
    fn match_elems<'a, F>(&'a mut self, predicate: F) -> impl Iterator<Item = &'a mut ItemStorage>
    where
        F: Fn(&&'a mut ItemStorage) -> bool + Clone + 'a,
        ItemStorage: 'a,
    {
        // Storage can be declared only in the root of a contract.
        self.root.match_elems(predicate)
    }
}

impl LexedElementsMatcher<ItemStorage> for LexedModule {
    fn match_elems<'a, F>(&'a mut self, predicate: F) -> impl Iterator<Item = &'a mut ItemStorage>
    where
        F: Fn(&&'a mut ItemStorage) -> bool + Clone + 'a,
        ItemStorage: 'a,
    {
        self.tree
            .items
            .iter_mut()
            .map(|annotated_item| &mut annotated_item.value)
            .filter_map(move |decl| match decl {
                ItemKind::Storage(ref mut item_storage) => {
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

impl LexedElementsMatcher<StorageField> for ItemStorage {
    fn match_elems<'a, F>(&'a mut self, predicate: F) -> impl Iterator<Item = &'a mut StorageField>
    where
        F: Fn(&&'a mut StorageField) -> bool + Clone + 'a,
        StorageField: 'a,
    {
        self.entries
            .inner
            .iter_mut()
            .map(|annotated_item| &mut annotated_item.value)
            .filter_map(move |storage_entry| {
                storage_entry.field.as_mut().filter(|sf| predicate(sf))
            })
    }
}

impl LexedElementsMatcherDeep<StorageField> for ItemStorage {
    fn match_elems_deep<'a, F>(&'a mut self, predicate: F) -> Vec<&'a mut StorageField>
    where
        F: Fn(&&'a mut StorageField) -> bool + Clone + 'a,
        StorageField: 'a,
    {
        fn recursively_collect_storage_fields_in_storage_entry<'a, P>(
            result: &mut Vec<&'a mut StorageField>,
            predicate: P,
            storage_entry: &'a mut StorageEntry,
        ) where
            P: Fn(&&'a mut StorageField) -> bool + Clone + 'a,
        {
            if let Some(ref mut sf) = storage_entry.field {
                if predicate(&sf) {
                    result.push(sf)
                }
            }

            if let Some(ref mut namespace) = storage_entry.namespace {
                namespace
                    .inner
                    .iter_mut()
                    .map(|annotated_item| &mut annotated_item.value)
                    .for_each(|storage_entry| {
                        recursively_collect_storage_fields_in_storage_entry(
                            result,
                            predicate.clone(),
                            storage_entry.as_mut(),
                        )
                    });
            }
        }

        let mut result = vec![];
        self.entries
            .inner
            .iter_mut()
            .map(|annotated_item| &mut annotated_item.value)
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

pub mod matchers {
    use super::*;

    pub(crate) fn storage_decl<P>(parent: &mut P) -> Option<&mut ItemStorage>
    where
        P: LexedElementsMatcher<ItemStorage>,
    {
        parent.match_elems(any_mut).next()
    }

    #[allow(dead_code)]
    pub(crate) fn storage_fields<'a, P, F>(
        parent: &'a mut P,
        predicate: F,
    ) -> impl Iterator<Item = &'a mut StorageField>
    where
        F: Fn(&&'a mut StorageField) -> bool + Clone + 'a,
        P: LexedElementsMatcher<StorageField>,
    {
        parent.match_elems(predicate)
    }

    pub(crate) fn storage_fields_deep<'a, S, F>(
        scope: &'a mut S,
        predicate: F,
    ) -> Vec<&'a mut StorageField>
    where
        F: Fn(&&'a mut StorageField) -> bool + Clone + 'a,
        S: LexedElementsMatcherDeep<StorageField>,
    {
        scope.match_elems_deep(predicate)
    }
}

pub mod predicates {
    pub mod lexed_storage_field {
        use super::super::*;

        #[allow(dead_code)]
        pub(crate) fn with_in_keyword(storage_field: &&mut StorageField) -> bool {
            storage_field.key_expr.is_some()
        }

        pub(crate) fn without_in_keyword(storage_field: &&mut StorageField) -> bool {
            storage_field.key_expr.is_none()
        }
    }
}
