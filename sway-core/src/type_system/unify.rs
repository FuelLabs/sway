use sway_error::{
    type_error::TypeError,
    warning::{CompileWarning, Warning},
};
use sway_types::{integer_bits::IntegerBits, Ident, Span};

use crate::{language::ty, type_system::*};
use sway_types::Spanned;

pub(super) fn unify(
    type_engine: &TypeEngine,
    received: TypeId,
    expected: TypeId,
    span: &Span,
    help_text: &str,
    arguments_are_flipped: bool,
) -> (Vec<CompileWarning>, Vec<TypeError>) {
    use TypeInfo::*;

    // a curried version of this method to use in the helper functions
    let curried = |received: TypeId, expected: TypeId, span: &Span, help_text: &str| {
        unify(
            type_engine,
            received,
            expected,
            span,
            help_text,
            arguments_are_flipped,
        )
    };

    match (
        type_engine.slab.get(received.index()),
        type_engine.slab.get(expected.index()),
    ) {
        // If they have the same `TypeInfo`, then we either compare them for
        // correctness or perform further unification.
        (Boolean, Boolean) => (vec![], vec![]),
        (SelfType, SelfType) => (vec![], vec![]),
        (B256, B256) => (vec![], vec![]),
        (Numeric, Numeric) => (vec![], vec![]),
        (Contract, Contract) => (vec![], vec![]),
        (RawUntypedPtr, RawUntypedPtr) => (vec![], vec![]),
        (RawUntypedSlice, RawUntypedSlice) => (vec![], vec![]),
        (Str(l), Str(r)) => unify::unify_strs(
            received,
            expected,
            span,
            help_text,
            l.val(),
            r.val(),
            arguments_are_flipped,
            type_engine,
        ),
        (Tuple(rfs), Tuple(efs)) if rfs.len() == efs.len() => {
            unify::unify_tuples(help_text, rfs, efs, curried)
        }
        (UnsignedInteger(r), UnsignedInteger(e)) => {
            unify::unify_unsigned_ints(span, r, e, arguments_are_flipped)
        }
        (Numeric, e @ UnsignedInteger(_)) => {
            match type_engine.slab.replace(received, &Numeric, e, type_engine) {
                None => (vec![], vec![]),
                Some(_) => unify(
                    type_engine,
                    received,
                    expected,
                    span,
                    help_text,
                    arguments_are_flipped,
                ),
            }
        }
        (r @ UnsignedInteger(_), Numeric) => {
            match type_engine.slab.replace(expected, &Numeric, r, type_engine) {
                None => (vec![], vec![]),
                Some(_) => unify(
                    type_engine,
                    received,
                    expected,
                    span,
                    help_text,
                    arguments_are_flipped,
                ),
            }
        }
        (
            Struct {
                name: rn,
                type_parameters: rpts,
                fields: rfs,
            },
            Struct {
                name: en,
                type_parameters: etps,
                fields: efs,
            },
        ) => unify::unify_structs(
            received,
            expected,
            span,
            help_text,
            (rn, rpts, rfs),
            (en, etps, efs),
            curried,
            arguments_are_flipped,
            type_engine,
        ),
        (
            Enum {
                name: rn,
                type_parameters: rtps,
                variant_types: rvs,
            },
            Enum {
                name: en,
                type_parameters: etps,
                variant_types: evs,
            },
        ) => unify::unify_enums(
            received,
            expected,
            span,
            help_text,
            (rn, rtps, rvs),
            (en, etps, evs),
            curried,
            arguments_are_flipped,
            type_engine,
        ),
        (Array(re, rc), Array(ee, ec)) if rc.val() == ec.val() => unify::unify_arrays(
            received,
            expected,
            span,
            help_text,
            re.type_id,
            ee.type_id,
            curried,
            arguments_are_flipped,
            type_engine,
        ),
        (
            ref r @ TypeInfo::ContractCaller {
                abi_name: ref ran,
                address: ref rra,
            },
            TypeInfo::ContractCaller {
                abi_name: ref ean, ..
            },
        ) if (ran == ean && rra.is_none()) || matches!(ran, AbiName::Deferred) => {
            // if one address is empty, coerce to the other one
            match type_engine.slab.replace(
                received,
                r,
                type_engine.slab.get(expected.index()),
                type_engine,
            ) {
                None => (vec![], vec![]),
                Some(_) => unify(
                    type_engine,
                    received,
                    expected,
                    span,
                    help_text,
                    arguments_are_flipped,
                ),
            }
        }
        (
            TypeInfo::ContractCaller {
                abi_name: ref ran, ..
            },
            ref e @ TypeInfo::ContractCaller {
                abi_name: ref ean,
                address: ref ea,
            },
        ) if (ran == ean && ea.is_none()) || matches!(ean, AbiName::Deferred) => {
            // if one address is empty, coerce to the other one
            match type_engine.slab.replace(
                expected,
                e,
                type_engine.slab.get(received.index()),
                type_engine,
            ) {
                None => (vec![], vec![]),
                Some(_) => unify(
                    type_engine,
                    received,
                    expected,
                    span,
                    help_text,
                    arguments_are_flipped,
                ),
            }
        }
        (ref r @ TypeInfo::ContractCaller { .. }, ref e @ TypeInfo::ContractCaller { .. })
            if r.eq(e, type_engine) =>
        {
            // if they are the same, then it's ok
            (vec![], vec![])
        }

        // When we don't know anything about either term, assume that
        // they match and make the one we know nothing about reference the
        // one we may know something about
        (Unknown, Unknown) => (vec![], vec![]),
        (Unknown, e) => match type_engine.slab.replace(received, &Unknown, e, type_engine) {
            None => (vec![], vec![]),
            Some(_) => unify(
                type_engine,
                received,
                expected,
                span,
                help_text,
                arguments_are_flipped,
            ),
        },
        (r, Unknown) => match type_engine.slab.replace(expected, &Unknown, r, type_engine) {
            None => (vec![], vec![]),
            Some(_) => unify(
                type_engine,
                received,
                expected,
                span,
                help_text,
                arguments_are_flipped,
            ),
        },

        (
            UnknownGeneric {
                name: rn,
                trait_constraints: rtc,
            },
            UnknownGeneric {
                name: en,
                trait_constraints: etc,
            },
        ) if rn.as_str() == en.as_str() && rtc.eq(&etc, type_engine) => {
            type_engine.insert_unified_type(received, expected);
            type_engine.insert_unified_type(expected, received);
            (vec![], vec![])
        }
        (ref r @ UnknownGeneric { .. }, e) => {
            match type_engine.slab.replace(received, r, e, type_engine) {
                None => (vec![], vec![]),
                Some(_) => unify(
                    type_engine,
                    received,
                    expected,
                    span,
                    help_text,
                    arguments_are_flipped,
                ),
            }
        }
        (r, ref e @ UnknownGeneric { .. }) => {
            match type_engine.slab.replace(expected, e, r, type_engine) {
                None => (vec![], vec![]),
                Some(_) => unify(
                    type_engine,
                    received,
                    expected,
                    span,
                    help_text,
                    arguments_are_flipped,
                ),
            }
        }

        // If no previous attempts to unify were successful, raise an error
        (TypeInfo::ErrorRecovery, _) => (vec![], vec![]),
        (_, TypeInfo::ErrorRecovery) => (vec![], vec![]),
        (r, e) => {
            let e = type_engine.help_out(e).to_string();
            let r = type_engine.help_out(r).to_string();
            let (expected, received) = if !arguments_are_flipped {
                (e, r)
            } else {
                (r, e)
            };
            let errors = vec![TypeError::MismatchedType {
                expected,
                received,
                help_text: help_text.to_string(),
                span: span.clone(),
            }];
            (vec![], errors)
        }
    }
}

