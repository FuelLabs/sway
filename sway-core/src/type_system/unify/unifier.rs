use std::fmt;

use sway_error::{handler::Handler, type_error::TypeError};
use sway_types::{Ident, Span};

use crate::{engine_threading::*, language::ty, type_system::priv_prelude::*};

use super::occurs_check::OccursCheck;

pub(crate) enum UnifyKind {
    /// Make the types of `received` and `expected` equivalent (or produce an
    /// error if there is a conflict between them).
    ///
    /// More specifically, this function tries to make `received` equivalent to
    /// `expected`.
    Default,
    /// Make the types of `received` and `expected` equivalent (or produce an
    /// error if there is a conflict between them).
    ///
    /// More specifically, this function tries to make `received` equivalent to
    /// `expected`, except in cases where `received` has more type information
    /// than `expected` (e.g. when `expected` is a self type and `received`
    /// is not).
    WithSelf,
    /// Make the types of `received` and `expected` equivalent (or produce an
    /// error if there is a conflict between them).
    ///
    /// More specifically, this function tries to make `received` equivalent to
    /// `expected`, except in cases where `received` has more type information
    /// than `expected` (e.g. when `expected` is a generic type and `received`
    /// is not).
    WithGeneric,
}

/// Helper struct to aid in type unification.
pub(crate) struct Unifier<'a> {
    engines: &'a Engines,
    help_text: String,
    unify_kind: UnifyKind,
}

