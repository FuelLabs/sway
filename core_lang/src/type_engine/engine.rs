use super::*;
use crate::{
    build_config::BuildConfig, error::*, semantic_analysis::ast_node::TypedStructField,
    semantic_analysis::TypedExpression, types::ResolvedType, CallPath, Ident, Rule, Span,
};
use derivative::Derivative;
use std::collections::HashMap;
use std::iter::FromIterator;

use pest::iterators::Pair;
#[derive(Default, Clone, Debug)]
pub(crate) struct Engine<'sc> {
    id_counter: usize, // Used to generate unique IDs
    vars: HashMap<TypeId, TypeInfo<'sc>>,
}

impl<'sc> TypeEngine<'sc> for Engine<'sc> {
    type TypeId = usize;
    type TypeInfo = TypeInfo<'sc>;
    type ResolvedType = ResolvedType<'sc>;
    type Error = TypeError<'sc>;
    /// Create a new type term with whatever we have about its type
    fn insert(&mut self, info: TypeInfo<'sc>) -> TypeId {
        // Generate a new ID for our type term
        self.id_counter += 1;
        let id = self.id_counter;
        self.vars.insert(id, info);
        id
    }

    fn unify_with_self(
        &mut self,
        a: Self::TypeId,
        b: Self::TypeId,
        self_type: Self::TypeId,
        span: &Span<'sc>,
    ) -> Result<Option<Warning<'sc>>, Self::Error> {
        let a = if self.vars[&a] == TypeInfo::SelfType {
            self_type
        } else {
            a
        };
        let b = if self.vars[&b] == TypeInfo::SelfType {
            self_type
        } else {
            b
        };

        self.unify(a, b, span)
    }
    /// Make the types of two type terms equivalent (or produce an error if
    /// there is a conflict between them)
    fn unify(
        &mut self,
        a: Self::TypeId,
        b: Self::TypeId,
        span: &Span<'sc>,
    ) -> Result<Option<Warning<'sc>>, Self::Error> {
        use TypeInfo::*;
        match (self.vars[&a].clone(), self.vars[&b].clone()) {
            // Follow any references
            (Ref(a), _) => self.unify(a, b, span),
            (_, Ref(b)) => self.unify(a, b, span),

            // When we don't know anything about either term, assume that
            // they match and make the one we know nothing about reference the
            // one we may know something about
            (Unknown, _) => {
                self.vars.insert(a, TypeInfo::Ref(b));
                Ok(None)
            }
            (_, Unknown) => {
                self.vars.insert(b, TypeInfo::Ref(a));
                Ok(None)
            }

            // Primitives are trivial to unify
            (Numeric, Numeric) => Ok(None),
            (Boolean, Boolean) => Ok(None),
            (B256, B256) => Ok(None),
            (Byte, Byte) => Ok(None),
            (UnsignedInteger(x), UnsignedInteger(y)) => match numeric_cast_compat(x, y) {
                NumericCastCompatResult::CastableWithWarning(warn) => {
                    // cast the one on the right to the one on the left
                    self.vars.insert(a, UnsignedInteger(x));
                    Ok(Some(warn))
                }
                // do nothing if compatible
                NumericCastCompatResult::Compatible => Ok(None),
            },
            (Numeric, b @ UnsignedInteger(_)) => {
                self.vars.insert(a, b);
                Ok(None)
            }
            (a @ UnsignedInteger(_), Numeric) => {
                self.vars.insert(b, a);
                Ok(None)
            }
            (Enum { .. }, _) | (_, Enum { .. }) => todo!("enum ty"),
            (Struct { .. }, _) | (_, Struct { .. }) => todo!("struct ty"),

            // When unifying complex types, we must check their sub-types. This
            // can be trivially implemented for tuples, sum types, etc.
            // (List(a_item), List(b_item)) => self.unify(a_item, b_item),
            // this can be used for curried function types but we might not want that
            // (Func(a_i, a_o), Func(b_i, b_o)) => {
            //     self.unify(a_i, b_i).and_then(|_| self.unify(a_o, b_o))
            // }

            // If no previous attempts to unify were successful, raise an error
            (a, b) => Err(TypeError::MismatchedType {
                expected: a.friendly_type_str(),
                received: b.friendly_type_str(),
                help_text: Default::default(),
                span: span.clone(),
            }),
        }
    }

    fn resolve(
        &self,
        id: Self::TypeId,
        span: &Span<'sc>,
    ) -> Result<Self::ResolvedType, Self::Error> {
        use TypeInfo::*;
        match self.vars[&id] {
            Unknown => Err(TypeError::UnknownType { span: span.clone() }),
            Ref(id) => self.resolve(id, span),
            // defaults to u64
            Numeric => Ok(ResolvedType::UnsignedInteger(IntegerBits::SixtyFour)),
            Boolean => Ok(ResolvedType::Boolean),
            // List(item) => todo!("Ok(ResolvedType::List(Box::new(self.reconstruct(item)?)))"),
            // Func(i, o) => Ok(ResolvedType::Function {
            //     from: Box::new(self.resolve(i)?),
            //     to: Box::new(self.resolve(o)?),
            // }),
            UnsignedInteger(x) => Ok(ResolvedType::UnsignedInteger(x)),
            ref a => todo!("{:?}", a),
        }
    }
    fn look_up_type_id(&self, id: TypeId) -> ResolvedType<'sc> {
        self.resolve(
            id,
            &Span {
                span: pest::Span::new(
                    "because we \"expect\" here, we don't need this error span.",
                    0,
                    0,
                )
                .unwrap(),
                path: Default::default(),
            },
        )
        .expect("Internal error: type ID did not exist in type engine")
    }
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