pub(super) fn unify_right(
    type_engine: &TypeEngine,
    received: TypeId,
    expected: TypeId,
    span: &Span,
    help_text: &str,
) -> (Vec<CompileWarning>, Vec<TypeError>) {
    use TypeInfo::*;

    // a curried version of this method to use in the helper functions
    let curried = |received: TypeId, expected: TypeId, span: &Span, help_text: &str| {
        unify_right(type_engine, received, expected, span, help_text)
    };

    match (
        type_engine.slab.get(received.index()),
        type_engine.slab.get(expected.index()),
    ) {
        // If they have the same `TypeInfo`, then we either compare them for
        // correctness or perform further unification.
        (Boolean, Boolean) => (vec![], vec![]),
        (SelfType, SelfType) => (vec![], vec![]),
        (B256, B256) => (vec![], vec![]),
        (Numeric, Numeric) => (vec![], vec![]),
        (Contract, Contract) => (vec![], vec![]),
        (RawUntypedPtr, RawUntypedPtr) => (vec![], vec![]),
        (RawUntypedSlice, RawUntypedSlice) => (vec![], vec![]),
        (Str(l), Str(r)) => unify::unify_strs(
            received,
            expected,
            span,
            help_text,
            l.val(),
            r.val(),
            false,
            type_engine,
        ),
        (Tuple(rfs), Tuple(efs)) if rfs.len() == efs.len() => {
            unify::unify_tuples(help_text, rfs, efs, curried)
        }
        (UnsignedInteger(r), UnsignedInteger(e)) => unify::unify_unsigned_ints(span, r, e, false),
        (Numeric, UnsignedInteger(_)) => (vec![], vec![]),
        (r @ UnsignedInteger(_), Numeric) => {
            match type_engine.slab.replace(expected, &Numeric, r, type_engine) {
                None => (vec![], vec![]),
                Some(_) => unify_right(type_engine, received, expected, span, help_text),
            }
        }
        (
            Struct {
                name: rn,
                type_parameters: rpts,
                fields: rfs,
            },
            Struct {
                name: en,
                type_parameters: etps,
                fields: efs,
            },
        ) => unify::unify_structs(
            received,
            expected,
            span,
            help_text,
            (rn, rpts, rfs),
            (en, etps, efs),
            curried,
            false,
            type_engine,
        ),
        (
            Enum {
                name: rn,
                type_parameters: rtps,
                variant_types: rvs,
            },
            Enum {
                name: en,
                type_parameters: etps,
                variant_types: evs,
            },
        ) => unify::unify_enums(
            received,
            expected,
            span,
            help_text,
            (rn, rtps, rvs),
            (en, etps, evs),
            curried,
            false,
            type_engine,
        ),
        (Array(re, rc), Array(ee, ec)) if rc.val() == ec.val() => unify::unify_arrays(
            received,
            expected,
            span,
            help_text,
            re.type_id,
            ee.type_id,
            curried,
            false,
            type_engine,
        ),
        (
            TypeInfo::ContractCaller {
                abi_name: ref ran, ..
            },
            ref e @ TypeInfo::ContractCaller {
                abi_name: ref ean,
                address: ref ea,
            },
        ) if (ran == ean && ea.is_none()) || matches!(ean, AbiName::Deferred) => {
            // if one address is empty, coerce to the other one
            match type_engine.slab.replace(
                expected,
                e,
                type_engine.slab.get(received.index()),
                type_engine,
            ) {
                None => (vec![], vec![]),
                Some(_) => unify_right(type_engine, received, expected, span, help_text),
            }
        }
        (
            TypeInfo::ContractCaller {
                abi_name: ref ran,
                address: ref ra,
            },
            TypeInfo::ContractCaller {
                abi_name: ref ean, ..
            },
        ) if (ran == ean && ra.is_none()) || matches!(ran, AbiName::Deferred) => (vec![], vec![]),
        (ref r @ TypeInfo::ContractCaller { .. }, ref e @ TypeInfo::ContractCaller { .. })
            if r.eq(e, type_engine) =>
        {
            // if they are the same, then it's ok
            (vec![], vec![])
        }

        // When we don't know anything about either term, assume that
        // they match and make the one we know nothing about reference the
        // one we may know something about
        (Unknown, Unknown) => (vec![], vec![]),
        (r, Unknown) => match type_engine.slab.replace(expected, &Unknown, r, type_engine) {
            None => (vec![], vec![]),
            Some(_) => unify_right(type_engine, received, expected, span, help_text),
        },
        (Unknown, _) => (vec![], vec![]),

        (
            UnknownGeneric {
                name: rn,
                trait_constraints: rtc,
            },
            UnknownGeneric {
                name: en,
                trait_constraints: etc,
            },
        ) if rn.as_str() == en.as_str() && rtc.eq(&etc, type_engine) => {
            type_engine.insert_unified_type(received, expected);
            (vec![], vec![])
        }
        (r, ref e @ UnknownGeneric { .. }) => {
            type_engine.insert_unified_type(received, expected);
            match type_engine.slab.replace(expected, e, r, type_engine) {
                None => (vec![], vec![]),
                Some(_) => unify_right(type_engine, received, expected, span, help_text),
            }
        }
        // this case is purposefully removed because it should cause an
        // error. trying to unify_right a generic with anything other an an
        // unknown or another generic is a type error
        // (UnknownGeneric { .. }, _) => (vec![], vec![]),

        // If no previous attempts to unify were successful, raise an error
        (TypeInfo::ErrorRecovery, _) => (vec![], vec![]),
        (_, TypeInfo::ErrorRecovery) => (vec![], vec![]),
        (r, e) => {
            let errors = vec![TypeError::MismatchedType {
                expected: type_engine.help_out(e).to_string(),
                received: type_engine.help_out(r).to_string(),
                help_text: help_text.to_string(),
                span: span.clone(),
            }];
            (vec![], errors)
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn unify_strs(
    received: TypeId,
    expected: TypeId,
    span: &Span,
    help_text: &str,
    r: usize,
    e: usize,
    arguments_are_flipped: bool,
    type_engine: &TypeEngine,
) -> (Vec<CompileWarning>, Vec<TypeError>) {
    let warnings = vec![];
    let mut errors = vec![];
    if r != e {
        let expected = type_engine.help_out(expected).to_string();
        let received = type_engine.help_out(received).to_string();
        let (expected, received) = if arguments_are_flipped {
            (received, expected)
        } else {
            (expected, received)
        };
        errors.push(TypeError::MismatchedType {
            expected,
            received,
            help_text: help_text.to_string(),
            span: span.clone(),
        });
    }
    (warnings, errors)
}

fn unify_tuples<F>(
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

fn unify_unsigned_ints(
    span: &Span,
    r: IntegerBits,
    e: IntegerBits,
    arguments_are_flipped: bool,
) -> (Vec<CompileWarning>, Vec<TypeError>) {
    // E.g., in a variable declaration `let a: u32 = 10u64` the 'expected' type will be
    // the annotation `u32`, and the 'received' type is 'self' of the initialiser, or
    // `u64`.  So we're casting received TO expected.
    let warnings = match numeric_cast_compat(e, r, arguments_are_flipped) {
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

fn numeric_cast_compat(
    new_size: IntegerBits,
    old_size: IntegerBits,
    arguments_are_flipped: bool,
) -> NumericCastCompatResult {
    let (new_size, old_size) = if !arguments_are_flipped {
        (new_size, old_size)
    } else {
        (old_size, new_size)
    };
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

#[allow(clippy::too_many_arguments)]
fn unify_structs<F>(
    received: TypeId,
    expected: TypeId,
    span: &Span,
    help_text: &str,
    r: (Ident, Vec<TypeParameter>, Vec<ty::TyStructField>),
    e: (Ident, Vec<TypeParameter>, Vec<ty::TyStructField>),
    unifier: F,
    arguments_are_flipped: bool,
    type_engine: &TypeEngine,
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
        let expected = type_engine.help_out(expected).to_string();
        let received = type_engine.help_out(received).to_string();
        let (expected, received) = if arguments_are_flipped {
            (received, expected)
        } else {
            (expected, received)
        };
        errors.push(TypeError::MismatchedType {
            expected,
            received,
            help_text: help_text.to_string(),
            span: span.clone(),
        });
    }
    (warnings, errors)
}

#[allow(clippy::too_many_arguments)]
fn unify_enums<F>(
    received: TypeId,
    expected: TypeId,
    span: &Span,
    help_text: &str,
    r: (Ident, Vec<TypeParameter>, Vec<ty::TyEnumVariant>),
    e: (Ident, Vec<TypeParameter>, Vec<ty::TyEnumVariant>),
    unifier: F,
    arguments_are_flipped: bool,
    type_engine: &TypeEngine,
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
        let expected = type_engine.help_out(expected).to_string();
        let received = type_engine.help_out(received).to_string();
        let (expected, received) = if arguments_are_flipped {
            (received, expected)
        } else {
            (expected, received)
        };
        errors.push(TypeError::MismatchedType {
            expected,
            received,
            help_text: help_text.to_string(),
            span: span.clone(),
        });
    }
    (warnings, errors)
}

#[allow(clippy::too_many_arguments)]
fn unify_arrays<F>(
    received: TypeId,
    expected: TypeId,
    span: &Span,
    help_text: &str,
    r: TypeId,
    e: TypeId,
    unifier: F,
    arguments_are_flipped: bool,
    type_engine: &TypeEngine,
) -> (Vec<CompileWarning>, Vec<TypeError>)
where
    F: Fn(TypeId, TypeId, &Span, &str) -> (Vec<CompileWarning>, Vec<TypeError>),
{
    let (warnings, new_errors) = unifier(r, e, span, help_text);

    // If there was an error then we want to report the array types as mismatching, not
    // the elem types.
    let mut errors = vec![];
    if !new_errors.is_empty() {
        let expected = type_engine.help_out(expected).to_string();
        let received = type_engine.help_out(received).to_string();
        let (expected, received) = if arguments_are_flipped {
            (received, expected)
        } else {
            (expected, received)
        };
        errors.push(TypeError::MismatchedType {
            expected,
            received,
            help_text: help_text.to_string(),
            span: span.clone(),
        });
    }
    (warnings, errors)
}
