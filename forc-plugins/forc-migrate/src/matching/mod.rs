//! This module contains common API for matching elements
//! within a lexed or a typed tree.
//!
//! A typical migration will search for certain elements in the
//! lexed or typed tree and modify them within the lexed tree.
//!
//! In the long term we want to have advanced infrastructure for both
//! matching and modifying parts of the trees, as discussed in
//! https://github.com/FuelLabs/sway/issues/6836.
//!
//! Currently, we will start (very) small, by providing reusable
//! module functions for matching parts of the trees.
//!
//! For concrete examples, see the match functions and trait impls
//! implemented in the sub-modules.
//!
//! ## Design decisions
//!
//! The goal was pragmatic. To create a simple to develop and extend API that
//! will offer easy discoverability of provided functions and methods, all in
//! order to move cumbersome and error-prone matching code out of the migration
//! logic.
//!
//! Migrations will use module level match functions to either search directly
//! within a parent or recursively (deep) within a scope. Match functions can
//! accept predicates to filter the searched elements. The predicates deliberately
//! accept `&&TElement` or `&&mut TElement` so that can be easily passed to
//! [Iterator::filter] function.
//!
//! ## Matching elements in trees
//!
//! Functions matching on lexed tree require mutable references as
//! input and return mutable references as output. This is according
//! to the premise that the non-code-modifying analysis will be done
//! on typed trees, while the code-modifying will be done on the
//! mutable lexed tree, as well as the typed tree.
//!
//! Matching can be done either directly within a parent, or recursively
//! within a scope. E.g., we can match for `StorageField`s that are
//! directly under the `storage` declaration, or for all `StorageField`s
//! that are in the `storage` declaration, in any of the namespaces,
//! recursively.
//!
//! Searching for elements "in-between", e.g., `StorageField`s in a particular
//! sub-namespace, is currently not supported, and must be done manually
//! within a migration.
//!
//! Matching is done on lexical or typed elements like, e.g., `StorageField`,
//! or `TyStorageField`, without any more convenient abstraction provided for
//! matching. This is also a simple beginning. A better matching framework
//! would expose a stable higher level abstraction for matching and modifying.
//!
//! ## Locating equivalent elements across trees
//!
//! Often we will find an element in the lexed tree, e.g., a `StorageField` in
//! order to change it, but will need additional information from its typed tree
//! counterpart, `TyStorageField`, or vice versa. The [TyLocate] trait offers
//! the [TyLocate::locate] method for finding a typed equivalent of a lexed
//! element. The [LexedLocate] does the opposite. 
//!
//! Locating an equivalent will in most of the cases be implemented via equality
//! of spans. Locating can also cause multiple traversals of the same part of
//! a tree. For migrations, this will not cause a performance problem.

mod typed_tree;
mod lexed_tree;

pub(crate) use typed_tree::matchers as ty_match; 
pub(crate) use typed_tree::predicates::ty_storage_field as ty_storage_field; 

pub(crate) use lexed_tree::matchers as lexed_match; 
pub(crate) use lexed_tree::predicates::lexed_storage_field as lexed_storage_field; 

/// Matches for typed tree elements of type `T` located **directly** within
/// the typed tree element `self`.
///
/// The matched elements must satisfy the `predicate`.
pub(crate) trait TyElementsMatcher<T> {
    fn match_elems<'a, P>(&'a self, predicate: P) -> impl Iterator<Item=&'a T>
    where
        P: Fn(&&'a T) -> bool + Clone + 'a,
        T: 'a;
}

/// Matches for typed tree elements of type `T` located **recursively** within
/// the typed tree element `self` or any of its children. The meaning of a
/// "child" depends on the exact tree element `self`.
///
/// The matched elements must satisfy the `predicate`.
pub(crate) trait TyElementsMatcherDeep<T> {
    fn match_elems_deep<'a, F>(&'a self, predicate: F) -> Vec<&'a T>
    where
        F: Fn(&&'a T) -> bool + Clone + 'a,
        T: 'a;
}

/// Within a typed tree element `self`, locates and returns the element of type `Ty`,
/// that is the typed equivalent of the `lexed_element`.
pub(crate) trait TyLocate<Lexed, Ty> {
    fn locate(&self, lexed_element: &Lexed) -> Option<&Ty>;
}

/// Matches for lexed tree elements of type `T` located **directly** within
/// the lexed tree element `self`.
///
/// The matched elements must satisfy the `predicate`.
pub(crate) trait LexedElementsMatcher<T> {
    fn match_elems<'a, F>(&'a mut self, predicate: F) -> impl Iterator<Item=&'a mut T>
    where
        F: Fn(&&'a mut T) -> bool + Clone + 'a,
        T: 'a;
}

