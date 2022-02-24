use std::fmt;
use std::ops::Sub;
use std::slice::Iter;
use std::vec::IntoIter;

use sway_types::Ident;
use sway_types::Span;

use crate::error::err;
use crate::error::ok;
use crate::CompileError;
use crate::CompileResult;
use crate::Literal;
use crate::MatchCondition;
use crate::Scrutinee;
use crate::StructScrutineeField;
use crate::TypeInfo;

use itertools::Itertools;
use std::fmt::Debug;

/// A `WitnessReport` is a report of the witnesses to a `Pattern` being useful
/// and is used in the match expression exhaustivity checking algorithm.
#[derive(Debug)]
pub(crate) enum WitnessReport {
    NoWitnesses,
    Witnesses(PatStack),
}

impl WitnessReport {
    /// Joins two `WitnessReport`s together.
    fn join_witness_reports(a: WitnessReport, b: WitnessReport) -> Self {
        match (a, b) {
            (WitnessReport::NoWitnesses, WitnessReport::NoWitnesses) => WitnessReport::NoWitnesses,
            (WitnessReport::NoWitnesses, WitnessReport::Witnesses(wits)) => {
                WitnessReport::Witnesses(wits)
            }
            (WitnessReport::Witnesses(wits), WitnessReport::NoWitnesses) => {
                WitnessReport::Witnesses(wits)
            }
            (WitnessReport::Witnesses(wits1), WitnessReport::Witnesses(mut wits2)) => {
                let mut wits = wits1;
                wits.append(&mut wits2);
                WitnessReport::Witnesses(wits)
            }
        }
    }

    /// Given a `WitnessReport` *wr* and a constructor *c* with *a* number of
    /// sub-patterns, creates a new `Pattern` *p* and a new `WitnessReport`
    /// *wr'*. *p* is created by applying *c* to the first *a* elements of *wr*.
    /// *wr'* is created by taking the remaining elements of *wr* after *a*
    /// elements have been removed from the front of *wr*.
    fn split_into_leading_constructor(
        witness_report: WitnessReport,
        c: &Pattern,
        span: &Span,
    ) -> CompileResult<(Pattern, Self)> {
        let mut warnings = vec![];
        let mut errors = vec![];
        match witness_report {
            WitnessReport::NoWitnesses => {
                errors.push(CompileError::ExhaustivityCheckingAlgorithmFailure(
                    "expected to find witnesses to use as arguments to a constructor",
                    span.clone(),
                ));
                err(warnings, errors)
            }
            WitnessReport::Witnesses(witnesses) => {
                let (rs, ps) = check!(
                    witnesses.split_at(c.a(), span),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                let pat = check!(
                    Pattern::from_constructor_and_arguments(c, rs, span),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                ok((pat, WitnessReport::Witnesses(ps)), warnings, errors)
            }
        }
    }

    /// Prepends a witness `Pattern` onto the `WitnessReport`.
    fn add_witness(&mut self, witness: Pattern, span: &Span) -> CompileResult<()> {
        let warnings = vec![];
        let mut errors = vec![];
        match self {
            WitnessReport::NoWitnesses => {
                errors.push(CompileError::ExhaustivityCheckingAlgorithmFailure(
                    "expected to find witnesses",
                    span.clone(),
                ));
                err(warnings, errors)
            }
            WitnessReport::Witnesses(witnesses) => {
                witnesses.prepend(witness);
                ok((), warnings, errors)
            }
        }
    }

    /// Reports if this `WitnessReport` has witnesses.
    pub(crate) fn has_witnesses(&self) -> bool {
        match self {
            WitnessReport::NoWitnesses => false,
            WitnessReport::Witnesses(_) => true, // !witnesses.is_empty()
        }
    }
}

impl fmt::Display for WitnessReport {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let witnesses = match self {
            WitnessReport::NoWitnesses => PatStack::empty(),
            WitnessReport::Witnesses(witnesses) => witnesses.clone(),
        };
        let s = witnesses
            .flatten()
            .into_iter()
            .map(|x| format!("`{}`", x))
            .join(", ");
        write!(f, "{}", s)
    }
}

pub(crate) trait MyMath<T> {
    fn global_max() -> T;
    fn global_min() -> T;

    fn incr(&self) -> T;
    fn decr(&self) -> T;
}

impl MyMath<u8> for u8 {
    fn global_max() -> u8 {
        std::u8::MAX
    }
    fn global_min() -> u8 {
        std::u8::MIN
    }

    fn incr(&self) -> u8 {
        self + 1
    }
    fn decr(&self) -> u8 {
        self - 1
    }
}

impl MyMath<u16> for u16 {
    fn global_max() -> u16 {
        std::u16::MAX
    }
    fn global_min() -> u16 {
        std::u16::MIN
    }

    fn incr(&self) -> u16 {
        self + 1
    }
    fn decr(&self) -> u16 {
        self - 1
    }
}

impl MyMath<u32> for u32 {
    fn global_max() -> u32 {
        std::u32::MAX
    }
    fn global_min() -> u32 {
        std::u32::MIN
    }

    fn incr(&self) -> u32 {
        self + 1
    }
    fn decr(&self) -> u32 {
        self - 1
    }
}

impl MyMath<u64> for u64 {
    fn global_max() -> u64 {
        std::u64::MAX
    }
    fn global_min() -> u64 {
        std::u64::MIN
    }

    fn incr(&self) -> u64 {
        self + 1
    }
    fn decr(&self) -> u64 {
        self - 1
    }
}

/// A `Range<T>` is a range of values of type T. Given this range:
///
/// ```ignore
/// Range {
///     first: 0,
///     last: 3
/// }
/// ```
///
/// This represents the inclusive range `[0, 3]`. (Where '[' and ']' represent
/// inclusive contains.) More specifically: it is equivalent to `0, 1, 2, 3`.
///
/// ---
///
/// `Range<T>`s are only useful in cases in which `T` is an integer. AKA when
/// `T` has discrete values. Because Sway does not have floats, this means that
/// `Range<T>` can be used for all numeric and integer Sway types.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Range<T>
where
    T: Debug
        + fmt::Display
        + Eq
        + Ord
        + PartialEq
        + PartialOrd
        + Clone
        + MyMath<T>
        + Sub<Output = T>
        + Into<u64>,
{
    first: T,
    last: T,
}

impl Range<u8> {
    fn u8() -> Range<u8> {
        Range {
            first: std::u8::MIN,
            last: std::u8::MAX,
        }
    }
}

impl Range<u16> {
    fn u16() -> Range<u16> {
        Range {
            first: std::u16::MIN,
            last: std::u16::MAX,
        }
    }
}

impl Range<u32> {
    fn u32() -> Range<u32> {
        Range {
            first: std::u32::MIN,
            last: std::u32::MAX,
        }
    }
}

impl Range<u64> {
    fn u64() -> Range<u64> {
        Range {
            first: std::u64::MIN,
            last: std::u64::MAX,
        }
    }
}

impl<T> Range<T>
where
    T: Debug
        + fmt::Display
        + Eq
        + Ord
        + PartialEq
        + PartialOrd
        + Clone
        + MyMath<T>
        + Sub<Output = T>
        + Into<u64>,
{
    /// Creates a `Range<T>` from a single value of type `T`, where the value is used
    /// both as the lower inclusive contains and the upper inclusive contains.
    fn from_single(x: T) -> Range<T> {
        Range {
            first: x.clone(),
            last: x,
        }
    }

    /// Creates a `Range<T>` and ensures that it is a "valid `Range<T>`"
    /// (i.e.) that `first` is <= to `last`
    fn from_double(first: T, last: T, span: &Span) -> CompileResult<Range<T>> {
        let warnings = vec![];
        let mut errors = vec![];
        if last < first {
            errors.push(CompileError::ExhaustivityCheckingAlgorithmFailure(
                "attempted to create an invalid range",
                span.clone(),
            ));
            err(warnings, errors)
        } else {
            ok(Range { first, last }, warnings, errors)
        }
    }

    /// Combines two ranges that overlap. There are 6 ways
    /// in which this might be the case:
    ///
    /// ```ignore
    /// A: |------------|
    /// B:    |------|
    /// -> |------------|
    ///
    /// A:   |------|
    /// B: |------------|
    /// -> |------------|
    ///
    /// A: |---------|
    /// B:      |---------|
    /// -> |--------------|
    ///
    /// A:      |---------|
    /// B: |---------|
    /// -> |--------------|
    ///
    /// A: |------|
    /// B:         |------|
    /// -> |--------------|
    ///
    /// A:         |------|
    /// B: |------|
    /// -> |--------------|
    /// ```
    ///
    /// ---
    ///
    /// Note that becaues `Range<T>` relies on the assumption that `T` is an
    /// integer value, this algorithm joins `Range<T>`s that are within ± 1 of
    /// one another. Given these two `Range<T>`s:
    ///
    /// ```ignore
    /// Range {
    ///     first: 0,
    ///     last: 3
    /// }
    /// Range {
    ///     first: 4,
    ///     last: 7
    /// }
    /// ```
    ///
    /// They can be joined into this `Range<T>`:
    ///
    /// ```ignore
    /// Range {
    ///     first: 0,
    ///     last: 7
    /// }
    /// ```
    fn join_ranges(a: &Range<T>, b: &Range<T>, span: &Span) -> CompileResult<Range<T>> {
        let mut warnings = vec![];
        let mut errors = vec![];
        if !a.overlaps(b) && !a.within_one(b) {
            errors.push(CompileError::ExhaustivityCheckingAlgorithmFailure(
                "these two ranges cannot be joined",
                span.clone(),
            ));
            err(warnings, errors)
        } else {
            let first = if a.first < b.first {
                a.first.clone()
            } else {
                b.first.clone()
            };
            let last = if a.last > b.last {
                a.last.clone()
            } else {
                b.last.clone()
            };
            let range = check!(
                Range::from_double(first, last, span),
                return err(warnings, errors),
                warnings,
                errors
            );
            ok(range, warnings, errors)
        }
    }

    /// Condenses a `Vec<Range<T>>` to a `Vec<Range<T>>` of ordered, distinct,
    /// non-overlapping ranges.
    ///
    /// Modeled after the algorithm here: https://www.geeksforgeeks.org/merging-intervals/
    ///
    /// 1. Sort the intervals based on increasing order of starting time.
    /// 2. Push the first interval on to a stack.
    /// 3. For each interval do the following
    ///     3a. If the current interval does not overlap with the stack
    ///         top, push it.
    ///     3b. If the current interval overlaps with stack top (or is within ± 1)
    ///         and ending time of current interval is more than that of stack top,
    ///         update stack top with the ending time of current interval.
    /// 4. At the end stack contains the merged intervals.
    fn condense_ranges(ranges: Vec<Range<T>>, span: &Span) -> CompileResult<Vec<Range<T>>> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let mut ranges = ranges;
        let mut stack: Vec<Range<T>> = vec![];

        // 1. Sort the intervals based on increasing order of starting time.
        ranges.sort_by(|a, b| b.first.cmp(&a.first));

        // 2. Push the first interval on to a stack.
        let (first, rest) = match ranges.split_first() {
            Some((first, rest)) => (first.to_owned(), rest.to_owned()),
            None => {
                errors.push(CompileError::ExhaustivityCheckingAlgorithmFailure(
                    "unable to split vec",
                    span.clone(),
                ));
                return err(warnings, errors);
            }
        };
        stack.push(first);

        for range in rest.iter() {
            let top = match stack.pop() {
                Some(top) => top,
                None => {
                    errors.push(CompileError::ExhaustivityCheckingAlgorithmFailure(
                        "stack empty",
                        span.clone(),
                    ));
                    return err(warnings, errors);
                }
            };
            if range.overlaps(&top) || range.within_one(&top) {
                // 3b. If the current interval overlaps with stack top (or is within ± 1)
                //     and ending time of current interval is more than that of stack top,
                //     update stack top with the ending time of current interval.
                stack.push(check!(
                    Range::join_ranges(range, &top, span),
                    return err(warnings, errors),
                    warnings,
                    errors
                ));
            } else {
                // 3a. If the current interval does not overlap with the stack
                //     top, push it.
                stack.push(top);
                stack.push(range.clone());
            }
        }
        stack.reverse();
        ok(stack, warnings, errors)
    }

