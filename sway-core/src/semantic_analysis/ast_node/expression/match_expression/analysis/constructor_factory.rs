use std::collections::HashSet;

use sway_types::Span;

use crate::{
    error::{err, ok},
    type_engine::{look_up_type_id, TypeId},
    CompileError, CompileResult, TypeInfo,
};

use super::{
    patstack::PatStack,
    pattern::{EnumPattern, Pattern, StructPattern},
    range::Range,
};

pub(crate) struct ConstructorFactory {
    value_type: TypeInfo,
}

impl ConstructorFactory {
    pub(crate) fn new(type_id: TypeId) -> Self {
        ConstructorFactory {
            value_type: look_up_type_id(type_id),
        }
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
            Pattern::Enum(enum_pattern) => {
                let (enum_name, enum_variants) = match &self.value_type {
                    TypeInfo::Enum {
                        name,
                        variant_types,
                    } => (name, variant_types),
                    _ => {
                        errors.push(CompileError::Internal("type mismatch", span.clone()));
                        return err(warnings, errors);
                    }
                };
                if enum_pattern.enum_name.as_str() != enum_name.as_str() {
                    errors.push(CompileError::Internal("type mismatch", span.clone()));
                    return err(warnings, errors);
                }
                let mut all_variants: HashSet<String> = HashSet::new();
                for variant in enum_variants.iter() {
                    all_variants.insert(variant.name.to_string().clone());
                }
                let mut variant_tracker: HashSet<String> = HashSet::new();
                for pat in rest.iter() {
                    match pat {
                        Pattern::Enum(enum_pattern2) => {
                            if enum_pattern2.enum_name.as_str() != enum_name.as_str() {
                                errors.push(CompileError::Internal("type mismatch", span.clone()));
                                return err(warnings, errors);
                            }
                            variant_tracker.insert(enum_pattern2.variant_name.to_string());
                        }
                        _ => {
                            errors.push(CompileError::Internal("type mismatch", span.clone()));
                            return err(warnings, errors);
                        }
                    }
                }
                check!(
                    Pattern::from_pat_stack(
                        PatStack::from(
                            all_variants
                                .difference(&variant_tracker)
                                .into_iter()
                                .map(|x| {
                                    Pattern::Enum(EnumPattern {
                                        enum_name: enum_name.to_string(),
                                        variant_name: x.clone(),
                                        value: Box::new(Pattern::Wildcard),
                                    })
                                })
                                .collect::<Vec<_>>()
                        ),
                        span
                    ),
                    return err(warnings, errors),
                    warnings,
                    errors
                )
            }
            Pattern::Tuple(elems) => Pattern::Tuple(PatStack::fill_wildcards(elems.len())),
            Pattern::Or(_) => unreachable!(),
        };
        ok(pat, warnings, errors)
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
    pub(crate) fn is_complete_signature(
        &self,
        pat_stack: &PatStack,
        span: &Span,
    ) -> CompileResult<bool> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let preprocessed = pat_stack.flatten().filter_out_wildcards();
        if preprocessed.is_empty() {
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
            Pattern::Enum(enum_pattern) => {
                let (enum_name, enum_variants) = match &self.value_type {
                    TypeInfo::Enum {
                        name,
                        variant_types,
                    } => (name, variant_types),
                    _ => {
                        errors.push(CompileError::Internal("type mismatch", span.clone()));
                        return err(warnings, errors);
                    }
                };
                if enum_pattern.enum_name.as_str() != enum_name.as_str() {
                    errors.push(CompileError::Internal("type mismatch", span.clone()));
                    return err(warnings, errors);
                }
                let mut all_variants: HashSet<String> = HashSet::new();
                for variant in enum_variants.iter() {
                    all_variants.insert(variant.name.to_string().clone());
                }
                let mut variant_tracker: HashSet<String> = HashSet::new();
                variant_tracker.insert(enum_pattern.variant_name.to_string());
                for pat in rest.iter() {
                    match pat {
                        Pattern::Enum(enum_pattern2) => {
                            if enum_pattern2.enum_name.as_str() != enum_name.as_str() {
                                errors.push(CompileError::Internal("type mismatch", span.clone()));
                                return err(warnings, errors);
                            }
                            variant_tracker.insert(enum_pattern2.variant_name.to_string());
                        }
                        _ => {
                            errors.push(CompileError::Internal("type mismatch", span.clone()));
                            return err(warnings, errors);
                        }
                    }
                }
                ok(
                    all_variants.difference(&variant_tracker).next().is_none(),
                    warnings,
                    errors,
                )
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
}
