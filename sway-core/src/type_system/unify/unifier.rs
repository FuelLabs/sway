use std::fmt;

use sway_error::{type_error::TypeError, warning::CompileWarning};
use sway_types::{Ident, Span};

use crate::{engine_threading::*, language::ty, type_system::priv_prelude::*};

use super::occurs_check::OccursCheck;

/// Helper struct to aid in type unification.
pub(crate) struct Unifier<'a> {
    engines: &'a Engines,
    help_text: String,
}

impl<'a> Unifier<'a> {
    /// Creates a new [Unifier].
    pub(crate) fn new(engines: &'a Engines, help_text: &str) -> Unifier<'a> {
        Unifier {
            engines,
            help_text: help_text.to_string(),
        }
    }

    /// Helper method for replacing the values in the [TypeEngine].
    fn replace_received_with_expected(
        &self,
        received: TypeId,
        expected: TypeId,
        received_type_info: &TypeInfo,
        expected_type_info: TypeInfo,
        span: &Span,
    ) -> (Vec<CompileWarning>, Vec<TypeError>) {
        match self.engines.te().slab.replace(
            received,
            received_type_info,
            expected_type_info,
            self.engines,
        ) {
            None => (vec![], vec![]),
            Some(_) => self.unify(received, expected, span),
        }
    }

    /// Helper method for replacing the values in the [TypeEngine].
    fn replace_expected_with_received(
        &self,
        received: TypeId,
        expected: TypeId,
        received_type_info: TypeInfo,
        expected_type_info: &TypeInfo,
        span: &Span,
    ) -> (Vec<CompileWarning>, Vec<TypeError>) {
        match self.engines.te().slab.replace(
            expected,
            expected_type_info,
            received_type_info,
            self.engines,
        ) {
            None => (vec![], vec![]),
            Some(_) => self.unify(received, expected, span),
        }
    }

    /// Performs type unification with `received` and `expected`.
    pub(crate) fn unify(
        &self,
        received: TypeId,
        expected: TypeId,
        span: &Span,
    ) -> (Vec<CompileWarning>, Vec<TypeError>) {
        use TypeInfo::*;

        if received == expected {
            return (vec![], vec![]);
        }

        match (
            self.engines.te().slab.get(received.index()),
            self.engines.te().slab.get(expected.index()),
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
            (Str(l), Str(r)) => self.unify_strs(received, expected, span, l.val(), r.val()),
            (Tuple(rfs), Tuple(efs)) if rfs.len() == efs.len() => self.unify_tuples(rfs, efs),
            (Array(re, rc), Array(ee, ec)) if rc.val() == ec.val() => {
                self.unify_arrays(received, expected, span, re.type_id, ee.type_id)
            }
            (Struct(r_decl_ref), Struct(e_decl_ref)) => {
                let r_decl = self.engines.de().get_struct(&r_decl_ref);
                let e_decl = self.engines.de().get_struct(&e_decl_ref);

                self.unify_structs(
                    received,
                    expected,
                    span,
                    (
                        r_decl.call_path.suffix,
                        r_decl.type_parameters,
                        r_decl.fields,
                    ),
                    (
                        e_decl.call_path.suffix,
                        e_decl.type_parameters,
                        e_decl.fields,
                    ),
                )
            }

            // Type aliases and the types they encapsulate coerce to each other.
            (Alias { ty, .. }, _) => self.unify(ty.type_id, expected, span),
            (_, Alias { ty, .. }) => self.unify(received, ty.type_id, span),

            // Let empty enums to coerce to any other type. This is useful for Never enum.
            (Enum(r_decl_ref), _)
                if self.engines.de().get_enum(&r_decl_ref).variants.is_empty() =>
            {
                (vec![], vec![])
            }
            (Enum(r_decl_ref), Enum(e_decl_ref)) => {
                let r_decl = self.engines.de().get_enum(&r_decl_ref);
                let e_decl = self.engines.de().get_enum(&e_decl_ref);

                self.unify_enums(
                    received,
                    expected,
                    span,
                    (
                        r_decl.call_path.suffix,
                        r_decl.type_parameters,
                        r_decl.variants,
                    ),
                    (
                        e_decl.call_path.suffix,
                        e_decl.type_parameters,
                        e_decl.variants,
                    ),
                )
            }

            // For integers and numerics, we (potentially) unify the numeric
            // with the integer.
            (UnsignedInteger(r), UnsignedInteger(e)) if r == e => (vec![], vec![]),
            (Numeric, e @ UnsignedInteger(_)) => {
                self.replace_received_with_expected(received, expected, &Numeric, e, span)
            }
            (r @ UnsignedInteger(_), Numeric) => {
                self.replace_expected_with_received(received, expected, r, &Numeric, span)
            }

            // For contract callers, we (potentially) unify them if they have
            // the same name and their address is `None`
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
                self.replace_received_with_expected(
                    received,
                    expected,
                    r,
                    self.engines.te().slab.get(expected.index()),
                    span,
                )
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
                self.replace_expected_with_received(
                    received,
                    expected,
                    self.engines.te().slab.get(received.index()),
                    e,
                    span,
                )
            }
            (ref r @ TypeInfo::ContractCaller { .. }, ref e @ TypeInfo::ContractCaller { .. })
                if r.eq(e, self.engines) =>
            {
                // if they are the same, then it's ok
                (vec![], vec![])
            }

            // When we don't know anything about either term, assume that
            // they match and make the one we know nothing about reference the
            // one we may know something about.
            (Unknown, Unknown) => (vec![], vec![]),
            (Unknown, e) => {
                self.replace_received_with_expected(received, expected, &Unknown, e, span)
            }
            (r, Unknown) => {
                self.replace_expected_with_received(received, expected, r, &Unknown, span)
            }

            (r @ Placeholder(_), e @ Placeholder(_)) => {
                self.replace_expected_with_received(received, expected, r, &e, span)
            }
            (r @ Placeholder(_), e) => {
                self.replace_received_with_expected(received, expected, &r, e, span)
            }
            (r, e @ Placeholder(_)) => {
                self.replace_expected_with_received(received, expected, r, &e, span)
            }

            // Generics are handled similarly to the case for unknowns, except
            // we take more careful consideration for the type/purpose for the
            // unification that we are performing.
            (
                UnknownGeneric {
                    name: rn,
                    trait_constraints: rtc,
                },
                UnknownGeneric {
                    name: en,
                    trait_constraints: etc,
                },
            ) if rn.as_str() == en.as_str() && rtc.eq(&etc, self.engines) => (vec![], vec![]),
            (r @ UnknownGeneric { .. }, e) if !self.occurs_check(r.clone(), &e) => {
                self.replace_received_with_expected(received, expected, &r, e, span)
            }
            (r, e @ UnknownGeneric { .. }) if !self.occurs_check(e.clone(), &r) => {
                self.replace_expected_with_received(received, expected, r, &e, span)
            }

            // If no previous attempts to unify were successful, raise an error.
            (TypeInfo::ErrorRecovery, _) => (vec![], vec![]),
            (_, TypeInfo::ErrorRecovery) => (vec![], vec![]),
            (r, e) => {
                let (received, expected) = self.assign_args(r, e);
                let errors = vec![TypeError::MismatchedType {
                    expected,
                    received,
                    help_text: self.help_text.clone(),
                    span: span.clone(),
                }];
                (vec![], errors)
            }
        }
    }

