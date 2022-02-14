use std::fmt;
use std::ops::Sub;
use std::slice::Iter;
use std::vec::IntoIter;

use sway_types::Ident;
use sway_types::Span;

use crate::error::err;
use crate::error::ok;
use crate::type_engine::IntegerBits;
use crate::CompileError;
use crate::CompileResult;
use crate::Literal;
use crate::MatchCondition;
use crate::Scrutinee;
use crate::TypeInfo;

use itertools::Itertools;
use std::fmt::Debug;

#[derive(Debug)]
pub(crate) enum WitnessReport {
    NoWitnesses,
    Witnesses(PatStack),
}

impl WitnessReport {
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

    fn resolve_with_constructor(
        witness_report: WitnessReport,
        c: &Pattern,
        n: usize,
        span: &Span,
    ) -> CompileResult<Self> {
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
                let (rs, mut ps) = witnesses.split_at(witnesses.len() - n + 1);
                let mut pat_stack = PatStack::empty();
                pat_stack.push(check!(
                    Pattern::from_constructor_and_arguments(c, rs, span),
                    return err(warnings, errors),
                    warnings,
                    errors
                ));
                pat_stack.append(&mut ps);
                ok(WitnessReport::Witnesses(pat_stack), vec![], vec![])
            }
        }
    }

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

    pub(crate) fn has_witnesses(&self) -> bool {
        match self {
            WitnessReport::NoWitnesses => false,
            WitnessReport::Witnesses(_) => true, // !witnesses.is_empty()
        }
    }
}

impl fmt::Display for WitnessReport {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let flattened = match self {
            WitnessReport::NoWitnesses => PatStack::empty(),
            WitnessReport::Witnesses(witnesses) => witnesses.flatten(),
        };
        let s = flattened.into_iter().map(|x| format!("`{}`", x)).join(", ");
        write!(f, "{}", s)
    }
}

pub(crate) trait IncrDecr<T> {
    fn incr(&self) -> T;
    fn decr(&self) -> T;
}

impl IncrDecr<u8> for u8 {
    fn incr(&self) -> u8 {
        self + 1
    }
    fn decr(&self) -> u8 {
        self - 1
    }
}

impl IncrDecr<u16> for u16 {
    fn incr(&self) -> u16 {
        self + 1
    }
    fn decr(&self) -> u16 {
        self - 1
    }
}

impl IncrDecr<u32> for u32 {
    fn incr(&self) -> u32 {
        self + 1
    }
    fn decr(&self) -> u32 {
        self - 1
    }
}

impl IncrDecr<u64> for u64 {
    fn incr(&self) -> u64 {
        self + 1
    }
    fn decr(&self) -> u64 {
        self - 1
    }
}

