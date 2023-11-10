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

use crate::{context::Context, pretty::DebugWithContext, Constant, ConstantValue, Value};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, DebugWithContext)]
pub struct Type(pub generational_arena::Index);

#[derive(Debug, Clone, DebugWithContext, Hash, PartialEq, Eq)]
pub enum TypeContent {
    Unit,
    Bool,
    Uint(u16),
    B256,
    StringSlice,
    StringArray(u64),
    Array(Type, u64),
    Union(Vec<Type>),
    Struct(Vec<Type>),
    Slice,
    Pointer(Type),
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
        Self::get_or_create_unique_type(context, TypeContent::Uint(64));
        Self::get_or_create_unique_type(context, TypeContent::Uint(256));
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
    pub fn new_uint(context: &mut Context, width: u16) -> Type {
        Self::get_or_create_unique_type(context, TypeContent::Uint(width))
    }

    /// New u8 type
    pub fn get_uint8(context: &Context) -> Type {
        Self::get_type(context, &TypeContent::Uint(8)).expect("create_basic_types not called")
    }

    /// New u64 type
    pub fn get_uint64(context: &Context) -> Type {
        Self::get_type(context, &TypeContent::Uint(64)).expect("create_basic_types not called")
    }

    /// New u64 type
    pub fn get_uint256(context: &Context) -> Type {
        Self::get_type(context, &TypeContent::Uint(256)).expect("create_basic_types not called")
    }

    /// Get unsigned integer type
    pub fn get_uint(context: &Context, width: u16) -> Option<Type> {
        Self::get_type(context, &TypeContent::Uint(width))
    }

    /// Get B256 type
    pub fn get_b256(context: &Context) -> Type {
        Self::get_type(context, &TypeContent::B256).expect("create_basic_types not called")
    }

