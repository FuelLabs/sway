use std::{
    cmp::Ordering,
    fmt::{self, Write},
    ops::Sub,
};

use crate::CompileError;
use itertools::Itertools;
use sway_error::handler::{ErrorEmitted, Handler};
use sway_types::Span;

pub(crate) trait MyMath<T> {
    fn global_max() -> T;
    fn global_min() -> T;

    fn incr(&self) -> T;
    fn decr(&self) -> T;
}

impl MyMath<u8> for u8 {
    fn global_max() -> u8 {
        u8::MAX
    }
    fn global_min() -> u8 {
        u8::MIN
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
        u16::MAX
    }
    fn global_min() -> u16 {
        u16::MIN
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
        u32::MAX
    }
    fn global_min() -> u32 {
        u32::MIN
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
        u64::MAX
    }
    fn global_min() -> u64 {
        u64::MIN
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
            first: u8::MIN,
            last: u8::MAX,
        }
    }
}

impl Range<u16> {
    pub(crate) fn u16() -> Range<u16> {
        Range {
            first: u16::MIN,
            last: u16::MAX,
        }
    }
}

impl Range<u32> {
    pub(crate) fn u32() -> Range<u32> {
        Range {
            first: u32::MIN,
            last: u32::MAX,
        }
    }
}