    /// Given an *oracle* `Range<T>` and a vec *guides* of `Range<T>`, this
    /// function returns the subdivided `Range<T>`s that are both within
    /// *oracle* not within *guides*.
    ///
    /// The steps are as follows:
    ///
    /// 1. Convert *guides* to a vec of ordered, distinct, non-overlapping
    ///    ranges *guides*'
    /// 2. Check to ensure that *oracle* fully encompasses all ranges in
    ///    *guides*'. For example, this would pass the check:
    ///    ```ignore
    ///    oracle: |--------------|
    ///    guides:   |--|  |--|
    ///    ```
    ///    But this would not:
    ///    ```ignore
    ///    oracle: |--------------|
    ///    guides:   |--|  |--| |---|
    ///    ```
    /// 3. Given the *oracle* range `[a, b]` and the *guides*'₀ range of
    ///    `[c, d]`, and `a != c`, construct a range of `[a, c]`.
    /// 4. Given *guides*' of length *n*, for every *k* 0..*n-1*, find the
    ///    *guides*'ₖ range of `[a,b]` and the *guides*'ₖ₊₁ range of `[c, d]`,
    ///    construct a range of `[b, c]`. You can assume that `b != d` because
    ///    of step (1)
    /// 5. Given the *oracle* range of `[a, b]`, *guides*' of length *n*, and
    ///    the *guides*'ₙ range of `[c, d]`, and `b != d`, construct a range of
    ///    `[b, d]`.
    /// 6. Combine the range given from step (3), the ranges given from step
    ///    (4), and the range given from step (5) for your result.
    fn find_exclusionary_ranges(
        guides: Vec<Range<T>>,
        oracle: Range<T>,
        span: &Span,
    ) -> CompileResult<Vec<Range<T>>> {
        let mut warnings = vec![];
        let mut errors = vec![];

        // 1. Convert *guides* to a vec of ordered, distinct, non-overlapping
        //    ranges *guides*'
        let condensed = check!(
            Range::condense_ranges(guides, span),
            return err(warnings, errors),
            warnings,
            errors
        );

        // 2. Check to ensure that *oracle* fully encompasses all ranges in
        //    *guides*'.
        if !oracle.encompasses_all(&condensed) {
            errors.push(CompileError::ExhaustivityCheckingAlgorithmFailure(
                "ranges OOB with the oracle",
                span.clone(),
            ));
            return err(warnings, errors);
        }

        // 3. Given the *oracle* range `[a, b]` and the *guides*'₀ range of
        //    `[c, d]`, and `a != c`, construct a range of `[a, c]`.
        let mut exclusionary = vec![];
        let (first, last) = match (condensed.split_first(), condensed.split_last()) {
            (Some((first, _)), Some((last, _))) => (first, last),
            _ => {
                errors.push(CompileError::ExhaustivityCheckingAlgorithmFailure(
                    "could not split vec",
                    span.clone(),
                ));
                return err(warnings, errors);
            }
        };
        if oracle.first != first.first {
            exclusionary.push(check!(
                Range::from_double(oracle.first.clone(), first.first.decr(), span),
                return err(warnings, errors),
                warnings,
                errors
            ));
        }

        // 4. Given *guides*' of length *n*, for every *k* 0..*n-1*, find the
        //    *guides*'ₖ range of `[a,b]` and the *guides*'ₖ₊₁ range of `[c, d]`,
        //    construct a range of `[b, c]`. You can assume that `b != d` because
        //    of step (1)
        for (left, right) in condensed.iter().tuple_windows() {
            exclusionary.push(check!(
                Range::from_double(left.last.incr(), right.first.decr(), span),
                return err(warnings, errors),
                warnings,
                errors
            ));
        }

        // 5. Given the *oracle* range of `[a, b]`, *guides*' of length *n*, and
        //    the *guides*'ₙ range of `[c, d]`, and `b != d`, construct a range of
        //    `[b, d]`.
        if oracle.last != last.last {
            exclusionary.push(check!(
                Range::from_double(last.last.incr(), oracle.last, span),
                return err(warnings, errors),
                warnings,
                errors
            ));
        }

        // 6. Combine the range given from step (3), the ranges given from step
        //    (4), and the range given from step (5) for your result.
        ok(exclusionary, warnings, errors)
    }

    /// Condenses a vec of ranges and checks to see if the condensed ranges
    /// equal an oracle range.
    fn do_ranges_equal_range(
        ranges: Vec<Range<T>>,
        oracle: Range<T>,
        span: &Span,
    ) -> CompileResult<bool> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let condensed_ranges = check!(
            Range::condense_ranges(ranges, span),
            return err(warnings, errors),
            warnings,
            errors
        );
        if condensed_ranges.len() > 1 {
            ok(false, warnings, errors)
        } else {
            let first_range = match condensed_ranges.first() {
                Some(first_range) => first_range.clone(),
                _ => {
                    errors.push(CompileError::ExhaustivityCheckingAlgorithmFailure(
                        "vec empty",
                        span.clone(),
                    ));
                    return err(warnings, errors);
                }
            };
            ok(first_range == oracle, warnings, errors)
        }
    }

    /// Checks to see if two ranges overlap. There are 4 ways in which this
    /// might be the case:
    ///
    /// ```ignore
    /// A: |------------|
    /// B:    |------|
    ///
    /// A:    |------|
    /// B: |------------|
    ///
    /// A: |---------|
    /// B:      |---------|
    ///
    /// A:      |---------|
    /// B: |---------|
    /// ```
    fn overlaps(&self, other: &Range<T>) -> bool {
        other.first >= self.first && other.last <= self.last
            || other.first <= self.first && other.last >= self.last
            || other.first <= self.first && other.last <= self.last && other.last >= self.first
            || other.first >= self.first && other.first <= self.last && other.last >= self.last
    }

    /// Checks to see if the first range encompasses the second range. There are
    /// 2 ways in which this might be the case:
    ///
    /// ```ignore
    /// A: |------------|
    /// B:    |------|
    ///
    /// A: |------------|
    /// B: |------------|
    /// ```
    fn encompasses(&self, other: &Range<T>) -> bool {
        self.first <= other.first && self.last >= other.last
    }

    fn encompasses_all(&self, others: &[Range<T>]) -> bool {
        others
            .iter()
            .map(|other| self.encompasses(other))
            .all(|x| x)
    }

    /// Checks to see if two ranges are within ± 1 of one another. There are 2
    /// ways in which this might be the case:
    ///
    /// ```ignore
    /// A: |------|
    /// B:         |------|
    ///
    /// A:         |------|
    /// B: |------|
    /// ```
    fn within_one(&self, other: &Range<T>) -> bool {
        !self.overlaps(other)
            && (other.first > self.last && (other.first.clone() - self.last.clone()).into() == 1u64
                || self.first > other.last
                    && (self.first.clone() - other.last.clone()).into() == 1u64)
    }
}

impl<T> fmt::Display for Range<T>
where
    T: Debug
        + fmt::Display
        + Eq
        + Ord
        + PartialEq
        + PartialOrd
        + Clone
        + MyMath<T>
        + Sub<Output = T>
        + Into<u64>,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut builder = String::new();
        builder.push('[');
        if self.first == T::global_min() {
            builder.push_str("MIN");
        } else {
            builder.push_str(&format!("{}", self.first));
        }
        builder.push_str("...");
        if self.last == T::global_max() {
            builder.push_str("MAX");
        } else {
            builder.push_str(&format!("{}", self.last));
        }
        builder.push(']');
        write!(f, "{}", builder)
    }
}

