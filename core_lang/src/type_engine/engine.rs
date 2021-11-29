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
    /// there is a conflict between them)
    pub(crate) fn unify<'sc>(
        &self,
        a: TypeId,
        b: TypeId,
        span: &Span<'sc>,
    ) -> Result<Vec<Warning<'sc>>, TypeError<'sc>> {
        use TypeInfo::*;
        match (self.slab.get(a), self.slab.get(b)) {
            // If the types are exactly the same, we are done.
            (a, b) if a == b => Ok(vec![]),

            // Follow any references
            (Ref(a), _) => self.unify(a, b, span),
            (_, Ref(b)) => self.unify(a, b, span),

            // When we don't know anything about either term, assume that
            // they match and make the one we know nothing about reference the
            // one we may know something about
            (Unknown, _) => match self.slab.replace(a, &Unknown, TypeInfo::Ref(b)) {
                None => Ok(vec![]),
                Some(_) => self.unify(a, b, span),
            },
            (_, Unknown) => match self.slab.replace(b, &Unknown, TypeInfo::Ref(a)) {
                None => Ok(vec![]),
                Some(_) => self.unify(a, b, span),
            },

            (UnsignedInteger(x), UnsignedInteger(y)) => match numeric_cast_compat(x, y) {
                NumericCastCompatResult::CastableWithWarning(warn) => {
                    // cast the one on the right to the one on the left
                    Ok(vec![warn])
                }
                // do nothing if compatible
                NumericCastCompatResult::Compatible => Ok(vec![]),
            },

            (ref a_info @ UnknownGeneric { .. }, _) => {
                self.slab.replace(a, a_info, TypeInfo::Ref(b));
                Ok(vec![])
            }

            (_, ref b_info @ UnknownGeneric { .. }) => {
                self.slab.replace(b, b_info, TypeInfo::Ref(a));
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

            (Numeric, b_info @ UnsignedInteger(_)) => {
                match self.slab.replace(a, &Numeric, b_info) {
                    None => Ok(vec![]),
                    Some(_) => self.unify(a, b, span),
                }
            }
            (a_info @ UnsignedInteger(_), Numeric) => {
                match self.slab.replace(b, &Numeric, a_info) {
                    None => Ok(vec![]),
                    Some(_) => self.unify(a, b, span),
                }
            }

            (Array(a_elem, a_count), Array(b_elem, b_count)) if a_count == b_count => self
                .unify(a_elem, b_elem, span)
                // If there was an error then we want to report the array types as mismatching, not
                // the elem types.
                .map_err(|_| TypeError::MismatchedType {
                    expected: b,
                    received: a,
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
                expected: b,
                received: a,
                help_text: Default::default(),
                span: span.clone(),
            }),
        }
    }

    pub fn unify_with_self<'sc>(
        &self,
        a: TypeId,
        b: TypeId,
        self_type: TypeId,
        span: &Span<'sc>,
    ) -> Result<Vec<Warning<'sc>>, TypeError<'sc>> {
        let a = if self.look_up_type_id(a) == TypeInfo::SelfType {
            self_type
        } else {
            a
        };
        let b = if self.look_up_type_id(b) == TypeInfo::SelfType {
            self_type
        } else {
            b
        };

        self.unify(a, b, span)
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

pub(crate) fn insert_type(ty: TypeInfo) -> TypeId {
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
) -> Result<Vec<Warning<'sc>>, TypeError<'sc>> {
    TYPE_ENGINE.unify_with_self(a, b, self_type, span)
}

pub fn resolve_type<'sc>(id: TypeId, error_span: &Span<'sc>) -> Result<TypeInfo, TypeError<'sc>> {
    TYPE_ENGINE.resolve_type(id, error_span)
}

fn numeric_cast_compat<'sc>(a: IntegerBits, b: IntegerBits) -> NumericCastCompatResult<'sc> {
    // if this is a downcast, warn for loss of precision. if upcast, then no warning.
    use IntegerBits::*;
    match (a, b) {
        // these should generate a downcast warning
        (Eight, Sixteen)
        | (Eight, ThirtyTwo)
        | (Eight, SixtyFour)
        | (Sixteen, ThirtyTwo)
        | (Sixteen, SixtyFour)
        | (ThirtyTwo, SixtyFour) => {
            NumericCastCompatResult::CastableWithWarning(Warning::LossOfPrecision {
                initial_type: a,
                cast_to: b,
            })
        }
        // upcasting is ok, so everything else is ok
        _ => NumericCastCompatResult::Compatible,
    }
}
enum NumericCastCompatResult<'sc> {
    Compatible,
    CastableWithWarning(Warning<'sc>),
}