impl Range<u64> {
    pub(crate) fn u64() -> Range<u64> {
        Range {
            first: u64::MIN,
            last: u64::MAX,
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
    fn from_double(
        handler: &Handler,
        first: T,
        last: T,
        span: &Span,
    ) -> Result<Range<T>, ErrorEmitted> {
        if last < first {
            Err(handler.emit_err(CompileError::Internal(
                "attempted to create an invalid range",
                span.clone(),
            )))
        } else {
            Ok(Range { first, last })
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
    /// Note that because `Range<T>` relies on the assumption that `T` is an
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
    fn join_ranges(
        handler: &Handler,
        a: &Range<T>,
        b: &Range<T>,
        span: &Span,
    ) -> Result<Range<T>, ErrorEmitted> {
        if !a.overlaps(b) && !a.within_one(b) {
            Err(handler.emit_err(CompileError::Internal(
                "these two ranges cannot be joined",
                span.clone(),
            )))
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
            let range = Range::from_double(handler, first, last, span)?;
            Ok(range)
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
    ///    3a. If the current interval does not overlap with the stack
    ///    top, push it.
    ///    3b. If the current interval overlaps with stack top (or is within ± 1)
    ///    and ending time of current interval is more than that of stack top,
    ///    update stack top with the ending time of current interval.
    /// 4. At the end stack contains the merged intervals.
    fn condense_ranges(
        handler: &Handler,
        ranges: Vec<Range<T>>,
        span: &Span,
    ) -> Result<Vec<Range<T>>, ErrorEmitted> {
        let mut ranges = ranges;
        let mut stack: Vec<Range<T>> = vec![];

        // 1. Sort the intervals based on increasing order of starting time.
        ranges.sort_by(|a, b| b.first.cmp(&a.first));

        // 2. Push the first interval on to a stack.
        let (first, rest) = match ranges.split_first() {
            Some((first, rest)) => (first.to_owned(), rest.to_owned()),
            None => {
                return Err(
                    handler.emit_err(CompileError::Internal("unable to split vec", span.clone()))
                );
            }
        };
        stack.push(first);

        for range in rest.iter() {
            let top = match stack.pop() {
                Some(top) => top,
                None => {
                    return Err(
                        handler.emit_err(CompileError::Internal("stack empty", span.clone()))
                    );
                }
            };
            if range.overlaps(&top) || range.within_one(&top) {
                // 3b. If the current interval overlaps with stack top (or is within ± 1)
                //     and ending time of current interval is more than that of stack top,
                //     update stack top with the ending time of current interval.
                stack.push(Range::join_ranges(handler, range, &top, span)?);
            } else {
                // 3a. If the current interval does not overlap with the stack
                //     top, push it.
                stack.push(top);
                stack.push(range.clone());
            }
        }
        stack.reverse();
        Ok(stack)
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
        handler: &Handler,
        guides: Vec<Range<T>>,
        oracle: Range<T>,
        span: &Span,
    ) -> Result<Vec<Range<T>>, ErrorEmitted> {
        // 1. Convert *guides* to a vec of ordered, distinct, non-overlapping
        //    ranges *guides*'
        let condensed = Range::condense_ranges(handler, guides, span)?;

        // 2. Check to ensure that *oracle* fully encompasses all ranges in
        //    *guides*'.
        if !oracle.encompasses_all(&condensed) {
            return Err(handler.emit_err(CompileError::Internal(
                "ranges OOB with the oracle",
                span.clone(),
            )));
        }

        // 3. Given the *oracle* range `[a, b]` and the *guides*'₀ range of
        //    `[c, d]`, and `a != c`, construct a range of `[a, c]`.
        let mut exclusionary = vec![];
        let (first, last) = match (condensed.split_first(), condensed.split_last()) {
            (Some((first, _)), Some((last, _))) => (first, last),
            _ => {
                return Err(
                    handler.emit_err(CompileError::Internal("could not split vec", span.clone()))
                );
            }
        };
        if oracle.first != first.first {
            exclusionary.push(Range::from_double(
                handler,
                oracle.first.clone(),
                first.first.decr(),
                span,
            )?);
        }

        // 4. Given *guides*' of length *n*, for every *k* 0..*n-1*, find the
        //    *guides*'ₖ range of `[a,b]` and the *guides*'ₖ₊₁ range of `[c, d]`,
        //    construct a range of `[b, c]`. You can assume that `b != d` because
        //    of step (1)
        for (left, right) in condensed.iter().tuple_windows() {
            exclusionary.push(Range::from_double(
                handler,
                left.last.incr(),
                right.first.decr(),
                span,
            )?);
        }

        // 5. Given the *oracle* range of `[a, b]`, *guides*' of length *n*, and
        //    the *guides*'ₙ range of `[c, d]`, and `b != d`, construct a range of
        //    `[b, d]`.
        if oracle.last != last.last {
            exclusionary.push(Range::from_double(
                handler,
                last.last.incr(),
                oracle.last,
                span,
            )?);
        }

        // 6. Combine the range given from step (3), the ranges given from step
        //    (4), and the range given from step (5) for your result.
        Ok(exclusionary)
    }

    /// Condenses a vec of ranges and checks to see if the condensed ranges
    /// equal an oracle range.
    pub(crate) fn do_ranges_equal_range(
        handler: &Handler,
        ranges: Vec<Range<T>>,
        oracle: Range<T>,
        span: &Span,
    ) -> Result<bool, ErrorEmitted> {
        let condensed_ranges = Range::condense_ranges(handler, ranges, span)?;
        if condensed_ranges.len() > 1 {
            Ok(false)
        } else {
            let first_range = match condensed_ranges.first() {
                Some(first_range) => first_range.clone(),
                _ => {
                    return Err(handler.emit_err(CompileError::Internal("vec empty", span.clone())));
                }
            };
            Ok(first_range == oracle)
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
        others.iter().all(|other| self.encompasses(other))
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

        // Because [Range]s represent the `[n, m]` (fully) inclusive 'contains',
        // it is entirely possible (and normal) for the occasional [Range] to
        // have the same `first` and `last`. For example, if the user is
        // matching on `u64` values and specifies a match arm for `2` but does
        // not specify a match arm for `1`, then this would otherwise display as
        // `[MIN...1]`. While not incorrect, it looks kind of weird. So instead
        // we bypass this problem when displaying [Range]s.
        if self.first == self.last {
            write!(builder, "{}", self.first)?;
            return write!(f, "{builder}");
        }

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
        write!(f, "{builder}")
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