/// A `Pattern` represents something that could be on the LHS of a match
/// expression arm.
///
/// For instance this match expression:
///
/// ```ignore
/// let x = (0, 5);
/// match x {
///     (0, 1) => true,
///     (2, 3) => true,
///     _ => false
/// }
/// ```
///
/// would result in these patterns:
///
/// ```ignore
/// Pattern::Tuple([
///     Pattern::U64(Range { first: 0, last: 0 }),
///     Pattern::U64(Range { first: 1, last: 1 })
/// ])
/// Pattern::Tuple([
///     Pattern::U64(Range { first: 2, last: 2 }),
///     Pattern::U64(Range { first: 3, last: 3 })
/// ])
/// Pattern::Wildcard
/// ```
///
/// ---
///
/// A `Pattern` is semantically constructed from a "constructor" and its
/// "arguments." Given the `Pattern`:
///
/// ```ignore
/// Pattern::Tuple([
///     Pattern::U64(Range { first: 0, last: 0 }),
///     Pattern::U64(Range { first: 1, last: 1 })
/// ])
/// ```
///
/// the constructor is:
///
/// ```ignore
/// Pattern::Tuple([.., ..])
/// ```
///
/// and the arguments are:
///
/// ```ignore
/// [
///     Pattern::U64(Range { first: 0, last: 0 }),
///     Pattern::U64(Range { first: 1, last: 1 })
/// ]
/// ```
///
/// Given the `Pattern`:
///
/// ```ignore
/// Pattern::U64(Range { first: 0, last: 0 })
/// ```
///
/// the constructor is:
///
/// ```ignore
/// Pattern::U64(Range { first: 0, last: 0 })
/// ```
/// and the arguments are empty. More specifically, in the case of u64 (and
/// other numbers), we can think of u64 as a giant enum, where every u64 value
/// is one variant of the enum, and each of these variants maps to a `Pattern`.
/// So "2u64" can be mapped to a `Pattern` with the constructor "2u64"
/// (represented as a `Range<u64>`) and with empty arguments.
///
/// This idea of a constructor and arguments is used in the match exhaustivity
/// algorithm.
///
/// ---
///
/// The variants of `Pattern` can be semantically categorized into 3 categories:
///
/// 1. the wildcard pattern (Pattern::Wildcard)
/// 2. the or pattern (Pattern::Or(..))
/// 3. constructed patterns (everything else)
///
/// This idea of semantic categorization is used in the match exhaustivity
/// algorithm.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Pattern {
    Wildcard,
    U8(Range<u8>),
    U16(Range<u16>),
    U32(Range<u32>),
    U64(Range<u64>),
    B256([u8; 32]),
    Boolean(bool),
    Byte(Range<u8>),
    Numeric(Range<u64>),
    String(String),
    Struct(StructPattern),
    Tuple(PatStack),
    Or(PatStack),
}

impl Pattern {
    /// Converts a `MatchCondition` to a `Pattern`.
    fn from_match_condition(match_condition: MatchCondition) -> CompileResult<Self> {
        match match_condition {
            MatchCondition::CatchAll(_) => ok(Pattern::Wildcard, vec![], vec![]),
            MatchCondition::Scrutinee(scrutinee) => Pattern::from_scrutinee(scrutinee),
        }
    }

    /// Converts a `Scrutinee` to a `Pattern`.
    fn from_scrutinee(scrutinee: Scrutinee) -> CompileResult<Self> {
        let mut warnings = vec![];
        let mut errors = vec![];
        match scrutinee {
            Scrutinee::Variable { .. } => ok(Pattern::Wildcard, warnings, errors),
            Scrutinee::Literal { value, .. } => match value {
                Literal::U8(x) => ok(Pattern::U8(Range::from_single(x)), warnings, errors),
                Literal::U16(x) => ok(Pattern::U16(Range::from_single(x)), warnings, errors),
                Literal::U32(x) => ok(Pattern::U32(Range::from_single(x)), warnings, errors),
                Literal::U64(x) => ok(Pattern::U64(Range::from_single(x)), warnings, errors),
                Literal::B256(x) => ok(Pattern::B256(x), warnings, errors),
                Literal::Boolean(b) => ok(Pattern::Boolean(b), warnings, errors),
                Literal::Byte(x) => ok(Pattern::Byte(Range::from_single(x)), warnings, errors),
                Literal::Numeric(x) => {
                    ok(Pattern::Numeric(Range::from_single(x)), warnings, errors)
                }
                Literal::String(s) => ok(Pattern::String(s.as_str().to_string()), warnings, errors),
            },
            Scrutinee::StructScrutinee {
                struct_name,
                fields,
                ..
            } => {
                let mut new_fields = vec![];
                for StructScrutineeField {
                    field, scrutinee, ..
                } in fields.into_iter()
                {
                    let f = match scrutinee {
                        Some(scrutinee) => check!(
                            Pattern::from_scrutinee(scrutinee),
                            return err(warnings, errors),
                            warnings,
                            errors
                        ),
                        None => Pattern::Wildcard,
                    };
                    new_fields.push((field.as_str().to_string(), f));
                }
                let pat = Pattern::Struct(StructPattern {
                    struct_name,
                    fields: new_fields,
                });
                ok(pat, warnings, errors)
            }
            Scrutinee::Tuple { elems, .. } => {
                let mut new_elems = PatStack::empty();
                for elem in elems.into_iter() {
                    new_elems.push(check!(
                        Pattern::from_scrutinee(elem),
                        return err(warnings, errors),
                        warnings,
                        errors
                    ));
                }
                ok(Pattern::Tuple(new_elems), warnings, errors)
            }
            Scrutinee::Unit { span } => {
                errors.push(CompileError::Unimplemented(
                    "unit exhaustivity checking",
                    span,
                ));
                err(warnings, errors)
            }
            Scrutinee::EnumScrutinee { span, .. } => {
                errors.push(CompileError::Unimplemented(
                    "enum exhaustivity checking",
                    span,
                ));
                err(warnings, errors)
            }
        }
    }

    /// Converts a `PatStack` to a `Pattern`. If the `PatStack` is of lenth 1,
    /// this function returns the single element, if it is of length > 1, this
    /// function wraps the provided `PatStack` in a `Pattern::Or(..)`.
    fn from_pat_stack(pat_stack: PatStack, span: &Span) -> CompileResult<Pattern> {
        if pat_stack.len() == 1 {
            pat_stack.first(span)
        } else {
            ok(Pattern::Or(pat_stack), vec![], vec![])
        }
    }

