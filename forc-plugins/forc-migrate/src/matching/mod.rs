#![allow(dead_code)]
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
//! Note that, although similar to static analysis tools like, e.g. Rust's
//! [Clippy](https://doc.rust-lang.org/clippy/), `forc migrate` is significantly
//! different. Instead of providing hundreds of independent lints that
//! automatically check for localized issues, migrations provide only a handful
//! of migration steps, that are orchestrated within a single migration process,
//! some of them possibly being interactive.
//!
//! Each migration step, in general, wants to take a look at a larger scope at a time,
//! often a module. This makes a typical approach, of using fine-grain visitor functions
//! less applicable. Also, the goal is to empower non-compiler developers to write
//! migrations.
//!
//! All this led to the design in which a single migration step is in focus, and can:
//! - search for elements of interest using the match functions,
//! - build new and modify existing lexed elements using the [super::modifying],
//!
//! Migrations will use match functions to either search directly
//! within a parent or recursively (deep) within a scope. Match functions can
//! accept predicates to filter the searched elements. The predicates deliberately
//! accept `&&TElement` or `&&mut TElement` so that can be easily passed to
//! [Iterator::filter] function.
//!
//! For the cases when migrations do target individual expressions, and do not need
//! to inspect a larger scope, the visitor pattern is still supported and available
//! via the tree visitors that are defined in [super::visiting].
//!
//! ## Matching elements in trees
//!
//! Functions matching on lexed trees are coming in two variants, immutable and mutable.
//! They differ in the mutability of their arguments and returned types, but
//! otherwise implement the same matching logic.
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
//! element. The [LexedLocate] and [LexedLocateMut] do the opposite.
//!
//! Locating an equivalent will in most of the cases be implemented via equality
//! of spans. Locating can also cause multiple traversals of the same part of
//! a tree. For migrations, this will not cause a performance problem.

mod lexed_tree;
mod typed_tree;

use sway_ast::attribute::{Annotated, Attribute, AttributeArg};
use sway_ast::{AttributeDecl, ItemFn, PathType};
pub(crate) use typed_tree::matchers as ty_match;
pub(crate) use typed_tree::predicates::*;

pub(crate) use lexed_tree::matchers as lexed_match;
pub(crate) use lexed_tree::matchers_mut as lexed_match_mut;
pub(crate) use lexed_tree::predicates::*;

/// Matches for typed tree elements of type `T` located **directly** within
/// the typed tree element `self`.
///
/// The matched elements must satisfy the `predicate`.
pub(crate) trait TyElementsMatcher<T> {
    fn match_elems<'a, P>(&'a self, predicate: P) -> impl Iterator<Item = &'a T>
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
pub(crate) trait LexedElementsMatcherMut<T> {
    fn match_elems_mut<'a, F>(&'a mut self, predicate: F) -> impl Iterator<Item = &'a mut T>
    where
        F: Fn(&&'a mut T) -> bool + Clone + 'a,
        T: 'a;
}

pub(crate) trait LexedElementsMatcher<T> {
    fn match_elems<'a, F>(&'a self, predicate: F) -> impl Iterator<Item = &'a T>
    where
        F: Fn(&&'a T) -> bool + Clone + 'a,
        T: 'a;
}

/// Matches for lexed tree elements of type `T` located **recursively** within
/// the lexed tree element `self` or any of its children. The meaning of a
/// "child" depends on the exact tree element `self`.
///
/// The matched elements must satisfy the `predicate`.
pub(crate) trait LexedElementsMatcherDeepMut<T> {
    fn match_elems_deep_mut<'a, F>(&'a mut self, predicate: F) -> Vec<&'a mut T>
    where
        F: Fn(&&'a mut T) -> bool + Clone + 'a,
        T: 'a;
}

pub(crate) trait LexedElementsMatcherDeep<T> {
    fn match_elems_deep<'a, F>(&'a self, predicate: F) -> Vec<&'a T>
    where
        F: Fn(&&'a T) -> bool + Clone + 'a,
        T: 'a;
}

/// Within a lexed tree element `self`, locates and returns the element of type `Lexed`,
/// that is the lexed equivalent of the `ty_element`.
pub(crate) trait LexedLocateMut<Ty, Lexed> {
    fn locate_mut(&mut self, ty_element: &Ty) -> Option<&mut Lexed>;
}

/// Within a lexed tree element `self`, locates and returns the element of type `Lexed`,
/// that is the lexed equivalent of the `ty_element`.
pub(crate) trait LexedLocate<Ty, Lexed> {
    fn locate(&self, ty_element: &Ty) -> Option<&Lexed>;
}

