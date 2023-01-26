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

use crate::{context::Context, pretty::DebugWithContext};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, DebugWithContext)]
pub struct Type(pub generational_arena::Index);

#[derive(Debug, Clone, DebugWithContext, Hash, PartialEq, Eq)]
pub enum TypeContent {
    Unit,
    Bool,
    Uint(u8),
    B256,
    String(u64),
    Array(Type, u64),
    Union(Vec<Type>),
    Struct(Vec<Type>),
    Slice,
}

impl Type {
    fn get_or_create_unique_type(context: &mut Context, t: TypeContent) -> Type {
        // Trying to avoiding cloning t unless we're creating a new type.
        #[allow(clippy::map_entry)]
        if !context.type_map.contains_key(&t) {
            let new_type = Type(context.types.insert(t.clone()));
            context.type_map.insert(t, new_type);
            new_type
        } else {
            context.type_map.get(&t).copied().unwrap()
        }
    }

    /// Get Type if it already exists.
    pub fn get_type(context: &Context, t: &TypeContent) -> Option<Type> {
        context.type_map.get(t).copied()
    }

    pub fn create_basic_types(context: &mut Context) {
        Self::get_or_create_unique_type(context, TypeContent::Unit);
        Self::get_or_create_unique_type(context, TypeContent::Bool);
        Self::get_or_create_unique_type(context, TypeContent::Uint(8));
        Self::get_or_create_unique_type(context, TypeContent::Uint(32));
        Self::get_or_create_unique_type(context, TypeContent::Uint(64));
        Self::get_or_create_unique_type(context, TypeContent::B256);
        Self::get_or_create_unique_type(context, TypeContent::Slice);
    }