    /// Given a `Pattern` *c* and a `PatStack` *args*, extracts the constructor
    /// from *c* and applies it to *args*. For example, given:
    ///
    /// ```ignore
    /// c:    Pattern::Tuple([
    ///         Pattern::U64(Range { first: 5, last: 7, }),
    ///         Pattern::U64(Range { first: 10, last: 12 })
    ///       ])
    /// args: [
    ///         Pattern::U64(Range { first: 0, last: 0 }),
    ///         Pattern::U64(Range { first: 1, last: 1 })
    ///       ]
    /// ```
    ///
    /// the extracted constructor *ctor* from *c* would be:
    ///
    /// ```ignore
    /// Pattern::Tuple([.., ..])
    /// ```
    ///
    /// Applying *args* to *ctor* would give:
    ///
    /// ```ignore
    /// Pattern::Tuple([
    ///     Pattern::U64(Range { first: 0, last: 0 }),
    ///     Pattern::U64(Range { first: 1, last: 1 })
    /// ])
    /// ```
    ///
    /// ---
    ///
    /// If if is the case that at lease one element of *args* is a
    /// or-pattern, then *args* is first "serialized". Meaning, that all
    /// or-patterns are extracted to create a vec of `PatStack`s *args*' where
    /// each `PatStack` is a copy of *args* where the index of the or-pattern is
    /// instead replaced with one element from the or-patterns contents. More
    /// specifically, given an *args* with one or-pattern that contains n
    /// elements, this "serialization" would result in *args*' of length n.
    /// Given an *args* with two or-patterns that contain n elements and m
    /// elements, this would result in *args*' of length n*m.
    ///
    /// Once *args*' is constructed, *ctor* is applied to every element of
    /// *args*' and the resulting `Pattern`s are wrapped inside of an
    /// or-pattern.
    ///
    /// For example, given:
    ///
    /// ```ignore
    /// ctor: Pattern::Tuple([.., ..])
    /// args: [
    ///         Pattern::Or([
    ///             Pattern::U64(Range { first: 0, last: 0 }),
    ///             Pattern::U64(Range { first: 1, last: 1 })
    ///         ]),
    ///         Pattern::Wildcard
    ///       ]
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
    /// applying *ctor* would create:
    ///
    /// ```ignore
    /// [
    ///     Pattern::Tuple([
    ///         Pattern::U64(Range { first: 0, last: 0 }),
    ///         Pattern::Wildcard
    ///     ]),
    ///     Pattern::Tuple([
    ///         Pattern::U64(Range { first: 1, last: 1 }),
    ///         Pattern::Wildcard
    ///     ]),
    /// ]
    /// ```
    ///
    /// and wrapping this in an or-pattern would create:
    ///
    /// ```ignore
    /// Pattern::Or([
    ///     Pattern::Tuple([
    ///         Pattern::U64(Range { first: 0, last: 0 }),
    ///         Pattern::Wildcard
    ///     ]),
    ///     Pattern::Tuple([
    ///         Pattern::U64(Range { first: 1, last: 1 }),
    ///         Pattern::Wildcard
    ///     ]),
    /// ])
    /// ```
    fn from_constructor_and_arguments(
        c: &Pattern,
        args: PatStack,
        span: &Span,
    ) -> CompileResult<Self> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let pat = match c {
            Pattern::Wildcard => unreachable!(),
            Pattern::U8(range) => {
                if !args.is_empty() {
                    errors.push(CompileError::ExhaustivityCheckingAlgorithmFailure(
                        "malformed constructor request",
                        span.clone(),
                    ));
                    return err(warnings, errors);
                }
                Pattern::U8(range.clone())
            }
            Pattern::U16(range) => {
                if !args.is_empty() {
                    errors.push(CompileError::ExhaustivityCheckingAlgorithmFailure(
                        "malformed constructor request",
                        span.clone(),
                    ));
                    return err(warnings, errors);
                }
                Pattern::U16(range.clone())
            }
            Pattern::U32(range) => {
                if !args.is_empty() {
                    errors.push(CompileError::ExhaustivityCheckingAlgorithmFailure(
                        "malformed constructor request",
                        span.clone(),
                    ));
                    return err(warnings, errors);
                }
                Pattern::U32(range.clone())
            }
            Pattern::U64(range) => {
                if !args.is_empty() {
                    errors.push(CompileError::ExhaustivityCheckingAlgorithmFailure(
                        "malformed constructor request",
                        span.clone(),
                    ));
                    return err(warnings, errors);
                }
                Pattern::U64(range.clone())
            }
            Pattern::B256(b) => {
                if !args.is_empty() {
                    errors.push(CompileError::ExhaustivityCheckingAlgorithmFailure(
                        "malformed constructor request",
                        span.clone(),
                    ));
                    return err(warnings, errors);
                }
                Pattern::B256(*b)
            }
            Pattern::Boolean(b) => {
                if !args.is_empty() {
                    errors.push(CompileError::ExhaustivityCheckingAlgorithmFailure(
                        "malformed constructor request",
                        span.clone(),
                    ));
                    return err(warnings, errors);
                }
                Pattern::Boolean(*b)
            }
            Pattern::Byte(range) => {
                if !args.is_empty() {
                    errors.push(CompileError::ExhaustivityCheckingAlgorithmFailure(
                        "malformed constructor request",
                        span.clone(),
                    ));
                    return err(warnings, errors);
                }
                Pattern::Byte(range.clone())
            }
            Pattern::Numeric(range) => {
                if !args.is_empty() {
                    errors.push(CompileError::ExhaustivityCheckingAlgorithmFailure(
                        "malformed constructor request",
                        span.clone(),
                    ));
                    return err(warnings, errors);
                }
                Pattern::Numeric(range.clone())
            }
            Pattern::String(s) => {
                if !args.is_empty() {
                    errors.push(CompileError::ExhaustivityCheckingAlgorithmFailure(
                        "malformed constructor request",
                        span.clone(),
                    ));
                    return err(warnings, errors);
                }
                Pattern::String(s.clone())
            }
            Pattern::Struct(struct_pattern) => {
                if args.len() != struct_pattern.fields.len() {
                    errors.push(CompileError::ExhaustivityCheckingAlgorithmFailure(
                        "malformed constructor request",
                        span.clone(),
                    ));
                    return err(warnings, errors);
                }
                let pats: PatStack = check!(
                    args.serialize_multi_patterns(span),
                    return err(warnings, errors),
                    warnings,
                    errors
                )
                .into_iter()
                .map(|args| {
                    Pattern::Struct(StructPattern {
                        struct_name: struct_pattern.struct_name.clone(),
                        fields: struct_pattern
                            .fields
                            .iter()
                            .zip(args.into_iter())
                            .map(|((name, _), arg)| (name.clone(), arg))
                            .collect::<Vec<_>>(),
                    })
                })
                .collect::<Vec<_>>()
                .into();
                check!(
                    Pattern::from_pat_stack(pats, span),
                    return err(warnings, errors),
                    warnings,
                    errors
                )
            }
            Pattern::Tuple(elems) => {
                if elems.len() != args.len() {
                    errors.push(CompileError::ExhaustivityCheckingAlgorithmFailure(
                        "malformed constructor request",
                        span.clone(),
                    ));
                    return err(warnings, errors);
                }
                let pats: PatStack = check!(
                    args.serialize_multi_patterns(span),
                    return err(warnings, errors),
                    warnings,
                    errors
                )
                .into_iter()
                .map(Pattern::Tuple)
                .collect::<Vec<_>>()
                .into();
                check!(
                    Pattern::from_pat_stack(pats, span),
                    return err(warnings, errors),
                    warnings,
                    errors
                )
            }
            Pattern::Or(_) => unreachable!(),
        };
        ok(pat, warnings, errors)
    }

    /// Create a `Pattern::Wildcard`
    fn wild_pattern() -> Self {
        Pattern::Wildcard
    }

    /// Finds the "a value" of the `Pattern`, AKA the number of sub-patterns
    /// used in the pattern's constructor. For example, the pattern
    /// `Pattern::Tuple([.., ..])` would have an "a value" of 2.
    fn a(&self) -> usize {
        match self {
            Pattern::U8(_) => 0,
            Pattern::U16(_) => 0,
            Pattern::U32(_) => 0,
            Pattern::U64(_) => 0,
            Pattern::B256(_) => 0,
            Pattern::Boolean(_) => 0,
            Pattern::Byte(_) => 0,
            Pattern::Numeric(_) => 0,
            Pattern::String(_) => 0,
            Pattern::Struct(StructPattern { fields, .. }) => fields.len(),
            Pattern::Tuple(elems) => elems.len(),
            Pattern::Wildcard => unreachable!(),
            Pattern::Or(_) => unreachable!(),
        }
    }

    /// Checks to see if two `Pattern` have the same constructor. For example,
    /// given the patterns:
    ///
    /// ```ignore
    /// A: Pattern::U64(Range { first: 0, last: 0 })
    /// B: Pattern::U64(Range { first: 0, last: 0 })
    /// C: Pattern::U64(Range { first: 1, last: 1 })
    /// ```
    ///
    /// A and B have the same constructor but A and C do not.
    ///
    /// Given the patterns:
    ///
    /// ```ignore
    /// A: Pattern::Tuple([
    ///     Pattern::U64(Range { first: 0, last: 0 }),
    ///     Pattern::U64(Range { first: 1, last: 1 }),
    ///    ])
    /// B: Pattern::Tuple([
    ///     Pattern::U64(Range { first: 2, last: 2 }),
    ///     Pattern::U64(Range { first: 3, last: 3 }),
    ///    ])
    /// C: Pattern::Tuple([
    ///     Pattern::U64(Range { first: 4, last: 4 }),
    ///    ])
    /// ```
    ///
    /// A and B have the same constructor but A and C do not.
    fn has_the_same_constructor(&self, other: &Pattern) -> bool {
        match (self, other) {
            (Pattern::Wildcard, Pattern::Wildcard) => true,
            (Pattern::U8(a), Pattern::U8(b)) => a == b,
            (Pattern::U16(a), Pattern::U16(b)) => a == b,
            (Pattern::U32(a), Pattern::U32(b)) => a == b,
            (Pattern::U64(a), Pattern::U64(b)) => a == b,
            (Pattern::B256(x), Pattern::B256(y)) => x == y,
            (Pattern::Boolean(x), Pattern::Boolean(y)) => x == y,
            (Pattern::Byte(a), Pattern::Byte(b)) => a == b,
            (Pattern::Numeric(a), Pattern::Numeric(b)) => a == b,
            (Pattern::String(x), Pattern::String(y)) => x == y,
            (
                Pattern::Struct(StructPattern {
                    struct_name: struct_name1,
                    fields: fields1,
                }),
                Pattern::Struct(StructPattern {
                    struct_name: struct_name2,
                    fields: fields2,
                }),
            ) => struct_name1 == struct_name2 && fields1.len() == fields2.len(),
            (Pattern::Tuple(elems1), Pattern::Tuple(elems2)) => elems1.len() == elems2.len(),
            (Pattern::Or(_), Pattern::Or(_)) => unreachable!(),
            _ => false,
        }
    }

    /// Extracts the "sub-patterns" of a `Pattern`, aka the "arguments" to the
    /// patterns "constructor". Some patterns have 0 sub-patterns and some
    /// patterns have >0 sub-patterns. For example, this pattern:
    ///
    /// ```ignore
    /// Pattern::U64(Range { first: 0, last: 0 }),
    /// ```
    ///
    /// has 0 sub-patterns. While this pattern:
    ///
    /// ```ignore
    /// Pattern::Tuple([
    ///     Pattern::U64(Range { first: 0, last: 0 }),
    ///     Pattern::U64(Range { first: 1, last: 1 })
    /// ])
    /// ```
    ///
    /// has 2 sub-patterns:
    ///
    /// ```ignore
    /// [
    ///     Pattern::U64(Range { first: 0, last: 0 }),
    ///     Pattern::U64(Range { first: 1, last: 1 })
    /// ]
    /// ```
    fn sub_patterns(&self, span: &Span) -> CompileResult<PatStack> {
        let warnings = vec![];
        let mut errors = vec![];
        let pats = match self {
            Pattern::Struct(StructPattern { fields, .. }) => fields
                .iter()
                .map(|(_, field)| field.to_owned())
                .collect::<Vec<_>>()
                .into(),
            Pattern::Tuple(elems) => elems.to_owned(),
            _ => PatStack::empty(),
        };
        if self.a() != pats.len() {
            errors.push(CompileError::ExhaustivityCheckingAlgorithmFailure(
                "invariant self.a() == pats.len() broken",
                span.clone(),
            ));
            return err(warnings, errors);
        }
        ok(pats, warnings, errors)
    }

    /// Flattens a `Pattern` into a `PatStack`. If the pattern is an
    /// "or-pattern", return its contents, otherwise return the pattern as a
    /// `PatStack`
    fn flatten(&self) -> PatStack {
        match self {
            Pattern::Or(pats) => pats.to_owned(),
            pat => PatStack::from_pattern(pat.to_owned()),
        }
    }
}

impl fmt::Display for Pattern {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Pattern::Wildcard => "_".to_string(),
            Pattern::U8(range) => format!("{}", range),
            Pattern::U16(range) => format!("{}", range),
            Pattern::U32(range) => format!("{}", range),
            Pattern::U64(range) => format!("{}", range),
            Pattern::Numeric(range) => format!("{}", range),
            Pattern::B256(n) => format!("{:#?}", n),
            Pattern::Boolean(b) => format!("{}", b),
            Pattern::Byte(range) => format!("{}", range),
            Pattern::String(s) => s.clone(),
            Pattern::Struct(struct_pattern) => format!("{}", struct_pattern),
            Pattern::Tuple(elems) => {
                let mut builder = String::new();
                builder.push('(');
                builder.push_str(&format!("{}", elems));
                builder.push(')');
                builder
            }
            Pattern::Or(_) => unreachable!(),
        };
        write!(f, "{}", s)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct StructPattern {
    struct_name: Ident,
    fields: Vec<(String, Pattern)>,
}

