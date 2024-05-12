//! [`Constant`] is a typed constant value.

use crate::{context::Context, irtype::Type, pretty::DebugWithContext, value::Value, Padding};
use sway_types::u256::U256;

/// A [`Type`] and constant value, including [`ConstantValue::Undef`] for uninitialized constants.
#[derive(Debug, Clone, DebugWithContext, Hash)]
pub struct Constant {
    pub ty: Type,
    pub value: ConstantValue,
}

pub type B256 = U256;

/// A constant representation of each of the supported [`Type`]s.
#[derive(Debug, Clone, DebugWithContext, Hash)]
pub enum ConstantValue {
    Undef,
    Unit,
    Bool(bool),
    Uint(u64),
    U256(U256),
    B256(B256),
    String(Vec<u8>),
    Array(Vec<Constant>),
    Struct(Vec<Constant>),
    Reference(Box<Constant>),
    RawUntypedSlice(Vec<u8>),
}

/// A [Constant] with its required [Padding].
/// If the [Padding] is `None` the default [Padding] for the
/// [Constant] type is expected.
type ConstantWithPadding<'a> = (&'a Constant, Option<Padding>);

impl Constant {
    pub fn new_unit(context: &Context) -> Self {
        Constant {
            ty: Type::get_unit(context),
            value: ConstantValue::Unit,
        }
    }

    pub fn new_bool(context: &Context, b: bool) -> Self {
        Constant {
            ty: Type::get_bool(context),
            value: ConstantValue::Bool(b),
        }
    }

    /// For numbers bigger than u64 see `new_uint256`.
    pub fn new_uint(context: &mut Context, nbits: u16, n: u64) -> Self {
        Constant {
            ty: Type::new_uint(context, nbits),
            value: match nbits {
                256 => ConstantValue::U256(n.into()),
                _ => ConstantValue::Uint(n),
            },
        }
    }

    pub fn new_uint256(context: &mut Context, n: U256) -> Self {
        Constant {
            ty: Type::new_uint(context, 256),
            value: ConstantValue::U256(n),
        }
    }

    pub fn new_b256(context: &Context, bytes: [u8; 32]) -> Self {
        Constant {
            ty: Type::get_b256(context),
            value: ConstantValue::B256(B256::from_be_bytes(&bytes)),
        }
    }

    pub fn new_string(context: &mut Context, string: Vec<u8>) -> Self {
        Constant {
            ty: Type::new_string_array(context, string.len() as u64),
            value: ConstantValue::String(string),
        }
    }

    pub fn new_array(context: &mut Context, elm_ty: Type, elems: Vec<Constant>) -> Self {
        Constant {
            ty: Type::new_array(context, elm_ty, elems.len() as u64),
            value: ConstantValue::Array(elems),
        }
    }

    pub fn new_struct(context: &mut Context, field_tys: Vec<Type>, fields: Vec<Constant>) -> Self {
        Constant {
            ty: Type::new_struct(context, field_tys),
            value: ConstantValue::Struct(fields),
        }
    }

    pub fn get_undef(ty: Type) -> Self {
        Constant {
            ty,
            value: ConstantValue::Undef,
        }
    }

    pub fn get_unit(context: &mut Context) -> Value {
        let new_const = Constant::new_unit(context);
        Value::new_constant(context, new_const)
    }

    pub fn get_bool(context: &mut Context, value: bool) -> Value {
        let new_const = Constant::new_bool(context, value);
        Value::new_constant(context, new_const)
    }

    pub fn get_uint(context: &mut Context, nbits: u16, value: u64) -> Value {
        let new_const = Constant::new_uint(context, nbits, value);
        Value::new_constant(context, new_const)
    }

    pub fn get_uint256(context: &mut Context, value: U256) -> Value {
        let new_const = Constant::new_uint256(context, value);
        Value::new_constant(context, new_const)
    }

    pub fn get_b256(context: &mut Context, value: [u8; 32]) -> Value {
        let new_const = Constant::new_b256(context, value);
        Value::new_constant(context, new_const)
    }

    pub fn get_string(context: &mut Context, value: Vec<u8>) -> Value {
        let new_const = Constant::new_string(context, value);
        Value::new_constant(context, new_const)
    }

    /// `value` must be created as an array constant first, using [`Constant::new_array()`].
    pub fn get_array(context: &mut Context, value: Constant) -> Value {
        assert!(value.ty.is_array(context));
        Value::new_constant(context, value)
    }

    /// `value` must be created as a struct constant first, using [`Constant::new_struct()`].
    pub fn get_struct(context: &mut Context, value: Constant) -> Value {
        assert!(value.ty.is_struct(context));
        Value::new_constant(context, value)
    }