    /// Get string type
    pub fn new_string_array(context: &mut Context, len: u64) -> Type {
        Self::get_or_create_unique_type(context, TypeContent::StringArray(len))
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

    /// New pointer type
    pub fn new_ptr(context: &mut Context, to_ty: Type) -> Type {
        Self::get_or_create_unique_type(context, TypeContent::Pointer(to_ty))
    }

    /// Get slice type
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
            TypeContent::StringSlice => "str".into(),
            TypeContent::StringArray(n) => format!("string<{n}>"),
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
            TypeContent::Pointer(ty) => format!("ptr {}", ty.as_string(context)),
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

            (TypeContent::StringSlice, TypeContent::StringSlice) => true,
            (TypeContent::StringArray(l), TypeContent::StringArray(r)) => l == r,

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
            (TypeContent::Pointer(l), TypeContent::Pointer(r)) => l.eq(context, r),

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
    pub fn is_uint_of(&self, context: &Context, width: u16) -> bool {
        matches!(*self.get_content(context), TypeContent::Uint(width_) if width == width_)
    }

    /// Is B256 type
    pub fn is_b256(&self, context: &Context) -> bool {
        matches!(*self.get_content(context), TypeContent::B256)
    }

    /// Is string type
    pub fn is_string_slice(&self, context: &Context) -> bool {
        matches!(*self.get_content(context), TypeContent::StringSlice)
    }

    /// Is string type
    pub fn is_string_array(&self, context: &Context) -> bool {
        matches!(*self.get_content(context), TypeContent::StringArray(_))
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

    /// Is enum type
    pub fn is_enum(&self, context: &Context) -> bool {
        // We have to do some painful special handling here for enums, which are tagged unions.
        // This really should be handled by the IR more explicitly and is something that will
        // hopefully be addressed by https://github.com/FuelLabs/sway/issues/2819#issuecomment-1256930392

        // Enums are at the moment represented as structs with two fields, first one being
        // the tag and second the union of variants. Enums are the only place we currently use unions
        // which makes the below heuristics valid.
        if !self.is_struct(context) {
            return false;
        }

        let field_tys = self.get_field_types(context);

        field_tys.len() == 2 && field_tys[0].is_uint(context) && field_tys[1].is_union(context)
    }

    /// Is aggregate type: struct, union, enum or array.
    pub fn is_aggregate(&self, context: &Context) -> bool {
        // Notice that enums are structs of tags and unions.
        self.is_struct(context) || self.is_union(context) || self.is_array(context)
    }

    /// Returns true if this is a slice type.
    pub fn is_slice(&self, context: &Context) -> bool {
        matches!(*self.get_content(context), TypeContent::Slice)
    }

    /// Returns true if this is a pointer type.
    pub fn is_ptr(&self, context: &Context) -> bool {
        matches!(*self.get_content(context), TypeContent::Pointer(_))
    }

    /// Get pointed to type iff self is a Pointer.
    pub fn get_pointee_type(&self, context: &Context) -> Option<Type> {
        if let TypeContent::Pointer(to_ty) = self.get_content(context) {
            Some(*to_ty)
        } else {
            None
        }
    }

    /// Get width of an integer type.
    pub fn get_uint_width(&self, context: &Context) -> Option<u16> {
        if let TypeContent::Uint(width) = self.get_content(context) {
            Some(*width)
        } else {
            None
        }
    }

    /// What's the type of the struct/array value indexed by indices.
    pub fn get_indexed_type(&self, context: &Context, indices: &[u64]) -> Option<Type> {
        if indices.is_empty() {
            return None;
        }

        indices.iter().try_fold(*self, |ty, idx| {
            ty.get_field_type(context, *idx)
                .or_else(|| match ty.get_content(context) {
                    TypeContent::Array(ty, len) if idx < len => Some(*ty),
                    _ => None,
                })
        })
    }

    /// What's the offset, in bytes, of the indexed element?
    /// Returns `None` on invalid indices.
    /// Panics if `self` is not an aggregate (struct, enum, union, or array).
    pub fn get_indexed_offset(&self, context: &Context, indices: &[u64]) -> Option<u64> {
        indices
            .iter()
            .try_fold((*self, 0), |(ty, accum_offset), idx| {
                if ty.is_struct(context) {
                    // Sum up all sizes of all previous fields.
                    // Every struct field is aligned to word boundary.
                    let prev_idxs_offset = (0..(*idx)).try_fold(0, |accum, pre_idx| {
                        ty.get_field_type(context, pre_idx)
                            .map(|field_ty| accum + field_ty.size(context).in_bytes_aligned())
                    })?;
                    ty.get_field_type(context, *idx)
                        .map(|field_ty| (field_ty, accum_offset + prev_idxs_offset))
                } else if ty.is_union(context) {
                    ty.get_field_type(context, *idx)
                        .map(|field_ty| (field_ty, accum_offset))
                } else {
                    assert!(
                        ty.is_array(context),
                        "Expected aggregate type. Got {}.",
                        ty.as_string(context)
                    );
                    // size_of_element * idx will be the offset of idx.
                    ty.get_array_elem_type(context).map(|elm_ty| {
                        let prev_idxs_offset = ty
                            .get_array_elem_type(context)
                            .unwrap()
                            .size(context)
                            .in_bytes()
                            * idx;
                        (elm_ty, accum_offset + prev_idxs_offset)
                    })
                }
            })
            .map(|pair| pair.1)
    }

    /// What's the offset, in bytes, of the value indexed element?
    /// It may not always be possible to determine statically.
    pub fn get_value_indexed_offset(&self, context: &Context, indices: &[Value]) -> Option<u64> {
        let const_indices: Vec<_> = indices
            .iter()
            .map_while(|idx| {
                if let Some(Constant {
                    value: ConstantValue::Uint(idx),
                    ty: _,
                }) = idx.get_constant(context)
                {
                    Some(*idx)
                } else {
                    None
                }
            })
            .collect();
        (const_indices.len() == indices.len())
            .then(|| self.get_indexed_offset(context, &const_indices))
            .flatten()
    }

    pub fn get_field_type(&self, context: &Context, idx: u64) -> Option<Type> {
        if let TypeContent::Struct(fields) | TypeContent::Union(fields) = self.get_content(context)
        {
            fields.get(idx as usize).cloned()
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

    /// Get the length of a string.
    pub fn get_string_len(&self, context: &Context) -> Option<u64> {
        if let TypeContent::StringArray(n) = *self.get_content(context) {
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

    /// Get the offset, in bytes, and the [Type] of the struct field at the index `field_idx`, if `self` is a struct,
    /// otherwise `None`.
    /// Panics if the `field_idx` is out of bounds.
    pub fn get_struct_field_offset_and_type(
        &self,
        context: &Context,
        field_idx: u64,
    ) -> Option<(u64, Type)> {
        if !self.is_struct(context) {
            return None;
        }

        let field_idx = field_idx as usize;
        let field_types = self.get_field_types(context);
        let field_offs_in_bytes = field_types
            .iter()
            .take(field_idx)
            .map(|field_ty| {
                // Struct fields are aligned to word boundary.
                field_ty.size(context).in_bytes_aligned()
            })
            .sum::<u64>();

        Some((field_offs_in_bytes, field_types[field_idx]))
    }

    /// Get the offset, in bytes, and the [Type] of the union field at the index `field_idx`, if `self` is a union,
    /// otherwise `None`.
    /// Panics if the `field_idx` is out of bounds.
    pub fn get_union_field_offset_and_type(
        &self,
        context: &Context,
        field_idx: u64,
    ) -> Option<(u64, Type)> {
        if !self.is_union(context) {
            return None;
        }

        let field_idx = field_idx as usize;
        let field_type = self.get_field_types(context)[field_idx];
        let union_size_in_bytes = self.size(context).in_bytes();
        let field_size_in_bytes = field_type.size(context).in_bytes();

        // The union fields are at offset (union_size - field_size) due to left padding.
        Some((union_size_in_bytes - field_size_in_bytes, field_type))
    }

    pub fn size(&self, context: &Context) -> TypeSize {
        match self.get_content(context) {
            TypeContent::Uint(8) | TypeContent::Bool | TypeContent::Unit => TypeSize::new(1),
            // All integers larger than a byte are words since FuelVM only has memory operations on those two units.
            TypeContent::Uint(16)
            | TypeContent::Uint(32)
            | TypeContent::Uint(64)
            | TypeContent::Pointer(_) => TypeSize::new(8),
            TypeContent::Uint(256) => TypeSize::new(32),
            TypeContent::Uint(_) => unreachable!(),
            TypeContent::Slice => TypeSize::new(16),
            TypeContent::B256 => TypeSize::new(32),
            TypeContent::StringSlice => TypeSize::new(16),
            TypeContent::StringArray(n) => {
                TypeSize::new(super::size_bytes_round_up_to_word_alignment!(*n))
            }
            TypeContent::Array(el_ty, cnt) => TypeSize::new(cnt * el_ty.size(context).in_bytes()),
            TypeContent::Struct(field_tys) => {
                // Sum up all the field sizes, aligned to words.
                TypeSize::new(
                    field_tys
                        .iter()
                        .map(|field_ty| field_ty.size(context).in_bytes_aligned())
                        .sum(),
                )
            }
            TypeContent::Union(field_tys) => {
                // Find the max size for field sizes.
                TypeSize::new(
                    field_tys
                        .iter()
                        .map(|field_ty| field_ty.size(context).in_bytes_aligned())
                        .max()
                        .unwrap_or(0),
                )
            }
        }
    }
}

// This is a mouthful...
#[macro_export]
macro_rules! size_bytes_round_up_to_word_alignment {
    ($bytes_expr: expr) => {
        ($bytes_expr + 7) - (($bytes_expr + 7) % 8)
    };
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

/// Provides information about a size of a type, raw and aligned to word boundaries.
#[derive(Clone, Debug)]
pub struct TypeSize {
    size_in_bytes: u64,
}

impl TypeSize {
    pub(crate) fn new(size_in_bytes: u64) -> Self {
        Self { size_in_bytes }
    }

    /// Returns the actual (unaligned) size of the type in bytes.
    pub fn in_bytes(&self) -> u64 {
        self.size_in_bytes
    }

    /// Returns the size of the type in bytes, aligned to word boundary.
    pub fn in_bytes_aligned(&self) -> u64 {
        (self.size_in_bytes + 7) - ((self.size_in_bytes + 7) % 8)
    }

    /// Returns the size of the type in words (aligned to word boundary).
    pub fn in_words(&self) -> u64 {
        (self.size_in_bytes + 7) / 8
    }
}

/// Provides information about padding expected when laying values in memory.
/// Padding depends on the type of the value, but also on the embedding of
/// the value in aggregates. E.g., in an array of `u8`, each `u8` is "padded"
/// to its size of one byte while as a struct field, it will be right padded
/// to 8 bytes.
#[derive(Clone, Debug)]
pub enum Padding {
    Left { target_size: usize },
    Right { target_size: usize },
}

impl Padding {
    /// Returns the default [Padding] for `u8`.
    pub fn default_for_u8(_value: u8) -> Self {
        // Dummy _value is used only to ensure correct usage at the call site.
        Self::Right { target_size: 1 }
    }

    /// Returns the default [Padding] for `u64`.
    pub fn default_for_u64(_value: u64) -> Self {
        // Dummy _value is used only to ensure correct usage at the call site.
        Self::Right { target_size: 8 }
    }

    /// Returns the default [Padding] for a byte array.
    pub fn default_for_byte_array(value: &[u8]) -> Self {
        Self::Right {
            target_size: value.len(),
        }
    }

    /// Returns the default [Padding] for an aggregate.
    /// `aggregate_size` is the overall size of the aggregate in bytes.
    pub fn default_for_aggregate(aggregate_size: usize) -> Self {
        Self::Right {
            target_size: aggregate_size,
        }
    }

    /// The target size in bytes.
    pub fn target_size(&self) -> usize {
        use Padding::*;
        match self {
            Left { target_size } | Right { target_size } => *target_size,
        }
    }
}
