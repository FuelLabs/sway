//! [`Constant`] is a typed constant value.

use crate::{
    context::Context,
    irtype::{Aggregate, Type},
    metadata::MetadataIndex,
    value::Value,
};

/// A [`Type`] and constant value, including [`ConstantValue::Undef`] for uninitialized constants.
#[derive(Debug, Clone)]
pub struct Constant {
    pub ty: Type,
    pub value: ConstantValue,
}

/// A constant representation of each of the supported [`Type`]s.
#[derive(Debug, Clone)]
pub enum ConstantValue {
    Undef,
    Unit,
    Bool(bool),
    Uint(u64),
    B256([u8; 32]),
    String(String),
    Array(Vec<Constant>),
    Struct(Vec<Constant>),
}

impl Constant {
    pub fn new_undef(context: &Context, ty: Type) -> Self {
        match ty {
            Type::Array(aggregate) => {
                let (elem_type, count) = context.aggregates[aggregate.0].array_type();
                let elem_init = Self::new_undef(context, *elem_type);
                Constant::new_array(&aggregate, vec![elem_init; *count as usize])
            }
            Type::Struct(aggregate) => {
                let field_types = context.aggregates[aggregate.0].field_types().clone();
                let field_inits = field_types
                    .into_iter()
                    .map(|field_type| Self::new_undef(context, field_type)) // Hrmm, recursive structures would break here.
                    .collect();
                Constant::new_struct(&aggregate, field_inits)
            }
            _otherwise => Constant {
                ty,
                value: ConstantValue::Undef,
            },
        }
    }

    pub fn new_unit() -> Self {
        Constant {
            ty: Type::Unit,
            value: ConstantValue::Unit,
        }
    }

    pub fn new_bool(b: bool) -> Self {
        Constant {
            ty: Type::Bool,
            value: ConstantValue::Bool(b),
        }
    }

    pub fn new_uint(nbits: u8, n: u64) -> Self {
        Constant {
            ty: Type::Uint(nbits),
            value: ConstantValue::Uint(n),
        }
    }

    pub fn new_b256(bytes: [u8; 32]) -> Self {
        Constant {
            ty: Type::B256,
            value: ConstantValue::B256(bytes),
        }
    }

    pub fn new_string(string: String) -> Self {
        // XXX Need to parse the string for escapes?  To do.
        Constant {
            ty: Type::String(string.chars().count() as u64),
            value: ConstantValue::String(string),
        }
    }

    pub fn new_array(aggregate: &Aggregate, elems: Vec<Constant>) -> Self {
        Constant {
            ty: Type::Array(*aggregate),
            value: ConstantValue::Array(elems),
        }
    }

    pub fn new_struct(aggregate: &Aggregate, fields: Vec<Constant>) -> Self {
        Constant {
            ty: Type::Struct(*aggregate),
            value: ConstantValue::Struct(fields),
        }
    }

    pub fn get_undef(context: &mut Context, ty: Type, span_md_idx: Option<MetadataIndex>) -> Value {
        Value::new_constant(context, Constant::new_undef(context, ty), span_md_idx)
    }

    pub fn get_unit(context: &mut Context, span_md_idx: Option<MetadataIndex>) -> Value {
        Value::new_constant(context, Constant::new_unit(), span_md_idx)
    }

    pub fn get_bool(
        context: &mut Context,
        value: bool,
        span_md_idx: Option<MetadataIndex>,
    ) -> Value {
        Value::new_constant(context, Constant::new_bool(value), span_md_idx)
    }

    pub fn get_uint(
        context: &mut Context,
        nbits: u8,
        value: u64,
        span_md_idx: Option<MetadataIndex>,
    ) -> Value {
        Value::new_constant(context, Constant::new_uint(nbits, value), span_md_idx)
    }

    pub fn get_b256(
        context: &mut Context,
        value: [u8; 32],
        span_md_idx: Option<MetadataIndex>,
    ) -> Value {
        Value::new_constant(context, Constant::new_b256(value), span_md_idx)
    }

    pub fn get_string(
        context: &mut Context,
        value: String,
        span_md_idx: Option<MetadataIndex>,
    ) -> Value {
        Value::new_constant(context, Constant::new_string(value), span_md_idx)
    }

    /// `value` must be created as an array constant first, using [`Constant::new_array()`].
    pub fn get_array(
        context: &mut Context,
        value: Constant,
        span_md_idx: Option<MetadataIndex>,
    ) -> Value {
        assert!(matches!(
            value,
            Constant {
                ty: Type::Array(_),
                ..
            }
        ));
        Value::new_constant(context, value, span_md_idx)
    }

    /// `value` must be created as a struct constant first, using [`Constant::new_struct()`].
    pub fn get_struct(
        context: &mut Context,
        value: Constant,
        span_md_idx: Option<MetadataIndex>,
    ) -> Value {
        assert!(matches!(
            value,
            Constant {
                ty: Type::Struct(_),
                ..
            }
        ));
        Value::new_constant(context, value, span_md_idx)
    }
}