impl fmt::Display for StructPattern {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut builder = String::new();
        builder.push_str(self.struct_name.as_str());
        builder.push_str(" { ");
        let mut start_of_wildcard_tail = None;
        for (i, (_, pat)) in self.fields.iter().enumerate().rev() {
            match (pat, start_of_wildcard_tail) {
                (Pattern::Wildcard, None) => {}
                (_, None) => start_of_wildcard_tail = Some(i + 1),
                (_, _) => {}
            }
        }
        let s: String = match start_of_wildcard_tail {
            Some(start_of_wildcard_tail) => {
                let (front, _) = self.fields.split_at(start_of_wildcard_tail);
                let mut inner_builder = front
                    .iter()
                    .map(|(name, field)| {
                        let mut inner_builder = String::new();
                        inner_builder.push_str(name);
                        inner_builder.push_str(": ");
                        inner_builder.push_str(&format!("{}", field));
                        inner_builder
                    })
                    .join(", ");
                inner_builder.push_str(", ...");
                inner_builder
            }
            None => self
                .fields
                .iter()
                .map(|(name, field)| {
                    let mut inner_builder = String::new();
                    inner_builder.push_str(name);
                    inner_builder.push_str(": ");
                    inner_builder.push_str(&format!("{}", field));
                    inner_builder
                })
                .join(", "),
        };
        builder.push_str(&s);
        builder.push_str(" }");
        write!(f, "{}", builder)
    }
}

/// A `PatStack` is a `Vec<Pattern>` that is implemented with special methods
/// particular to the match exhaustivity algorithm.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PatStack {
    pats: Vec<Pattern>,
}

impl PatStack {
    /// Creates an empty `PatStack`.
    fn empty() -> Self {
        PatStack { pats: vec![] }
    }

    /// Given a `Pattern` *p*, creates a `PatStack` with one element *p*.
    fn from_pattern(p: Pattern) -> Self {
        PatStack { pats: vec![p] }
    }

    /// Given a usize *n*, creates a `PatStack` filled with *n*
    /// `Pattern::Wildcard` elements.
    fn fill_wildcards(n: usize) -> Self {
        let mut pats = vec![];
        for _ in 0..n {
            pats.push(Pattern::Wildcard);
        }
        PatStack { pats }
    }

    /// Returns the first element of a `PatStack`.
    fn first(&self, span: &Span) -> CompileResult<Pattern> {
        let warnings = vec![];
        let mut errors = vec![];
        match self.pats.first() {
            Some(first) => ok(first.to_owned(), warnings, errors),
            None => {
                errors.push(CompileError::ExhaustivityCheckingAlgorithmFailure(
                    "empty PatStack",
                    span.clone(),
                ));
                err(warnings, errors)
            }
        }
    }

