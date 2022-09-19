use std::collections::HashSet;

use sway_types::{Ident, Span};

use crate::{
    error::{err, ok},
    semantic_analysis::TypedEnumVariant,
    type_system::{look_up_type_id, TypeId},
    CompileError, CompileResult, TypeInfo,
};

use super::{
    patstack::PatStack,
    pattern::{EnumPattern, Pattern, StructPattern},
    range::Range,
};

pub(crate) struct ConstructorFactory {
    possible_types: Vec<TypeInfo>,
}

impl ConstructorFactory {
    pub(crate) fn new(type_id: TypeId, span: &Span) -> CompileResult<Self> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let possible_types = check!(
            look_up_type_id(type_id).extract_nested_types(span),
            return err(warnings, errors),
            warnings,
            errors
        );
        let factory = ConstructorFactory { possible_types };
        ok(factory, warnings, errors)
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
                            errors.push(CompileError::Internal(
                                "expected all patterns to be of the same type",
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
                            errors.push(CompileError::Internal(
                                "expected all patterns to be of the same type",
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
                            errors.push(CompileError::Internal(
                                "expected all patterns to be of the same type",
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
                            errors.push(CompileError::Internal(
                                "expected all patterns to be of the same type",
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
                            errors.push(CompileError::Internal(
                                "expected all patterns to be of the same type",
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
            Pattern::Wildcard => Pattern::Wildcard,
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
                            errors.push(CompileError::Internal(
                                "expected all patterns to be of the same type",
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
            ref pat @ Pattern::Enum(ref enum_pattern) => {
                let type_info = check!(
                    self.resolve_possible_types(pat, span),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                let (enum_name, enum_variants) = check!(
                    type_info.expect_enum("", span),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                let (all_variants, variant_tracker) = check!(
                    ConstructorFactory::resolve_enum(
                        enum_name,
                        enum_variants,
                        enum_pattern,
                        rest,
                        span
                    ),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
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
            Pattern::Or(_) => {
                errors.push(CompileError::Unimplemented(
                    "or patterns are not supported",
                    span.clone(),
                ));
                return err(warnings, errors);
            }
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
                            errors.push(CompileError::Internal(
                                "expected all patterns to be of the same type",
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
                            errors.push(CompileError::Internal(
                                "expected all patterns to be of the same type",
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
                            errors.push(CompileError::Internal(
                                "expected all patterns to be of the same type",
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
                            errors.push(CompileError::Internal(
                                "expected all patterns to be of the same type",
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
                            errors.push(CompileError::Internal(
                                "expected all patterns to be of the same type",
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
                            errors.push(CompileError::Internal(
                                "expected all patterns to be of the same type",
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
                            errors.push(CompileError::Internal(
                                "expected all patterns to be of the same type",
                                span.clone(),
                            ));
                            return err(warnings, errors);
                        }
                    }
                }
                ok(true_found && false_found, warnings, errors)
            }
            ref pat @ Pattern::Enum(ref enum_pattern) => {
                let type_info = check!(
                    self.resolve_possible_types(pat, span),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                let (enum_name, enum_variants) = check!(
                    type_info.expect_enum("", span),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                let (all_variants, variant_tracker) = check!(
                    ConstructorFactory::resolve_enum(
                        enum_name,
                        enum_variants,
                        enum_pattern,
                        rest,
                        span
                    ),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
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
            Pattern::Wildcard => {
                errors.push(CompileError::Internal(
                    "expected the wildcard pattern to be filtered out here",
                    span.clone(),
                ));
                err(warnings, errors)
            }
            Pattern::Or(_) => {
                errors.push(CompileError::Unimplemented(
                    "or patterns are not supported",
                    span.clone(),
                ));
                err(warnings, errors)
            }
        }
    }

    fn resolve_possible_types(&self, pattern: &Pattern, span: &Span) -> CompileResult<&TypeInfo> {
        let warnings = vec![];
        let mut errors = vec![];
        let mut type_info = None;
        for possible_type in self.possible_types.iter() {
            let matches = pattern.matches_type_info(possible_type, span);
            if matches {
                type_info = Some(possible_type);
                break;
            }
        }
        match type_info {
            Some(type_info) => ok(type_info, warnings, errors),
            None => {
                errors.push(CompileError::Internal(
                    "there is no type that matches this pattern",
                    span.clone(),
                ));
                err(warnings, errors)
            }
        }
    }

    fn resolve_enum(
        enum_name: &Ident,
        enum_variants: &[TypedEnumVariant],
        enum_pattern: &EnumPattern,
        rest: PatStack,
        span: &Span,
    ) -> CompileResult<(HashSet<String>, HashSet<String>)> {
        let warnings = vec![];
        let mut errors = vec![];
        if enum_pattern.enum_name.as_str() != enum_name.as_str() {
            errors.push(CompileError::Internal(
                "expected matching enum names",
                span.clone(),
            ));
            return err(warnings, errors);
        }
        let mut all_variants: HashSet<String> = HashSet::new();
        for variant in enum_variants.iter() {
            all_variants.insert(variant.name.to_string().clone());
        }
        let mut variant_tracker: HashSet<String> = HashSet::new();
        variant_tracker.insert(enum_pattern.variant_name.clone());
        for pat in rest.iter() {
            match pat {
                Pattern::Enum(enum_pattern2) => {
                    if enum_pattern2.enum_name.as_str() != enum_name.as_str() {
                        errors.push(CompileError::Internal(
                            "expected matching enum names",
                            span.clone(),
                        ));
                        return err(warnings, errors);
                    }
                    variant_tracker.insert(enum_pattern2.variant_name.to_string());
                }
                _ => {
                    errors.push(CompileError::Internal(
                        "expected all patterns to be of the same type",
                        span.clone(),
                    ));
                    return err(warnings, errors);
                }
            }
        }
        ok((all_variants, variant_tracker), warnings, errors)
    }
}