/// A `Range<T>` is a range of values of type T.
/// Given this range:
///
/// ```ignore
/// Range {
///     first: 0,
///     last: 3
/// }
/// ```
///
/// This represents the inclusive range `[0, 3]`.
/// (Where '[' and ']' represent inclusive contains.)
/// More specifically: `0, 1, 2, 3`.
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
        + IncrDecr<T>
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
        + IncrDecr<T>
        + Sub<Output = T>
        + Into<u64>,
{
    fn from_single(x: T) -> Range<T> {
        Range {
            first: x.clone(),
            last: x,
        }
    }

    /// Create a `Range` and ensures that it is a "valid `Range`"
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
    /// |------------|
    ///    |------|
    /// ->
    /// |------------|
    ///
    ///    |------|
    /// |------------|
    /// ->
    /// |------------|
    ///
    /// |---------|
    ///      |---------|
    /// ->
    /// |--------------|
    ///
    ///      |---------|
    /// |---------|
    /// ->
    /// |--------------|
    ///
    /// |------|
    ///         |------|
    /// ->
    /// |--------------|
    ///
    ///         |------|
    /// |------|
    /// ->
    /// |--------------|
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
    ///     3b. If the current interval overlaps with stack top and ending
    ///         time of current interval is more than that of stack top,
    ///         update stack top with the ending  time of current interval.
    /// 4. At the end stack contains the merged intervals.
    ///
    /// However there is a small modification that is Sway-specific. Because Sway does not
    /// have floating point numbers at the language level this algorithm can further condense
    /// ranges that are located ± 1 to one another. For instance, these two `Range`s:
    ///
    /// ```ignore
    /// Range {
    ///     first: 0,
    ///     last: 0,
    /// }
    ///
    /// Range {
    ///     first: 1,
    ///     last: 1,
    /// }
    /// ```
    ///
    /// become this `Range`:
    ///
    /// ```ignore
    /// Range {
    ///     first: 0,
    ///     last: 1
    /// }
    /// ```
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

    fn find_exclusionary_ranges(
        ranges: Vec<Range<T>>,
        oracle: Range<T>,
        span: &Span,
    ) -> CompileResult<Vec<Range<T>>> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let condensed = check!(
            Range::condense_ranges(ranges, span),
            return err(warnings, errors),
            warnings,
            errors
        );
        if !oracle.overlaps_all(&condensed) {
            errors.push(CompileError::ExhaustivityCheckingAlgorithmFailure(
                "ranges OOB with the oracle",
                span.clone(),
            ));
            return err(warnings, errors);
        }
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
        for (left, right) in condensed.iter().tuple_windows() {
            exclusionary.push(check!(
                Range::from_double(left.last.incr(), right.first.decr(), span),
                return err(warnings, errors),
                warnings,
                errors
            ));
        }
        if oracle.last != last.last {
            exclusionary.push(check!(
                Range::from_double(last.last.incr(), oracle.last, span),
                return err(warnings, errors),
                warnings,
                errors
            ));
        }
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

    /// Checks to see if two ranges overlap. There are 4 ways in which this might be the case:
    ///
    /// ```ignore
    /// |------------|
    ///    |------|
    ///
    ///    |------|
    /// |------------|
    ///
    /// |---------|
    ///      |---------|
    ///
    ///      |---------|
    /// |---------|
    /// ```
    fn overlaps(&self, other: &Range<T>) -> bool {
        other.first >= self.first && other.last <= self.last
            || other.first <= self.first && other.last >= self.last
            || other.first <= self.first && other.last <= self.last && other.last >= self.first
            || other.first >= self.first && other.first <= self.last && other.last >= self.last
    }

    fn overlaps_all(&self, others: &[Range<T>]) -> bool {
        others.iter().map(|other| self.overlaps(other)).all(|x| x)
    }

    /// Checks to see if two ranges are within ± 1 of one another.
    /// There are 2 ways in which this might be the case:
    ///
    /// ```ignore
    /// |------|
    ///         |------|
    ///
    ///         |------|
    /// |------|
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
        + IncrDecr<T>
        + Sub<Output = T>
        + Into<u64>,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut builder = String::new();
        builder.push('[');
        builder.push_str(&format!("{}", self.first));
        builder.push_str("...");
        builder.push_str(&format!("{}", self.last));
        builder.push(']');
        write!(f, "{}", builder)
    }
}

/// A `Pattern` represents something that could be on the LHS of a match expression arm.
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
/// Pattern::Tuple(
///     PatStack {
///         pats: [
///             Pattern::U64(Range { first: 0, last: 0 }),
///             Pattern::U64(Range { first: 1, last: 1 })
///         ]
///     }
/// )
///
/// Pattern::Tuple(
///     PatStack {
///         pats: [
///             Pattern::U64(Range { first: 2, last: 2 }),
///             Pattern::U64(Range { first: 3, last: 3 })
///         ]
///     }
/// )
///
/// Pattern::Wildcard
/// ```
///
/// A `Pattern` is semantically constructed from a "constructor" and its "arguments."
///
/// Given the `Pattern`:
///
/// ```ignore
/// Pattern::Tuple(
///     PatStack {
///         pats: [
///             Pattern::U64(Range { first: 0, last: 0 }),
///             Pattern::U64(Range { first: 1, last: 1 })
///         ]
///     }
/// )
/// ```
///
/// The constructor is "Pattern::Tuple" and its arguments are the contents of `pats`.
/// This idea of a constructor and arguments is used in the match exhaustivity algorithm.
#[derive(Clone, Debug)]
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
    String(Span),
    Struct(StructPattern),
    Tuple(PatStack),
    Or(Box<Pattern>, Box<Pattern>),
}

impl Pattern {
    fn from_match_condition(match_condition: MatchCondition) -> Self {
        match match_condition {
            MatchCondition::CatchAll(_) => Pattern::Wildcard,
            MatchCondition::Scrutinee(scrutinee) => Pattern::from_scrutinee(scrutinee),
        }
    }

