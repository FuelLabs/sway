//! Each of the valid `Value` types.
//!
//! These generally mimic the Sway types with a couple of exceptions:
//! - [`Type::Unit`] is still a discrete type rather than an empty tuple.  This may change in the
//!   future.
//! - [`Type::Union`] is a sum type which resembles a C union.  Each member of the union uses the
//!   same storage and the size of the union is the size of the largest member.
//!
//! [`Aggregate`] is an abstract collection of [`Type`]s used for structs, unions and arrays,
//! though see below for future improvements around splitting arrays into a different construct.

use crate::{context::Context, pretty::DebugWithContext, Pointer};

#[derive(Debug, Clone, Copy, DebugWithContext)]
pub enum Type {
    Unit,
    Bool,
    Uint(u8),
    B256,
    String(u64),
    Array(Aggregate),
    Union(Aggregate),
    Struct(Aggregate),
    Pointer(Pointer),
}

impl Type {
    /// Return whether this is a 'copy' type, one whose value will always fit in a register.
    pub fn is_copy_type(&self) -> bool {
        matches!(
            self,
            Type::Unit | Type::Bool | Type::Uint(_) | Type::Pointer(_)
        )
    }

    /// Return a string representation of type, used for printing.
    pub fn as_string(&self, context: &Context) -> String {
        let sep_types_str = |agg_content: &AggregateContent, sep: &str| {
            agg_content
                .field_types()
                .iter()
                .map(|ty| ty.as_string(context))
                .collect::<Vec<_>>()
                .join(sep)
        };

        match self {
            Type::Unit => "()".into(),
            Type::Bool => "bool".into(),
            Type::Uint(nbits) => format!("u{}", nbits),
            Type::B256 => "b256".into(),
            Type::String(n) => format!("string<{}>", n),
            Type::Array(agg) => {
                let (ty, cnt) = &context.aggregates[agg.0].array_type();
                format!("[{}; {}]", ty.as_string(context), cnt)
            }
            Type::Union(agg) => {
                let agg_content = &context.aggregates[agg.0];
                format!("( {} )", sep_types_str(agg_content, " | "))
            }
            Type::Struct(agg) => {
                let agg_content = &context.aggregates[agg.0];
                format!("{{ {} }}", sep_types_str(agg_content, ", "))
            }
            Type::Pointer(ptr) => ptr.as_string(context, None),
        }
    }

    /// Compare a type to this one for equivalence.  We're unable to use `PartialEq` as we need the
    /// `Context` to compare structs and arrays.
    pub fn eq(&self, context: &Context, other: &Type) -> bool {
        match (self, other) {
            (Type::Unit, Type::Unit) => true,
            (Type::Bool, Type::Bool) => true,
            (Type::Uint(l), Type::Uint(r)) => l == r,
            (Type::B256, Type::B256) => true,
            (Type::String(l), Type::String(r)) => l == r,

            (Type::Array(l), Type::Array(r)) => l.is_equivalent(context, r),
            (Type::Struct(l), Type::Struct(r)) => l.is_equivalent(context, r),

            // Unions are special.  We say unions are equivalent to any of their variant types.
            (Type::Union(l), Type::Union(r)) => l.is_equivalent(context, r),
            (l, r @ Type::Union(_)) => r.eq(context, l),
            (Type::Union(l), r) => context.aggregates[l.0]
                .field_types()
                .iter()
                .any(|field_ty| r.eq(context, field_ty)),

            (Type::Pointer(l), Type::Pointer(r)) => l.is_equivalent(context, r),
            _ => false,
        }
    }

    pub fn strip_ptr_type(&self, context: &Context) -> Type {
        if let Type::Pointer(ptr) = self {
            *ptr.get_type(context)
        } else {
            *self
        }
    }

    /// Gets the inner pointer type if its a pointer.
    pub fn get_inner_ptr_type(&self, context: &Context) -> Option<Type> {
        match self {
            Type::Pointer(ptr) => Some(*ptr.get_type(context)),
            _ => None,
        }
    }

    /// Returns true if this is a pointer type.
    pub fn is_ptr_type(&self) -> bool {
        matches!(self, Type::Pointer(_))
    }
}

