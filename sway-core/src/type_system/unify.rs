use sway_types::{Ident, Span};

use crate::{
    error::{TypeError, Warning},
    semantic_analysis::{TyEnumVariant, TyStructField},
    CompileWarning, IntegerBits, TypeArgument, TypeId, TypeParameter,
};
use sway_types::Spanned;

pub(super) fn unify_strs(
    received: TypeId,
    expected: TypeId,
    span: &Span,
    help_text: &str,
    r: u64,
    e: u64,
) -> (Vec<CompileWarning>, Vec<TypeError>) {
    let warnings = vec![];
    let mut errors = vec![];
    if r != e {
        errors.push(TypeError::MismatchedType {
            expected,
            received,
            help_text: help_text.to_string(),
            span: span.clone(),
        });
    }
    (warnings, errors)
}

pub(super) fn unify_tuples<F>(
    help_text: &str,
    rfs: Vec<TypeArgument>,
    efs: Vec<TypeArgument>,
    unifier: F,
) -> (Vec<CompileWarning>, Vec<TypeError>)
where
    F: Fn(TypeId, TypeId, &Span, &str) -> (Vec<CompileWarning>, Vec<TypeError>),
{
    let mut warnings = vec![];
    let mut errors = vec![];
    for (rf, ef) in rfs.iter().zip(efs.iter()) {
        append!(
            unifier(rf.type_id, ef.type_id, &rf.span, help_text),
            warnings,
            errors
        );
    }
    (warnings, errors)
}

pub(super) fn unify_unsigned_ints(
    span: &Span,
    r: IntegerBits,
    e: IntegerBits,
) -> (Vec<CompileWarning>, Vec<TypeError>) {
    // E.g., in a variable declaration `let a: u32 = 10u64` the 'expected' type will be
    // the annotation `u32`, and the 'received' type is 'self' of the initialiser, or
    // `u64`.  So we're casting received TO expected.
    let warnings = match numeric_cast_compat(e, r) {
        NumericCastCompatResult::CastableWithWarning(warn) => {
            vec![CompileWarning {
                span: span.clone(),
                warning_content: warn,
            }]
        }
        NumericCastCompatResult::Compatible => {
            vec![]
        }
    };

    // we don't want to do a slab replacement here, because
    // we don't want to overwrite the original numeric type with the new one.
    // This isn't actually inferencing the original type to the new numeric type.
    // We just want to say "up until this point, this was a u32 (eg) and now it is a
    // u64 (eg)". If we were to do a slab replace here, we'd be saying "this was always a
    // u64 (eg)".
    (warnings, vec![])
}

fn numeric_cast_compat(new_size: IntegerBits, old_size: IntegerBits) -> NumericCastCompatResult {
    // If this is a downcast, warn for loss of precision. If upcast, then no warning.
    use IntegerBits::*;
    match (new_size, old_size) {
        // These should generate a downcast warning.
        (Eight, Sixteen)
        | (Eight, ThirtyTwo)
        | (Eight, SixtyFour)
        | (Sixteen, ThirtyTwo)
        | (Sixteen, SixtyFour)
        | (ThirtyTwo, SixtyFour) => {
            NumericCastCompatResult::CastableWithWarning(Warning::LossOfPrecision {
                initial_type: old_size,
                cast_to: new_size,
            })
        }
        // Upcasting is ok, so everything else is ok.
        _ => NumericCastCompatResult::Compatible,
    }
}
enum NumericCastCompatResult {
    Compatible,
    CastableWithWarning(Warning),
}

pub(super) fn unify_structs<F>(
    received: TypeId,
    expected: TypeId,
    span: &Span,
    help_text: &str,
    r: (Ident, Vec<TypeParameter>, Vec<TyStructField>),
    e: (Ident, Vec<TypeParameter>, Vec<TyStructField>),
    unifier: F,
) -> (Vec<CompileWarning>, Vec<TypeError>)
where
    F: Fn(TypeId, TypeId, &Span, &str) -> (Vec<CompileWarning>, Vec<TypeError>),
{
    let mut warnings = vec![];
    let mut errors = vec![];
    let (rn, rtps, rfs) = r;
    let (en, etps, efs) = e;
    if rn == en && rfs.len() == efs.len() && rtps.len() == etps.len() {
        rfs.iter().zip(efs.iter()).for_each(|(rf, ef)| {
            append!(
                unifier(rf.type_id, ef.type_id, &rf.span, help_text),
                warnings,
                errors
            );
        });
        rtps.iter().zip(etps.iter()).for_each(|(rtp, etp)| {
            append!(
                unifier(rtp.type_id, etp.type_id, &rtp.name_ident.span(), help_text,),
                warnings,
                errors
            );
        });
    } else {
        errors.push(TypeError::MismatchedType {
            expected,
            received,
            help_text: help_text.to_string(),
            span: span.clone(),
        });
    }
    (warnings, errors)
}

pub(super) fn unify_enums<F>(
    received: TypeId,
    expected: TypeId,
    span: &Span,
    help_text: &str,
    r: (Ident, Vec<TypeParameter>, Vec<TyEnumVariant>),
    e: (Ident, Vec<TypeParameter>, Vec<TyEnumVariant>),
    unifier: F,
) -> (Vec<CompileWarning>, Vec<TypeError>)
where
    F: Fn(TypeId, TypeId, &Span, &str) -> (Vec<CompileWarning>, Vec<TypeError>),
{
    let mut warnings = vec![];
    let mut errors = vec![];
    let (rn, rtps, rvs) = r;
    let (en, etps, evs) = e;
    if rn == en && rvs.len() == evs.len() && rtps.len() == etps.len() {
        rvs.iter().zip(evs.iter()).for_each(|(rv, ev)| {
            append!(
                unifier(rv.type_id, ev.type_id, &rv.span, help_text),
                warnings,
                errors
            );
        });
        rtps.iter().zip(etps.iter()).for_each(|(rtp, etp)| {
            append!(
                unifier(rtp.type_id, etp.type_id, &rtp.name_ident.span(), help_text,),
                warnings,
                errors
            );
        });
    } else {
        errors.push(TypeError::MismatchedType {
            expected,
            received,
            help_text: help_text.to_string(),
            span: span.clone(),
        });
    }
    (warnings, errors)
}

pub(super) fn unify_arrays<F>(
    received: TypeId,
    expected: TypeId,
    span: &Span,
    help_text: &str,
    r: TypeId,
    e: TypeId,
    unifier: F,
) -> (Vec<CompileWarning>, Vec<TypeError>)
where
    F: Fn(TypeId, TypeId, &Span, &str) -> (Vec<CompileWarning>, Vec<TypeError>),
{
    let (warnings, new_errors) = unifier(r, e, span, help_text);

    // If there was an error then we want to report the array types as mismatching, not
    // the elem types.
    let mut errors = vec![];
    if !new_errors.is_empty() {
        errors.push(TypeError::MismatchedType {
            expected,
            received,
            help_text: help_text.to_string(),
            span: span.clone(),
        });
    }
    (warnings, errors)
}
