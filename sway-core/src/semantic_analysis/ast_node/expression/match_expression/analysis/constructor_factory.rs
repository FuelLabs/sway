use std::collections::HashSet;

use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
};
use sway_types::{Ident, Span};

use crate::{decl_engine::DeclEngine, language::ty, type_system::TypeId, Engines, TypeInfo};

use super::{
    patstack::PatStack,
    pattern::{EnumPattern, Pattern, StructPattern},
    range::Range,
};

pub(crate) struct ConstructorFactory {
    possible_types: Vec<TypeInfo>,
}

impl ConstructorFactory {
    pub(crate) fn new(engines: &Engines, type_id: TypeId) -> Self {
        let possible_types = type_id.extract_nested_types(engines);
        ConstructorFactory { possible_types }
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
        handler: &Handler,
        engines: &Engines,
        sigma: PatStack,
        span: &Span,
    ) -> Result<Pattern, ErrorEmitted> {
        let (first, rest) = sigma
            .flatten()
            .filter_out_wildcards()
            .split_first(handler, span)?;
        let pat = match first {
            Pattern::U8(range) => {
                let mut ranges = vec![range];
                for pat in rest.into_iter() {
                    match pat {
                        Pattern::U8(range) => ranges.push(range),
                        _ => {
                            return Err(handler.emit_err(CompileError::Internal(
                                "expected all patterns to be of the same type",
                                span.clone(),
                            )));
                        }
                    }
                }
                let unincluded: PatStack =
                    Range::find_exclusionary_ranges(handler, ranges, Range::u8(), span)?
                        .into_iter()
                        .map(Pattern::U8)
                        .collect::<Vec<_>>()
                        .into();
                Pattern::from_pat_stack(handler, unincluded, span)?
            }
            Pattern::U16(range) => {
                let mut ranges = vec![range];
                for pat in rest.into_iter() {
                    match pat {
                        Pattern::U16(range) => ranges.push(range),
                        _ => {
                            return Err(handler.emit_err(CompileError::Internal(
                                "expected all patterns to be of the same type",
                                span.clone(),
                            )));
                        }
                    }
                }
                let unincluded: PatStack =
                    Range::find_exclusionary_ranges(handler, ranges, Range::u16(), span)?
                        .into_iter()
                        .map(Pattern::U16)
                        .collect::<Vec<_>>()
                        .into();
                Pattern::from_pat_stack(handler, unincluded, span)?
            }
            Pattern::U32(range) => {
                let mut ranges = vec![range];
                for pat in rest.into_iter() {
                    match pat {
                        Pattern::U32(range) => ranges.push(range),
                        _ => {
                            return Err(handler.emit_err(CompileError::Internal(
                                "expected all patterns to be of the same type",
                                span.clone(),
                            )));
                        }
                    }
                }
                let unincluded: PatStack =
                    Range::find_exclusionary_ranges(handler, ranges, Range::u32(), span)?
                        .into_iter()
                        .map(Pattern::U32)
                        .collect::<Vec<_>>()
                        .into();
                Pattern::from_pat_stack(handler, unincluded, span)?
            }
            Pattern::U64(range) => {
                let mut ranges = vec![range];
                for pat in rest.into_iter() {
                    match pat {
                        Pattern::U64(range) => ranges.push(range),
                        _ => {
                            return Err(handler.emit_err(CompileError::Internal(
                                "expected all patterns to be of the same type",
                                span.clone(),
                            )));
                        }
                    }
                }
                let unincluded: PatStack =
                    Range::find_exclusionary_ranges(handler, ranges, Range::u64(), span)?
                        .into_iter()
                        .map(Pattern::U64)
                        .collect::<Vec<_>>()
                        .into();
                Pattern::from_pat_stack(handler, unincluded, span)?
            }
            Pattern::Numeric(range) => {
                let mut ranges = vec![range];
                for pat in rest.into_iter() {
                    match pat {
                        Pattern::Numeric(range) => ranges.push(range),
                        _ => {
                            return Err(handler.emit_err(CompileError::Internal(
                                "expected all patterns to be of the same type",
                                span.clone(),
                            )));
                        }
                    }
                }
                let unincluded: PatStack =
                    Range::find_exclusionary_ranges(handler, ranges, Range::u64(), span)?
                        .into_iter()
                        .map(Pattern::Numeric)
                        .collect::<Vec<_>>()
                        .into();
                Pattern::from_pat_stack(handler, unincluded, span)?
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
                    return Err(handler.emit_err(CompileError::Internal(
                        "unable to create a new pattern",
                        span.clone(),
                    )));
                } else if true_found {
                    Pattern::Boolean(false)
                } else {
                    Pattern::Boolean(true)
                }
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
                let type_info = self.resolve_possible_types(handler, pat, span, engines.de())?;
                let enum_decl = engines
                    .de()
                    .get_enum(&type_info.expect_enum(handler, engines, "", span)?);
                let enum_name = &enum_decl.call_path.suffix;
                let enum_variants = &enum_decl.variants;
                let (all_variants, variant_tracker) = ConstructorFactory::resolve_enum(
                    handler,
                    enum_name,
                    enum_variants,
                    enum_pattern,
                    rest,
                    span,
                )?;
                Pattern::from_pat_stack(
                    handler,
                    PatStack::from(
                        all_variants
                            .difference(&variant_tracker)
                            .map(|x| {
                                Pattern::Enum(EnumPattern {
                                    enum_name: enum_name.to_string(),
                                    variant_name: x.clone(),
                                    value: Box::new(Pattern::Wildcard),
                                })
                            })
                            .collect::<Vec<_>>(),
                    ),
                    span,
                )?
            }
            Pattern::Tuple(elems) => Pattern::Tuple(PatStack::fill_wildcards(elems.len())),
            Pattern::Or(elems) => {
                let mut pat_stack = PatStack::empty();
                for pat in elems.into_iter() {
                    pat_stack.push(self.create_pattern_not_present(
                        handler,
                        engines,
                        PatStack::from_pattern(pat),
                        span,
                    )?);
                }
                Pattern::from_pat_stack(handler, pat_stack, span)?
            }
        };
        Ok(pat)
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
        handler: &Handler,
        engines: &Engines,
        pat_stack: &PatStack,
        span: &Span,
    ) -> Result<bool, ErrorEmitted> {
        // flatten or patterns
        let pat_stack = pat_stack
            .clone()
            .serialize_multi_patterns(handler, span)?
            .into_iter()
            .fold(PatStack::empty(), |mut acc, mut pats| {
                acc.append(&mut pats);
                acc
            });

        if pat_stack.is_empty() {
            return Ok(false);
        }
        if pat_stack.contains(&Pattern::Wildcard) {
            return Ok(true);
        }

        let (first, mut rest) = pat_stack.split_first(handler, span)?;
        match first {
            // its assumed that no one is ever going to list every string
            Pattern::String(_) => Ok(false),
            // its assumed that no one is ever going to list every B256
            Pattern::B256(_) => Ok(false),
            Pattern::U8(range) => {
                let mut ranges = vec![range];
                for pat in rest.into_iter() {
                    match pat {
                        Pattern::U8(range) => ranges.push(range),
                        _ => {
                            return Err(handler.emit_err(CompileError::Internal(
                                "expected all patterns to be of the same type",
                                span.clone(),
                            )));
                        }
                    }
                }
                Range::do_ranges_equal_range(handler, ranges, Range::u8(), span)
            }
            Pattern::U16(range) => {
                let mut ranges = vec![range];
                for pat in rest.into_iter() {
                    match pat {
                        Pattern::U16(range) => ranges.push(range),
                        _ => {
                            return Err(handler.emit_err(CompileError::Internal(
                                "expected all patterns to be of the same type",
                                span.clone(),
                            )));
                        }
                    }
                }
                Range::do_ranges_equal_range(handler, ranges, Range::u16(), span)
            }
            Pattern::U32(range) => {
                let mut ranges = vec![range];
                for pat in rest.into_iter() {
                    match pat {
                        Pattern::U32(range) => ranges.push(range),
                        _ => {
                            return Err(handler.emit_err(CompileError::Internal(
                                "expected all patterns to be of the same type",
                                span.clone(),
                            )));
                        }
                    }
                }
                Range::do_ranges_equal_range(handler, ranges, Range::u32(), span)
            }
            Pattern::U64(range) => {
                let mut ranges = vec![range];
                for pat in rest.into_iter() {
                    match pat {
                        Pattern::U64(range) => ranges.push(range),
                        _ => {
                            return Err(handler.emit_err(CompileError::Internal(
                                "expected all patterns to be of the same type",
                                span.clone(),
                            )));
                        }
                    }
                }
                Range::do_ranges_equal_range(handler, ranges, Range::u64(), span)
            }
            Pattern::Numeric(range) => {
                let mut ranges = vec![range];
                for pat in rest.into_iter() {
                    match pat {
                        Pattern::Numeric(range) => ranges.push(range),
                        _ => {
                            return Err(handler.emit_err(CompileError::Internal(
                                "expected all patterns to be of the same type",
                                span.clone(),
                            )));
                        }
                    }
                }
                Range::do_ranges_equal_range(handler, ranges, Range::u64(), span)
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
                            return Err(handler.emit_err(CompileError::Internal(
                                "expected all patterns to be of the same type",
                                span.clone(),
                            )));
                        }
                    }
                }
                Ok(true_found && false_found)
            }
            ref pat @ Pattern::Enum(ref enum_pattern) => {
                let type_info = self.resolve_possible_types(handler, pat, span, engines.de())?;
                let enum_decl = engines
                    .de()
                    .get_enum(&type_info.expect_enum(handler, engines, "", span)?);
                let enum_name = &enum_decl.call_path.suffix;
                let enum_variants = &enum_decl.variants;
                let (all_variants, variant_tracker) = ConstructorFactory::resolve_enum(
                    handler,
                    enum_name,
                    enum_variants,
                    enum_pattern,
                    rest,
                    span,
                )?;
                Ok(all_variants.difference(&variant_tracker).next().is_none())
            }
            ref tup @ Pattern::Tuple(_) => {
                for pat in rest.iter() {
                    if !pat.has_the_same_constructor(tup) {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            ref strct @ Pattern::Struct(_) => {
                for pat in rest.iter() {
                    if !pat.has_the_same_constructor(strct) {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            Pattern::Wildcard => Err(handler.emit_err(CompileError::Internal(
                "expected the wildcard pattern to be filtered out here",
                span.clone(),
            ))),
            Pattern::Or(mut elems) => {
                elems.append(&mut rest);
                Ok(self.is_complete_signature(handler, engines, &elems, span)?)
            }
        }
    }

    fn resolve_possible_types(
        &self,
        handler: &Handler,
        pattern: &Pattern,
        span: &Span,
        decl_engine: &DeclEngine,
    ) -> Result<&TypeInfo, ErrorEmitted> {
        let mut type_info = None;
        for possible_type in self.possible_types.iter() {
            let matches = pattern.matches_type_info(possible_type, decl_engine);
            if matches {
                type_info = Some(possible_type);
                break;
            }
        }
        match type_info {
            Some(type_info) => Ok(type_info),
            None => Err(handler.emit_err(CompileError::Internal(
                "there is no type that matches this pattern",
                span.clone(),
            ))),
        }
    }

    fn resolve_enum(
        handler: &Handler,
        enum_name: &Ident,
        enum_variants: &[ty::TyEnumVariant],
        enum_pattern: &EnumPattern,
        rest: PatStack,
        span: &Span,
    ) -> Result<(HashSet<String>, HashSet<String>), ErrorEmitted> {
        if enum_pattern.enum_name.as_str() != enum_name.as_str() {
            return Err(handler.emit_err(CompileError::Internal(
                "expected matching enum names",
                span.clone(),
            )));
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
                        return Err(handler.emit_err(CompileError::Internal(
                            "expected matching enum names",
                            span.clone(),
                        )));
                    }
                    variant_tracker.insert(enum_pattern2.variant_name.to_string());
                }
                _ => {
                    return Err(handler.emit_err(CompileError::Internal(
                        "expected all patterns to be of the same type",
                        span.clone(),
                    )));
                }
            }
        }
        Ok((all_variants, variant_tracker))
    }
}