/// A collection of [`Type`]s.
///
/// XXX I've added Array as using Aggregate in the hope ExtractValue could be used just like with
/// struct aggregates, but it turns out we need ExtractElement (which takes an index Value).  So
/// Aggregate can be a 'struct' or 'array' but you only ever use them with Struct and Array types
/// and with ExtractValue and ExtractElement... so they're orthogonal and we can simplify aggregate
/// again to be only for structs.
///
/// But also to keep Type as Copy we need to put the Array meta into another copy type (rather than
/// recursing with Box<Type>, effectively a different Aggregate.  This could be OK though, still
/// simpler that what we have here.
///
/// NOTE: `Aggregate` derives `Eq` (and `PartialEq`) so that it can also derive `Hash`.  But we must
/// be careful not to use `==` or `!=` to compare `Aggregate` for equivalency -- i.e., to check
/// that they represent the same collection of types.  Instead the `is_equivalent()` method is
/// provided.  XXX Perhaps `Hash` should be impl'd directly without `Eq` if possible?

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, DebugWithContext)]
pub struct Aggregate(#[in_context(aggregates)] pub generational_arena::Index);

#[doc(hidden)]
#[derive(Debug, Clone, DebugWithContext)]
pub enum AggregateContent {
    ArrayType(Type, u64),
    FieldTypes(Vec<Type>),
}

impl Aggregate {
    /// Return a new struct specific aggregate.
    pub fn new_struct(context: &mut Context, field_types: Vec<Type>) -> Self {
        Aggregate(
            context
                .aggregates
                .insert(AggregateContent::FieldTypes(field_types)),
        )
    }

    /// Returna new array specific aggregate.
    pub fn new_array(context: &mut Context, element_type: Type, count: u64) -> Self {
        Aggregate(
            context
                .aggregates
                .insert(AggregateContent::ArrayType(element_type, count)),
        )
    }

    /// Tests whether an aggregate has the same sub-types.
    pub fn is_equivalent(&self, context: &Context, other: &Aggregate) -> bool {
        context.aggregates[self.0].eq(context, &context.aggregates[other.0])
    }

    /// Get a reference to the [`AggregateContent`] for this aggregate.
    pub fn get_content<'a>(&self, context: &'a Context) -> &'a AggregateContent {
        &context.aggregates[self.0]
    }

    /// Get the type of (nested) aggregate fields, if found.  If an index is into a `Union` then it
    /// will get the type of the indexed variant.
    pub fn get_field_type(&self, context: &Context, indices: &[u64]) -> Option<Type> {
        indices.iter().fold(Some(Type::Struct(*self)), |ty, idx| {
            ty.and_then(|ty| match ty {
                Type::Struct(agg) | Type::Union(agg) => context.aggregates[agg.0]
                    .field_types()
                    .get(*idx as usize)
                    .cloned(),

                // Trying to index a non-aggregate.
                _otherwise => None,
            })
        })
    }

    /// Get the type of the array element, if applicable.
    pub fn get_elem_type(&self, context: &Context) -> Option<Type> {
        if let AggregateContent::ArrayType(ty, _) = context.aggregates[self.0] {
            Some(ty)
        } else {
            None
        }
    }
}

impl AggregateContent {
    pub fn field_types(&self) -> &Vec<Type> {
        match self {
            AggregateContent::FieldTypes(types) => types,
            AggregateContent::ArrayType(..) => panic!("Getting field types from array aggregate."),
        }
    }

    pub fn array_type(&self) -> (&Type, &u64) {
        match self {
            AggregateContent::FieldTypes(..) => panic!("Getting array type from fields aggregate."),
            AggregateContent::ArrayType(ty, cnt) => (ty, cnt),
        }
    }

    /// Tests whether an aggregate has the same sub-types.
    pub fn eq(&self, context: &Context, other: &AggregateContent) -> bool {
        match (self, other) {
            (AggregateContent::FieldTypes(l_tys), AggregateContent::FieldTypes(r_tys)) => l_tys
                .iter()
                .zip(r_tys.iter())
                .all(|(l, r)| l.eq(context, r)),
            (
                AggregateContent::ArrayType(l_ty, l_cnt),
                AggregateContent::ArrayType(r_ty, r_cnt),
            ) => l_cnt == r_cnt && l_ty.eq(context, r_ty),

            _ => false,
        }
    }
}
