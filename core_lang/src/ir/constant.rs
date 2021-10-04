use super::*;

#[derive(Clone)]
pub(crate) struct Constant {
    // XXX Do we need ty, when the value implies its type?  For the struct?
    pub(crate) ty: Type,
    pub(crate) value: ConstantValue,
}

#[derive(Clone)]
pub(crate) enum ConstantValue {
    Undef,
    Unit,
    Bool(bool),
    Uint(u64),
    B256([u8; 32]),
    String(String),
    Struct(Vec<Constant>),
}

impl Constant {
    pub(super) fn new_undef(context: &Context, ty: Type) -> Self {
        match ty {
            Type::Struct(aggregate) => {
                let field_types = context.aggregates[aggregate.0].field_types.clone();
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

    pub(super) fn new_unit() -> Self {
        Constant {
            ty: Type::Unit,
            value: ConstantValue::Unit,
        }
    }

    pub(super) fn new_bool(b: bool) -> Self {
        Constant {
            ty: Type::Bool,
            value: ConstantValue::Bool(b),
        }
    }

    pub(super) fn new_uint(nbits: u8, n: u64) -> Self {
        Constant {
            ty: Type::Uint(nbits),
            value: ConstantValue::Uint(n),
        }
    }

    pub(super) fn new_b256(bytes: [u8; 32]) -> Self {
        Constant {
            ty: Type::B256,
            value: ConstantValue::B256(bytes),
        }
    }

    pub(super) fn new_string(string: String) -> Self {
        // XXX Need to parse the string for escapes?  To do.
        Constant {
            ty: Type::String(string.chars().count() as u64),
            value: ConstantValue::String(string),
        }
    }

    pub(super) fn new_struct(aggregate: &Aggregate, fields: Vec<Constant>) -> Self {
        Constant {
            ty: Type::Struct(*aggregate),
            value: ConstantValue::Struct(fields),
        }
    }

    pub(crate) fn get_undef(context: &mut Context, ty: Type) -> Value {
        Value::new_constant(context, Constant::new_undef(context, ty))
    }

    pub(crate) fn get_unit(context: &mut Context) -> Value {
        Value::new_constant(context, Constant::new_unit())
    }

    pub(crate) fn get_bool(context: &mut Context, value: bool) -> Value {
        Value::new_constant(context, Constant::new_bool(value))
    }

    pub(crate) fn get_uint(context: &mut Context, nbits: u8, value: u64) -> Value {
        Value::new_constant(context, Constant::new_uint(nbits, value))
    }

    pub(crate) fn get_b256(context: &mut Context, value: [u8; 32]) -> Value {
        Value::new_constant(context, Constant::new_b256(value))
    }

    pub(crate) fn get_string(context: &mut Context, value: String) -> Value {
        Value::new_constant(context, Constant::new_string(value))
    }

    // This requires you create a struct constant first, using `Constant::new_struct()`.
    pub(crate) fn get_struct(context: &mut Context, value: Constant) -> Value {
        assert!(matches!(
            value,
            Constant {
                ty: Type::Struct(_),
                ..
            }
        ));
        Value::new_constant(context, value)
    }
}
