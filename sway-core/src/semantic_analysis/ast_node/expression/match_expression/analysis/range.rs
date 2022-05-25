use std::{
    cmp::Ordering,
    fmt::{self, Write},
    ops::Sub,
};

use crate::{
    error::{err, ok},
    CompileError, CompileResult,
};
use itertools::Itertools;
use sway_types::Span;

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
    T: fmt::Debug
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
    pub(crate) fn u8() -> Range<u8> {
        Range {
            first: std::u8::MIN,
            last: std::u8::MAX,
        }
    }
}

impl Range<u16> {
    pub(crate) fn u16() -> Range<u16> {
        Range {
            first: std::u16::MIN,
            last: std::u16::MAX,
        }
    }
}

impl Range<u32> {
    pub(crate) fn u32() -> Range<u32> {
        Range {
            first: std::u32::MIN,
            last: std::u32::MAX,
        }
    }
}

impl Range<u64> {
    pub(crate) fn u64() -> Range<u64> {
        Range {
            first: std::u64::MIN,
            last: std::u64::MAX,
        }
    }
}

impl<T> Range<T>
where
    T: fmt::Debug
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
    pub(crate) fn from_single(x: T) -> Range<T> {
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
            errors.push(CompileError::Internal(
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
            errors.push(CompileError::Internal(
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
                errors.push(CompileError::Internal("unable to split vec", span.clone()));
                return err(warnings, errors);
            }
        };
        stack.push(first);

        for range in rest.iter() {
            let top = match stack.pop() {
                Some(top) => top,
                None => {
                    errors.push(CompileError::Internal("stack empty", span.clone()));
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
    pub(crate) fn find_exclusionary_ranges(
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
            errors.push(CompileError::Internal(
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
                errors.push(CompileError::Internal("could not split vec", span.clone()));
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
    pub(crate) fn do_ranges_equal_range(
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
                    errors.push(CompileError::Internal("vec empty", span.clone()));
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
    T: fmt::Debug
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
            write!(builder, "{}", self.first)?;
        }
        builder.push_str("...");
        if self.last == T::global_max() {
            builder.push_str("MAX");
        } else {
            write!(builder, "{}", self.last)?;
        }
        builder.push(']');
        write!(f, "{}", builder)
    }
}

/// Checks to see if two ranges are greater than or equal to one another.
impl<T> std::cmp::Ord for Range<T>
where
    T: fmt::Debug
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
    fn cmp(&self, other: &Self) -> Ordering {
        use Ordering::*;

        match (self.first.cmp(&other.first), self.last.cmp(&other.last)) {
            (Less, Less) => Less,
            (Less, Equal) => Less,
            (Less, Greater) => Less,
            (Equal, Less) => Less,
            (Equal, Equal) => Equal,
            (Equal, Greater) => Greater,
            (Greater, Less) => Greater,
            (Greater, Equal) => Greater,
            (Greater, Greater) => Greater,
        }
    }
}

impl<T> std::cmp::PartialOrd for Range<T>
where
    T: fmt::Debug
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
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