    fn occurs_check(&self, generic: TypeInfo, other: &TypeInfo) -> bool {
        OccursCheck::new(self.engines).check(generic, other)
    }

    fn unify_strs(
        &self,
        received: TypeId,
        expected: TypeId,
        span: &Span,
        r: usize,
        e: usize,
    ) -> (Vec<CompileWarning>, Vec<TypeError>) {
        let warnings = vec![];
        let mut errors = vec![];
        if r != e {
            let (received, expected) = self.assign_args(received, expected);
            errors.push(TypeError::MismatchedType {
                expected,
                received,
                help_text: self.help_text.clone(),
                span: span.clone(),
            });
        }
        (warnings, errors)
    }

    fn unify_tuples(
        &self,
        rfs: Vec<TypeArgument>,
        efs: Vec<TypeArgument>,
    ) -> (Vec<CompileWarning>, Vec<TypeError>) {
        let mut warnings = vec![];
        let mut errors = vec![];
        for (rf, ef) in rfs.iter().zip(efs.iter()) {
            let (mut warns, mut errs) = self.unify(rf.type_id, ef.type_id, &rf.span);
            warnings.append(&mut warns);
            errors.append(&mut errs);
        }
        (warnings, errors)
    }

    fn unify_structs(
        &self,
        received: TypeId,
        expected: TypeId,
        span: &Span,
        r: (Ident, Vec<TypeParameter>, Vec<ty::TyStructField>),
        e: (Ident, Vec<TypeParameter>, Vec<ty::TyStructField>),
    ) -> (Vec<CompileWarning>, Vec<TypeError>) {
        let mut warnings = vec![];
        let mut errors = vec![];
        let (rn, rtps, rfs) = r;
        let (en, etps, efs) = e;
        if rn == en && rfs.len() == efs.len() && rtps.len() == etps.len() {
            rfs.iter().zip(efs.iter()).for_each(|(rf, ef)| {
                let (mut warns, mut errs) =
                    self.unify(rf.type_argument.type_id, ef.type_argument.type_id, span);
                warnings.append(&mut warns);
                errors.append(&mut errs);
            });
            rtps.iter().zip(etps.iter()).for_each(|(rtp, etp)| {
                let (mut warns, mut errs) = self.unify(rtp.type_id, etp.type_id, span);
                warnings.append(&mut warns);
                errors.append(&mut errs);
            });
        } else {
            let (received, expected) = self.assign_args(received, expected);
            errors.push(TypeError::MismatchedType {
                expected,
                received,
                help_text: self.help_text.clone(),
                span: span.clone(),
            });
        }
        (warnings, errors)
    }