/// Within a lexed tree element `self`, locates and returns the element of type `Lexed`,
/// that is the lexed equivalent of the `ty_element`, together with its annotations.
pub(crate) trait LexedLocateAnnotatedMut<Ty, Lexed> {
    fn locate_annotated_mut<'a>(
        &'a mut self,
        ty_element: &Ty,
    ) -> Option<(&'a mut Vec<AttributeDecl>, &'a mut Lexed)>;
}

/// Within a lexed tree element `self`, locates and returns the element of type `Lexed`,
/// that is the lexed equivalent of the `ty_element`, together with its annotations.
pub(crate) trait LexedLocateAnnotated<Ty, Lexed> {
    fn locate_annotated<'a>(
        &'a self,
        ty_element: &Ty,
    ) -> Option<(&'a Vec<AttributeDecl>, &'a Lexed)>;
}

/// Within an annotated lexed tree element `self`, locates and returns the element of type `LexedAnnotated`,
/// that is the annotated lexed equivalent of the `ty_element`.
pub(crate) trait LexedLocateAsAnnotatedMut<Ty, LexedAnnotated> {
    fn locate_as_annotated_mut(
        &mut self,
        ty_element: &Ty,
    ) -> Option<&mut Annotated<LexedAnnotated>>;
}

/// Within an annotated lexed tree element `self`, locates and returns the element of type `LexedAnnotated`,
/// that is the annotated lexed equivalent of the `ty_element`.
pub(crate) trait LexedLocateAsAnnotated<Ty, LexedAnnotated> {
    fn locate_as_annotated(&self, ty_element: &Ty) -> Option<&Annotated<LexedAnnotated>>;
}

impl<T, Ty, Lexed> LexedLocateMut<Ty, Lexed> for T
where
    T: LexedLocateAnnotatedMut<Ty, Lexed>,
{
    fn locate_mut(&mut self, ty_element: &Ty) -> Option<&mut Lexed> {
        self.locate_annotated_mut(ty_element)
            .map(|annotated| annotated.1)
    }
}

impl<T, Ty, Lexed> LexedLocate<Ty, Lexed> for T
where
    T: LexedLocateAnnotated<Ty, Lexed>,
{
    fn locate(&self, ty_element: &Ty) -> Option<&Lexed> {
        self.locate_annotated(ty_element)
            .map(|annotated| annotated.1)
    }
}

/// A predicate that returns true for any immutable input.
pub(crate) fn any<T>(_t: &&T) -> bool {
    true
}

/// A predicate that returns true for any mutable input.
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
where
    P: Fn(&&T) -> bool + Clone,
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
where
    P: Fn(&&mut T) -> bool + Clone,
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
where
    P: Fn(&&T) -> bool + Clone,
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
where
    P: Fn(&&mut T) -> bool + Clone,
{
    move |t: &&mut T| {
        let mut res = false;
        for predicate in predicates {
            res |= predicate(t);
        }
        res
    }
}

/// Trait for inspecting if a tree element has the expected name.
pub(crate) trait WithName {
    /// Returns true if `Self` has the name `name`.
    fn with_name<N: AsRef<str> + ?Sized>(&self, name: &N) -> bool;
}

/// Returns a predicate that evaluates to true if a [WithName]
/// implementer has the name equal to `name`.
pub(crate) fn with_name<T, N>(name: &N) -> impl Fn(&&T) -> bool + Clone + '_
where
    T: WithName,
    N: AsRef<str> + ?Sized,
{
    move |t: &&T| t.with_name(name)
}

/// Returns a predicate that evaluates to true if a [WithName]
/// implementer has the name equal to `name`.
pub(crate) fn with_name_mut<T, N>(name: &N) -> impl Fn(&&mut T) -> bool + Clone + '_
where
    T: WithName,
    N: AsRef<str> + ?Sized,
{
    move |t: &&mut T| t.with_name(name)
}

impl WithName for Attribute {
    fn with_name<N: AsRef<str> + ?Sized>(&self, name: &N) -> bool {
        self.name.as_str() == name.as_ref()
    }
}

impl WithName for AttributeArg {
    fn with_name<N: AsRef<str> + ?Sized>(&self, name: &N) -> bool {
        self.name.as_str() == name.as_ref()
    }
}

impl WithName for PathType {
    fn with_name<N: AsRef<str> + ?Sized>(&self, name: &N) -> bool {
        self.last_segment().name.as_str() == name.as_ref()
    }
}

impl WithName for ItemFn {
    fn with_name<N: AsRef<str> + ?Sized>(&self, name: &N) -> bool {
        self.fn_signature.name.as_str() == name.as_ref()
    }
}
