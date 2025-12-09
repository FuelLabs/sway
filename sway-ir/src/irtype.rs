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

use crate::{context::Context, pretty::DebugWithContext, ConstantContent, ConstantValue, Value};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Type(pub slotmap::DefaultKey);

impl DebugWithContext for Type {
    fn fmt_with_context(
        &self,
        formatter: &mut std::fmt::Formatter,
        context: &Context,
    ) -> std::fmt::Result {
        self.get_content(context)
            .fmt_with_context(formatter, context)
    }
}

#[derive(Debug, Clone, DebugWithContext, Hash, PartialEq, Eq)]
pub enum TypeContent {
    Never,
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
    Pointer,
    TypedPointer(Type),
    TypedSlice(Type),
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
        Self::get_or_create_unique_type(context, TypeContent::Never);
        Self::get_or_create_unique_type(context, TypeContent::Unit);
        Self::get_or_create_unique_type(context, TypeContent::Bool);
        Self::get_or_create_unique_type(context, TypeContent::Uint(8));
        Self::get_or_create_unique_type(context, TypeContent::Uint(16));
        Self::get_or_create_unique_type(context, TypeContent::Uint(32));
        Self::get_or_create_unique_type(context, TypeContent::Uint(64));
        Self::get_or_create_unique_type(context, TypeContent::Uint(256));
        Self::get_or_create_unique_type(context, TypeContent::B256);
        Self::get_or_create_unique_type(context, TypeContent::Slice);
        Self::get_or_create_unique_type(context, TypeContent::Pointer);
    }

    /// Get the content for this [Type].
    pub fn get_content<'a>(&self, context: &'a Context) -> &'a TypeContent {
        &context.types[self.0]
    }

    /// Get never type
    pub fn get_never(context: &Context) -> Type {
        Self::get_type(context, &TypeContent::Never).expect("create_basic_types not called")
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

    /// New u16 type
    pub fn get_uint16(context: &Context) -> Type {
        Self::get_type(context, &TypeContent::Uint(16)).expect("create_basic_types not called")
    }

    /// New u32 type
    pub fn get_uint32(context: &Context) -> Type {
        Self::get_type(context, &TypeContent::Uint(32)).expect("create_basic_types not called")
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

    /// Get untyped pointer type
    pub fn get_ptr(context: &Context) -> Type {
        Self::get_type(context, &TypeContent::Pointer).expect("create_basic_types not called")
    }

    pub fn new_untyped_slice(context: &mut Context) -> Type {
        Self::get_or_create_unique_type(context, TypeContent::Slice)
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
    pub fn new_typed_pointer(context: &mut Context, to_ty: Type) -> Type {
        Self::get_or_create_unique_type(context, TypeContent::TypedPointer(to_ty))
    }

    /// Get slice type
    pub fn get_slice(context: &Context) -> Type {
        Self::get_type(context, &TypeContent::Slice).expect("create_basic_types not called")
    }

    /// Get typed slice type
    pub fn get_typed_slice(context: &mut Context, item_ty: Type) -> Type {
        Self::get_or_create_unique_type(context, TypeContent::TypedSlice(item_ty))
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
            TypeContent::Never => "never".into(),
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
            TypeContent::Pointer => "ptr".into(),
            TypeContent::TypedSlice(ty) => format!("__slice[{}]", ty.as_string(context)),
            TypeContent::TypedPointer(ty) => format!("__ptr {}", ty.as_string(context)),
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

            (TypeContent::TypedSlice(l), TypeContent::TypedSlice(r)) => l.eq(context, r),

            (TypeContent::Struct(l), TypeContent::Struct(r))
            | (TypeContent::Union(l), TypeContent::Union(r)) => {
                l.len() == r.len() && l.iter().zip(r.iter()).all(|(l, r)| l.eq(context, r))
            }
            // Unions are special.  We say unions are equivalent to any of their variant types.
            (_, TypeContent::Union(_)) => other.eq(context, self),
            (TypeContent::Union(l), _) => l.iter().any(|field_ty| other.eq(context, field_ty)),
            // Never type can coerce into any other type.
            (TypeContent::Never, _) => true,
            (TypeContent::Slice, TypeContent::Slice) => true,
            (TypeContent::Pointer, TypeContent::Pointer) => true,
            (TypeContent::TypedPointer(l), TypeContent::TypedPointer(r)) => l.eq(context, r),
            _ => false,
        }
    }

    /// Is Never type
    pub fn is_never(&self, context: &Context) -> bool {
        matches!(*self.get_content(context), TypeContent::Never)
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

    /// Returns true if `self` is a slice type.
    pub fn is_slice(&self, context: &Context) -> bool {
        matches!(*self.get_content(context), TypeContent::Slice)
    }

    // TODO: (REFERENCES) Check all the usages of `is_ptr`.
    /// Returns true if `self` is a pointer type.
    pub fn is_ptr(&self, context: &Context) -> bool {
        matches!(
            *self.get_content(context),
            TypeContent::TypedPointer(_) | TypeContent::Pointer
        )
    }

    /// Get pointed to type iff `self`` is a pointer.
    pub fn get_pointee_type(&self, context: &Context) -> Option<Type> {
        if let TypeContent::TypedPointer(to_ty) = self.get_content(context) {
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

    /// What's the type of the struct/array value indexed by indices.
    pub fn get_value_indexed_type(&self, context: &Context, indices: &[Value]) -> Option<Type> {
        // Fetch the field type from the vector of Values.  If the value is a constant int then
        // unwrap it and try to fetch the field type (which will fail for arrays) otherwise (i.e.,
        // not a constant int or not a struct) fetch the array element type, which will fail for
        // non-arrays.
        indices.iter().try_fold(*self, |ty, idx_val| {
            idx_val
                .get_constant(context)
                .and_then(|const_ref| {
                    if let ConstantValue::Uint(n) = const_ref.get_content(context).value {
                        Some(n)
                    } else {
                        None
                    }
                })
                .and_then(|idx| ty.get_field_type(context, idx))
                .or_else(|| ty.get_array_elem_type(context))
        })
    }

    /// What's the offset, in bytes, of the indexed element?
    /// Returns `None` on invalid indices.
    /// Panics if `self` is not an aggregate (struct, union, or array).
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
                    // Union variants have their raw size in bytes and are
                    // left padded within the union.
                    let union_size_in_bytes = ty.size(context).in_bytes();
                    ty.get_field_type(context, *idx).map(|field_ty| {
                        (
                            field_ty,
                            accum_offset
                                + (union_size_in_bytes - field_ty.size(context).in_bytes()),
                        )
                    })
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
                if let Some(ConstantContent {
                    value: ConstantValue::Uint(idx),
                    ty: _,
                }) = idx.get_constant(context).map(|c| c.get_content(context))
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

    /// Get the type of the array element, if applicable.
    pub fn get_typed_slice_elem_type(&self, context: &Context) -> Option<Type> {
        if let TypeContent::TypedSlice(ty) = *self.get_content(context) {
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
        // Union variants have their raw size in bytes and are
        // left padded within the union.
        let field_size_in_bytes = field_type.size(context).in_bytes();

        // The union fields are at offset (union_size - field_size) due to left padding.
        Some((union_size_in_bytes - field_size_in_bytes, field_type))
    }

    /// Returns the memory size of the [Type].
    /// The returned `TypeSize::in_bytes` will provide the raw memory size of the `self`,
    /// when it's not embedded in an aggregate.
    pub fn size(&self, context: &Context) -> TypeSize {
        match self.get_content(context) {
            TypeContent::Unit | TypeContent::Never => TypeSize::new(0),
            TypeContent::Uint(8) | TypeContent::Bool => {
                TypeSize::new(1)
            }
            // All integers larger than a byte are words since FuelVM only has memory operations on those two units.
            TypeContent::Uint(16)
            | TypeContent::Uint(32)
            | TypeContent::Uint(64)
            | TypeContent::TypedPointer(_)
            | TypeContent::Pointer => TypeSize::new(8),
            TypeContent::Uint(256) => TypeSize::new(32),
            TypeContent::Uint(_) => unreachable!(),
            TypeContent::Slice => TypeSize::new(16),
            TypeContent::TypedSlice(..) => TypeSize::new(16),
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
        self.size_in_bytes.div_ceil(8)
    }
}

/// Provides information about padding expected when laying values in memory.
/// Padding depends on the type of the value, but also on the embedding of
/// the value in aggregates. E.g., in an array of `u8`, each `u8` is "padded"
/// to its size of one byte while as a struct field, it will be right padded
/// to 8 bytes.
#[derive(Clone, Debug, serde::Serialize)]
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

#[cfg(test)]
mod tests {
    pub use super::*;
    /// Unit tests in this module document and assert decisions on memory layout.
    mod memory_layout {
        use super::*;
        use crate::{Backtrace, Context};
        use once_cell::sync::Lazy;
        use sway_features::ExperimentalFeatures;
        use sway_types::SourceEngine;

        #[test]
        /// Bool, when not embedded in aggregates, has a size of 1 byte.
        fn boolean() {
            let context = create_context();

            let s_bool = Type::get_bool(&context).size(&context);

            assert_eq!(s_bool.in_bytes(), 1);
        }

        #[test]
        /// Unit, when not embedded in aggregates, has a size of 1 byte.
        fn unit() {
            let context = create_context();

            let s_unit = Type::get_unit(&context).size(&context);

            assert_eq!(s_unit.in_bytes(), 1);
        }

        #[test]
        /// `u8`, when not embedded in aggregates, has a size of 1 byte.
        fn unsigned_u8() {
            let context = create_context();

            let s_u8 = Type::get_uint8(&context).size(&context);

            assert_eq!(s_u8.in_bytes(), 1);
        }

        #[test]
        /// `u16`, `u32`, and `u64,`, when not embedded in aggregates, have a size of 8 bytes/1 word.
        fn unsigned_u16_u32_u64() {
            let context = create_context();

            let s_u16 = Type::get_uint16(&context).size(&context);
            let s_u32 = Type::get_uint32(&context).size(&context);
            let s_u64 = Type::get_uint64(&context).size(&context);

            assert_eq!(s_u16.in_bytes(), 8);
            assert_eq!(s_u16.in_bytes(), s_u16.in_bytes_aligned());

            assert_eq!(s_u32.in_bytes(), 8);
            assert_eq!(s_u32.in_bytes(), s_u32.in_bytes_aligned());

            assert_eq!(s_u64.in_bytes(), 8);
            assert_eq!(s_u64.in_bytes(), s_u64.in_bytes_aligned());
        }

        #[test]
        /// `u256`, when not embedded in aggregates, has a size of 32 bytes.
        fn unsigned_u256() {
            let context = create_context();

            let s_u256 = Type::get_uint256(&context).size(&context);

            assert_eq!(s_u256.in_bytes(), 32);
            assert_eq!(s_u256.in_bytes(), s_u256.in_bytes_aligned());
        }

        #[test]
        /// Pointer to any type, when not embedded in aggregates, has a size of 8 bytes/1 word.
        fn pointer() {
            let mut context = create_context();

            for ty in all_sample_types(&mut context) {
                let s_ptr = Type::new_typed_pointer(&mut context, ty).size(&context);

                assert_eq!(s_ptr.in_bytes(), 8);
                assert_eq!(s_ptr.in_bytes(), s_ptr.in_bytes_aligned());
            }

            assert_eq!(Type::get_ptr(&context).size(&context).in_bytes(), 8);
            assert_eq!(
                Type::get_ptr(&context).size(&context).in_bytes(),
                Type::get_ptr(&context).size(&context).in_bytes_aligned()
            );
        }

        #[test]
        /// Slice, when not embedded in aggregates, has a size of 16 bytes/2 words.
        /// The first word is the pointer to the actual content, and the second the
        /// length of the slice.
        fn slice() {
            let context = create_context();

            let s_slice = Type::get_slice(&context).size(&context);

            assert_eq!(s_slice.in_bytes(), 16);
            assert_eq!(s_slice.in_bytes(), s_slice.in_bytes_aligned());
        }

        #[test]
        /// `B256`, when not embedded in aggregates, has a size of 32 bytes.
        fn b256() {
            let context = create_context();

            let s_b256 = Type::get_b256(&context).size(&context);

            assert_eq!(s_b256.in_bytes(), 32);
            assert_eq!(s_b256.in_bytes(), s_b256.in_bytes_aligned());
        }

        #[test]
        /// String slice, when not embedded in aggregates, has a size of 16 bytes/2 words.
        /// The first word is the pointer to the actual content, and the second the
        /// length of the slice.
        fn string_slice() {
            let mut context = create_context();

            let s_slice = Type::get_or_create_unique_type(&mut context, TypeContent::StringSlice)
                .size(&context);

            assert_eq!(s_slice.in_bytes(), 16);
            assert_eq!(s_slice.in_bytes(), s_slice.in_bytes_aligned());
        }

        #[test]
        /// String array, when not embedded in aggregates, has a size in bytes of its length, aligned to the word boundary.
        /// Note that this differs from other arrays, which are packed but not, in addition, aligned to the word boundary.
        /// The reason we have the alignment/padding in case of string arrays, is because of the current ABI encoding.
        /// The output receipt returned by a contract call can be a string array, and the way the output is encoded
        /// (at least for small strings) is by literally putting the ASCII bytes in the return value register.
        /// For string arrays smaller than 8 bytes this poses a problem, because we have to fill the register with something
        /// or start reading memory that isn't ours. And the workaround was to simply pad all string arrays with zeroes so
        /// they're all at least 8 bytes long.
        /// Thus, changing this behavior would be a breaking change in ABI compatibility.
        /// Note that we do want to change this behavior in the future, as a part of either refactoring the ABI encoding
        /// or proper support for slices.
        fn string_array() {
            let mut context = create_context();

            for (str_array_ty, len) in sample_string_arrays(&mut context) {
                assert!(str_array_ty.is_string_array(&context)); // Just in case.

                let s_str_array = str_array_ty.size(&context);

                assert_eq!(str_array_ty.get_string_len(&context).unwrap(), len);

                assert_eq!(
                    s_str_array.in_bytes(),
                    size_bytes_round_up_to_word_alignment!(len)
                );
                assert_eq!(s_str_array.in_bytes(), s_str_array.in_bytes_aligned());
            }
        }

        #[test]
        /// Array, when not embedded in aggregates, has a size in bytes of its length multiplied by the size of its element's type.
        /// Arrays are packed. The offset of the n-th element is `n * __size_of<ElementType>()`.
        fn array() {
            let mut context = create_context();

            for (array_ty, len, elem_size) in sample_arrays(&mut context) {
                assert!(array_ty.is_array(&context)); // Just in case.

                let s_array = array_ty.size(&context);

                assert_eq!(array_ty.get_array_len(&context).unwrap(), len);

                // The size of the array is the length multiplied by the element size.
                assert_eq!(s_array.in_bytes(), len * elem_size);

                for elem_index in 0..len {
                    let elem_offset = array_ty
                        .get_indexed_offset(&context, &[elem_index])
                        .unwrap();

                    // The offset of the element is its index multiplied by the element size.
                    assert_eq!(elem_offset, elem_index * elem_size);
                }
            }
        }

        #[test]
        /// Struct has a size in bytes of the sum of all of its fields.
        /// The size of each individual field is a multiple of the word size. Thus,
        /// if needed, fields are right-padded to the multiple of the word size.
        /// Each individual field is aligned to the word boundary.
        /// Struct fields are ordered in the order of their appearance in the struct definition.
        /// The offset of each field is the sum of the sizes of the preceding fields.
        /// Since the size of the each individual field is a multiple of the word size,
        /// the size of the struct is also always a multiple of the word size.
        fn r#struct() {
            let mut context = create_context();

            for (struct_ty, fields) in sample_structs(&mut context) {
                assert!(struct_ty.is_struct(&context)); // Just in case.

                let s_struct = struct_ty.size(&context);

                // The size of the struct is the sum of the field sizes,
                // where each field is, if needed, right-padded to the multiple of the
                // word size.
                assert_eq!(
                    s_struct.in_bytes(),
                    fields
                        .iter()
                        .map(|(_, raw_size)| size_bytes_round_up_to_word_alignment!(raw_size))
                        .sum::<u64>()
                );
                // Structs' sizes are always multiples of the word size.
                assert_eq!(s_struct.in_bytes(), s_struct.in_bytes_aligned());

                for field_index in 0..fields.len() {
                    // The offset of a field is the sum of the sizes of the previous fields.
                    let expected_offset = fields
                        .iter()
                        .take(field_index)
                        .map(|(_, raw_size)| size_bytes_round_up_to_word_alignment!(raw_size))
                        .sum::<u64>();

                    let field_offset = struct_ty
                        .get_indexed_offset(&context, &[field_index as u64])
                        .unwrap();
                    assert_eq!(field_offset, expected_offset);

                    let (field_offset, field_type) = struct_ty
                        .get_struct_field_offset_and_type(&context, field_index as u64)
                        .unwrap();
                    assert_eq!(field_offset, expected_offset);
                    assert_eq!(field_type, fields[field_index].0);
                }
            }
        }

        #[test]
        /// Union has a size in bytes of the largest of all of its variants,
        /// where the largest variant is, if needed, left-padded to the multiple of the word size.
        /// Variants overlap in memory and are left-padded (aligned to the right) to the size of the
        /// largest variant (already _right_ aligned/left-padded to the word boundary).
        /// Thus, a variant, in a general case, needs not to be aligned to the word boundary.
        /// The offset of a variant, relative to the union address is:
        ///
        ///  `__size_of<UnionType>() - __size_of<VariantType>()`.
        ///
        /// Since the size of the largest variant is a multiple of the word size,
        /// the size of the union is also always a multiple of the word size.
        fn union() {
            let mut context = create_context();

            for (union_ty, variants) in sample_unions(&mut context) {
                assert!(union_ty.is_union(&context)); // Just in case.

                let s_union = union_ty.size(&context);

                // The size of the union is the size of the largest variant,
                // where the largest variant is, if needed, left-padded to the multiple
                // of the word size.
                assert_eq!(
                    s_union.in_bytes(),
                    variants
                        .iter()
                        .map(|(_, raw_size)| size_bytes_round_up_to_word_alignment!(raw_size))
                        .max()
                        .unwrap_or_default()
                );
                // Unions' sizes are always multiples of the word size.
                assert_eq!(s_union.in_bytes(), s_union.in_bytes_aligned());

                for (variant_index, variant) in variants.iter().enumerate() {
                    // Variants are left-padded.
                    // The offset of a variant is the union size minus the raw variant size.
                    let expected_offset = s_union.in_bytes() - variant.1;

                    let variant_offset = union_ty
                        .get_indexed_offset(&context, &[variant_index as u64])
                        .unwrap();
                    assert_eq!(variant_offset, expected_offset);

                    let (variant_offset, field_type) = union_ty
                        .get_union_field_offset_and_type(&context, variant_index as u64)
                        .unwrap();
                    assert_eq!(variant_offset, expected_offset);
                    assert_eq!(field_type, variant.0);
                }
            }
        }

        // A bit of trickery just to avoid bloating test setups by having `SourceEngine`
        // instantiation in every test.
        // Not that we can't do the same with the `Context` because it must be isolated and
        // unique in every test.
        static SOURCE_ENGINE: Lazy<SourceEngine> = Lazy::new(SourceEngine::default);

        fn create_context() -> Context<'static> {
            Context::new(
                &SOURCE_ENGINE,
                ExperimentalFeatures::default(),
                Backtrace::default(),
            )
        }

        /// Creates sample types that are not aggregates and do not point to
        /// other types. Where applicable, several typical representatives of
        /// a type are created, e.g., string arrays of different sizes.
        fn sample_non_aggregate_types(context: &mut Context) -> Vec<Type> {
            let mut types = vec![
                Type::get_bool(context),
                Type::get_unit(context),
                Type::get_uint(context, 8).unwrap(),
                Type::get_uint(context, 16).unwrap(),
                Type::get_uint(context, 32).unwrap(),
                Type::get_uint(context, 64).unwrap(),
                Type::get_uint(context, 256).unwrap(),
                Type::get_b256(context),
                Type::get_slice(context),
                Type::get_or_create_unique_type(context, TypeContent::StringSlice),
            ];

            types.extend(
                sample_string_arrays(context)
                    .into_iter()
                    .map(|(string_array, _)| string_array),
            );

            types
        }

        /// Creates sample string array types of different lengths and
        /// returns the string array types and their respective lengths.
        fn sample_string_arrays(context: &mut Context) -> Vec<(Type, u64)> {
            let mut types = vec![];

            for len in [0, 1, 7, 8, 15] {
                types.push((Type::new_string_array(context, len), len));
            }

            types
        }

        /// Creates sample array types of different lengths and
        /// different element types and returns the created array types
        /// and their respective lengths and the size of the element type.
        fn sample_arrays(context: &mut Context) -> Vec<(Type, u64, u64)> {
            let mut types = vec![];

            for len in [0, 1, 7, 8, 15] {
                for ty in sample_non_aggregate_types(context) {
                    // As commented in other places, we trust the result of the
                    // `size` method for non-aggregate types.
                    types.push((
                        Type::new_array(context, ty, len),
                        len,
                        ty.size(context).in_bytes(),
                    ));
                }

                for (array_ty, array_len, elem_size) in sample_arrays_to_embed(context) {
                    // We cannot use the `size` methods on arrays here because we use this
                    // samples to actually test it. We calculate the expected size manually
                    // according to the definition of the layout for the arrays.
                    types.push((
                        Type::new_array(context, array_ty, len),
                        len,
                        array_len * elem_size,
                    ));
                }

                for (struct_ty, struct_size) in sample_structs_to_embed(context) {
                    types.push((Type::new_array(context, struct_ty, len), len, struct_size));
                }
            }

            types
        }

        /// Creates sample struct types and returns the created struct types
        /// and their respective field types and their raw size in bytes
        /// (when not embedded).
        fn sample_structs(context: &mut Context) -> Vec<(Type, Vec<(Type, u64)>)> {
            let mut types = vec![];

            // Empty struct.
            types.push((Type::new_struct(context, vec![]), vec![]));

            // Structs with only one 1-byte long field.
            add_structs_with_non_aggregate_types_of_length_in_bytes(&mut types, context, 1);
            // Structs with only one 1-word long field.
            add_structs_with_non_aggregate_types_of_length_in_bytes(&mut types, context, 8);

            // Complex struct with fields of all non aggregate types, arrays, and structs.
            let mut fields = vec![];
            for ty in sample_non_aggregate_types(context) {
                // We can trust the result of the `size` method here,
                // because it is tested in tests for individual non-aggregate types.
                fields.push((ty, ty.size(context).in_bytes()));
            }
            for (array_ty, len, elem_size) in sample_arrays(context) {
                // We can't trust the result of the `size` method here,
                // because tests for arrays test embedded structs and vice versa.
                // So we will manually calculate the expected raw size in bytes,
                // as per the definition of the memory layout for the arrays.
                fields.push((array_ty, len * elem_size));
            }
            for (struct_ty, struct_size) in sample_structs_to_embed(context) {
                fields.push((struct_ty, struct_size));
            }

            types.push((
                Type::new_struct(
                    context,
                    fields.iter().map(|(field_ty, _)| *field_ty).collect(),
                ),
                fields,
            ));

            return types;

            fn add_structs_with_non_aggregate_types_of_length_in_bytes(
                types: &mut Vec<(Type, Vec<(Type, u64)>)>,
                context: &mut Context,
                field_type_size_in_bytes: u64,
            ) {
                for ty in sample_non_aggregate_types(context) {
                    if ty.size(context).in_bytes() != field_type_size_in_bytes {
                        continue;
                    }

                    types.push((
                        Type::new_struct(context, vec![ty]),
                        vec![(ty, field_type_size_in_bytes)],
                    ));
                }
            }
        }

        /// Creates sample union types and returns the created union types
        /// and their respective variant types and their raw size in bytes
        /// (when not embedded).
        fn sample_unions(context: &mut Context) -> Vec<(Type, Vec<(Type, u64)>)> {
            let mut types = vec![];

            // Empty union.
            types.push((Type::new_union(context, vec![]), vec![]));

            // Unions with only one 1-byte long variant.
            add_unions_with_non_aggregate_types_of_length_in_bytes(&mut types, context, 1);
            // Unions with only one 1-word long variant.
            add_unions_with_non_aggregate_types_of_length_in_bytes(&mut types, context, 8);

            // Complex union with variants of all non aggregate types, arrays, and structs.
            // For the reasons for using the `size` method for non-aggregates vs
            // calculating sizes for non aggregates, see the comment in the
            // `sample_structs` function.
            let mut variants = vec![];
            for ty in sample_non_aggregate_types(context) {
                variants.push((ty, ty.size(context).in_bytes()));
            }
            for (array_ty, len, elem_size) in sample_arrays(context) {
                variants.push((array_ty, len * elem_size));
            }
            for (struct_ty, struct_size) in sample_structs_to_embed(context) {
                variants.push((struct_ty, struct_size));
            }

            types.push((
                Type::new_union(
                    context,
                    variants.iter().map(|(field_ty, _)| *field_ty).collect(),
                ),
                variants,
            ));

            return types;

            fn add_unions_with_non_aggregate_types_of_length_in_bytes(
                types: &mut Vec<(Type, Vec<(Type, u64)>)>,
                context: &mut Context,
                variant_type_size_in_bytes: u64,
            ) {
                for ty in sample_non_aggregate_types(context) {
                    if ty.size(context).in_bytes() != variant_type_size_in_bytes {
                        continue;
                    }

                    types.push((
                        Type::new_union(context, vec![ty]),
                        vec![(ty, variant_type_size_in_bytes)],
                    ));
                }
            }
        }

        /// Creates sample arrays to embed in other aggregates.
        /// Returns the created array types, its length, and the size
        /// of the element type.
        fn sample_arrays_to_embed(context: &mut Context) -> Vec<(Type, u64, u64)> {
            let mut types = vec![];

            for len in [0, 1, 7, 8, 15] {
                for elem_ty in sample_non_aggregate_types(context) {
                    types.push((
                        Type::new_array(context, elem_ty, len),
                        len,
                        elem_ty.size(context).in_bytes(),
                    ));
                }
            }

            types
        }

        /// Creates sample structs to embed in other aggregates.
        /// Returns the struct type and size in bytes for each created struct.
        fn sample_structs_to_embed(context: &mut Context) -> Vec<(Type, u64)> {
            let mut types = vec![];

            // Create structs with just one field for each non_aggregate type.
            for field_ty in sample_non_aggregate_types(context) {
                // We can trust the result of the `size` method here,
                // because it is tested in tests for individual non-aggregate types.
                // We align it to the word boundary to satisfy the layout of structs.
                types.push((
                    Type::new_struct(context, vec![field_ty]),
                    field_ty.size(context).in_bytes_aligned(),
                ));
            }

            // Create structs for pairwise combinations of field types.
            let field_types = sample_non_aggregate_types(context);
            for (index, first_field_ty) in field_types.iter().enumerate() {
                for second_field_type in field_types.iter().skip(index) {
                    // Again, we trust the `size` method called on non-aggregate types
                    // and calculate the struct size on our own.
                    let struct_size = first_field_ty.size(context).in_bytes_aligned()
                        + second_field_type.size(context).in_bytes_aligned();
                    types.push((
                        Type::new_struct(context, vec![*first_field_ty, *second_field_type]),
                        struct_size,
                    ));
                }
            }

            // Create a struct with a field for each aggregate type.
            let field_types = sample_non_aggregate_types(context);
            let struct_size = field_types
                .iter()
                .map(|ty| ty.size(context).in_bytes_aligned())
                .sum();
            types.push((Type::new_struct(context, field_types), struct_size));

            types
        }

        /// Returns all types that we can have, including several typical samples for
        /// aggregates like, e.g., arrays of different elements and different sizes.
        fn all_sample_types(context: &mut Context) -> Vec<Type> {
            let mut types = vec![];

            types.extend(sample_non_aggregate_types(context));
            types.extend(
                sample_arrays(context)
                    .into_iter()
                    .map(|(array_ty, _, _)| array_ty),
            );
            types.extend(
                sample_structs(context)
                    .into_iter()
                    .map(|(array_ty, __)| array_ty),
            );

            types
        }
    }
}