    fn from_scrutinee(scrutinee: Scrutinee) -> Self {
        match scrutinee {
            Scrutinee::Variable { .. } => Pattern::Wildcard,
            Scrutinee::Literal { value, .. } => match value {
                Literal::U8(x) => Pattern::U8(Range::from_single(x)),
                Literal::U16(x) => Pattern::U16(Range::from_single(x)),
                Literal::U32(x) => Pattern::U32(Range::from_single(x)),
                Literal::U64(x) => Pattern::U64(Range::from_single(x)),
                Literal::B256(x) => Pattern::B256(x),
                Literal::Boolean(b) => Pattern::Boolean(b),
                Literal::Byte(x) => Pattern::Byte(Range::from_single(x)),
                Literal::Numeric(x) => Pattern::Numeric(Range::from_single(x)),
                Literal::String(s) => Pattern::String(s),
            },
            Scrutinee::StructScrutinee {
                struct_name,
                fields,
                ..
            } => Pattern::Struct(StructPattern {
                struct_name,
                fields: PatStack::new(
                    fields
                        .into_iter()
                        .map(|x| match x.scrutinee {
                            Some(scrutinee) => Pattern::from_scrutinee(scrutinee),
                            None => Pattern::Wildcard,
                        })
                        .collect::<Vec<_>>(),
                ),
            }),
            Scrutinee::Tuple { elems, .. } => Pattern::Tuple(PatStack::new(
                elems
                    .into_iter()
                    .map(Pattern::from_scrutinee)
                    .collect::<Vec<_>>(),
            )),
            _ => unreachable!(),
        }
    }

    fn from_pat_stack(pat_stack: PatStack, span: &Span) -> CompileResult<Pattern> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let mut pat_stack = pat_stack;
        let mut pat: Option<Pattern> = None;
        while !pat_stack.is_empty() {
            let p = check!(
                pat_stack.pop(span),
                return err(warnings, errors),
                warnings,
                errors
            );
            match pat {
                Some(other) => pat = Some(Pattern::Or(Box::new(p), Box::new(other))),
                None => pat = Some(p),
            }
        }
        match pat {
            Some(pat) => ok(pat, warnings, errors),
            None => {
                errors.push(CompileError::ExhaustivityCheckingAlgorithmFailure(
                    "unable to build pattern",
                    span.clone(),
                ));
                err(warnings, errors)
            }
        }
    }

    fn from_constructor_and_arguments(
        c: &Pattern,
        args: PatStack,
        span: &Span,
    ) -> CompileResult<Self> {
        let warnings = vec![];
        let mut errors = vec![];
        match c {
            Pattern::Tuple(elems) => {
                if elems.len() != args.len() {
                    errors.push(CompileError::ExhaustivityCheckingAlgorithmFailure(
                        "tuple arity mismatch",
                        span.clone(),
                    ));
                    return err(warnings, errors);
                }
                ok(Pattern::Tuple(args), warnings, errors)
            }
            _ => unimplemented!(),
        }
    }

    fn wild_pattern() -> Self {
        Pattern::Wildcard
    }

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
            Pattern::Or(_, _) => unreachable!(),
        }
    }

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
            (Pattern::Or(_, _), Pattern::Or(_, _)) => unreachable!(),
            _ => false,
        }
    }

    /*
    fn constructor(&self) -> Self {
        match self {
            Pattern::U8(_) => Pattern::U8(Range::u8()),
            Pattern::U16(_) => Pattern::U16(Range::u16()),
            Pattern::U32(_) => Pattern::U32(Range::u32()),
            Pattern::U64(_) => Pattern::U64(Range::u64()),
            _ => unimplemented!(),
        }
    }
    */

    fn sub_patterns(&self) -> PatStack {
        match self {
            Pattern::Struct(StructPattern { fields, .. }) => fields.to_owned(),
            Pattern::Tuple(elems) => elems.to_owned(),
            _ => PatStack::empty(),
        }
    }

    fn flatten(&self) -> PatStack {
        match self {
            Pattern::Or(pat1, pat2) => {
                let mut pat_stack = (*pat1).flatten();
                pat_stack.append(&mut (*pat2).flatten());
                pat_stack
            }
            pat => PatStack::from_pattern(pat.to_owned()),
        }
    }
}

impl fmt::Display for Pattern {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Pattern::Numeric(range) => format!("{}", range),
            a => unimplemented!("{:?}", a),
        };
        write!(f, "{}", s)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct StructPattern {
    struct_name: Ident,
    fields: PatStack,
}

