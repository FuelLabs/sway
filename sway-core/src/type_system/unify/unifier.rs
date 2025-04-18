use std::fmt;

use sway_error::{
    handler::{ErrorEmitted, Handler},
    type_error::TypeError,
};
use sway_types::{Span, Spanned};

use crate::{
    ast_elements::type_parameter::ConstGenericExpr,
    decl_engine::{DeclEngineGet, DeclId},
    engine_threading::{Engines, PartialEqWithEngines, PartialEqWithEnginesContext, WithEngines},
    language::{ty::TyStructDecl, CallPath},
    type_system::{engine::Unification, priv_prelude::*},
};

use super::occurs_check::OccursCheck;

#[derive(Debug, Clone)]
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
        received: TypeId,
        expected_type_info: &TypeInfo,
        span: &Span,
    ) {
        self.engines.te().replace_with_new_source_id(
            self.engines,
            received,
            expected_type_info.clone(),
            span.source_id().copied(),
        );
    }

    /// Helper method for replacing the values in the [TypeEngine].
    fn replace_expected_with_received(
        &self,
        expected: TypeId,
        received_type_info: &TypeInfo,
        span: &Span,
    ) {
        self.engines.te().replace_with_new_source_id(
            self.engines,
            expected,
            received_type_info.clone(),
            span.source_id().copied(),
        );
    }

    /// Performs type unification with `received` and `expected`.
    pub(crate) fn unify(
        &self,
        handler: &Handler,
        received: TypeId,
        expected: TypeId,
        span: &Span,
        push_unification: bool,
    ) {
        if push_unification {
            let unification = Unification {
                received,
                expected,
                span: span.clone(),
                help_text: self.help_text.clone(),
                unify_kind: self.unify_kind.clone(),
            };

            self.engines.te().push_unification(unification);
        }

        use TypeInfo::{
            Alias, Array, Boolean, Contract, Enum, Never, Numeric, Placeholder, RawUntypedPtr,
            RawUntypedSlice, Ref, Slice, StringArray, StringSlice, Struct, Tuple, Unknown,
            UnknownGeneric, UnsignedInteger, B256,
        };

        if received == expected {
            return;
        }

        let r_type_source_info = self.engines.te().get(received);
        let e_type_source_info = self.engines.te().get(expected);

        match (&*r_type_source_info, &*e_type_source_info) {
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
                self.unify_strs(handler, received, expected, span, l.val(), r.val());
            }
            (Tuple(rfs), Tuple(efs)) if rfs.len() == efs.len() => {
                self.unify_tuples(handler, rfs, efs);
            }
            (r @ Array(re, rc), e @ Array(ee, ec)) => {
                if self
                    .unify_type_arguments_in_parents(handler, received, expected, span, re, ee)
                    .is_err()
                {
                    return;
                }

                if matches!(
                    rc.expr(),
                    ConstGenericExpr::AmbiguousVariableExpression { .. }
                ) {
                    self.replace_received_with_expected(received, e, span);
                }

                if matches!(
                    ec.expr(),
                    ConstGenericExpr::AmbiguousVariableExpression { .. }
                ) {
                    self.replace_expected_with_received(expected, r, span);
                }
            }
            (Slice(re), Slice(ee)) => {
                let _ =
                    self.unify_type_arguments_in_parents(handler, received, expected, span, re, ee);
            }
            (Struct(received_decl_id), Struct(expected_decl_id)) => {
                self.unify_structs(
                    handler,
                    received,
                    expected,
                    span,
                    received_decl_id,
                    expected_decl_id,
                );
            }
            // When we don't know anything about either term, assume that
            // they match and make the one we know nothing about reference the
            // one we may know something about.
            (Unknown, Unknown) => (),
            (Unknown, e) => self.replace_received_with_expected(received, e, span),
            (r, Unknown) => self.replace_expected_with_received(expected, r, span),

            (r @ Placeholder(_), _e @ Placeholder(_)) => {
                self.replace_expected_with_received(expected, r, span);
            }
            (_r @ Placeholder(_), e) => self.replace_received_with_expected(received, e, span),
            (r, _e @ Placeholder(_)) => self.replace_expected_with_received(expected, r, span),

            // Generics are handled similarly to the case for unknowns, except
            // we take more careful consideration for the type/purpose for the
            // unification that we are performing.
            (UnknownGeneric { parent: rp, .. }, e)
                if rp.is_some()
                    && self
                        .engines
                        .te()
                        .get(rp.unwrap())
                        .eq(e, &PartialEqWithEnginesContext::new(self.engines)) => {}
            (r, UnknownGeneric { parent: ep, .. })
                if ep.is_some()
                    && self
                        .engines
                        .te()
                        .get(ep.unwrap())
                        .eq(r, &PartialEqWithEnginesContext::new(self.engines)) => {}
            (UnknownGeneric { parent: rp, .. }, UnknownGeneric { parent: ep, .. })
                if rp.is_some()
                    && ep.is_some()
                    && self.engines.te().get(ep.unwrap()).eq(
                        &*self.engines.te().get(rp.unwrap()),
                        &PartialEqWithEnginesContext::new(self.engines),
                    ) => {}

            (
                UnknownGeneric {
                    name: rn,
                    trait_constraints: rtc,
                    parent: _,
                    is_from_type_parameter: _,
                },
                UnknownGeneric {
                    name: en,
                    trait_constraints: etc,
                    parent: _,
                    is_from_type_parameter: _,
                },
            ) if rn.as_str() == en.as_str()
                && rtc.eq(etc, &PartialEqWithEnginesContext::new(self.engines)) => {}

            (_r @ UnknownGeneric { .. }, e)
                if !self.occurs_check(received, expected)
                    && (matches!(self.unify_kind, UnifyKind::WithGeneric)
                        || !matches!(
                            &*self.engines.te().get(expected),
                            TypeInfo::UnknownGeneric { .. }
                        )) =>
            {
                self.replace_received_with_expected(received, e, span)
            }
            (r, e @ UnknownGeneric { .. })
                if !self.occurs_check(expected, received)
                    && e.is_self_type()
                    && matches!(self.unify_kind, UnifyKind::WithSelf) =>
            {
                self.replace_expected_with_received(expected, r, span);
            }

            // Never type coerces to any other type.
            // This should be after the unification of self types.
            (Never, _) => {}

            // Type aliases and the types they encapsulate coerce to each other.
            (Alias { ty, .. }, _) => self.unify(handler, ty.type_id(), expected, span, false),
            (_, Alias { ty, .. }) => self.unify(handler, received, ty.type_id(), span, false),

            (Enum(r_decl_ref), Enum(e_decl_ref)) => {
                let r_decl = self.engines.de().get_enum(r_decl_ref);
                let e_decl = self.engines.de().get_enum(e_decl_ref);

                self.unify_enums(
                    handler,
                    received,
                    expected,
                    span,
                    (r_decl.call_path.clone(), r_decl.generic_parameters.clone()),
                    (e_decl.call_path.clone(), e_decl.generic_parameters.clone()),
                );
            }

            // For integers and numerics, we (potentially) unify the numeric
            // with the integer.
            (UnsignedInteger(r), UnsignedInteger(e)) if r == e => (),
            (Numeric, e @ UnsignedInteger(_)) => {
                self.replace_received_with_expected(received, e, span);
            }
            (r @ UnsignedInteger(_), Numeric) => {
                self.replace_expected_with_received(expected, r, span);
            }

            // For contract callers, we (potentially) unify them if they have
            // the same name and their address is `None`
            (
                _r @ TypeInfo::ContractCaller {
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
                    &self.engines.te().get(expected),
                    span,
                );
            }
            (
                TypeInfo::ContractCaller {
                    abi_name: ref ran, ..
                },
                _e @ TypeInfo::ContractCaller {
                    abi_name: ref ean,
                    address: ref ea,
                },
            ) if (ran == ean && ea.is_none()) || matches!(ean, AbiName::Deferred) => {
                // if one address is empty, coerce to the other one
                self.replace_expected_with_received(
                    expected,
                    &self.engines.te().get(received),
                    span,
                );
            }
            (ref r @ TypeInfo::ContractCaller { .. }, ref e @ TypeInfo::ContractCaller { .. })
                if r.eq(e, &PartialEqWithEnginesContext::new(self.engines)) =>
            {
                // if they are the same, then it's ok
            }
            // Unification is possible in these situations, assuming that the referenced types
            // can unify:
            //  - `&` -> `&`
            //  - `&mut` -> `&`
            //  - `&mut` -> `&mut`
            (
                Ref {
                    to_mutable_value: r_to_mut,
                    referenced_type: r_ty,
                },
                Ref {
                    to_mutable_value: e_to_mut,
                    referenced_type: e_ty,
                },
            ) if *r_to_mut || !*e_to_mut => {
                let _ = self
                    .unify_type_arguments_in_parents(handler, received, expected, span, r_ty, e_ty);
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

    fn unify_tuples(&self, handler: &Handler, rfs: &[GenericArgument], efs: &[GenericArgument]) {
        for (rf, ef) in rfs.iter().zip(efs.iter()) {
            self.unify(handler, rf.type_id(), ef.type_id(), &rf.span(), false);
        }
    }

    fn unify_structs(
        &self,
        handler: &Handler,
        received_type_id: TypeId,
        expected_type_id: TypeId,
        span: &Span,
        received_decl_id: &DeclId<TyStructDecl>,
        expected_decl_id: &DeclId<TyStructDecl>,
    ) {
        let TyStructDecl {
            call_path: received_call_path,
            generic_parameters: received_parameters,
            ..
        } = &*self.engines.de().get(received_decl_id);
        let TyStructDecl {
            call_path: expected_call_path,
            generic_parameters: expected_parameters,
            ..
        } = &*self.engines.de().get(expected_decl_id);

        if received_parameters.len() == expected_parameters.len()
            && received_call_path == expected_call_path
        {
            for (received_parameter, expected_parameter) in
                received_parameters.iter().zip(expected_parameters.iter())
            {
                match (received_parameter, expected_parameter) {
                    (
                        TypeParameter::Type(received_parameter),
                        TypeParameter::Type(expected_parameter),
                    ) => self.unify(
                        handler,
                        received_parameter.type_id,
                        expected_parameter.type_id,
                        span,
                        false,
                    ),
                    (
                        TypeParameter::Const(received_parameter),
                        TypeParameter::Const(expected_parameter),
                    ) => {
                        match (&received_parameter.expr, &expected_parameter.expr) {
                            (Some(received_expr), Some(expected_expr)) => {
                                // We can unify if their are the same
                                if received_expr.as_literal_val() == expected_expr.as_literal_val()
                                {
                                } else {
                                    todo!("Will be implemented by https://github.com/FuelLabs/sway/issues/6860")
                                }
                            }
                            (Some(_), None) => {
                                self.replace_expected_with_received(
                                    expected_type_id,
                                    &TypeInfo::Struct(*received_decl_id),
                                    span,
                                );
                            }
                            (None, Some(_)) => {
                                self.replace_received_with_expected(
                                    received_type_id,
                                    &TypeInfo::Struct(*expected_decl_id),
                                    span,
                                );
                            }
                            (None, None) => {}
                        }
                    }
                    _ => todo!(),
                }
            }
        } else {
            let (received, expected) = self.assign_args(received_type_id, expected_type_id);
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
        r: (CallPath, Vec<TypeParameter>),
        e: (CallPath, Vec<TypeParameter>),
    ) {
        let (rn, rtps) = r;
        let (en, etps) = e;
        if rn == en && rtps.len() == etps.len() {
            rtps.iter().zip(etps.iter()).for_each(|(rtp, etp)| {
                let rtp = rtp
                    .as_type_parameter()
                    .expect("will only work with type parameters");
                let etp = etp
                    .as_type_parameter()
                    .expect("will only work with type parameters");
                self.unify(handler, rtp.type_id, etp.type_id, span, false);
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

    /// Unifies `received_type_argument` and `expected_type_argument`, and in case of a
    /// mismatch, reports the `received_parent` and `expected_parent` as mismatching.
    /// Useful for unifying types like arrays and references where issues in unification
    /// of their [TypeArgument]s directly corresponds to the unification of enclosed types themselves.
    fn unify_type_arguments_in_parents(
        &self,
        handler: &Handler,
        received_parent: TypeId,
        expected_parent: TypeId,
        span: &Span,
        received_type_argument: &GenericArgument,
        expected_type_argument: &GenericArgument,
    ) -> Result<(), ErrorEmitted> {
        let h = Handler::default();
        self.unify(
            &h,
            received_type_argument.type_id(),
            expected_type_argument.type_id(),
            span,
            false,
        );
        let (new_errors, warnings) = h.consume();

        for warn in warnings {
            handler.emit_warn(warn);
        }

        // If there was an error then we want to report the parent types as mismatching, not
        // the argument types.
        if !new_errors.is_empty() {
            let (received, expected) = self.assign_args(received_parent, expected_parent);
            Err(handler.emit_err(
                TypeError::MismatchedType {
                    expected,
                    received,
                    help_text: self.help_text.clone(),
                    span: span.clone(),
                }
                .into(),
            ))
        } else {
            Ok(())
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