impl<'a> Unifier<'a> {
    /// Creates a new [Unifier].
    pub(crate) fn new(engines: &'a Engines, help_text: &str, unify_kind: UnifyKind) -> Unifier<'a> {
        Unifier {
            engines,
            help_text: help_text.to_string(),
            unify_kind,
        }
    }

    /// Helper method for replacing the values in the [TypeEngine].
    fn replace_received_with_expected(
        &self,
        handler: &Handler,
        received: TypeId,
        expected: TypeId,
        received_type_info: &TypeInfo,
        expected_type_info: TypeInfo,
        span: &Span,
    ) {
        let type_engine = self.engines.te();
        if type_engine
            .slab
            .replace(
                received,
                received_type_info,
                expected_type_info,
                self.engines,
            )
            .is_some()
        {
            self.unify(handler, received, expected, span);
        }
    }

    /// Helper method for replacing the values in the [TypeEngine].
    fn replace_expected_with_received(
        &self,
        handler: &Handler,
        received: TypeId,
        expected: TypeId,
        received_type_info: TypeInfo,
        expected_type_info: &TypeInfo,
        span: &Span,
    ) {
        let type_engine = self.engines.te();
        if type_engine
            .slab
            .replace(
                expected,
                expected_type_info,
                received_type_info,
                self.engines,
            )
            .is_some()
        {
            self.unify(handler, received, expected, span);
        }
    }

    /// Performs type unification with `received` and `expected`.
    pub(crate) fn unify(&self, handler: &Handler, received: TypeId, expected: TypeId, span: &Span) {
        use TypeInfo::*;

        if received == expected {
            return;
        }

        match (
            self.engines.te().slab.get(received.index()),
            self.engines.te().slab.get(expected.index()),
        ) {
            // If they have the same `TypeInfo`, then we either compare them for
            // correctness or perform further unification.
            (Boolean, Boolean) => (),
            (B256, B256) => (),
            (Numeric, Numeric) => (),
            (Contract, Contract) => (),
            (RawUntypedPtr, RawUntypedPtr) => (),
            (RawUntypedSlice, RawUntypedSlice) => (),
            (StringSlice, StringSlice) => (),
            (StringArray(l), StringArray(r)) => {
                self.unify_strs(handler, received, expected, span, l.val(), r.val())
            }
            (Tuple(rfs), Tuple(efs)) if rfs.len() == efs.len() => {
                self.unify_tuples(handler, rfs, efs)
            }
            (Array(re, rc), Array(ee, ec)) if rc.val() == ec.val() => {
                self.unify_arrays(handler, received, expected, span, re.type_id, ee.type_id)
            }
            (Struct(r_decl_ref), Struct(e_decl_ref)) => {
                let r_decl = self.engines.de().get_struct(&r_decl_ref);
                let e_decl = self.engines.de().get_struct(&e_decl_ref);

                self.unify_structs(
                    handler,
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

            // When we don't know anything about either term, assume that
            // they match and make the one we know nothing about reference the
            // one we may know something about.
            (Unknown, Unknown) => (),
            (Unknown, e) => {
                self.replace_received_with_expected(handler, received, expected, &Unknown, e, span)
            }
            (r, Unknown) => {
                self.replace_expected_with_received(handler, received, expected, r, &Unknown, span)
            }

            (r @ Placeholder(_), e @ Placeholder(_)) => {
                self.replace_expected_with_received(handler, received, expected, r, &e, span)
            }
            (r @ Placeholder(_), e) => {
                self.replace_received_with_expected(handler, received, expected, &r, e, span)
            }
            (r, e @ Placeholder(_)) => {
                self.replace_expected_with_received(handler, received, expected, r, &e, span)
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
            ) if rn.as_str() == en.as_str() && rtc.eq(&etc, self.engines) => (),

            (r @ UnknownGeneric { .. }, e)
                if !self.occurs_check(received, expected)
                    && (matches!(self.unify_kind, UnifyKind::WithGeneric)
                        || !matches!(
                            self.engines.te().get(expected),
                            TypeInfo::UnknownGeneric { .. }
                        )) =>
            {
                self.replace_received_with_expected(handler, received, expected, &r, e, span)
            }
            (r, e @ UnknownGeneric { .. })
                if !self.occurs_check(expected, received)
                    && e.is_self_type()
                    && matches!(self.unify_kind, UnifyKind::WithSelf) =>
            {
                self.replace_expected_with_received(handler, received, expected, r, &e, span)
            }
            // Type aliases and the types they encapsulate coerce to each other.
            (Alias { ty, .. }, _) => self.unify(handler, ty.type_id, expected, span),
            (_, Alias { ty, .. }) => self.unify(handler, received, ty.type_id, span),

            // Let empty enums to coerce to any other type. This is useful for Never enum.
            (Enum(r_decl_ref), _)
                if self.engines.de().get_enum(&r_decl_ref).variants.is_empty() => {}

            (Enum(r_decl_ref), Enum(e_decl_ref)) => {
                let r_decl = self.engines.de().get_enum(&r_decl_ref);
                let e_decl = self.engines.de().get_enum(&e_decl_ref);

                self.unify_enums(
                    handler,
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
            (UnsignedInteger(r), UnsignedInteger(e)) if r == e => (),
            (Numeric, e @ UnsignedInteger(_)) => {
                self.replace_received_with_expected(handler, received, expected, &Numeric, e, span)
            }
            (r @ UnsignedInteger(_), Numeric) => {
                self.replace_expected_with_received(handler, received, expected, r, &Numeric, span)
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
                    handler,
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
                    handler,
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
            }

            // If no previous attempts to unify were successful, raise an error.
            (TypeInfo::ErrorRecovery(_), _) => (),
            (_, TypeInfo::ErrorRecovery(_)) => (),
            (r, e) => {
                let (received, expected) = self.assign_args(r, e);
                handler.emit_err(
                    TypeError::MismatchedType {
                        expected,
                        received,
                        help_text: self.help_text.clone(),
                        span: span.clone(),
                    }
                    .into(),
                );
            }
        }
    }

    fn occurs_check(&self, generic: TypeId, other: TypeId) -> bool {
        OccursCheck::new(self.engines).check(generic, other)
    }

    fn unify_strs(
        &self,
        handler: &Handler,
        received: TypeId,
        expected: TypeId,
        span: &Span,
        r: usize,
        e: usize,
    ) {
        if r != e {
            let (received, expected) = self.assign_args(received, expected);
            handler.emit_err(
                TypeError::MismatchedType {
                    expected,
                    received,
                    help_text: self.help_text.clone(),
                    span: span.clone(),
                }
                .into(),
            );
        }
    }

    fn unify_tuples(&self, handler: &Handler, rfs: Vec<TypeArgument>, efs: Vec<TypeArgument>) {
        for (rf, ef) in rfs.iter().zip(efs.iter()) {
            self.unify(handler, rf.type_id, ef.type_id, &rf.span);
        }
    }

    fn unify_structs(
        &self,
        handler: &Handler,
        received: TypeId,
        expected: TypeId,
        span: &Span,
        r: (Ident, Vec<TypeParameter>, Vec<ty::TyStructField>),
        e: (Ident, Vec<TypeParameter>, Vec<ty::TyStructField>),
    ) {
        let (rn, rtps, rfs) = r;
        let (en, etps, efs) = e;
        if rn == en && rfs.len() == efs.len() && rtps.len() == etps.len() {
            rfs.iter().zip(efs.iter()).for_each(|(rf, ef)| {
                self.unify(
                    handler,
                    rf.type_argument.type_id,
                    ef.type_argument.type_id,
                    span,
                );
            });
            rtps.iter().zip(etps.iter()).for_each(|(rtp, etp)| {
                self.unify(handler, rtp.type_id, etp.type_id, span);
            });
        } else {
            let (received, expected) = self.assign_args(received, expected);
            handler.emit_err(
                TypeError::MismatchedType {
                    expected,
                    received,
                    help_text: self.help_text.clone(),
                    span: span.clone(),
                }
                .into(),
            );
        }
    }

    fn unify_enums(
        &self,
        handler: &Handler,
        received: TypeId,
        expected: TypeId,
        span: &Span,
        r: (Ident, Vec<TypeParameter>, Vec<ty::TyEnumVariant>),
        e: (Ident, Vec<TypeParameter>, Vec<ty::TyEnumVariant>),
    ) {
        let (rn, rtps, rvs) = r;
        let (en, etps, evs) = e;
        if rn == en && rvs.len() == evs.len() && rtps.len() == etps.len() {
            rvs.iter().zip(evs.iter()).for_each(|(rv, ev)| {
                self.unify(
                    handler,
                    rv.type_argument.type_id,
                    ev.type_argument.type_id,
                    span,
                );
            });
            rtps.iter().zip(etps.iter()).for_each(|(rtp, etp)| {
                self.unify(handler, rtp.type_id, etp.type_id, span);
            });
        } else {
            let (received, expected) = self.assign_args(received, expected);
            handler.emit_err(
                TypeError::MismatchedType {
                    expected,
                    received,
                    help_text: self.help_text.clone(),
                    span: span.clone(),
                }
                .into(),
            );
        }
    }

    fn unify_arrays(
        &self,
        handler: &Handler,
        received: TypeId,
        expected: TypeId,
        span: &Span,
        r: TypeId,
        e: TypeId,
    ) {
        let h = Handler::default();
        self.unify(&h, r, e, span);
        let (new_errors, warnings) = h.consume();

        // If there was an error then we want to report the array types as mismatching, not
        // the elem types.
        if !new_errors.is_empty() {
            let (received, expected) = self.assign_args(received, expected);
            handler.emit_err(
                TypeError::MismatchedType {
                    expected,
                    received,
                    help_text: self.help_text.clone(),
                    span: span.clone(),
                }
                .into(),
            );
        }

        for warn in warnings {
            handler.emit_warn(warn);
        }
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
