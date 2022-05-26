use std::{cmp::Ordering, fmt, slice::Iter, vec::IntoIter};

use itertools::Itertools;
use sway_types::Span;

use crate::{
    error::{err, ok},
    CompileError, CompileResult,
};

use super::pattern::Pattern;

/// A `PatStack` is a `Vec<Pattern>` that is implemented with special methods
/// particular to the match exhaustivity algorithm.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PatStack {
    pats: Vec<Pattern>,
}

impl PatStack {
    /// Creates an empty `PatStack`.
    pub(crate) fn empty() -> Self {
        PatStack { pats: vec![] }
    }

    /// Given a `Pattern` *p*, creates a `PatStack` with one element *p*.
    pub(crate) fn from_pattern(p: Pattern) -> Self {
        PatStack { pats: vec![p] }
    }

    /// Given a usize *n*, creates a `PatStack` filled with *n*
    /// `Pattern::Wildcard` elements.
    pub(crate) fn fill_wildcards(n: usize) -> Self {
        let mut pats = vec![];
        for _ in 0..n {
            pats.push(Pattern::Wildcard);
        }
        PatStack { pats }
    }

    /// Returns the first element of a `PatStack`.
    pub(crate) fn first(&self, span: &Span) -> CompileResult<Pattern> {
        let warnings = vec![];
        let mut errors = vec![];
        match self.pats.first() {
            Some(first) => ok(first.to_owned(), warnings, errors),
            None => {
                errors.push(CompileError::Internal("empty PatStack", span.clone()));
                err(warnings, errors)
            }
        }
    }

    /// Returns a tuple of the first element of a `PatStack` and the rest of the
    /// elements.
    pub(crate) fn split_first(&self, span: &Span) -> CompileResult<(Pattern, PatStack)> {
        let warnings = vec![];
        let mut errors = vec![];
        match self.pats.split_first() {
            Some((first, pat_stack_contents)) => {
                let pat_stack = PatStack {
                    pats: pat_stack_contents.to_vec(),
                };
                ok((first.to_owned(), pat_stack), warnings, errors)
            }
            None => {
                errors.push(CompileError::Internal("empty PatStack", span.clone()));
                err(warnings, errors)
            }
        }
    }

    /// Given a usize *n*, splits the `PatStack` at *n* and returns both halves.
    pub(crate) fn split_at(&self, n: usize, span: &Span) -> CompileResult<(PatStack, PatStack)> {
        let warnings = vec![];
        let mut errors = vec![];
        if n > self.len() {
            errors.push(CompileError::Internal(
                "attempting to split OOB",
                span.clone(),
            ));
            return err(warnings, errors);
        }
        let (a, b) = self.pats.split_at(n);
        let x = PatStack { pats: a.to_vec() };
        let y = PatStack { pats: b.to_vec() };
        ok((x, y), warnings, errors)
    }

    /// Pushes a `Pattern` onto the `PatStack`
    pub(crate) fn push(&mut self, other: Pattern) {
        self.pats.push(other)
    }

    /// Given a usize *n*, returns a mutable reference to the `PatStack` at
    /// index *n*.
    fn get_mut(&mut self, n: usize, span: &Span) -> CompileResult<&mut Pattern> {
        let warnings = vec![];
        let mut errors = vec![];
        match self.pats.get_mut(n) {
            Some(elem) => ok(elem, warnings, errors),
            None => {
                errors.push(CompileError::Internal(
                    "cant retrieve mutable reference to element",
                    span.clone(),
                ));
                err(warnings, errors)
            }
        }
    }

    /// Appends a `PatStack` onto the `PatStack`.
    pub(crate) fn append(&mut self, others: &mut PatStack) {
        self.pats.append(&mut others.pats);
    }

    /// Prepends a `Pattern` onto the `PatStack`.
    pub(crate) fn prepend(&mut self, other: Pattern) {
        self.pats.insert(0, other);
    }

    /// Returns the length of the `PatStack`.
    pub(crate) fn len(&self) -> usize {
        self.pats.len()
    }

    /// Reports if the `PatStack` is empty.
    pub(crate) fn is_empty(&self) -> bool {
        self.flatten().filter_out_wildcards().pats.is_empty()
    }

    /// Reports if the `PatStack` contains a given `Pattern`.
    pub(crate) fn contains(&self, pat: &Pattern) -> bool {
        self.pats.contains(pat)
    }

    /// Reports if the `PatStack` contains an or-pattern at the top level.
    fn contains_or_pattern(&self) -> bool {
        for pat in self.pats.iter() {
            if let Pattern::Or(_) = pat {
                return true;
            }
        }
        false
    }

