use super::*;
use crate::concurrent_slab::ConcurrentSlab;
use crate::Span;
use lazy_static::lazy_static;

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
    pub(crate) fn unify<'sc>(
        &self,
        received: TypeId,
        expected: TypeId,
        span: &Span<'sc>,
    ) -> Result<Vec<CompileWarning<'sc>>, TypeError<'sc>> {
        use TypeInfo::*;
        match (self.slab.get(received), self.slab.get(expected)) {
            // If the types are exactly the same, we are done.
            (received_info, expected_info) if received_info == expected_info => Ok(vec![]),

            // Follow any references
            (Ref(received), _) => self.unify(received, expected, span),
            (_, Ref(expected)) => self.unify(received, expected, span),

            // When we don't know anything about either term, assume that
            // they match and make the one we know nothing about reference the
            // one we may know something about
            (Unknown, _) => match self
                .slab
                .replace(received, &Unknown, TypeInfo::Ref(expected))
            {
                None => Ok(vec![]),
                Some(_) => self.unify(received, expected, span),
            },
            (_, Unknown) => match self
                .slab
                .replace(expected, &Unknown, TypeInfo::Ref(received))
            {
                None => Ok(vec![]),
                Some(_) => self.unify(received, expected, span),
            },

            (
                ref received_info @ UnsignedInteger(recieved_width),
                ref expected_info @ UnsignedInteger(expected_width),
            ) => {
                // E.g., in a variable declaration `let a: u32 = 10u64` the 'expected' type will be
                // the annotation `u32`, and the 'received' type is 'self' of the initialiser, or
                // `u64`.  So we're casting received TO expected.
                let warn = match numeric_cast_compat(expected_width, recieved_width) {
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

                // Cast the expected type to the recieved type.
                self.slab
                    .replace(received, received_info, expected_info.clone());
                Ok(warn)
            }

            (ref received_info @ UnknownGeneric { .. }, _) => {
                self.slab
                    .replace(received, received_info, TypeInfo::Ref(expected));
                Ok(vec![])
            }

            (_, ref expected_info @ UnknownGeneric { .. }) => {
                self.slab
                    .replace(expected, expected_info, TypeInfo::Ref(received));
                Ok(vec![])
            }

            // if the types, once their ids have been looked up, are the same, we are done
            (
                Struct {
                    fields: a_fields, ..
                },
                Struct {
                    fields: b_fields, ..
                },
            ) if {
                let a_fields = a_fields.iter().map(|x| x.r#type);
                let b_fields = b_fields.iter().map(|x| x.r#type);

                let mut zipped = a_fields.zip(b_fields);
                zipped.all(|(a, b)| self.unify(a, b, span).is_ok())
            } =>
            {
                Ok(vec![])
            }
            (
                Enum {
                    variant_types: a_variants,
                    ..
                },
                Enum {
                    variant_types: b_variants,
                    ..
                },
            ) if {
                let a_variants = a_variants.iter().map(|x| x.r#type);
                let b_variants = b_variants.iter().map(|x| x.r#type);

                let mut zipped = a_variants.zip(b_variants);
                zipped.all(|(a, b)| self.unify(a, b, span).is_ok())
            } =>
            {
                Ok(vec![])
            }

            (Numeric, expected_info @ UnsignedInteger(_)) => {
                match self.slab.replace(received, &Numeric, expected_info) {
                    None => Ok(vec![]),
                    Some(_) => self.unify(received, expected, span),
                }
            }
            (received_info @ UnsignedInteger(_), Numeric) => {
                match self.slab.replace(expected, &Numeric, received_info) {
                    None => Ok(vec![]),
                    Some(_) => self.unify(received, expected, span),
                }
            }

            (Array(a_elem, a_count), Array(b_elem, b_count)) if a_count == b_count => self
                .unify(a_elem, b_elem, span)
                // If there was an error then we want to report the array types as mismatching, not
                // the elem types.
                .map_err(|_| TypeError::MismatchedType {
                    expected,
                    received,
                    help_text: Default::default(),
                    span: span.clone(),
                }),

            // When unifying complex types, we must check their sub-types. This
            // can be trivially implemented for tuples, sum types, etc.
            // (List(a_item), List(b_item)) => self.unify(a_item, b_item),
            // this can be used for curried function types but we might not want that
            // (Func(a_i, a_o), Func(b_i, b_o)) => {
            //     self.unify(a_i, b_i).and_then(|_| self.unify(a_o, b_o))
            // }

            // If no previous attempts to unify were successful, raise an error
            (_, _) => Err(TypeError::MismatchedType {
                expected,
                received,
                help_text: Default::default(),
                span: span.clone(),
            }),
        }
    }

    pub fn unify_with_self<'sc>(
        &self,
        received: TypeId,
        expected: TypeId,
        self_type: TypeId,
        span: &Span<'sc>,
    ) -> Result<Vec<CompileWarning<'sc>>, TypeError<'sc>> {
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

        self.unify(received, expected, span)
    }

    pub fn resolve_type<'sc>(
        &self,
        id: TypeId,
        error_span: &Span<'sc>,
    ) -> Result<TypeInfo, TypeError<'sc>> {
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

pub fn unify_with_self<'sc>(
    a: TypeId,
    b: TypeId,
    self_type: TypeId,
    span: &Span<'sc>,
) -> Result<Vec<CompileWarning<'sc>>, TypeError<'sc>> {
    TYPE_ENGINE.unify_with_self(a, b, self_type, span)
}

pub fn resolve_type<'sc>(id: TypeId, error_span: &Span<'sc>) -> Result<TypeInfo, TypeError<'sc>> {
    TYPE_ENGINE.resolve_type(id, error_span)
}

fn numeric_cast_compat<'sc>(
    new_size: IntegerBits,
    old_size: IntegerBits,
) -> NumericCastCompatResult<'sc> {
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
enum NumericCastCompatResult<'sc> {
    Compatible,
    CastableWithWarning(Warning<'sc>),
}
