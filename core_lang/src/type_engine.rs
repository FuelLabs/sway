use crate::error::*;
use crate::types::{IntegerBits, ResolvedType};
use std::collections::HashMap;

trait TypeEngine<'sc> {
    type TypeId;
    type TypeInfo;
    type ResolvedType;
    type Error;
    /// Insert a new bit of inference information about a specific type. Receive a [Self::TypeId]
    /// representing that information in return.
    fn insert(&mut self, info: Self::TypeInfo) -> Self::TypeId;
    /// Attempt to unify two type ids into one equivalence class. Throw an error if it is impossible.
    fn unify(
        &mut self,
        a: Self::TypeId,
        b: Self::TypeId,
    ) -> Result<Option<Warning<'sc>>, Self::Error>;
    /// Attempt to reconstruct a concrete type from the given type term ID. This
    /// may fail if we don't yet have enough information to figure out what the
    /// type is.
    fn resolve(&self, id: Self::TypeId) -> Result<Self::ResolvedType, Self::Error>;
}

/// A concrete type that has been fully inferred
#[derive(Debug)]
enum Type {
    Num,
    Bool,
    List(Box<Type>),
    Func(Box<Type>, Box<Type>),
}

/// A identifier to uniquely refer to our type terms
pub type TypeId = usize;

// /// Information about a type term
// #[derive(Clone, Debug)]
// enum TypeInfo {
//     // No information about the type of this type term
//     Unknown,
//     // This type term is the same as another type term
//     Ref(TypeId),
//     // This type term is definitely a number
//     Num,
//     // This type term is definitely a boolean
//     Bool,
//     // This type term is definitely a list
//     List(TypeId),
//     // This type term is definitely a function
//     Func(TypeId, TypeId),
// }
/// Type information without an associated value, used for type inferencing and definition.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeInfo<'sc> {
    Unknown,
    Str(u64),
    UnsignedInteger(IntegerBits),
    Boolean,
    /// A custom type could be a struct or similar if the name is in scope,
    /// or just a generic parameter if it is not.
    /// At parse time, there is no sense of scope, so this determination is not made
    /// until the semantic analysis stage.
    Custom {
        name: crate::Ident<'sc>,
    },
    /// For the type inference engine to use when a type references another type
    Ref(TypeId),
    Unit,
    SelfType,
    Byte,
    B256,
    /// This means that specific type of a number is not yet known. It will be
    /// determined via inference at a later time.
    Numeric,
    Contract,
    // used for recovering from errors in the ast
    ErrorRecovery,
}

#[derive(Default)]
struct Engine<'sc> {
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

    /// Make the types of two type terms equivalent (or produce an error if
    /// there is a conflict between them)
    fn unify(
        &mut self,
        a: Self::TypeId,
        b: Self::TypeId,
    ) -> Result<Option<Warning<'sc>>, Self::Error> {
        use TypeInfo::*;
        match (self.vars[&a].clone(), self.vars[&b].clone()) {
            // Follow any references
            (Ref(a), _) => self.unify(a, b),
            (_, Ref(b)) => self.unify(a, b),

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
                    // then push warning (the todo)
                    todo!("put together compile warning")
                }
                // do nothing if compatible
                NumericCastCompatResult::Compatible => Ok(None),
            },
            (Numeric, b @ UnsignedInteger(_)) => {
                self.vars.insert(a, b);
                Ok(None)
            }
            (b @ UnsignedInteger(_), Numeric) => {
                self.vars.insert(a, b);
                Ok(None)
            }

            // When unifying complex types, we must check their sub-types. This
            // can be trivially implemented for tuples, sum types, etc.
            // (List(a_item), List(b_item)) => self.unify(a_item, b_item),
            // this can be used for curried function types but we might not want that
            // (Func(a_i, a_o), Func(b_i, b_o)) => {
            //     self.unify(a_i, b_i).and_then(|_| self.unify(a_o, b_o))
            // }

            // If no previous attempts to unify were successful, raise an error
            (a, b) => todo!("Conflict between {:?} and {:?}", a, b),
        }
    }

    fn resolve(&self, id: Self::TypeId) -> Result<Self::ResolvedType, Self::Error> {
        use TypeInfo::*;
        match self.vars[&id] {
            Unknown => todo!("Cannot infer"),
            Ref(id) => self.resolve(id),
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
}

enum NumericCastCompatResult<'sc> {
    Compatible,
    CastableWithWarning(Warning<'sc>),
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

// # Example usage
// In reality, the most common approach will be to walk your AST, assigning type
// terms to each of your nodes with whatever information you have available. You
// will also need to call `engine.unify(x, y)` when you know two nodes have the
// same type, such as in the statement `x = y;`.

#[test]
fn main() {
    let mut engine = Engine::default();

    // numerics
    let id = engine.insert(TypeInfo::Numeric);
    let id2 = engine.insert(TypeInfo::UnsignedInteger(IntegerBits::Eight));

    // Unify them together...
    engine.unify(id, id2).unwrap();

    assert_eq!(
        engine.resolve(id).unwrap(),
        ResolvedType::UnsignedInteger(IntegerBits::Eight)
    );
}