    /// Returns a tuple of the first element of a `PatStack` and the rest of the
    /// elements.
    fn split_first(&self, span: &Span) -> CompileResult<(Pattern, PatStack)> {
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
                errors.push(CompileError::ExhaustivityCheckingAlgorithmFailure(
                    "empty PatStack",
                    span.clone(),
                ));
                err(warnings, errors)
            }
        }
    }

    /// Given a usize *n*, splits the `PatStack` at *n* and returns both halves.
    fn split_at(&self, n: usize, span: &Span) -> CompileResult<(PatStack, PatStack)> {
        let warnings = vec![];
        let mut errors = vec![];
        if n > self.len() {
            errors.push(CompileError::ExhaustivityCheckingAlgorithmFailure(
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
    fn push(&mut self, other: Pattern) {
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
                errors.push(CompileError::ExhaustivityCheckingAlgorithmFailure(
                    "cant retrieve mutable reference to element",
                    span.clone(),
                ));
                err(warnings, errors)
            }
        }
    }

    /// Appends a `PatStack` onto the `PatStack`.
    fn append(&mut self, others: &mut PatStack) {
        self.pats.append(&mut others.pats);
    }

    /// Prepends a `Pattern` onto the `PatStack`.
    fn prepend(&mut self, other: Pattern) {
        self.pats.insert(0, other);
    }

    /// Returns the length of the `PatStack`.
    fn len(&self) -> usize {
        self.pats.len()
    }

    /// Reports if the `PatStack` is empty.
    fn is_empty(&self) -> bool {
        self.flatten().filter_out_wildcards().pats.is_empty()
    }

    /// Reports if the `PatStack` contains a given `Pattern`.
    fn contains(&self, pat: &Pattern) -> bool {
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

    fn iter(&self) -> Iter<'_, Pattern> {
        self.pats.iter()
    }

    fn into_iter(self) -> IntoIter<Pattern> {
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
    fn is_complete_signature(&self, span: &Span) -> CompileResult<bool> {
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
                            errors.push(CompileError::ExhaustivityCheckingAlgorithmFailure(
                                "type mismatch",
                                span.clone(),
                            ));
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
                            errors.push(CompileError::ExhaustivityCheckingAlgorithmFailure(
                                "type mismatch",
                                span.clone(),
                            ));
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
                            errors.push(CompileError::ExhaustivityCheckingAlgorithmFailure(
                                "type mismatch",
                                span.clone(),
                            ));
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
                            errors.push(CompileError::ExhaustivityCheckingAlgorithmFailure(
                                "type mismatch",
                                span.clone(),
                            ));
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
                            errors.push(CompileError::ExhaustivityCheckingAlgorithmFailure(
                                "type mismatch",
                                span.clone(),
                            ));
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
                            errors.push(CompileError::ExhaustivityCheckingAlgorithmFailure(
                                "type mismatch",
                                span.clone(),
                            ));
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
                            errors.push(CompileError::ExhaustivityCheckingAlgorithmFailure(
                                "type mismatch",
                                span.clone(),
                            ));
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
    fn flatten(&self) -> PatStack {
        let mut flattened = PatStack::empty();
        for pat in self.pats.iter() {
            flattened.append(&mut pat.flatten());
        }
        flattened
    }

    /// Returns the given `PatStack` with wildcard patterns filtered out.
    fn filter_out_wildcards(&self) -> PatStack {
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
    fn serialize_multi_patterns(self, span: &Span) -> CompileResult<Vec<PatStack>> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let mut output: Vec<PatStack> = vec![];
        let mut stack: Vec<PatStack> = vec![self];
        while !stack.is_empty() {
            let top = match stack.pop() {
                Some(top) => top,
                None => {
                    errors.push(CompileError::ExhaustivityCheckingAlgorithmFailure(
                        "can't pop Vec",
                        span.clone(),
                    ));
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

/// A `Matrix` is a `Vec<PatStack>` that is implemented with special methods
/// particular to the match exhaustivity algorithm.
///
/// The number of rows of the `Matrix` is equal to the number of `PatStack`s and
/// the number of columns of the `Matrix` is equal to the number of elements in
/// the `PatStack`s. Each `PatStack` should contains the same number of
/// elements.
#[derive(Clone, Debug)]
struct Matrix {
    rows: Vec<PatStack>,
}

impl Matrix {
    /// Creates an empty `Matrix`.
    fn empty() -> Self {
        Matrix { rows: vec![] }
    }

    /// Creates a `Matrix` with one row from a `PatStack`.
    fn from_pat_stack(pat_stack: PatStack) -> Self {
        Matrix {
            rows: vec![pat_stack],
        }
    }

    /// Pushes a `PatStack` onto the `Matrix`.
    fn push(&mut self, row: PatStack) {
        self.rows.push(row);
    }

    /// Appends a `Vec<PatStack>` onto the `Matrix`.
    fn append(&mut self, rows: &mut Vec<PatStack>) {
        self.rows.append(rows);
    }

    /// Returns a reference to the rows of the `Matrix`.
    fn rows(&self) -> &Vec<PatStack> {
        &self.rows
    }

    /// Returns the rows of the `Matrix`.
    fn into_rows(self) -> Vec<PatStack> {
        self.rows
    }

    /// Returns the number of rows *m* and the number of columns *n* of the
    /// `Matrix` in the form (*m*, *n*).
    fn m_n(&self, span: &Span) -> CompileResult<(usize, usize)> {
        let warnings = vec![];
        let mut errors = vec![];
        let first = match self.rows.first() {
            Some(first) => first,
            None => return ok((0, 0), warnings, errors),
        };
        let n = first.len();
        for row in self.rows.iter().skip(1) {
            let l = row.len();
            if l != n {
                errors.push(CompileError::ExhaustivityCheckingAlgorithmFailure(
                    "found invalid matrix size",
                    span.clone(),
                ));
                return err(warnings, errors);
            }
        }
        ok((self.rows.len(), n), warnings, errors)
    }

    /// Reports if the `Matrix` is equivalent to a vector (aka a single
    /// `PatStack`).
    fn is_a_vector(&self) -> bool {
        self.rows.len() == 1
    }

    /// Checks to see if the `Matrix` is a vector, and if it is, returns the
    /// single `PatStack` from its elements.
    fn unwrap_vector(&self, span: &Span) -> CompileResult<PatStack> {
        let warnings = vec![];
        let mut errors = vec![];
        if self.is_a_vector() {
            errors.push(CompileError::ExhaustivityCheckingAlgorithmFailure(
                "found invalid matrix size",
                span.clone(),
            ));
            return err(warnings, errors);
        }
        match self.rows.first() {
            Some(first) => ok(first.clone(), warnings, errors),
            None => ok(PatStack::empty(), warnings, errors),
        }
    }

    /// Computes Σ, where Σ is a `PatStack` containing the first element of
    /// every row of the `Matrix`.
    fn compute_sigma(&self, span: &Span) -> CompileResult<PatStack> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let mut pat_stack = PatStack::empty();
        for row in self.rows.iter() {
            let first = check!(
                row.first(span),
                return err(warnings, errors),
                warnings,
                errors
            );
            pat_stack.push(first)
        }
        ok(pat_stack.flatten().filter_out_wildcards(), warnings, errors)
    }
}

struct ConstructorFactory {}

impl ConstructorFactory {
    fn new(_type_info: TypeInfo) -> Self {
        ConstructorFactory {}
    }

    /// Given Σ, computes a `Pattern` not present in Σ from the type of the
    /// elements of Σ. If more than one `Pattern` is found, these patterns are
    /// wrapped in an or-pattern.
    ///
    /// For example, given this Σ:
    ///
    /// ```ignore
    /// [
    ///     Pattern::U64(Range { first: std::u64::MIN, last: 3 }),
    ///     Pattern::U64(Range { first: 16, last: std::u64::MAX })
    /// ]
    /// ```
    ///
    /// this would result in this `Pattern`:
    ///
    /// ```ignore
    /// Pattern::U64(Range { first: 4, last: 15 })
    /// ```
    ///
    /// Given this Σ (which is more likely to occur than the above example):
    ///
    /// ```ignore
    /// [
    ///     Pattern::U64(Range { first: 2, last: 3 }),
    ///     Pattern::U64(Range { first: 16, last: 17 })
    /// ]
    /// ```
    ///
    /// this would result in this `Pattern`:
    ///
    /// ```ignore
    /// Pattern::Or([
    ///     Pattern::U64(Range { first: std::u64::MIN, last: 1 }),
    ///     Pattern::U64(Range { first: 4, last: 15 }),
    ///     Pattern::U64(Range { first: 18, last: std::u64::MAX })
    /// ])
    /// ```
    fn create_pattern_not_present(&self, sigma: PatStack, span: &Span) -> CompileResult<Pattern> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let (first, rest) = check!(
            sigma.flatten().filter_out_wildcards().split_first(span),
            return err(warnings, errors),
            warnings,
            errors
        );
        let pat = match first {
            Pattern::U8(range) => {
                let mut ranges = vec![range];
                for pat in rest.into_iter() {
                    match pat {
                        Pattern::U8(range) => ranges.push(range),
                        _ => {
                            errors.push(CompileError::ExhaustivityCheckingAlgorithmFailure(
                                "type mismatch",
                                span.clone(),
                            ));
                            return err(warnings, errors);
                        }
                    }
                }
                let unincluded: PatStack = check!(
                    Range::find_exclusionary_ranges(ranges, Range::u8(), span),
                    return err(warnings, errors),
                    warnings,
                    errors
                )
                .into_iter()
                .map(Pattern::U8)
                .collect::<Vec<_>>()
                .into();
                check!(
                    Pattern::from_pat_stack(unincluded, span),
                    return err(warnings, errors),
                    warnings,
                    errors
                )
            }
            Pattern::U16(range) => {
                let mut ranges = vec![range];
                for pat in rest.into_iter() {
                    match pat {
                        Pattern::U16(range) => ranges.push(range),
                        _ => {
                            errors.push(CompileError::ExhaustivityCheckingAlgorithmFailure(
                                "type mismatch",
                                span.clone(),
                            ));
                            return err(warnings, errors);
                        }
                    }
                }
                let unincluded: PatStack = check!(
                    Range::find_exclusionary_ranges(ranges, Range::u16(), span),
                    return err(warnings, errors),
                    warnings,
                    errors
                )
                .into_iter()
                .map(Pattern::U16)
                .collect::<Vec<_>>()
                .into();
                check!(
                    Pattern::from_pat_stack(unincluded, span),
                    return err(warnings, errors),
                    warnings,
                    errors
                )
            }
            Pattern::U32(range) => {
                let mut ranges = vec![range];
                for pat in rest.into_iter() {
                    match pat {
                        Pattern::U32(range) => ranges.push(range),
                        _ => {
                            errors.push(CompileError::ExhaustivityCheckingAlgorithmFailure(
                                "type mismatch",
                                span.clone(),
                            ));
                            return err(warnings, errors);
                        }
                    }
                }
                let unincluded: PatStack = check!(
                    Range::find_exclusionary_ranges(ranges, Range::u32(), span),
                    return err(warnings, errors),
                    warnings,
                    errors
                )
                .into_iter()
                .map(Pattern::U32)
                .collect::<Vec<_>>()
                .into();
                check!(
                    Pattern::from_pat_stack(unincluded, span),
                    return err(warnings, errors),
                    warnings,
                    errors
                )
            }
            Pattern::U64(range) => {
                let mut ranges = vec![range];
                for pat in rest.into_iter() {
                    match pat {
                        Pattern::U64(range) => ranges.push(range),
                        _ => {
                            errors.push(CompileError::ExhaustivityCheckingAlgorithmFailure(
                                "type mismatch",
                                span.clone(),
                            ));
                            return err(warnings, errors);
                        }
                    }
                }
                let unincluded: PatStack = check!(
                    Range::find_exclusionary_ranges(ranges, Range::u64(), span),
                    return err(warnings, errors),
                    warnings,
                    errors
                )
                .into_iter()
                .map(Pattern::U64)
                .collect::<Vec<_>>()
                .into();
                check!(
                    Pattern::from_pat_stack(unincluded, span),
                    return err(warnings, errors),
                    warnings,
                    errors
                )
            }
            Pattern::Numeric(range) => {
                let mut ranges = vec![range];
                for pat in rest.into_iter() {
                    match pat {
                        Pattern::Numeric(range) => ranges.push(range),
                        _ => {
                            errors.push(CompileError::ExhaustivityCheckingAlgorithmFailure(
                                "type mismatch",
                                span.clone(),
                            ));
                            return err(warnings, errors);
                        }
                    }
                }
                let unincluded: PatStack = check!(
                    Range::find_exclusionary_ranges(ranges, Range::u64(), span),
                    return err(warnings, errors),
                    warnings,
                    errors
                )
                .into_iter()
                .map(Pattern::Numeric)
                .collect::<Vec<_>>()
                .into();
                check!(
                    Pattern::from_pat_stack(unincluded, span),
                    return err(warnings, errors),
                    warnings,
                    errors
                )
            }
            // we will not present every string case
            Pattern::String(_) => Pattern::Wildcard,
            Pattern::Wildcard => unreachable!(),
            // we will not present every b256 case
            Pattern::B256(_) => Pattern::Wildcard,
            Pattern::Boolean(b) => {
                let mut true_found = false;
                let mut false_found = false;
                if b {
                    true_found = true;
                } else {
                    false_found = true;
                }
                if rest.contains(&Pattern::Boolean(true)) {
                    true_found = true;
                } else if rest.contains(&Pattern::Boolean(false)) {
                    false_found = true;
                }
                if true_found && false_found {
                    errors.push(CompileError::ExhaustivityCheckingAlgorithmFailure(
                        "unable to create a new pattern",
                        span.clone(),
                    ));
                    return err(warnings, errors);
                } else if true_found {
                    Pattern::Boolean(false)
                } else {
                    Pattern::Boolean(true)
                }
            }
            Pattern::Byte(range) => {
                let mut ranges = vec![range];
                for pat in rest.into_iter() {
                    match pat {
                        Pattern::Byte(range) => ranges.push(range),
                        _ => {
                            errors.push(CompileError::ExhaustivityCheckingAlgorithmFailure(
                                "type mismatch",
                                span.clone(),
                            ));
                            return err(warnings, errors);
                        }
                    }
                }
                let unincluded: PatStack = check!(
                    Range::find_exclusionary_ranges(ranges, Range::u8(), span),
                    return err(warnings, errors),
                    warnings,
                    errors
                )
                .into_iter()
                .map(Pattern::Byte)
                .collect::<Vec<_>>()
                .into();
                check!(
                    Pattern::from_pat_stack(unincluded, span),
                    return err(warnings, errors),
                    warnings,
                    errors
                )
            }
            Pattern::Struct(struct_pattern) => Pattern::Struct(StructPattern {
                struct_name: struct_pattern.struct_name,
                fields: struct_pattern
                    .fields
                    .into_iter()
                    .map(|(name, _)| (name, Pattern::Wildcard))
                    .collect::<Vec<_>>(),
            }),
            Pattern::Tuple(elems) => Pattern::Tuple(PatStack::fill_wildcards(elems.len())),
            Pattern::Or(_) => unreachable!(),
        };
        ok(pat, warnings, errors)
    }
}

/// Algorithm modeled after this paper:
/// http://moscova.inria.fr/%7Emaranget/papers/warn/warn004.html
/// and resembles the one here:
/// https://doc.rust-lang.org/nightly/nightly-rustc/rustc_mir_build/thir/pattern/usefulness/index.html
pub(crate) fn check_match_expression_usefulness(
    type_info: TypeInfo,
    arms: Vec<MatchCondition>,
    span: Span,
) -> CompileResult<(WitnessReport, Vec<(MatchCondition, bool)>)> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let mut matrix = Matrix::empty();
    let mut arms_reachability = vec![];
    let factory = ConstructorFactory::new(type_info);
    match arms.split_first() {
        Some((first_arm, arms_rest)) => {
            let pat = check!(
                Pattern::from_match_condition(first_arm.clone()),
                return err(warnings, errors),
                warnings,
                errors
            );
            matrix.push(PatStack::from_pattern(pat));
            arms_reachability.push((first_arm.clone(), true));
            for arm in arms_rest.iter() {
                let pat = check!(
                    Pattern::from_match_condition(arm.clone()),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                let v = PatStack::from_pattern(pat);
                let witness_report = check!(
                    is_useful(&factory, &matrix, &v, &span),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                matrix.push(v);
                // if an arm has witnesses to its usefulness then it is reachable
                arms_reachability.push((arm.clone(), witness_report.has_witnesses()));
            }
        }
        None => {
            errors.push(CompileError::ExhaustivityCheckingAlgorithmFailure(
                "empty match arms",
                span,
            ));
            return err(warnings, errors);
        }
    }
    let v = PatStack::from_pattern(Pattern::wild_pattern());
    let witness_report = check!(
        is_useful(&factory, &matrix, &v, &span),
        return err(warnings, errors),
        warnings,
        errors
    );
    // if a wildcard case has no witnesses to its usefulness, then the match arms are exhaustive
    ok((witness_report, arms_reachability), warnings, errors)
}

/// Given a `Matrix` *P* and a `PatStack` *q*, computes a `WitnessReport` from
/// algorithm *U(P, q)*.
///
/// This recursive algorithm is basically an induction proof with 2 base cases.
/// The first base case is when *P* is the empty `Matrix`. In this case, we
/// return a witness report where the witnesses are wildcard patterns for every
/// element of *q*. The second base case is when *P* has at least one row but
/// does not have any columns. In this case, we return a witness report with no
/// witnesses. This case indicates exhaustivity. The induction case covers
/// everything else, and what we do for induction depends on what the first
/// element of *q* is. Depending on if the first element of *q* is a wildcard
/// pattern, or-pattern, or constructed pattern we do something different. Each
/// case returns a witness report that we propogate through the recursive steps.
fn is_useful(
    factory: &ConstructorFactory,
    p: &Matrix,
    q: &PatStack,
    span: &Span,
) -> CompileResult<WitnessReport> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let (m, n) = check!(p.m_n(span), return err(warnings, errors), warnings, errors);
    match (m, n) {
        (0, 0) => ok(
            WitnessReport::Witnesses(PatStack::fill_wildcards(q.len())),
            warnings,
            errors,
        ),
        (_, 0) => ok(WitnessReport::NoWitnesses, warnings, errors),
        (_, _) => {
            let c = check!(
                q.first(span),
                return err(warnings, errors),
                warnings,
                errors
            );
            let witness_report = match c {
                Pattern::Wildcard => check!(
                    is_useful_wildcard(factory, p, q, span),
                    return err(warnings, errors),
                    warnings,
                    errors
                ),
                Pattern::Or(pats) => check!(
                    is_useful_or(factory, p, q, pats, span),
                    return err(warnings, errors),
                    warnings,
                    errors
                ),
                c => check!(
                    is_useful_constructed(factory, p, q, c, span),
                    return err(warnings, errors),
                    warnings,
                    errors
                ),
            };
            ok(witness_report, warnings, errors)
        }
    }
}

/// Computes a witness report from *U(P, q)* when *q* is a wildcard pattern.
///
/// Because *q* is a wildcard pattern, this means we are checking to see if the
/// wildcard pattern is useful given *P*. We can do this by investigating the
/// first column Σ of *P*. If Σ is a complete signature (that is if Σ contains
/// every constructor for the type of elements in Σ), then we can recursively
/// compute the witnesses for every element of Σ and aggregate them. If Σ is not
/// a complete signature, then we can compute the default `Matrix` for *P* (i.e.
/// a version of *P* that is agnostic to *c*) and recursively compute the
/// witnesses for if q is useful given the new default `Matrix`.
///
/// ---
///
/// 1. Compute Σ = {c₁, ... , cₙ}, which is the set of constructors that appear
///    as root constructors of the patterns of *P*'s first column.
/// 2. Determine if Σ is a complete signature.
/// 3. If it is a complete signature:
///     1. For every every *k* 0..*n*, compute the specialized `Matrix`
///        *S(cₖ, P)*
///     2. Compute the specialized `Matrix` *S(cₖ, q)*
///     3. Recursively compute U(S(cₖ, P), S(cₖ, q))
///     4. If the recursive call to (3.3) returns a non-empty witness report,
///        create a new pattern from *cₖ* and the witness report and a create a
///        new witness report from the elements not used to create the new
///        pattern
///     5. Aggregate a new patterns and new witness reports from every call of
///        (3.4)
///     6. Transform the aggregated patterns from (3.5) into a single pattern
///        and prepend it to the aggregated witness report
///     7. Return the witness report
/// 4. If it is not a complete signature:
///     1. Compute the default `Matrix` *D(P)*
///     2. Compute *q'* as \[q₂ ... qₙ*\].
///     3. Recursively compute *U(D(P), q')*.
///     4. If Σ is empty, create a pattern not present in Σ
///     5. Add this new pattern to the resulting witness report
///     6. Return the witness report
fn is_useful_wildcard(
    factory: &ConstructorFactory,
    p: &Matrix,
    q: &PatStack,
    span: &Span,
) -> CompileResult<WitnessReport> {
    let mut warnings = vec![];
    let mut errors = vec![];

    // 1. Compute Σ = {c₁, ... , cₙ}, which is the set of constructors that appear
    //    as root constructors of the patterns of *P*'s first column.
    let sigma = check!(
        p.compute_sigma(span),
        return err(warnings, errors),
        warnings,
        errors
    );

    // 2. Determine if Σ is a complete signature.
    let is_complete_signature = check!(
        sigma.is_complete_signature(span),
        return err(warnings, errors),
        warnings,
        errors
    );
    if is_complete_signature {
        // 3. If it is a complete signature:

        let mut witness_report = WitnessReport::NoWitnesses;
        let mut pat_stack = PatStack::empty();
        for c_k in sigma.iter() {
            //     3.1. For every every *k* 0..*n*, compute the specialized `Matrix`
            //        *S(cₖ, P)*
            let s_c_k_p = check!(
                compute_specialized_matrix(c_k, p, span),
                return err(warnings, errors),
                warnings,
                errors
            );

            //     3.2. Compute the specialized `Matrix` *S(cₖ, q)*
            let s_c_k_q = check!(
                compute_specialized_matrix(c_k, &Matrix::from_pat_stack(q.clone()), span),
                return err(warnings, errors),
                warnings,
                errors
            );
            let s_c_k_q = check!(
                s_c_k_q.unwrap_vector(span),
                return err(warnings, errors),
                warnings,
                errors
            );

            //     3.3. Recursively compute U(S(cₖ, P), S(cₖ, q))
            let wr = check!(
                is_useful(factory, &s_c_k_p, &s_c_k_q, span),
                return err(warnings, errors),
                warnings,
                errors
            );

            //     3.4. If the recursive call to (3.3) returns a non-empty witness report,
            //        create a new pattern from *cₖ* and the witness report and a create a
            //        new witness report from the elements not used to create the new
            //        pattern
            //     3.5. Aggregate the new patterns and new witness reports from every call of
            //        (3.4)
            match (&witness_report, wr) {
                (WitnessReport::NoWitnesses, WitnessReport::NoWitnesses) => {}
                (WitnessReport::NoWitnesses, wr) => {
                    let (pat, wr) = check!(
                        WitnessReport::split_into_leading_constructor(wr, c_k, span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    if !pat_stack.contains(&pat) {
                        pat_stack.push(pat);
                    }
                    witness_report = wr;
                }
                (_, wr) => {
                    let (pat, _) = check!(
                        WitnessReport::split_into_leading_constructor(wr, c_k, span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    if !pat_stack.contains(&pat) {
                        pat_stack.push(pat);
                    }
                }
            }
        }

        //     3.6. Transform the aggregated patterns from (3.5) into a single pattern
        //        and prepend it to the aggregated witness report
        match &mut witness_report {
            WitnessReport::NoWitnesses => {}
            witness_report => {
                let pat_stack = check!(
                    Pattern::from_pat_stack(pat_stack, span),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                check!(
                    witness_report.add_witness(pat_stack, span),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
            }
        }

        //     7. Return the witness report
        ok(witness_report, warnings, errors)
    } else {
        // 4. If it is not a complete signature:

        //     4.1. Compute the default `Matrix` *D(P)*
        let d_p = check!(
            compute_default_matrix(p, span),
            return err(warnings, errors),
            warnings,
            errors
        );

        //     4.2. Compute *q'* as \[q₂ ... qₙ*\].
        let (_, q_rest) = check!(
            q.split_first(span),
            return err(warnings, errors),
            warnings,
            errors
        );

        //     4.3. Recursively compute *U(D(P), q')*.
        let mut witness_report = check!(
            is_useful(factory, &d_p, &q_rest, span),
            return err(warnings, errors),
            warnings,
            errors
        );

        //     4.4. If Σ is empty, create a pattern not present in Σ
        let witness_to_add = if sigma.is_empty() {
            Pattern::Wildcard
        } else {
            check!(
                factory.create_pattern_not_present(sigma, span),
                return err(warnings, errors),
                warnings,
                errors
            )
        };

        //     4.5. Add this new pattern to the resulting witness report
        match &mut witness_report {
            WitnessReport::NoWitnesses => {}
            witness_report => check!(
                witness_report.add_witness(witness_to_add, span),
                return err(warnings, errors),
                warnings,
                errors
            ),
        }

        //     4.6. Return the witness report
        ok(witness_report, warnings, errors)
    }
}

/// Computes a witness report from *U(P, q)* when *q* is a constructed pattern
/// *c(r₁, ..., rₐ)*.
///
/// Given a specialized `Matrix` that specializes *P* to *c* and another
/// specialized `Matrix` that specializes *q* to *c*, recursively compute if the
/// latter `Matrix` is useful to the former.
///
/// ---
///
/// 1. Extract the specialized `Matrix` *S(c, P)*
/// 2. Extract the specialized `Matrix` *S(c, q)*
/// 3. Recursively compute *U(S(c, P), S(c, q))*
fn is_useful_constructed(
    factory: &ConstructorFactory,
    p: &Matrix,
    q: &PatStack,
    c: Pattern,
    span: &Span,
) -> CompileResult<WitnessReport> {
    let mut warnings = vec![];
    let mut errors = vec![];

    // 1. Extract the specialized `Matrix` *S(c, P)*
    let s_c_p = check!(
        compute_specialized_matrix(&c, p, span),
        return err(warnings, errors),
        warnings,
        errors
    );
    let (s_c_p_m, s_c_p_n) = check!(
        s_c_p.m_n(span),
        return err(warnings, errors),
        warnings,
        errors
    );
    if s_c_p_m > 0 && s_c_p_n != (c.a() + q.len() - 1) {
        errors.push(CompileError::ExhaustivityCheckingAlgorithmFailure(
            "S(c,P) matrix is misshappen",
            span.clone(),
        ));
        return err(warnings, errors);
    }

    // 2. Extract the specialized `Matrix` *S(c, q)*
    let s_c_q = check!(
        compute_specialized_matrix(&c, &Matrix::from_pat_stack(q.clone()), span),
        return err(warnings, errors),
        warnings,
        errors
    );
    let s_c_q = check!(
        s_c_q.unwrap_vector(span),
        return err(warnings, errors),
        warnings,
        errors
    );

    // 3. Recursively compute *U(S(c, P), S(c, q))*
    is_useful(factory, &s_c_p, &s_c_q, span)
}

/// Computes a witness report from *U(P, q)* when *q* is an or-pattern
/// *(r₁ | ... | rₐ)*.
///
/// Compute the witness report for each element of q and aggregate them
/// together.
///
/// ---
///
/// 1. For each *k* 0..*a* compute *q'* as \[*rₖ q₂ ... qₙ*\].
/// 2. Compute the witnesses from *U(P, q')*
/// 3. Aggregate the witnesses from every *U(P, q')*
fn is_useful_or(
    factory: &ConstructorFactory,
    p: &Matrix,
    q: &PatStack,
    pats: PatStack,
    span: &Span,
) -> CompileResult<WitnessReport> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let (_, q_rest) = check!(
        q.split_first(span),
        return err(warnings, errors),
        warnings,
        errors
    );
    let mut p = p.clone();
    let mut witness_report = WitnessReport::Witnesses(PatStack::empty());
    for pat in pats.into_iter() {
        // 1. For each *k* 0..*a* compute *q'* as \[*rₖ q₂ ... qₙ*\].
        let mut v = PatStack::from_pattern(pat);
        v.append(&mut q_rest.clone());

        // 2. Compute the witnesses from *U(P, q')*
        let wr = check!(
            is_useful(factory, &p, &v, span),
            return err(warnings, errors),
            warnings,
            errors
        );
        p.push(v);

        // 3. Aggregate the witnesses from every *U(P, q')*
        witness_report = WitnessReport::join_witness_reports(witness_report, wr);
    }
    ok(witness_report, warnings, errors)
}

/// Given a `Matrix` *P*, constructs the default `Matrix` *D(P). This is done by
/// sequentially computing the rows of *D(P)*.
///
/// Intuition: A default `Matrix` is a transformation upon *P* that "shrinks"
/// the rows of *P* depending on if the row is able to generally match all
/// patterns in a default case.
fn compute_default_matrix(p: &Matrix, span: &Span) -> CompileResult<Matrix> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let mut d_p = Matrix::empty();
    for p_i in p.rows().iter() {
        d_p.append(&mut check!(
            compute_default_matrix_row(p_i, span),
            return err(warnings, errors),
            warnings,
            errors
        ));
    }
    ok(d_p, warnings, errors)
}

/// Given a `PatStack` *pⁱ* from `Matrix` *P*, compute the resulting row of the
/// default `Matrix` *D(P)*.
///
/// A row in the default `Matrix` "shrinks itself" or "eliminates itself"
/// depending on if its possible to make general claims the first element of the
/// row *pⁱ₁*. It is possible to make a general claim *pⁱ₁* when *pⁱ₁* is the
/// wildcard pattern (in which case it could match anything) and when *pⁱ₁* is
/// an or-pattern (in which case we can do recursion while pretending that the
/// or-pattern is itself a `Matrix`). A row "eliminates itself" when *pⁱ₁* is a
/// constructed pattern (in which case it could only make a specific constructed
/// pattern and we could not make any general claims about it).
///
/// ---
///
/// Rows are defined according to the first component of the row:
///
/// 1. *pⁱ₁* is a constructed pattern *c'(r₁, ..., rₐ)*:
///     1. no row is produced
/// 2. *pⁱ₁* is a wildcard pattern:
///     1. the resulting row equals \[pⁱ₂ ... pⁱₙ*\]
/// 3. *pⁱ₁* is an or-pattern *(r₁ | ... | rₐ)*:
///     1. Construct a new `Matrix` *P'*, where given *k* 0..*a*, the rows of
///        *P'* are defined as \[*rₖ pⁱ₂ ... pⁱₙ*\] for every *k*.
///     2. The resulting rows are the rows obtained from calling the recursive
///        *D(P')*
fn compute_default_matrix_row(p_i: &PatStack, span: &Span) -> CompileResult<Vec<PatStack>> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let mut rows: Vec<PatStack> = vec![];
    let (p_i_1, mut p_i_rest) = check!(
        p_i.split_first(span),
        return err(warnings, errors),
        warnings,
        errors
    );
    match p_i_1 {
        Pattern::Wildcard => {
            // 2. *pⁱ₁* is a wildcard pattern:
            //     1. the resulting row equals \[pⁱ₂ ... pⁱₙ*\]
            let mut row = PatStack::empty();
            row.append(&mut p_i_rest);
            rows.push(row);
        }
        Pattern::Or(pats) => {
            // 3. *pⁱ₁* is an or-pattern *(r₁ | ... | rₐ)*:
            //     1. Construct a new `Matrix` *P'*, where given *k* 0..*a*, the rows of
            //        *P'* are defined as \[*rₖ pⁱ₂ ... pⁱₙ*\] for every *k*.
            let mut m = Matrix::empty();
            for pat in pats.iter() {
                let mut m_row = PatStack::from_pattern(pat.clone());
                m_row.append(&mut p_i_rest.clone());
                m.push(m_row);
            }
            //     2. The resulting rows are the rows obtained from calling the recursive
            //        *D(P')*
            let d_p = check!(
                compute_default_matrix(&m, span),
                return err(warnings, errors),
                warnings,
                errors
            );
            rows.append(&mut d_p.into_rows());
        }
        // 1. *pⁱ₁* is a constructed pattern *c'(r₁, ..., rₐ)*:
        //     1. no row is produced
        _ => {}
    }
    ok(rows, warnings, errors)
}

/// Given a constructor *c* and a `Matrix` *P*, constructs the specialized
/// `Matrix` *S(c, P)*. This is done by sequentially computing the rows of
/// *S(c, P)*.
///
/// Intuition: A specialized `Matrix` is a transformation upon *P* that
/// "unwraps" the rows of *P* depending on if they are congruent with *c*.
fn compute_specialized_matrix(c: &Pattern, p: &Matrix, span: &Span) -> CompileResult<Matrix> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let mut s_c_p = Matrix::empty();
    for p_i in p.rows().iter() {
        s_c_p.append(&mut check!(
            compute_specialized_matrix_row(c, p_i, span),
            return err(warnings, errors),
            warnings,
            errors
        ));
    }
    let (m, _) = check!(
        s_c_p.m_n(span),
        return err(warnings, errors),
        warnings,
        errors
    );
    if p.is_a_vector() && m > 1 {
        errors.push(CompileError::ExhaustivityCheckingAlgorithmFailure(
            "S(c,p) must be a vector",
            span.clone(),
        ));
        return err(warnings, errors);
    }
    ok(s_c_p, warnings, errors)
}

/// Given a constructor *c* and a `PatStack` *pⁱ* from `Matrix` *P*, compute the
/// resulting row of the specialized `Matrix` *S(c, P)*.
///
/// Intuition: a row in the specialized `Matrix` "expands itself" or "eliminates
/// itself" depending on if its possible to furthur "drill down" into the
/// elements of *P* given a *c* that we are specializing for. It is possible to
/// "drill down" when the first element of a row of *P* *pⁱ₁* matches *c* (in
/// which case it is possible to "drill down" into the arguments for *pⁱ₁*),
/// when *pⁱ₁* is the wildcard case (in which case it is possible to "drill
/// down" into "fake" arguments for *pⁱ₁* as it does not matter if *c* matches
/// or not), and when *pⁱ₁* is an or-pattern (in which case we can do recursion
/// while pretending that the or-pattern is itself a `Matrix`). A row
/// "eliminates itself" when *pⁱ₁* does not match *c* (in which case it is not
/// possible to "drill down").
///
/// ---
///
/// Rows are defined according to the first component of the row:
///
/// 1. *pⁱ₁* is a constructed pattern *c'(r₁, ..., rₐ)* where *c* == *c'*:
///     1. the resulting row equals \[*r₁ ... rₐ pⁱ₂ ... pⁱₙ*\]
/// 2. *pⁱ₁* is a constructed pattern *c'(r₁, ..., rₐ)* where *c* != *c'*:
///     1. no row is produced
/// 3. *pⁱ₁* is a wildcard pattern and the number of sub-patterns in *c* is *a*:
///     1. the resulting row equals \[*_₁ ... _ₐ pⁱ₂ ... pⁱₙ*\]
/// 4. *pⁱ₁* is an or-pattern *(r₁ | ... | rₐ)*:
///     1. Construct a new `Matrix` *P'* where, given *k* 0..*a*, the rows of
///        *P'* are defined as \[*rₖ pⁱ₂ ... pⁱₙ*\] for every *k*
///     2. The resulting rows are the rows obtained from calling the recursive
///        *S(c, P')*
fn compute_specialized_matrix_row(
    c: &Pattern,
    p_i: &PatStack,
    span: &Span,
) -> CompileResult<Vec<PatStack>> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let mut rows: Vec<PatStack> = vec![];
    let (p_i_1, mut p_i_rest) = check!(
        p_i.split_first(span),
        return err(warnings, errors),
        warnings,
        errors
    );
    match p_i_1 {
        Pattern::Wildcard => {
            // 3. *pⁱ₁* is a wildcard pattern and the number of sub-patterns in *c* is *a*:
            //     3.1. the resulting row equals \[*_₁ ... _ₐ pⁱ₂ ... pⁱₙ*\]
            let mut row: PatStack = PatStack::fill_wildcards(c.a());
            row.append(&mut p_i_rest);
            rows.push(row);
        }
        Pattern::Or(pats) => {
            // 4. *pⁱ₁* is an or-pattern *(r₁ | ... | rₐ)*:
            //     4.1. Construct a new `Matrix` *P'* where, given *k* 0..*a*, the rows of
            //        *P'* are defined as \[*rₖ pⁱ₂ ... pⁱₙ*\] for every *k*
            let mut m = Matrix::empty();
            for pat in pats.iter() {
                let mut m_row = PatStack::from_pattern(pat.clone());
                m_row.append(&mut p_i_rest.clone());
                m.push(m_row);
            }

            //     4.2. The resulting rows are the rows obtained from calling the recursive
            //        *S(c, P')*
            let s_c_p = check!(
                compute_specialized_matrix(c, &m, span),
                return err(warnings, errors),
                warnings,
                errors
            );
            rows.append(&mut s_c_p.into_rows());
        }
        other => {
            if c.has_the_same_constructor(&other) {
                // 1. *pⁱ₁* is a constructed pattern *c'(r₁, ..., rₐ)* where *c* == *c'*:
                //     1.1. the resulting row equals \[*r₁ ... rₐ pⁱ₂ ... pⁱₙ*\]
                let mut row: PatStack = check!(
                    other.sub_patterns(span),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                row.append(&mut p_i_rest);
                rows.push(row);
            }
            // 2. *pⁱ₁* is a constructed pattern *c'(r₁, ..., rₐ)* where *c* != *c'*:
            //     2.1. no row is produced
        }
    }
    ok(rows, warnings, errors)
}