    /// Get the content for this Type.
    pub fn get_content<'a>(&self, context: &'a Context) -> &'a TypeContent {
        &context.types[self.0]
    }

    /// Get unit type
    pub fn get_unit(context: &Context) -> Type {
        Self::get_type(context, &TypeContent::Unit).expect("create_basic_types not called")
    }

    /// Get bool type
    pub fn get_bool(context: &Context) -> Type {
        Self::get_type(context, &TypeContent::Bool).expect("create_basic_types not called")
    }

    /// New unsigned integer type
    pub fn new_uint(context: &mut Context, width: u8) -> Type {
        Self::get_or_create_unique_type(context, TypeContent::Uint(width))
    }

    /// New u8 type
    pub fn get_uint8(context: &Context) -> Type {
        Self::get_type(context, &TypeContent::Uint(8)).expect("create_basic_types not called")
    }

    /// New u32 type
    pub fn get_uint32(context: &Context) -> Type {
        Self::get_type(context, &TypeContent::Uint(32)).expect("create_basic_types not called")
    }

    /// New u64 type
    pub fn get_uint64(context: &Context) -> Type {
        Self::get_type(context, &TypeContent::Uint(64)).expect("create_basic_types not called")
    }

    /// Get unsigned integer type
    pub fn get_uint(context: &Context, width: u8) -> Option<Type> {
        Self::get_type(context, &TypeContent::Uint(width))
    }

    /// Get B256 type
    pub fn get_b256(context: &Context) -> Type {
        Self::get_type(context, &TypeContent::B256).expect("create_basic_types not called")
    }

    /// Get string type
    pub fn new_string(context: &mut Context, len: u64) -> Type {
        Self::get_or_create_unique_type(context, TypeContent::String(len))
    }

    /// Get array type
    pub fn new_array(context: &mut Context, elm_ty: Type, len: u64) -> Type {
        Self::get_or_create_unique_type(context, TypeContent::Array(elm_ty, len))
    }

    /// Get union type
    pub fn new_union(context: &mut Context, fields: Vec<Type>) -> Type {
        Self::get_or_create_unique_type(context, TypeContent::Union(fields))
    }

    /// Get struct type
    pub fn new_struct(context: &mut Context, fields: Vec<Type>) -> Type {
        Self::get_or_create_unique_type(context, TypeContent::Struct(fields))
    }

    /// Get pointer type
    pub fn get_slice(context: &mut Context) -> Type {
        Self::get_type(context, &TypeContent::Slice).expect("create_basic_types not called")
    }

    /// Return a string representation of type, used for printing.
    pub fn as_string(&self, context: &Context) -> String {
        let sep_types_str = |agg_content: &Vec<Type>, sep: &str| {
            agg_content
                .iter()
                .map(|ty| ty.as_string(context))
                .collect::<Vec<_>>()
                .join(sep)
        };

        match self.get_content(context) {
            TypeContent::Unit => "()".into(),
            TypeContent::Bool => "bool".into(),
            TypeContent::Uint(nbits) => format!("u{nbits}"),
            TypeContent::B256 => "b256".into(),
            TypeContent::String(n) => format!("string<{n}>"),
            TypeContent::Array(ty, cnt) => {
                format!("[{}; {}]", ty.as_string(context), cnt)
            }
            TypeContent::Union(agg) => {
                format!("( {} )", sep_types_str(agg, " | "))
            }
            TypeContent::Struct(agg) => {
                format!("{{ {} }}", sep_types_str(agg, ", "))
            }
            TypeContent::Slice => "slice".into(),
        }
    }

    /// Compare a type to this one for equivalence.
    /// `PartialEq` does not take into account the special case for Unions below.
    pub fn eq(&self, context: &Context, other: &Type) -> bool {
        match (self.get_content(context), other.get_content(context)) {
            (TypeContent::Unit, TypeContent::Unit) => true,
            (TypeContent::Bool, TypeContent::Bool) => true,
            (TypeContent::Uint(l), TypeContent::Uint(r)) => l == r,
            (TypeContent::B256, TypeContent::B256) => true,
            (TypeContent::String(l), TypeContent::String(r)) => l == r,

            (TypeContent::Array(l, llen), TypeContent::Array(r, rlen)) => {
                llen == rlen && l.eq(context, r)
            }
            (TypeContent::Struct(l), TypeContent::Struct(r))
            | (TypeContent::Union(l), TypeContent::Union(r)) => {
                l.len() == r.len() && l.iter().zip(r.iter()).all(|(l, r)| l.eq(context, r))
            }
            // Unions are special.  We say unions are equivalent to any of their variant types.
            (_, TypeContent::Union(_)) => other.eq(context, self),
            (TypeContent::Union(l), _) => l.iter().any(|field_ty| other.eq(context, field_ty)),

            (TypeContent::Slice, TypeContent::Slice) => true,
            _ => false,
        }
    }

    /// Is bool type
    pub fn is_bool(&self, context: &Context) -> bool {
        matches!(*self.get_content(context), TypeContent::Bool)
    }

    /// Is unit type
    pub fn is_unit(&self, context: &Context) -> bool {
        matches!(*self.get_content(context), TypeContent::Unit)
    }

    /// Is unsigned integer type
    pub fn is_uint(&self, context: &Context) -> bool {
        matches!(*self.get_content(context), TypeContent::Uint(_))
    }

    /// Is u8 type
    pub fn is_uint8(&self, context: &Context) -> bool {
        matches!(*self.get_content(context), TypeContent::Uint(8))
    }

    /// Is u32 type
    pub fn is_uint32(&self, context: &Context) -> bool {
        matches!(*self.get_content(context), TypeContent::Uint(32))
    }

    /// Is u64 type
    pub fn is_uint64(&self, context: &Context) -> bool {
        matches!(*self.get_content(context), TypeContent::Uint(64))
    }

    /// Is unsigned integer type of specific width
    pub fn is_uint_of(&self, context: &Context, width: u8) -> bool {
        matches!(*self.get_content(context), TypeContent::Uint(width_) if width == width_)
    }

    /// Is B256 type
    pub fn is_b256(&self, context: &Context) -> bool {
        matches!(*self.get_content(context), TypeContent::B256)
    }

    /// Is string type
    pub fn is_string(&self, context: &Context) -> bool {
        matches!(*self.get_content(context), TypeContent::String(_))
    }

    /// Is array type
    pub fn is_array(&self, context: &Context) -> bool {
        matches!(*self.get_content(context), TypeContent::Array(..))
    }

    /// Is union type
    pub fn is_union(&self, context: &Context) -> bool {
        matches!(*self.get_content(context), TypeContent::Union(_))
    }

    /// Is struct type
    pub fn is_struct(&self, context: &Context) -> bool {
        matches!(*self.get_content(context), TypeContent::Struct(_))
    }

    /// Returns true if this is a slice type.
    pub fn is_slice(&self, context: &Context) -> bool {
        matches!(*self.get_content(context), TypeContent::Slice)
    }

    /// Get width of an integer type.
    pub fn get_uint_width(&self, context: &Context) -> Option<u8> {
        if let TypeContent::Uint(width) = self.get_content(context) {
            Some(*width)
        } else {
            None
        }
    }

    /// What's the type of the struct value indexed by indices.
    pub fn get_indexed_type(&self, context: &Context, indices: &[u64]) -> Option<Type> {
        if indices.is_empty() {
            return None;
        }

        indices.iter().fold(Some(*self), |ty, idx| {
            ty.and_then(|ty| {
                ty.get_field_type(context, *idx)
                    .or_else(|| ty.get_array_elem_type(context))
            })
        })
    }

    pub fn get_field_type(&self, context: &Context, idx: u64) -> Option<Type> {
        if let TypeContent::Struct(agg) | TypeContent::Union(agg) = self.get_content(context) {
            agg.get(idx as usize).cloned()
        } else {
            // Trying to index a non-aggregate.
            None
        }
    }

    /// Get the type of the array element, if applicable.
    pub fn get_array_elem_type(&self, context: &Context) -> Option<Type> {
        if let TypeContent::Array(ty, _) = *self.get_content(context) {
            Some(ty)
        } else {
            None
        }
    }

    /// Get the length of the array , if applicable.
    pub fn get_array_len(&self, context: &Context) -> Option<u64> {
        if let TypeContent::Array(_, n) = *self.get_content(context) {
            Some(n)
        } else {
            None
        }
    }

    /// Get the length of a string
    pub fn get_string_len(&self, context: &Context) -> Option<u64> {
        if let TypeContent::String(n) = *self.get_content(context) {
            Some(n)
        } else {
            None
        }
    }

    /// Get the type of each field of a struct Type. Empty vector otherwise.
    pub fn get_field_types(&self, context: &Context) -> Vec<Type> {
        match self.get_content(context) {
            TypeContent::Struct(fields) | TypeContent::Union(fields) => fields.clone(),
            _ => vec![],
        }
    }
}

/// A helper to check if an Option<Type> value is of a particular Type.
pub trait TypeOption {
    fn is(&self, pred: fn(&Type, &Context) -> bool, context: &Context) -> bool;
}

impl TypeOption for Option<Type> {
    fn is(&self, pred: fn(&Type, &Context) -> bool, context: &Context) -> bool {
        self.filter(|ty| pred(ty, context)).is_some()
    }
}