/// Matches for lexed tree elements of type `T` located **recursively** within
/// the lexed tree element `self` or any of its children. The meaning of a
/// "child" depends on the exact tree element `self`.
///
/// The matched elements must satisfy the `predicate`.
pub(crate) trait LexedElementsMatcherDeep<T> {
    fn match_elems_deep<'a, F>(&'a mut self, predicate: F) -> Vec<&'a mut T>
    where
        F: Fn(&&'a mut T) -> bool + Clone + 'a,
        T: 'a;
}

/// Within a lexed tree element `self`, locates and returns the element of type `Lexed`,
/// that is the lexed equivalent of the `ty_element`.
#[allow(dead_code)]
pub(crate) trait LexedLocate<Ty, Lexed> {
    fn locate(&mut self, ty_element: &Ty) -> Option<&mut Lexed>;
}

/// A predicate that returns true for any input.
/// Convenient to use in [TyElementsMatcher] and [TyElementsMatcherDeep].
pub(crate) fn any<T>(_t: &&T) -> bool {
    true
}

/// A predicate that returns true for any input.
/// Convenient to use in [LexedElementsMatcher] and [LexedElementsMatcherDeep].
pub(crate) fn any_mut<T>(_t: &&mut T) -> bool {
    true
}

/// Returns a predicate that evaluates to true if all the predicates passed
/// as arguments evaluate to true.
#[macro_export]
macro_rules! all_of {
    ($($i:expr),+) => {
       $crate::matching::all_of([$($i, )*].as_slice())
    };
}

/// Returns a predicate that evaluates to true if all the `predicates`
/// evaluate to true.
///
/// Not intended to be used directly. Use [all_of!] macro instead.
#[allow(dead_code)]
pub(crate) fn all_of<T, P>(predicates: &[P]) -> impl Fn(&&T) -> bool + Clone + '_
where P: Fn(&&T) -> bool + Clone,
{
    move |t: &&T| {
        let mut res = true;
        for predicate in predicates {
            res &= predicate(t);
        }
        res
    }
}

/// Returns a predicate that evaluates to true if all the predicates passed
/// as arguments evaluate to true.
#[macro_export]
macro_rules! all_of_mut {
    ($($i:expr),+) => {
       $crate::matching::all_of_mut([$($i, )*].as_slice())
    };
}

/// Returns a predicate that evaluates to true if all the `predicates`
/// evaluate to true.
///
/// Not intended to be used directly. Use [all_of_mut!] macro instead.
#[allow(dead_code)]
pub(crate) fn all_of_mut<T, P>(predicates: &[P]) -> impl Fn(&&mut T) -> bool + Clone + '_
where P: Fn(&&mut T) -> bool + Clone,
{
    move |t: &&mut T| {
        let mut res = true;
        for predicate in predicates {
            res &= predicate(t);
        }
        res
    }
}

/// Returns a predicate that evaluates to true if any of the predicates passed
/// as arguments evaluate to true.
#[macro_export]
macro_rules! any_of {
    ($($i:expr),+) => {
       $crate::matching::any_of([$($i, )*].as_slice())
    };
}

/// Returns a predicate that evaluates to true if any of the `predicates`
/// evaluate to true.
///
/// Not intended to be used directly. Use [any_of!] macro instead.
#[allow(dead_code)]
pub(crate) fn any_of<T, P>(predicates: &[P]) -> impl Fn(&&T) -> bool + Clone + '_
where P: Fn(&&T) -> bool + Clone,
{
    move |t: &&T| {
        let mut res = false;
        for predicate in predicates {
            res |= predicate(t);
        }
        res
    }
}

/// Returns a predicate that evaluates to true if any of the predicates passed
/// as arguments evaluate to true.
#[macro_export]
macro_rules! any_of_mut {
    ($($i:expr),+) => {
       $crate::matching::any_of_mut([$($i, )*].as_slice())
    };
}

/// Returns a predicate that evaluates to true if any of the `predicates`
/// evaluate to true.
///
/// Not intended to be used directly. Use [any_of_mut!] macro instead.
#[allow(dead_code)]
pub(crate) fn any_of_mut<T, P>(predicates: &[P]) -> impl Fn(&&mut T) -> bool + Clone + '_
where P: Fn(&&mut T) -> bool + Clone,
{
    move |t: &&mut T| {
        let mut res = false;
        for predicate in predicates {
            res |= predicate(t);
        }
        res
    }
}