    /// Returns the tag and the value of an enum constant if `self` is an enum constant,
    /// otherwise `None`.
    fn extract_enum_tag_and_value(&self, context: &Context) -> Option<(&Constant, &Constant)> {
        if !self.ty.is_enum(context) {
            return None;
        }

        let elems = match &self.value {
            ConstantValue::Struct(elems) if elems.len() == 2 => elems,
            _ => return None, // This should never be the case. If we have an enum, it is a struct with exactly two elements.
        };

        Some((&elems[0], &elems[1]))
    }

    /// Returns enum tag and value as [Constant]s, together with their [Padding]s,
    /// if `self` is an enum [Constant], otherwise `None`.
    pub fn enum_tag_and_value_with_paddings(
        &self,
        context: &Context,
    ) -> Option<(ConstantWithPadding, ConstantWithPadding)> {
        if !self.ty.is_enum(context) {
            return None;
        }

        let tag_and_value_with_paddings = self
            .elements_of_aggregate_with_padding(context)
            .expect("Enums are aggregates.");

        debug_assert!(tag_and_value_with_paddings.len() == 2, "In case of enums, `elements_of_aggregate_with_padding` must return exactly two elements, the tag and the value.");

        let tag = tag_and_value_with_paddings[0].clone();
        let value = tag_and_value_with_paddings[1].clone();

        Some((tag, value))
    }

    /// Returns elements of an array with the expected padding for each array element
    /// if `self` is an array [Constant], otherwise `None`.
    pub fn array_elements_with_padding(
        &self,
        context: &Context,
    ) -> Option<Vec<ConstantWithPadding>> {
        if !self.ty.is_array(context) {
            return None;
        }

        self.elements_of_aggregate_with_padding(context)
    }

    /// Returns fields of a struct with the expected padding for each field
    /// if `self` is a struct [Constant], otherwise `None`.
    pub fn struct_fields_with_padding(
        &self,
        context: &Context,
    ) -> Option<Vec<ConstantWithPadding>> {
        if !self.ty.is_struct(context) {
            return None;
        }

        self.elements_of_aggregate_with_padding(context)
    }

    /// Returns elements of an aggregate constant with the expected padding for each element
    /// if `self` is an aggregate (struct, enum, or array), otherwise `None`.
    /// If the returned [Padding] is `None` the default [Padding] for the type
    /// is expected.
    /// If the aggregate constant is an enum, the returned [Vec] has exactly two elements,
    /// the first being the tag and the second the value of the enum variant.
    fn elements_of_aggregate_with_padding(
        &self,
        context: &Context,
    ) -> Option<Vec<(&Constant, Option<Padding>)>> {
        // We need a special handling in case of enums.
        if let Some((tag, value)) = self.extract_enum_tag_and_value(context) {
            let tag_with_padding = (tag, None);

            // Enum variants are left padded to the word boundary, and the size
            // of each variant is the size of the union.
            // We know we have an enum here, means exactly two fields in the struct
            // second of which is the union.
            let target_size = self.ty.get_field_types(context)[1]
                .size(context)
                .in_bytes_aligned() as usize;

            let value_with_padding = (value, Some(Padding::Left { target_size }));

            return Some(vec![tag_with_padding, value_with_padding]);
        }

        match &self.value {
            // Individual array elements do not have additional padding.
            ConstantValue::Array(elems) => Some(elems.iter().map(|el| (el, None)).collect()),
            // Each struct field is right padded to the word boundary.
            ConstantValue::Struct(elems) => Some(
                elems
                    .iter()
                    .map(|el| {
                        let target_size = el.ty.size(context).in_bytes_aligned() as usize;
                        (el, Some(Padding::Right { target_size }))
                    })
                    .collect(),
            ),
            _ => None,
        }
    }

    /// Compare two Constant values. Can't impl PartialOrder because of context.
    pub fn eq(&self, context: &Context, other: &Self) -> bool {
        self.ty.eq(context, &other.ty)
            && match (&self.value, &other.value) {
                // Two Undefs are *NOT* equal (PartialEq allows this).
                (ConstantValue::Undef, _) | (_, ConstantValue::Undef) => false,
                (ConstantValue::Unit, ConstantValue::Unit) => true,
                (ConstantValue::Bool(l0), ConstantValue::Bool(r0)) => l0 == r0,
                (ConstantValue::Uint(l0), ConstantValue::Uint(r0)) => l0 == r0,
                (ConstantValue::U256(l0), ConstantValue::U256(r0)) => l0 == r0,
                (ConstantValue::B256(l0), ConstantValue::B256(r0)) => l0 == r0,
                (ConstantValue::String(l0), ConstantValue::String(r0)) => l0 == r0,
                (ConstantValue::Array(l0), ConstantValue::Array(r0))
                | (ConstantValue::Struct(l0), ConstantValue::Struct(r0)) => {
                    l0.iter().zip(r0.iter()).all(|(l0, r0)| l0.eq(context, r0))
                }
                _ => false,
            }
    }
}