    pub(crate) fn iter(&self) -> Iter<'_, Pattern> {
        self.pats.iter()
    }

    pub(crate) fn into_iter(self) -> IntoIter<Pattern> {
        self.pats.into_iter()
    }

    /// Flattens the contents of a `PatStack` into a `PatStack`.
    pub(crate) fn flatten(&self) -> PatStack {
        let mut flattened = PatStack::empty();
        for pat in self.pats.iter() {
            flattened.append(&mut pat.flatten());
        }
        flattened
    }

    /// Orders a `PatStack` into a human-readable order.
    pub(crate) fn sort(self) -> PatStack {
        let mut sorted = self.pats;
        sorted.sort();
        PatStack::from(sorted)
    }

    /// Returns the given `PatStack` with wildcard patterns filtered out.
    pub(crate) fn filter_out_wildcards(&self) -> PatStack {
        let mut pats = PatStack::empty();
        for pat in self.pats.iter() {
            match pat {
                Pattern::Wildcard => {}
                pat => pats.push(pat.to_owned()),
            }
        }
        pats
    }

    /// Given a `PatStack` *args*, return a `Vec<PatStack>` *args*'
    /// "serialized" from *args*.
    ///
    /// Or-patterns are extracted to create a vec of `PatStack`s *args*' where
    /// each `PatStack` is a copy of *args* where the index of the or-pattern is
    /// instead replaced with one element from the or-patterns contents. More
    /// specifically, given an *args* with one or-pattern that contains n
    /// elements, this "serialization" would result in *args*' of length n.
    /// Given an *args* with two or-patterns that contain n elements and m
    /// elements, this would result in *args*' of length n*m.
    ///
    /// For example, given an *args*:
    ///
    /// ```ignore
    /// [
    ///     Pattern::Or([
    ///         Pattern::U64(Range { first: 0, last: 0 }),
    ///         Pattern::U64(Range { first: 1, last: 1 })
    ///     ]),
    ///     Pattern::Wildcard
    /// ]
    /// ```
    ///
    /// *args* would serialize to:
    ///
    /// ```ignore
    /// [
    ///     [
    ///         Pattern::U64(Range { first: 0, last: 0 }),
    ///         Pattern::Wildcard
    ///     ],
    ///     [
    ///         Pattern::U64(Range { first: 1, last: 1 }),
    ///         Pattern::Wildcard
    ///     ]
    /// ]
    /// ```
    ///
    /// Or, given an *args*:
    ///
    /// ```ignore
    /// [
    ///     Pattern::Or([
    ///         Pattern::U64(Range { first: 0, last: 0 }),
    ///         Pattern::U64(Range { first: 1, last: 1 })
    ///     ]),
    ///     Pattern::Or([
    ///         Pattern::U64(Range { first: 2, last: 2 }),
    ///         Pattern::U64(Range { first: 3, last: 3 }),
    ///         Pattern::U64(Range { first: 4, last: 4 }),
    ///     ]),
    /// ]
    /// ```
    ///
    /// *args* would serialize to:
    ///
    /// ```ignore
    /// [
    ///     [
    ///         Pattern::U64(Range { first: 0, last: 0 }),
    ///         Pattern::U64(Range { first: 2, last: 2 })
    ///     ],
    ///     [
    ///         Pattern::U64(Range { first: 0, last: 0 }),
    ///         Pattern::U64(Range { first: 3, last: 3 })
    ///     ],
    ///     [
    ///         Pattern::U64(Range { first: 0, last: 0 }),
    ///         Pattern::U64(Range { first: 4, last: 4 })
    ///     ],
    ///     [
    ///         Pattern::U64(Range { first: 1, last: 1 }),
    ///         Pattern::U64(Range { first: 2, last: 2 })
    ///     ],
    ///     [
    ///         Pattern::U64(Range { first: 1, last: 1 }),
    ///         Pattern::U64(Range { first: 3, last: 3 })
    ///     ],
    ///     [
    ///         Pattern::U64(Range { first: 1, last: 1 }),
    ///         Pattern::U64(Range { first: 4, last: 4 })
    ///     ],
    /// ]
    /// ```
    pub(crate) fn serialize_multi_patterns(self, span: &Span) -> CompileResult<Vec<PatStack>> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let mut output: Vec<PatStack> = vec![];
        let mut stack: Vec<PatStack> = vec![self];
        while !stack.is_empty() {
            let top = match stack.pop() {
                Some(top) => top,
                None => {
                    errors.push(CompileError::Internal("can't pop Vec", span.clone()));
                    return err(warnings, errors);
                }
            };
            if !top.contains_or_pattern() {
                output.push(top);
            } else {
                for (i, pat) in top.clone().into_iter().enumerate() {
                    if let Pattern::Or(elems) = pat {
                        for elem in elems.into_iter() {
                            let mut top = top.clone();
                            let r = check!(
                                top.get_mut(i, span),
                                return err(warnings, errors),
                                warnings,
                                errors
                            );
                            let _ = std::mem::replace(r, elem);
                            stack.push(top);
                        }
                    }
                }
            }
        }
        output.reverse();
        ok(output, warnings, errors)
    }

    /// Orders a `PatStack` into a human-readable order.
    ///
    /// For error reporting only.
    pub(crate) fn remove_duplicates(self) -> PatStack {
        let mut new_pats = vec![];
        for pat in self.pats.into_iter() {
            if !new_pats.contains(&pat) {
                new_pats.push(pat);
            }
        }
        PatStack::from(new_pats)
    }
}

impl From<Vec<Pattern>> for PatStack {
    fn from(pats: Vec<Pattern>) -> Self {
        PatStack { pats }
    }
}

impl fmt::Display for PatStack {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = self
            .flatten()
            .sort()
            .remove_duplicates()
            .into_iter()
            .map(|x| format!("{}", x))
            .join(", ");
        write!(f, "{}", s)
    }
}

impl std::cmp::Ord for PatStack {
    fn cmp(&self, other: &Self) -> Ordering {
        let sorted_self = self.clone().sort();
        let sorted_other = other.clone().sort();
        sorted_self.pats.cmp(&sorted_other.pats)
    }
}

impl std::cmp::PartialOrd for PatStack {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