#[derive(Clone, Debug)]
pub(crate) struct PatStack {
    pats: Vec<Pattern>,
}

impl PatStack {
    fn empty() -> Self {
        PatStack { pats: vec![] }
    }

    fn new(pats: Vec<Pattern>) -> Self {
        PatStack { pats }
    }

    fn from_pattern(pattern: Pattern) -> Self {
        PatStack {
            pats: vec![pattern],
        }
    }

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

    fn split_at(&self, n: usize) -> (PatStack, PatStack) {
        let (a, b) = self.pats.split_at(n);
        let x = PatStack { pats: a.to_vec() };
        let y = PatStack { pats: b.to_vec() };
        (x, y)
    }

    fn push(&mut self, other: Pattern) {
        self.pats.push(other)
    }

    fn pop(&mut self, span: &Span) -> CompileResult<Pattern> {
        let warnings = vec![];
        let mut errors = vec![];
        match self.pats.pop() {
            Some(pat) => ok(pat, warnings, errors),
            None => {
                errors.push(CompileError::ExhaustivityCheckingAlgorithmFailure(
                    "unable to pop patstack",
                    span.clone(),
                ));
                err(warnings, errors)
            }
        }
    }

    fn append(&mut self, others: &mut PatStack) {
        self.pats.append(&mut others.pats);
    }

    fn prepend(&mut self, other: Pattern) {
        self.pats.insert(0, other);
    }

    fn len(&self) -> usize {
        self.pats.len()
    }

    fn is_empty(&self) -> bool {
        self.pats.is_empty()
    }

    fn iter(&self) -> Iter<'_, Pattern> {
        self.pats.iter()
    }

    fn into_iter(self) -> IntoIter<Pattern> {
        self.pats.into_iter()
    }

    fn is_complete_signature(&self, span: &Span) -> CompileResult<bool> {
        let mut warnings = vec![];
        let mut errors = vec![];
        if self.pats.is_empty() {
            return ok(false, warnings, errors);
        }
        let (first, rest) = check!(
            self.flatten().split_first(span),
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
            Pattern::Tuple(elems) => {
                for pat in rest.iter() {
                    if !pat.has_the_same_constructor(&Pattern::Tuple(elems.clone())) {
                        return ok(false, warnings, errors);
                    }
                }
                ok(true, warnings, errors)
            }
            Pattern::Struct(_) => unimplemented!(),
            Pattern::Wildcard => unreachable!(),
            Pattern::Or(_, _) => unreachable!(),
        }
    }

    fn flatten(&self) -> PatStack {
        let mut flattened = PatStack::empty();
        for pat in self.pats.iter() {
            flattened.append(&mut pat.flatten());
        }
        flattened
    }
}

impl From<Vec<Pattern>> for PatStack {
    fn from(pats: Vec<Pattern>) -> Self {
        PatStack { pats }
    }
}

#[derive(Clone, Debug)]
struct Matrix {
    rows: Vec<PatStack>,
}

impl Matrix {
    fn empty() -> Self {
        Matrix { rows: vec![] }
    }

    fn from_pat_stack(pat_stack: PatStack) -> Self {
        Matrix {
            rows: vec![pat_stack],
        }
    }

    fn from_pat_stacks(pat_stacks: Vec<PatStack>) -> Self {
        Matrix { rows: pat_stacks }
    }

    fn push(&mut self, row: PatStack) {
        self.rows.push(row);
    }

    fn append(&mut self, rows: &mut Vec<PatStack>) {
        self.rows.append(rows);
    }

    fn rows(&self) -> &Vec<PatStack> {
        &self.rows
    }

    fn into_rows(self) -> Vec<PatStack> {
        self.rows
    }

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

    fn is_a_vector(&self) -> bool {
        self.rows.len() == 1
    }

    fn unwrap_vector(&self, span: &Span) -> CompileResult<PatStack> {
        let warnings = vec![];
        let mut errors = vec![];
        if self.rows.len() > 1 {
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
            match first {
                Pattern::Wildcard => {}
                other => pat_stack.push(other),
            }
        }
        ok(pat_stack, warnings, errors)
    }
}

struct ConstructorFactory {
    type_info: TypeInfo,
    constructors: PatStack,
}

