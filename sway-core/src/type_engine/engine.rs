use super::*;
use crate::concurrent_slab::ConcurrentSlab;
use indexmap::IndexSet;
use lazy_static::lazy_static;
use sway_types::span::Span;

lazy_static! {
    static ref TYPE_ENGINE: Engine = Engine::default();
}

#[derive(Debug, Default)]
pub(crate) struct Engine {
    slab: ConcurrentSlab<TypeInfo>,
}

impl Engine {
    pub fn insert_type(&self, ty: TypeInfo) -> TypeId {
        self.slab.insert(ty)
    }

    pub fn look_up_type_id_raw(&self, id: TypeId) -> TypeInfo {
        self.slab.get(id)
    }

    pub fn look_up_type_id(&self, id: TypeId) -> TypeInfo {
        match self.slab.get(id) {
            TypeInfo::Ref(other) => self.look_up_type_id(other),
            ty => ty,
        }
    }

    /// Make the types of two type terms equivalent (or produce an error if
    /// there is a conflict between them).
    //
    // When reporting type errors we will report 'received' and 'expected' as such.
    pub(crate) fn unify(
        &self,
        received: TypeId,
        expected: TypeId,
        span: &Span,
        help_text: impl Into<String>,
    ) -> (IndexSet<CompileWarning>, IndexSet<TypeError>) {
        use TypeInfo::*;
        let help_text = help_text.into();
        match (self.slab.get(received), self.slab.get(expected)) {
            // If the types are exactly the same, we are done.
            (Boolean, Boolean) => (IndexSet::new(), IndexSet::new()),
            (SelfType, SelfType) => (IndexSet::new(), IndexSet::new()),
            (Byte, Byte) => (IndexSet::new(), IndexSet::new()),
            (B256, B256) => (IndexSet::new(), IndexSet::new()),
            (Numeric, Numeric) => (IndexSet::new(), IndexSet::new()),
            (Contract, Contract) => (IndexSet::new(), IndexSet::new()),
            (Str(l), Str(r)) => {
                let warnings = IndexSet::new();
                let mut errors = IndexSet::new();
                if l != r {
                    errors.insert(TypeError::MismatchedType {
                        expected,
                        received,
                        help_text,
                        span: span.clone(),
                    });
                }
                (warnings, errors)
            }
            //(received_info, expected_info) if received_info == expected_info => (IndexSet::new(), IndexSet::new()),

            // Follow any references
            (Ref(received), Ref(expected)) if received == expected => {
                (IndexSet::new(), IndexSet::new())
            }
            (Ref(received), _) => self.unify(received, expected, span, help_text),
            (_, Ref(expected)) => self.unify(received, expected, span, help_text),

            // When we don't know anything about either term, assume that
            // they match and make the one we know nothing about reference the
            // one we may know something about
            (Unknown, Unknown) => (IndexSet::new(), IndexSet::new()),
            (Unknown, _) => {
                match self
                    .slab
                    .replace(received, &Unknown, TypeInfo::Ref(expected))
                {
                    None => (IndexSet::new(), IndexSet::new()),
                    Some(_) => self.unify(received, expected, span, help_text),
                }
            }
            (_, Unknown) => {
                match self
                    .slab
                    .replace(expected, &Unknown, TypeInfo::Ref(received))
                {
                    None => (IndexSet::new(), IndexSet::new()),
                    Some(_) => self.unify(received, expected, span, help_text),
                }
            }

            (Tuple(fields_a), Tuple(fields_b)) if fields_a.len() == fields_b.len() => {
                let mut warnings = IndexSet::new();
                let mut errors = IndexSet::new();
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

            (
                ref received_info @ UnsignedInteger(received_width),
                ref expected_info @ UnsignedInteger(expected_width),
            ) => {
                // E.g., in a variable declaration `let a: u32 = 10u64` the 'expected' type will be
                // the annotation `u32`, and the 'received' type is 'self' of the initialiser, or
                // `u64`.  So we're casting received TO expected.
                let warnings = match numeric_cast_compat(expected_width, received_width) {
                    NumericCastCompatResult::CastableWithWarning(warn) => {
                        IndexSet::from([CompileWarning {
                            span: span.clone(),
                            warning_content: warn,
                        }])
                    }
                    NumericCastCompatResult::Compatible => IndexSet::new(),
                };

                // Cast the expected type to the received type.
                self.slab
                    .replace(received, received_info, expected_info.clone());
                (warnings, IndexSet::new())
            }

            (UnknownGeneric { name: l_name }, UnknownGeneric { name: r_name })
                if l_name == r_name =>
            {
                (IndexSet::new(), IndexSet::new())
            }
            (ref received_info @ UnknownGeneric { .. }, _) => {
                self.slab
                    .replace(received, received_info, TypeInfo::Ref(expected));
                (IndexSet::new(), IndexSet::new())
            }

            (_, ref expected_info @ UnknownGeneric { .. }) => {
                self.slab
                    .replace(expected, expected_info, TypeInfo::Ref(received));
                (IndexSet::new(), IndexSet::new())
            }

            // if the types, once their ids have been looked up, are the same, we are done
            (
                Struct {
                    name: a_name,
                    fields: a_fields,
                    ..
                },
                Struct {
                    name: b_name,
                    fields: b_fields,
                    ..
                },
            ) => {
                let mut warnings = IndexSet::new();
                let mut errors = IndexSet::new();
                if a_name == b_name && a_fields.len() == b_fields.len() {
                    a_fields.iter().zip(b_fields.iter()).for_each(|(a, b)| {
                        let (new_warnings, new_errors) =
                            self.unify(a.r#type, b.r#type, &a.span, help_text.clone());
                        warnings.extend(new_warnings);
                        errors.extend(new_errors);
                    });
                } else {
                    errors.insert(TypeError::MismatchedType {
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
                },
                Enum {
                    name: b_name,
                    variant_types: b_variants,
                },
            ) => {
                let mut warnings = IndexSet::new();
                let mut errors = IndexSet::new();
                if a_name == b_name && a_variants.len() == b_variants.len() {
                    a_variants.iter().zip(b_variants.iter()).for_each(|(a, b)| {
                        let (new_warnings, new_errors) =
                            self.unify(a.r#type, b.r#type, &a.span, help_text.clone());
                        warnings.extend(new_warnings);
                        errors.extend(new_errors);
                    });
                } else {
                    errors.insert(TypeError::MismatchedType {
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
                    None => (IndexSet::new(), IndexSet::new()),
                    Some(_) => self.unify(received, expected, span, help_text),
                }
            }
            (received_info @ UnsignedInteger(_), Numeric) => {
                match self.slab.replace(expected, &Numeric, received_info) {
                    None => (IndexSet::new(), IndexSet::new()),
                    Some(_) => self.unify(received, expected, span, help_text),
                }
            }

            (Array(a_elem, a_count), Array(b_elem, b_count)) if a_count == b_count => {
                let mut warnings = IndexSet::new();
                let mut errors = IndexSet::new();
                if a_count == b_count {
                    let (new_warnings, new_errors) =
                        self.unify(a_elem, b_elem, span, help_text.clone());
                    // If there was an error then we want to report the array types as mismatching, not
                    // the elem types.
                    if new_errors.is_empty() {
                        warnings.extend(new_warnings);
                    } else {
                        errors.insert(TypeError::MismatchedType {
                            expected,
                            received,
                            help_text,
                            span: span.clone(),
                        });
                    }
                } else {
                    errors.insert(TypeError::MismatchedType {
                        expected,
                        received,
                        help_text,
                        span: span.clone(),
                    });
                }
                (warnings, errors)
            }

            // When unifying complex types, we must check their sub-types. This
            // can be trivially implemented for tuples, sum types, etc.
            // (List(a_item), List(b_item)) => self.unify(a_item, b_item),
            // this can be used for curried function types but we might not want that
            // (Func(a_i, a_o), Func(b_i, b_o)) => {
            //     self.unify(a_i, b_i).and_then(|_| self.unify(a_o, b_o))
            // }

            // If no previous attempts to unify were successful, raise an error
            (TypeInfo::ErrorRecovery, _) => (IndexSet::new(), IndexSet::new()),
            (_, TypeInfo::ErrorRecovery) => (IndexSet::new(), IndexSet::new()),
            _ => {
                let errors = IndexSet::from([TypeError::MismatchedType {
                    expected,
                    received,
                    help_text,
                    span: span.clone(),
                }]);
                (IndexSet::new(), errors)
            }
        }
    }

    pub fn unify_with_self(
        &self,
        received: TypeId,
        expected: TypeId,
        self_type: TypeId,
        span: &Span,
        help_text: impl Into<String>,
    ) -> (IndexSet<CompileWarning>, IndexSet<TypeError>) {
        let received = if self.look_up_type_id(received) == TypeInfo::SelfType {
            self_type
        } else {
            received
        };
        let expected = if self.look_up_type_id(expected) == TypeInfo::SelfType {
            self_type
        } else {
            expected
        };

        self.unify(received, expected, span, help_text)
    }

    pub fn resolve_type(&self, id: TypeId, error_span: &Span) -> Result<TypeInfo, TypeError> {
        match self.look_up_type_id(id) {
            TypeInfo::Unknown => Err(TypeError::UnknownType {
                span: error_span.clone(),
            }),
            ty => Ok(ty),
        }
    }
}

pub fn insert_type(ty: TypeInfo) -> TypeId {
    TYPE_ENGINE.insert_type(ty)
}

pub(crate) fn look_up_type_id(id: TypeId) -> TypeInfo {
    TYPE_ENGINE.look_up_type_id(id)
}

pub(crate) fn look_up_type_id_raw(id: TypeId) -> TypeInfo {
    TYPE_ENGINE.look_up_type_id_raw(id)
}

pub fn unify_with_self(
    a: TypeId,
    b: TypeId,
    self_type: TypeId,
    span: &Span,
    help_text: impl Into<String>,
) -> (IndexSet<CompileWarning>, IndexSet<TypeError>) {
    TYPE_ENGINE.unify_with_self(a, b, self_type, span, help_text)
}

pub(crate) fn unify(
    a: TypeId,
    b: TypeId,
    span: &Span,
    help_text: impl Into<String>,
) -> (IndexSet<CompileWarning>, IndexSet<TypeError>) {
    TYPE_ENGINE.unify(a, b, span, help_text)
}

pub fn resolve_type(id: TypeId, error_span: &Span) -> Result<TypeInfo, TypeError> {
    TYPE_ENGINE.resolve_type(id, error_span)
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
