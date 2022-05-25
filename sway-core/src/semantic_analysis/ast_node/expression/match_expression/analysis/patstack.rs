use std::{fmt, slice::Iter, vec::IntoIter};

use itertools::Itertools;
use sway_types::Span;

use crate::{
    error::{err, ok},
    CompileError, CompileResult,
};

use super::{pattern::Pattern, range::Range};

/// A `PatStack` is a `Vec<Pattern>` that is implemented with special methods
/// particular to the match exhaustivity algorithm.
#[derive(Clone, Debug, PartialEq)]
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

    /// Reports if the `PatStack` Σ is a "complete signature" of the type of the
    /// elements of Σ.
    ///
    /// For example, a Σ composed of `Pattern::U64(..)`s would need to check for
    /// if it is a complete signature for the `U64` pattern type. Versus a Σ
    /// composed of `Pattern::Tuple([.., ..])` which would need to check for if
    /// it is a complete signature for "`Tuple` with 2 sub-patterns" type.
    ///
    /// There are several rules with which to determine if Σ is a complete
    /// signature:
    ///
    /// 1. If Σ is empty it is not a complete signature.
    /// 2. If Σ contains only wildcard patterns, it is not a complete signature.
    /// 3. If Σ contains all constructors for the type of the elements of Σ then
    ///    it is a complete signature.
    ///
    /// For example, given this Σ:
    ///
    /// ```ignore
    /// [
    ///     Pattern::U64(Range { first: 0, last: 0 }),
    ///     Pattern::U64(Range { first: 7, last: 7 })
    /// ]
    /// ```
    ///
    /// this would not be a complete signature as it does not contain all
    /// elements from the `U64` type.
    ///
    /// Given this Σ:
    ///
    /// ```ignore
    /// [
    ///     Pattern::U64(Range { first: std::u64::MIN, last: std::u64::MAX })
    /// ]
    /// ```
    ///
    /// this would be a complete signature as it does contain all elements from
    /// the `U64` type.
    ///
    /// Given this Σ:
    ///
    /// ```ignore
    /// [
    ///     Pattern::Tuple([
    ///         Pattern::U64(Range { first: 0, last: 0 }),
    ///         Pattern::Wildcard
    ///     ]),
    /// ]
    /// ```
    ///
    /// this would also be a complete signature as it does contain all elements
    /// from the "`Tuple` with 2 sub-patterns" type.
    pub(crate) fn is_complete_signature(&self, span: &Span) -> CompileResult<bool> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let preprocessed = self.flatten().filter_out_wildcards();
        if preprocessed.pats.is_empty() {
            return ok(false, warnings, errors);
        }
        let (first, rest) = check!(
            preprocessed.split_first(span),
            return err(warnings, errors),
            warnings,
            errors
        );
        match first {
            // its assumed that no one is ever going to list every string
            Pattern::String(_) => ok(false, warnings, errors),
            // its assumed that no one is ever going to list every B256
            Pattern::B256(_) => ok(false, warnings, errors),
            Pattern::U8(range) => {
                let mut ranges = vec![range];
                for pat in rest.into_iter() {
                    match pat {
                        Pattern::U8(range) => ranges.push(range),
                        _ => {
                            errors.push(CompileError::Internal("type mismatch", span.clone()));
                            return err(warnings, errors);
                        }
                    }
                }
                Range::do_ranges_equal_range(ranges, Range::u8(), span)
            }
            Pattern::U16(range) => {
                let mut ranges = vec![range];
                for pat in rest.into_iter() {
                    match pat {
                        Pattern::U16(range) => ranges.push(range),
                        _ => {
                            errors.push(CompileError::Internal("type mismatch", span.clone()));
                            return err(warnings, errors);
                        }
                    }
                }
                Range::do_ranges_equal_range(ranges, Range::u16(), span)
            }
            Pattern::U32(range) => {
                let mut ranges = vec![range];
                for pat in rest.into_iter() {
                    match pat {
                        Pattern::U32(range) => ranges.push(range),
                        _ => {
                            errors.push(CompileError::Internal("type mismatch", span.clone()));
                            return err(warnings, errors);
                        }
                    }
                }
                Range::do_ranges_equal_range(ranges, Range::u32(), span)
            }
            Pattern::U64(range) => {
                let mut ranges = vec![range];
                for pat in rest.into_iter() {
                    match pat {
                        Pattern::U64(range) => ranges.push(range),
                        _ => {
                            errors.push(CompileError::Internal("type mismatch", span.clone()));
                            return err(warnings, errors);
                        }
                    }
                }
                Range::do_ranges_equal_range(ranges, Range::u64(), span)
            }
            Pattern::Byte(range) => {
                let mut ranges = vec![range];
                for pat in rest.into_iter() {
                    match pat {
                        Pattern::Byte(range) => ranges.push(range),
                        _ => {
                            errors.push(CompileError::Internal("type mismatch", span.clone()));
                            return err(warnings, errors);
                        }
                    }
                }
                Range::do_ranges_equal_range(ranges, Range::u8(), span)
            }
            Pattern::Numeric(range) => {
                let mut ranges = vec![range];
                for pat in rest.into_iter() {
                    match pat {
                        Pattern::Numeric(range) => ranges.push(range),
                        _ => {
                            errors.push(CompileError::Internal("type mismatch", span.clone()));
                            return err(warnings, errors);
                        }
                    }
                }
                Range::do_ranges_equal_range(ranges, Range::u64(), span)
            }
            Pattern::Boolean(b) => {
                let mut true_found = false;
                let mut false_found = false;
                match b {
                    true => true_found = true,
                    false => false_found = true,
                }
                for pat in rest.iter() {
                    match pat {
                        Pattern::Boolean(b) => match b {
                            true => true_found = true,
                            false => false_found = true,
                        },
                        _ => {
                            errors.push(CompileError::Internal("type mismatch", span.clone()));
                            return err(warnings, errors);
                        }
                    }
                }
                ok(true_found && false_found, warnings, errors)
            }
            ref tup @ Pattern::Tuple(_) => {
                for pat in rest.iter() {
                    if !pat.has_the_same_constructor(tup) {
                        return ok(false, warnings, errors);
                    }
                }
                ok(true, warnings, errors)
            }
            ref strct @ Pattern::Struct(_) => {
                for pat in rest.iter() {
                    if !pat.has_the_same_constructor(strct) {
                        return ok(false, warnings, errors);
                    }
                }
                ok(true, warnings, errors)
            }
            Pattern::Wildcard => unreachable!(),
            Pattern::Or(_) => unreachable!(),
        }
    }

    /// Flattens the contents of a `PatStack` into a `PatStack`.
    pub(crate) fn flatten(&self) -> PatStack {
        let mut flattened = PatStack::empty();
        for pat in self.pats.iter() {
            flattened.append(&mut pat.flatten());
        }
        flattened
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
    /// Given an *args*:
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
            .into_iter()
            .map(|x| format!("{}", x))
            .join(", ");
        write!(f, "{}", s)
    }
}