impl ConstructorFactory {
    fn new(type_info: TypeInfo) -> Self {
        let constructors = match type_info {
            TypeInfo::UnsignedInteger(IntegerBits::SixtyFour) => vec![Pattern::U64(Range::u64())],
            TypeInfo::Numeric => vec![Pattern::Numeric(Range::u64())],
            _ => unimplemented!(),
        };
        ConstructorFactory {
            type_info,
            constructors: PatStack::new(constructors),
        }
    }

    fn create_constructor_not_present(
        &self,
        sigma: PatStack,
        span: &Span,
    ) -> CompileResult<Pattern> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let (first, rest) = check!(
            sigma.flatten().split_first(span),
            return err(warnings, errors),
            warnings,
            errors
        );
        match first {
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
                Pattern::from_pat_stack(unincluded, span)
            }
            a => unimplemented!("{:?}", a),
        }
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
            matrix.push(PatStack::from_pattern(Pattern::from_match_condition(
                first_arm.clone(),
            )));
            arms_reachability.push((first_arm.clone(), true));
            for arm in arms_rest.iter() {
                let pattern = Pattern::from_match_condition(arm.clone());
                let v = PatStack::from_pattern(pattern);
                /*
                let witness_report = check!(
                    is_useful(&factory, &matrix, &v, &span),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                */
                matrix.push(v);
                // if an arm has witnesses to its usefulness then it is reachable
                //arms_reachability.push((arm.clone(), witness_report.has_witnesses()));
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

fn is_useful(
    factory: &ConstructorFactory,
    p: &Matrix,
    q: &PatStack,
    span: &Span,
) -> CompileResult<WitnessReport> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let (m, n) = check!(p.m_n(span), return err(warnings, errors), warnings, errors);
    /*
    if n != q.len() {
        println!("p: {:?}", p);
        println!("q: {:?}", q);
        unimplemented!()
    }
    */
    match (m, n) {
        (0, 0) => ok(
            WitnessReport::Witnesses(PatStack::empty()),
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
            match c {
                Pattern::Wildcard => is_useful_wildcard(factory, p, q, n, span),
                Pattern::Or(pat1, pat2) => is_useful_or(factory, p, q, *pat1, *pat2, span),
                c => is_useful_constructed(factory, p, q, c, span),
            }
        }
    }
}

fn is_useful_wildcard(
    factory: &ConstructorFactory,
    p: &Matrix,
    q: &PatStack,
    n: usize,
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
    let sigma = check!(
        p.compute_sigma(span),
        return err(warnings, errors),
        warnings,
        errors
    );
    let is_complete_signature = check!(
        sigma.is_complete_signature(span),
        return err(warnings, errors),
        warnings,
        errors
    );
    if is_complete_signature {
        let mut joined_witness_report = WitnessReport::NoWitnesses;
        for c_k in sigma.iter() {
            let s_c_k_p = check!(
                compute_specialized_matrix(c_k, p, span),
                return err(warnings, errors),
                warnings,
                errors
            );
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
            let witness_report = check!(
                is_useful(factory, &s_c_k_p, &s_c_k_q, span),
                return err(warnings, errors),
                warnings,
                errors
            );
            let witness_report = check!(
                WitnessReport::resolve_with_constructor(witness_report, c_k, n, span),
                return err(warnings, errors),
                warnings,
                errors
            );
            joined_witness_report =
                WitnessReport::join_witness_reports(joined_witness_report, witness_report)
        }
        ok(joined_witness_report, warnings, errors)
    } else {
        let d_p = check!(
            compute_default_matrix(p, span),
            return err(warnings, errors),
            warnings,
            errors
        );
        let mut witness_report = check!(
            is_useful(factory, &d_p, &q_rest, span),
            return err(warnings, errors),
            warnings,
            errors
        );
        let witness_to_add = if sigma.is_empty() {
            Pattern::Wildcard
        } else {
            check!(
                factory.create_constructor_not_present(sigma, span),
                return err(warnings, errors),
                warnings,
                errors
            )
        };
        match &mut witness_report {
            WitnessReport::NoWitnesses => {}
            witness_report => check!(
                witness_report.add_witness(witness_to_add, span),
                return err(warnings, errors),
                warnings,
                errors
            ),
        }
        ok(witness_report, warnings, errors)
    }
}

fn is_useful_constructed(
    factory: &ConstructorFactory,
    p: &Matrix,
    q: &PatStack,
    c: Pattern,
    span: &Span,
) -> CompileResult<WitnessReport> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let s_c_p = check!(
        compute_specialized_matrix(&c, p, span),
        return err(warnings, errors),
        warnings,
        errors
    );
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
    is_useful(factory, &s_c_p, &s_c_q, span)
}

