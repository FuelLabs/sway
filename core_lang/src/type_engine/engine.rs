use super::*;
use crate::Span;
use crate::concurrent_slab::ConcurrentSlab;
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
    ) -> Result<Option<Warning<'sc>>, TypeError<'sc>> {
        use TypeInfo::*;
        match (self.slab.get(a), self.slab.get(b)) {
            // If the types are exactly the same, we are done.
            (a, b) if a == b => Ok(None),

            // Follow any references
            (Ref(a), _) => self.unify(a, b, span),
            (_, Ref(b)) => self.unify(a, b, span),

            // When we don't know anything about either term, assume that
            // they match and make the one we know nothing about reference the
            // one we may know something about
            (Unknown, _) => {
                match self.slab.replace(a, &Unknown, TypeInfo::Ref(b)) {
                    None => Ok(None),
                    Some(_) => self.unify(a, b, span),
                }
            }
            (_, Unknown) => {
                match self.slab.replace(b, &Unknown, TypeInfo::Ref(a)) {
                    None => Ok(None),
                    Some(_) => self.unify(a, b, span),
                }
            }

            (UnsignedInteger(x), UnsignedInteger(y)) => match numeric_cast_compat(x, y) {
                NumericCastCompatResult::CastableWithWarning(warn) => {
                    // cast the one on the right to the one on the left
                    Ok(Some(warn))
                }
                // do nothing if compatible
                NumericCastCompatResult::Compatible => Ok(None),
            },
            (Numeric, b_info @ UnsignedInteger(_)) => {
                match self.slab.replace(a, &Numeric, b_info) {
                    None => Ok(None),
                    Some(_) => self.unify(a, b, span),
                }
            }
            (a_info @ UnsignedInteger(_), Numeric) => {
                match self.slab.replace(b, &Numeric, a_info) {
                    None => Ok(None),
                    Some(_) => self.unify(a, b, span),
                }
            }

            // When unifying complex types, we must check their sub-types. This
            // can be trivially implemented for tuples, sum types, etc.
            // (List(a_item), List(b_item)) => self.unify(a_item, b_item),
            // this can be used for curried function types but we might not want that
            // (Func(a_i, a_o), Func(b_i, b_o)) => {
            //     self.unify(a_i, b_i).and_then(|_| self.unify(a_o, b_o))
            // }

            // If no previous attempts to unify were successful, raise an error
            (a, b) => Err(TypeError::MismatchedType {
                expected: b.friendly_type_str(),
                received: a.friendly_type_str(),
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
    ) -> Result<Option<Warning<'sc>>, TypeError<'sc>> {
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

pub fn unify_with_self<'sc>(
    a: TypeId,
    b: TypeId,
    self_type: TypeId,
    span: &Span<'sc>,
) -> Result<Option<Warning<'sc>>, TypeError<'sc>> {
    TYPE_ENGINE.unify_with_self(a, b, self_type, span)
}

pub fn resolve_type<'sc>(
    id: TypeId,
    error_span: &Span<'sc>,
) -> Result<TypeInfo, TypeError<'sc>> {
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
