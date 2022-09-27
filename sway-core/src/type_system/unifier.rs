use sway_types::Span;

use crate::{
    concurrent_slab::ConcurrentSlab,
    error::{TypeError, Warning},
    look_up_type_id, AbiName, CompileWarning, IntegerBits, ReplaceSelfType, TypeId, TypeInfo,
};

use sway_types::Spanned;

pub(super) struct Unifier<'cs> {
    slab: &'cs ConcurrentSlab<TypeInfo>,
    self_type: Option<TypeId>,
}

impl Unifier<'_> {
    pub(super) fn new(slab: &ConcurrentSlab<TypeInfo>, self_type: Option<TypeId>) -> Unifier {
        Unifier { slab, self_type }
    }

    /// Make the types of `received` and `expected` equivalent (or produce an
    /// error if there is a conflict between them).
    ///
    /// More specifically, this function tries to make `received` equivalent to
    /// `expected`, except in cases where `received` has more type information
    /// than `expected` (e.g. when `expected` is a generic type and `received`
    /// is not).
    pub(super) fn unify(
        &self,
        mut received: TypeId,
        mut expected: TypeId,
        span: &Span,
        help_text: impl Into<String>,
    ) -> (Vec<CompileWarning>, Vec<TypeError>) {
        use TypeInfo::*;

        if let Some(self_type) = self.self_type {
            received.replace_self_type(self_type);
            expected.replace_self_type(self_type);
        }

        let help_text = help_text.into();

        match (self.slab.get(*received), self.slab.get(*expected)) {
            // If the types are exactly the same, we are done.
            (Boolean, Boolean) => (vec![], vec![]),
            (SelfType, SelfType) => (vec![], vec![]),
            (Byte, Byte) => (vec![], vec![]),
            (B256, B256) => (vec![], vec![]),
            (Numeric, Numeric) => (vec![], vec![]),
            (Contract, Contract) => (vec![], vec![]),
            (Str(l), Str(r)) => {
                let warnings = vec![];
                let mut errors = vec![];
                if l != r {
                    errors.push(TypeError::MismatchedType {
                        expected,
                        received,
                        help_text,
                        span: span.clone(),
                    });
                }
                (warnings, errors)
            }

            // Follow any references
            (Ref(received, _sp1), Ref(expected, _sp2)) if received == expected => (vec![], vec![]),
            (Ref(received, _sp), _) => self.unify(received, expected, span, help_text),
            (_, Ref(expected, _sp)) => self.unify(received, expected, span, help_text),

            // When we don't know anything about either term, assume that
            // they match and make the one we know nothing about reference the
            // one we may know something about
            (Unknown, Unknown) => (vec![], vec![]),
            (Unknown, _) => {
                match self
                    .slab
                    .replace(received, &Unknown, TypeInfo::Ref(expected, span.clone()))
                {
                    None => (vec![], vec![]),
                    Some(_) => self.unify(received, expected, span, help_text),
                }
            }
            (_, Unknown) => {
                match self
                    .slab
                    .replace(expected, &Unknown, TypeInfo::Ref(received, span.clone()))
                {
                    None => (vec![], vec![]),
                    Some(_) => self.unify(received, expected, span, help_text),
                }
            }

            (Tuple(fields_a), Tuple(fields_b)) if fields_a.len() == fields_b.len() => {
                let mut warnings = vec![];
                let mut errors = vec![];
                for (field_a, field_b) in fields_a.iter().zip(fields_b.iter()) {
                    let (new_warnings, new_errors) = self.unify(
                        field_a.type_id,
                        field_b.type_id,
                        &field_a.span,
                        help_text.clone(),
                    );
                    warnings.extend(new_warnings);
                    errors.extend(new_errors);
                }
                (warnings, errors)
            }

            (UnsignedInteger(received_width), UnsignedInteger(expected_width)) => {
                // E.g., in a variable declaration `let a: u32 = 10u64` the 'expected' type will be
                // the annotation `u32`, and the 'received' type is 'self' of the initialiser, or
                // `u64`.  So we're casting received TO expected.
                let warnings = match numeric_cast_compat(expected_width, received_width) {
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

            (UnknownGeneric { name: l_name }, UnknownGeneric { name: r_name })
                if l_name.as_str() == r_name.as_str() =>
            {
                (vec![], vec![])
            }
            (ref received_info @ UnknownGeneric { .. }, _) => {
                self.slab.replace(
                    received,
                    received_info,
                    TypeInfo::Ref(expected, span.clone()),
                );
                (vec![], vec![])
            }

            (_, ref expected_info @ UnknownGeneric { .. }) => {
                self.slab.replace(
                    expected,
                    expected_info,
                    TypeInfo::Ref(received, span.clone()),
                );
                (vec![], vec![])
            }

            // if the types, once their ids have been looked up, are the same, we are done
            (
                Struct {
                    name: a_name,
                    fields: a_fields,
                    type_parameters: a_parameters,
                    ..
                },
                Struct {
                    name: b_name,
                    fields: b_fields,
                    type_parameters: b_parameters,
                    ..
                },
            ) => {
                let mut warnings = vec![];
                let mut errors = vec![];
                if a_name == b_name
                    && a_fields.len() == b_fields.len()
                    && a_parameters.len() == b_parameters.len()
                {
                    a_fields.iter().zip(b_fields.iter()).for_each(|(a, b)| {
                        let (new_warnings, new_errors) =
                            self.unify(a.type_id, b.type_id, &a.span, help_text.clone());
                        warnings.extend(new_warnings);
                        errors.extend(new_errors);
                    });
                    a_parameters
                        .iter()
                        .zip(b_parameters.iter())
                        .for_each(|(a, b)| {
                            let (new_warnings, new_errors) = self.unify(
                                a.type_id,
                                b.type_id,
                                &a.name_ident.span(),
                                help_text.clone(),
                            );
                            warnings.extend(new_warnings);
                            errors.extend(new_errors);
                        });
                } else {
                    errors.push(TypeError::MismatchedType {
                        expected,
                        received,
                        help_text,
                        span: span.clone(),
                    });
                }
                (warnings, errors)
            }
            (
                Enum {
                    name: a_name,
                    variant_types: a_variants,
                    type_parameters: a_parameters,
                },
                Enum {
                    name: b_name,
                    variant_types: b_variants,
                    type_parameters: b_parameters,
                },
            ) => {
                let mut warnings = vec![];
                let mut errors = vec![];
                if a_name == b_name
                    && a_variants.len() == b_variants.len()
                    && a_parameters.len() == b_parameters.len()
                {
                    a_variants.iter().zip(b_variants.iter()).for_each(|(a, b)| {
                        let (new_warnings, new_errors) =
                            self.unify(a.type_id, b.type_id, &a.span, help_text.clone());
                        warnings.extend(new_warnings);
                        errors.extend(new_errors);
                    });
                    a_parameters
                        .iter()
                        .zip(b_parameters.iter())
                        .for_each(|(a, b)| {
                            let (new_warnings, new_errors) = self.unify(
                                a.type_id,
                                b.type_id,
                                &a.name_ident.span(),
                                help_text.clone(),
                            );
                            warnings.extend(new_warnings);
                            errors.extend(new_errors);
                        });
                } else {
                    errors.push(TypeError::MismatchedType {
                        expected,
                        received,
                        help_text,
                        span: span.clone(),
                    });
                }
                (warnings, errors)
            }

            (Numeric, expected_info @ UnsignedInteger(_)) => {
                match self.slab.replace(received, &Numeric, expected_info) {
                    None => (vec![], vec![]),
                    Some(_) => self.unify(received, expected, span, help_text),
                }
            }
            (received_info @ UnsignedInteger(_), Numeric) => {
                match self.slab.replace(expected, &Numeric, received_info) {
                    None => (vec![], vec![]),
                    Some(_) => self.unify(received, expected, span, help_text),
                }
            }

            (Array(a_elem, a_count, _), Array(b_elem, b_count, _)) if a_count == b_count => {
                let (warnings, new_errors) = self.unify(a_elem, b_elem, span, help_text.clone());

                // If there was an error then we want to report the array types as mismatching, not
                // the elem types.
                let mut errors = vec![];
                if !new_errors.is_empty() {
                    errors.push(TypeError::MismatchedType {
                        expected,
                        received,
                        help_text,
                        span: span.clone(),
                    });
                }
                (warnings, errors)
            }

            (
                ref r @ TypeInfo::ContractCaller {
                    abi_name: ref abi_name_received,
                    address: ref received_address,
                },
                TypeInfo::ContractCaller {
                    abi_name: ref abi_name_expected,
                    ..
                },
            ) if (abi_name_received == abi_name_expected && received_address.is_none())
                || matches!(abi_name_received, AbiName::Deferred) =>
            {
                // if one address is empty, coerce to the other one
                match self.slab.replace(received, r, look_up_type_id(expected)) {
                    None => (vec![], vec![]),
                    Some(_) => self.unify(received, expected, span, help_text),
                }
            }
            (
                TypeInfo::ContractCaller {
                    abi_name: ref abi_name_received,
                    ..
                },
                ref e @ TypeInfo::ContractCaller {
                    abi_name: ref abi_name_expected,
                    ref address,
                },
            ) if (abi_name_received == abi_name_expected && address.is_none())
                || matches!(abi_name_expected, AbiName::Deferred) =>
            {
                // if one address is empty, coerce to the other one
                match self.slab.replace(expected, e, look_up_type_id(received)) {
                    None => (vec![], vec![]),
                    Some(_) => self.unify(received, expected, span, help_text),
                }
            }
            (ref r @ TypeInfo::ContractCaller { .. }, ref e @ TypeInfo::ContractCaller { .. })
                if r == e =>
            {
                // if they are the same, then it's ok
                (vec![], vec![])
            }

            // If no previous attempts to unify were successful, raise an error
            (TypeInfo::ErrorRecovery, _) => (vec![], vec![]),
            (_, TypeInfo::ErrorRecovery) => (vec![], vec![]),
            (_, _) => {
                let errors = vec![TypeError::MismatchedType {
                    expected,
                    received,
                    help_text,
                    span: span.clone(),
                }];
                (vec![], errors)
            }
        }
    }
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