fn is_useful_or(
    factory: &ConstructorFactory,
    p: &Matrix,
    q: &PatStack,
    pat1: Pattern,
    pat2: Pattern,
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
    let mut q1 = q_rest.clone();
    q1.prepend(pat1);
    let witness_report1 = check!(
        is_useful(factory, p, &q1, span),
        return err(warnings, errors),
        warnings,
        errors
    );
    let mut q2 = q_rest;
    q2.prepend(pat2);
    let witness_report2 = check!(
        is_useful(factory, p, &q2, span),
        return err(warnings, errors),
        warnings,
        errors
    );

    let witness_report = WitnessReport::join_witness_reports(witness_report1, witness_report2);
    ok(witness_report, warnings, errors)
}

fn compute_default_matrix(p: &Matrix, span: &Span) -> CompileResult<Matrix> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let mut d_p = Matrix::empty();
    for p_i in p.rows().iter() {
        let (p_i_1, mut p_i_rest) = check!(
            p_i.split_first(span),
            return err(warnings, errors),
            warnings,
            errors
        );
        let mut rows = check!(
            compute_default_matrix_row(&p_i_1, &mut p_i_rest, span),
            return err(warnings, errors),
            warnings,
            errors
        );
        d_p.append(&mut rows);
    }
    ok(d_p, warnings, errors)
}

fn compute_default_matrix_row(
    p_i_1: &Pattern,
    p_i_rest: &mut PatStack,
    span: &Span,
) -> CompileResult<Vec<PatStack>> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let mut rows: Vec<PatStack> = vec![];
    match p_i_1 {
        Pattern::Wildcard => {
            let mut row = PatStack::empty();
            row.append(p_i_rest);
            rows.push(row);
        }
        Pattern::Or(pat1, pat2) => {
            let mut row1 = PatStack::from_pattern(*pat1.clone());
            row1.append(&mut p_i_rest.clone());
            let mut row2 = PatStack::from_pattern(*pat2.clone());
            row2.append(p_i_rest);
            let d_p = check!(
                compute_default_matrix(&Matrix::from_pat_stacks(vec!(row1, row2)), span),
                return err(warnings, errors),
                warnings,
                errors
            );
            rows.append(&mut d_p.into_rows());
        }
        _ => {}
    }
    ok(rows, warnings, errors)
}

fn compute_specialized_matrix(c: &Pattern, p: &Matrix, span: &Span) -> CompileResult<Matrix> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let mut s_c_p = Matrix::empty();
    for p_i in p.rows().iter() {
        let (p_i_1, mut p_i_rest) = check!(
            p_i.split_first(span),
            return err(warnings, errors),
            warnings,
            errors
        );
        let mut rows = check!(
            compute_specialized_matrix_row(c, &p_i_1, &mut p_i_rest, span),
            return err(warnings, errors),
            warnings,
            errors
        );
        s_c_p.append(&mut rows);
    }
    let (m, _n) = check!(
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

fn compute_specialized_matrix_row(
    c: &Pattern,
    p_i_1: &Pattern,
    p_i_rest: &mut PatStack,
    span: &Span,
) -> CompileResult<Vec<PatStack>> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let mut rows: Vec<PatStack> = vec![];
    match p_i_1 {
        Pattern::Wildcard => {
            let mut row: PatStack = PatStack::empty();
            for _ in 0..c.a() {
                row.push(Pattern::Wildcard);
            }
            row.append(p_i_rest);
            rows.push(row);
        }
        Pattern::Or(pat1, pat2) => {
            let mut row1 = PatStack::from_pattern(*pat1.clone());
            row1.append(&mut p_i_rest.clone());
            let mut row2 = PatStack::from_pattern(*pat2.clone());
            row2.append(p_i_rest);
            let s_c_p = check!(
                compute_specialized_matrix(c, &Matrix::from_pat_stacks(vec!(row1, row2)), span),
                return err(warnings, errors),
                warnings,
                errors
            );
            rows.append(&mut s_c_p.into_rows());
        }
        other => {
            if c.has_the_same_constructor(other) {
                let mut row: PatStack = other.sub_patterns();
                row.append(p_i_rest);
                rows.push(row);
            }
        }
    }
    ok(rows, warnings, errors)
}