    fn unify_enums(
        &self,
        received: TypeId,
        expected: TypeId,
        span: &Span,
        r: (Ident, Vec<TypeParameter>, Vec<ty::TyEnumVariant>),
        e: (Ident, Vec<TypeParameter>, Vec<ty::TyEnumVariant>),
    ) -> (Vec<CompileWarning>, Vec<TypeError>) {
        let mut warnings = vec![];
        let mut errors = vec![];
        let (rn, rtps, rvs) = r;
        let (en, etps, evs) = e;
        if rn == en && rvs.len() == evs.len() && rtps.len() == etps.len() {
            rvs.iter().zip(evs.iter()).for_each(|(rv, ev)| {
                let (mut warns, mut errs) =
                    self.unify(rv.type_argument.type_id, ev.type_argument.type_id, span);
                warnings.append(&mut warns);
                errors.append(&mut errs);
            });
            rtps.iter().zip(etps.iter()).for_each(|(rtp, etp)| {
                let (mut warns, mut errs) = self.unify(rtp.type_id, etp.type_id, span);
                warnings.append(&mut warns);
                errors.append(&mut errs);
            });
        } else {
            let (received, expected) = self.assign_args(received, expected);
            errors.push(TypeError::MismatchedType {
                expected,
                received,
                help_text: self.help_text.clone(),
                span: span.clone(),
            });
        }
        (warnings, errors)
    }

    fn unify_arrays(
        &self,
        received: TypeId,
        expected: TypeId,
        span: &Span,
        r: TypeId,
        e: TypeId,
    ) -> (Vec<CompileWarning>, Vec<TypeError>) {
        let (warnings, new_errors) = self.unify(r, e, span);

        // If there was an error then we want to report the array types as mismatching, not
        // the elem types.
        let mut errors = vec![];
        if !new_errors.is_empty() {
            let (received, expected) = self.assign_args(received, expected);
            errors.push(TypeError::MismatchedType {
                expected,
                received,
                help_text: self.help_text.clone(),
                span: span.clone(),
            });
        }
        (warnings, errors)
    }

    fn assign_args<T>(&self, r: T, e: T) -> (String, String)
    where
        WithEngines<'a, T>: fmt::Debug,
    {
        let r = format!("{:?}", self.engines.help_out(r));
        let e = format!("{:?}", self.engines.help_out(e));
        (r, e)
    }
}
