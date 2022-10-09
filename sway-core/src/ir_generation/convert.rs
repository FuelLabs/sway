use crate::{
    error::CompileError,
    language::Literal,
    type_system::{to_typeinfo, TypeId, TypeInfo},
};

use super::types::{create_enum_aggregate, create_tuple_aggregate};

use sway_ir::{Aggregate, Constant, Context, Type, Value};
use sway_types::span::Span;

pub(super) fn convert_literal_to_value(context: &mut Context, ast_literal: &Literal) -> Value {
    match ast_literal {
        // In Sway for now we don't have `as` casting and for integers which may be implicitly cast
        // between widths we just emit a warning, and essentially ignore it.  We also assume a
        // 'Numeric' integer of undetermined width is 'u64`.  The IR would like to be type
        // consistent and doesn't tolerate mising integers of different width, so for now, until we
        // do introduce explicit `as` casting, all integers are `u64` as far as the IR is
        // concerned.
        Literal::U8(n) => Constant::get_uint(context, 64, *n as u64),
        Literal::U16(n) => Constant::get_uint(context, 64, *n as u64),
        Literal::U32(n) => Constant::get_uint(context, 64, *n as u64),
        Literal::U64(n) => Constant::get_uint(context, 64, *n),
        Literal::Numeric(n) => Constant::get_uint(context, 64, *n),
        Literal::String(s) => Constant::get_string(context, s.as_str().as_bytes().to_vec()),
        Literal::Boolean(b) => Constant::get_bool(context, *b),
        Literal::B256(bs) => Constant::get_b256(context, *bs),
    }
}

pub(super) fn convert_literal_to_constant(ast_literal: &Literal) -> Constant {
    match ast_literal {
        // All integers are `u64`.  See comment above.
        Literal::U8(n) => Constant::new_uint(64, *n as u64),
        Literal::U16(n) => Constant::new_uint(64, *n as u64),
        Literal::U32(n) => Constant::new_uint(64, *n as u64),
        Literal::U64(n) => Constant::new_uint(64, *n),
        Literal::Numeric(n) => Constant::new_uint(64, *n),
        Literal::String(s) => Constant::new_string(s.as_str().as_bytes().to_vec()),
        Literal::Boolean(b) => Constant::new_bool(*b),
        Literal::B256(bs) => Constant::new_b256(*bs),
    }
}

pub(super) fn convert_resolved_typeid(
    context: &mut Context,
    ast_type: &TypeId,
    span: &Span,
) -> Result<Type, CompileError> {
    // There's probably a better way to convert TypeError to String, but... we'll use something
    // other than String eventually?  IrError?
    convert_resolved_type(
        context,
        &to_typeinfo(*ast_type, span)
            .map_err(|ty_err| CompileError::InternalOwned(format!("{ty_err:?}"), span.clone()))?,
        span,
    )
}

pub(super) fn convert_resolved_typeid_no_span(
    context: &mut Context,
    ast_type: &TypeId,
) -> Result<Type, CompileError> {
    let msg = "unknown source location";
    let span = crate::span::Span::from_string(msg.to_string());
    convert_resolved_typeid(context, ast_type, &span)
}

fn convert_resolved_type(
    context: &mut Context,
    ast_type: &TypeInfo,
    span: &Span,
) -> Result<Type, CompileError> {
    // A handy macro for rejecting unsupported types.
    macro_rules! reject_type {
        ($name_str:literal) => {{
            return Err(CompileError::Internal(
                concat!($name_str, " type cannot be resolved in IR."),
                span.clone(),
            ));
        }};
    }

    Ok(match ast_type {
        // All integers are `u64`, see comment in convert_literal_to_value() above.
        TypeInfo::UnsignedInteger(_) => Type::Uint(64),
        TypeInfo::Numeric => Type::Uint(64),
        TypeInfo::Boolean => Type::Bool,
        TypeInfo::B256 => Type::B256,
        TypeInfo::Str(n) => Type::String(*n),
        TypeInfo::Struct { fields, .. } => super::types::get_aggregate_for_types(
            context,
            fields
                .iter()
                .map(|field| field.type_id)
                .collect::<Vec<_>>()
                .as_slice(),
        )
        .map(&Type::Struct)?,
        TypeInfo::Enum { variant_types, .. } => {
            create_enum_aggregate(context, variant_types.clone()).map(&Type::Struct)?
        }
        TypeInfo::Array(elem_type_id, count, _) => {
            let elem_type = convert_resolved_typeid(context, elem_type_id, span)?;
            Type::Array(Aggregate::new_array(context, elem_type, *count as u64))
        }
        TypeInfo::Tuple(fields) => {
            if fields.is_empty() {
                // XXX We've removed Unit from the core compiler, replaced with an empty Tuple.
                // Perhaps the same should be done for the IR, although it would use an empty
                // aggregate which might not make as much sense as a dedicated Unit type.
                Type::Unit
            } else {
                let new_fields = fields.iter().map(|x| x.type_id).collect();
                create_tuple_aggregate(context, new_fields).map(Type::Struct)?
            }
        }

        // Unsupported types which shouldn't exist in the AST after type checking and
        // monomorphisation.
        TypeInfo::Custom { .. } => reject_type!("Custom"),
        TypeInfo::SelfType { .. } => reject_type!("Self"),
        TypeInfo::Contract => reject_type!("Contract"),
        TypeInfo::ContractCaller { .. } => reject_type!("ContractCaller"),
        TypeInfo::Unknown => reject_type!("Unknown"),
        TypeInfo::UnknownGeneric { .. } => reject_type!("Generic"),
        TypeInfo::ErrorRecovery => reject_type!("Error recovery"),
        TypeInfo::Storage { .. } => reject_type!("Storage"),
    })
}
