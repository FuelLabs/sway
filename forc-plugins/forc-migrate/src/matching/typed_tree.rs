//! This module contains helper functions for matching elements within a typed program.

use super::{any, TyElementsMatcher, TyElementsMatcherDeep, TyLocate};
use sway_ast::StorageField;
use sway_core::{
    decl_engine::id::DeclId,
    language::ty::{TyAstNodeContent, TyDecl, TyModule, TyProgram, TyStorageDecl, TyStorageField},
};
use sway_types::Spanned;

impl TyElementsMatcher<DeclId<TyStorageDecl>> for TyProgram {
    fn match_elems<'a, F>(&'a self, predicate: F) -> impl Iterator<Item = &'a DeclId<TyStorageDecl>>
    where
        F: Fn(&&'a DeclId<TyStorageDecl>) -> bool + Clone + 'a,
        DeclId<TyStorageDecl>: 'a,
    {
        // Storage can be declared only in the root of a contract.
        self.root_module.match_elems(predicate)
    }
}

impl TyElementsMatcher<DeclId<TyStorageDecl>> for TyModule {
    fn match_elems<'a, F>(&'a self, predicate: F) -> impl Iterator<Item = &'a DeclId<TyStorageDecl>>
    where
        F: Fn(&&'a DeclId<TyStorageDecl>) -> bool + Clone + 'a,
        DeclId<TyStorageDecl>: 'a,
    {
        self.all_nodes
            .iter()
            .filter_map(move |decl| match &decl.content {
                TyAstNodeContent::Declaration(TyDecl::StorageDecl(storage_decl)) => {
                    if predicate(&&storage_decl.decl_id) {
                        Some(&storage_decl.decl_id)
                    } else {
                        None
                    }
                }
                _ => None,
            })
    }
}

impl TyElementsMatcher<TyStorageField> for TyStorageDecl {
    fn match_elems<'a, F>(&'a self, predicate: F) -> impl Iterator<Item = &'a TyStorageField>
    where
        F: Fn(&&'a TyStorageField) -> bool + Clone + 'a,
        TyStorageField: 'a,
    {
        self.fields
            .iter()
            // In the `TyStorageDecl`, all the fields are flattened.
            // But we need to preserve the semantics of non-deep matching
            // and return only those that are directly under the storage.
            .filter(|sf| sf.full_name().starts_with("storage."))
            .filter(predicate)
    }
}

impl TyElementsMatcherDeep<TyStorageField> for TyStorageDecl {
    fn match_elems_deep<'a, F>(&'a self, predicate: F) -> Vec<&'a TyStorageField>
    where
        F: Fn(&&'a TyStorageField) -> bool + Clone + 'a,
        TyStorageField: 'a,
    {
        self.fields.iter().filter(predicate).collect()
    }
}

impl TyLocate<StorageField, TyStorageField> for TyStorageDecl {
    fn locate(&self, lexed_element: &StorageField) -> Option<&TyStorageField> {
        self.fields
            .iter()
            .find(|field| field.name.span() == lexed_element.name.span())
    }
}

pub mod matchers {
    use super::*;

    pub(crate) fn storage_decl<P>(parent: &P) -> Option<DeclId<TyStorageDecl>>
    where
        P: TyElementsMatcher<DeclId<TyStorageDecl>>,
    {
        parent.match_elems(any).next().copied()
    }

    #[allow(dead_code)]
    pub(crate) fn storage_fields<'a, P, F>(
        parent: &'a P,
        predicate: F,
    ) -> impl Iterator<Item = &'a TyStorageField>
    where
        F: Fn(&&'a TyStorageField) -> bool + Clone + 'a,
        P: TyElementsMatcher<TyStorageField>,
    {
        parent.match_elems(predicate)
    }

    pub(crate) fn storage_fields_deep<'a, S, F>(
        scope: &'a S,
        predicate: F,
    ) -> Vec<&'a TyStorageField>
    where
        F: Fn(&&'a TyStorageField) -> bool + Clone + 'a,
        S: TyElementsMatcherDeep<TyStorageField>,
    {
        scope.match_elems_deep(predicate)
    }
}

pub mod predicates {
    pub mod ty_storage_field {
        use super::super::*;

        pub(crate) fn with_in_keyword(storage_field: &&TyStorageField) -> bool {
            storage_field.key_expression.is_some()
        }

        pub(crate) fn without_in_keyword(storage_field: &&TyStorageField) -> bool {
            storage_field.key_expression.is_none()
        }
    }
}
