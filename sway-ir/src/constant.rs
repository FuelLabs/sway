//! [`Constant`] is a typed constant value.

use std::hash::{Hash, Hasher};

use crate::{context::Context, irtype::Type, pretty::DebugWithContext, value::Value};
use rustc_hash::FxHasher;
use sway_types::u256::U256;

/// A wrapper around an [ECS](https://github.com/orlp/slotmap) handle into the
/// [`Context`].
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, DebugWithContext)]
pub struct Constant(#[in_context(values)] pub slotmap::DefaultKey);

impl Constant {
    /// Get or create a unique constant with given contents.
    pub fn unique(context: &mut Context, constant: ConstantContent) -> Constant {
        let mut hasher = FxHasher::default();
        constant.hash(&mut hasher);
        let hash = hasher.finish();
        // Insert a new entry if it doesn't exist.
        context.constants_map.entry(hash).or_default();
        let constants = context.constants_map.get(&hash).unwrap();
        // If the constant already exists, return it.
        for c in constants.iter() {
            if context.constants.get(c.0).unwrap().eq(context, &constant) {
                return *c;
            }
        }
        let constant = Constant(context.constants.insert(constant));
        // Re-borrow the constants map (mutably this time) to insert the new constant.
        let constants = context.constants_map.get_mut(&hash).unwrap();
        constants.push(constant);
        constant
    }

    /// Get the contents of a unique constant
    pub fn get_content<'a>(&self, context: &'a Context) -> &'a ConstantContent {
        context
            .constants
            .get(self.0)
            .expect("Constants are global immutable data, they must live through the context")
    }
}

/// A [`Type`] and constant value, including [`ConstantValue::Undef`] for uninitialized constants.
#[derive(Debug, Clone, DebugWithContext, Hash)]
pub struct ConstantContent {
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
    Array(Vec<ConstantContent>),
    Slice(Vec<ConstantContent>),
    Struct(Vec<ConstantContent>),
    Reference(Box<ConstantContent>),
    RawUntypedSlice(Vec<u8>),
}

impl ConstantContent {
    pub fn new_unit(context: &Context) -> Self {
        ConstantContent {
            ty: Type::get_unit(context),
            value: ConstantValue::Unit,
        }
    }

    pub fn new_bool(context: &Context, b: bool) -> Self {
        ConstantContent {
            ty: Type::get_bool(context),
            value: ConstantValue::Bool(b),
        }
    }

    /// For numbers bigger than u64 see `new_uint256`.
    pub fn new_uint(context: &mut Context, nbits: u16, n: u64) -> Self {
        ConstantContent {
            ty: Type::new_uint(context, nbits),
            value: match nbits {
                256 => ConstantValue::U256(n.into()),
                _ => ConstantValue::Uint(n),
            },
        }
    }

    pub fn new_uint256(context: &mut Context, n: U256) -> Self {
        ConstantContent {
            ty: Type::new_uint(context, 256),
            value: ConstantValue::U256(n),
        }
    }

    pub fn new_b256(context: &Context, bytes: [u8; 32]) -> Self {
        ConstantContent {
            ty: Type::get_b256(context),
            value: ConstantValue::B256(B256::from_be_bytes(&bytes)),
        }
    }

    pub fn new_string(context: &mut Context, string: Vec<u8>) -> Self {
        ConstantContent {
            ty: Type::new_string_array(context, string.len() as u64),
            value: ConstantValue::String(string),
        }
    }

    pub fn new_array(context: &mut Context, elm_ty: Type, elems: Vec<ConstantContent>) -> Self {
        ConstantContent {
            ty: Type::new_array(context, elm_ty, elems.len() as u64),
            value: ConstantValue::Array(elems),
        }
    }

    pub fn new_struct(
        context: &mut Context,
        field_tys: Vec<Type>,
        fields: Vec<ConstantContent>,
    ) -> Self {
        ConstantContent {
            ty: Type::new_struct(context, field_tys),
            value: ConstantValue::Struct(fields),
        }
    }

    pub fn get_undef(ty: Type) -> Self {
        ConstantContent {
            ty,
            value: ConstantValue::Undef,
        }
    }

    pub fn get_unit(context: &mut Context) -> Value {
        let new_const_contents = ConstantContent::new_unit(context);
        let new_const = Constant::unique(context, new_const_contents);
        Value::new_constant(context, new_const)
    }

    pub fn get_bool(context: &mut Context, value: bool) -> Value {
        let new_const_contents = ConstantContent::new_bool(context, value);
        let new_const = Constant::unique(context, new_const_contents);
        Value::new_constant(context, new_const)
    }

    pub fn get_uint(context: &mut Context, nbits: u16, value: u64) -> Value {
        let new_const_contents = ConstantContent::new_uint(context, nbits, value);
        let new_const = Constant::unique(context, new_const_contents);
        Value::new_constant(context, new_const)
    }

    pub fn get_uint256(context: &mut Context, value: U256) -> Value {
        let new_const_contents = ConstantContent::new_uint256(context, value);
        let new_const = Constant::unique(context, new_const_contents);
        Value::new_constant(context, new_const)
    }

    pub fn get_b256(context: &mut Context, value: [u8; 32]) -> Value {
        let new_const_contents = ConstantContent::new_b256(context, value);
        let new_const = Constant::unique(context, new_const_contents);
        Value::new_constant(context, new_const)
    }

    pub fn get_string(context: &mut Context, value: Vec<u8>) -> Value {
        let new_const_contents = ConstantContent::new_string(context, value);
        let new_const = Constant::unique(context, new_const_contents);
        Value::new_constant(context, new_const)
    }

    /// `value` must be created as an array constant first, using [`Constant::new_array()`].
    pub fn get_array(context: &mut Context, value: ConstantContent) -> Value {
        assert!(value.ty.is_array(context));
        let new_const = Constant::unique(context, value);
        Value::new_constant(context, new_const)
    }

    /// `value` must be created as a struct constant first, using [`Constant::new_struct()`].
    pub fn get_struct(context: &mut Context, value: ConstantContent) -> Value {
        assert!(value.ty.is_struct(context));
        let new_const = Constant::unique(context, value);
        Value::new_constant(context, new_const)
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

    pub fn as_uint(&self) -> Option<u64> {
        match &self.value {
            ConstantValue::Uint(v) => Some(*v),
            _ => None,
        }
    }
}
