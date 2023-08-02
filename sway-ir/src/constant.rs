//! [`Constant`] is a typed constant value.

use crate::{context::Context, irtype::Type, pretty::DebugWithContext, value::Value};

/// A [`Type`] and constant value, including [`ConstantValue::Undef`] for uninitialized constants.
#[derive(Debug, Clone, DebugWithContext)]
pub struct Constant {
    pub ty: Type,
    pub value: ConstantValue,
}

/// A constant representation of each of the supported [`Type`]s.
#[derive(Debug, Clone, DebugWithContext)]
pub enum ConstantValue {
    Undef,
    Unit,
    Bool(bool),
    Uint(u64),
    U256(u64), // TODO u256 limited to u64 values
    B256([u8; 32]),
    String(Vec<u8>),
    Array(Vec<Constant>),
    Struct(Vec<Constant>),
}

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

    pub fn new_uint(context: &mut Context, nbits: u16, n: u64) -> Self {
        Constant {
            ty: Type::new_uint(context, nbits),
            value:match nbits {
                256 => ConstantValue::U256(n),
                _ => ConstantValue::Uint(n)
            },
        }
    }

    pub fn new_b256(context: &Context, bytes: [u8; 32]) -> Self {
        Constant {
            ty: Type::get_b256(context),
            value: ConstantValue::B256(bytes),
        }
    }

    pub fn new_string(context: &mut Context, string: Vec<u8>) -> Self {
        Constant {
            ty: Type::new_string(context, string.len() as u64),
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
