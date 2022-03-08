use sway_types::Span;

use crate::{
    error::{err, ok},
    CompileError, CompileResult,
};

use super::{
    patstack::PatStack,
    pattern::{Pattern, StructPattern},
    range::Range,
};

pub(crate) struct ConstructorFactory {}

impl ConstructorFactory {
    pub(crate) fn new() -> Self {
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
    pub(crate) fn create_pattern_not_present(
        &self,
        sigma: PatStack,
        span: &Span,
    ) -> CompileResult<Pattern> {
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
                            errors.push(CompileError::Internal("type mismatch", span.clone()));
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
                            errors.push(CompileError::Internal("type mismatch", span.clone()));
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
                            errors.push(CompileError::Internal("type mismatch", span.clone()));
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
                            errors.push(CompileError::Internal("type mismatch", span.clone()));
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
                            errors.push(CompileError::Internal("type mismatch", span.clone()));
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
                    errors.push(CompileError::Internal(
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
                            errors.push(CompileError::Internal("type mismatch", span.clone()));
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
            Pattern::Struct(struct_pattern) => {
                let fields = struct_pattern
                    .fields()
                    .iter()
                    .map(|(name, _)| (name.clone(), Pattern::Wildcard))
                    .collect::<Vec<_>>();
                Pattern::Struct(StructPattern::new(
                    struct_pattern.struct_name().clone(),
                    fields,
                ))
            }
            Pattern::Tuple(elems) => Pattern::Tuple(PatStack::fill_wildcards(elems.len())),
            Pattern::Or(_) => unreachable!(),
        };
        ok(pat, warnings, errors)
    }
}
