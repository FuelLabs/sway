use crate::{
    decl_engine::DeclEngine,
    language::Literal,
    type_system::{TypeId, TypeInfo},
    TypeEngine,
};

use super::types::{create_tagged_union_type, create_tuple_aggregate};

use sway_error::error::CompileError;
use sway_ir::{Constant, Context, Type, Value};
use sway_types::{integer_bits::IntegerBits, span::Span};

pub(super) fn convert_literal_to_value(context: &mut Context, ast_literal: &Literal) -> Value {
    match ast_literal {
        // In Sway for now we don't have `as` casting and for integers which may be implicitly cast
        // between widths we just emit a warning, and essentially ignore it.  We also assume a
        // 'Numeric' integer of undetermined width is 'u64`.  The IR would like to be type
        // consistent and doesn't tolerate mising integers of different width, so for now, until we
        // do introduce explicit `as` casting, all integers are `u64` as far as the IR is
        // concerned.
        //
        // XXX The above isn't true for other targets.  We need to improved this.
        // FIXME
        Literal::U8(n) => Constant::get_uint(context, 8, *n as u64),
        Literal::U16(n) => Constant::get_uint(context, 64, *n as u64),
        Literal::U32(n) => Constant::get_uint(context, 64, *n as u64),
        Literal::U64(n) => Constant::get_uint(context, 64, *n),
        Literal::U256(n) => Constant::get_uint256(context, n.clone()),
        Literal::Numeric(_) => unreachable!(),
        Literal::String(s) => Constant::get_string(context, s.as_str().as_bytes().to_vec()),
        Literal::Boolean(b) => Constant::get_bool(context, *b),
        Literal::B256(bs) => Constant::get_b256(context, *bs),
    }
}

pub(super) fn convert_literal_to_constant(
    context: &mut Context,
    ast_literal: &Literal,
) -> Constant {
    match ast_literal {
        // All integers are `u64`.  See comment above.
        Literal::U8(n) => Constant::new_uint(context, 8, *n as u64),
        Literal::U16(n) => Constant::new_uint(context, 64, *n as u64),
        Literal::U32(n) => Constant::new_uint(context, 64, *n as u64),
        Literal::U64(n) => Constant::new_uint(context, 64, *n),
        Literal::U256(n) => Constant::new_uint256(context, n.clone()),
        Literal::Numeric(_) => unreachable!(),
        Literal::String(s) => Constant::new_string(context, s.as_str().as_bytes().to_vec()),
        Literal::Boolean(b) => Constant::new_bool(context, *b),
        Literal::B256(bs) => Constant::new_b256(context, *bs),
    }
}

pub(super) fn convert_resolved_typeid(
    type_engine: &TypeEngine,
    decl_engine: &DeclEngine,
    context: &mut Context,
    ast_type: &TypeId,
    span: &Span,
) -> Result<Type, CompileError> {
    // There's probably a better way to convert TypeError to String, but... we'll use something
    // other than String eventually?  IrError?
    convert_resolved_type(
        type_engine,
        decl_engine,
        context,
        &type_engine
            .to_typeinfo(*ast_type, span)
            .map_err(|ty_err| CompileError::InternalOwned(format!("{ty_err:?}"), span.clone()))?,
        span,
    )
}

pub(super) fn convert_resolved_typeid_no_span(
    type_engine: &TypeEngine,
    decl_engine: &DeclEngine,
    context: &mut Context,
    ast_type: &TypeId,
) -> Result<Type, CompileError> {
    let msg = "unknown source location";
    let span = crate::span::Span::from_string(msg.to_string());
    convert_resolved_typeid(type_engine, decl_engine, context, ast_type, &span)
}

fn convert_resolved_type(
    type_engine: &TypeEngine,
    decl_engine: &DeclEngine,
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
        // See comment in convert_literal_to_value() above.
        TypeInfo::UnsignedInteger(IntegerBits::V256) => Type::get_uint256(context),
        TypeInfo::UnsignedInteger(IntegerBits::Eight) => Type::get_uint8(context),
        TypeInfo::UnsignedInteger(IntegerBits::Sixteen)
        | TypeInfo::UnsignedInteger(IntegerBits::ThirtyTwo)
        | TypeInfo::UnsignedInteger(IntegerBits::SixtyFour)
        | TypeInfo::Numeric => Type::get_uint64(context),
        TypeInfo::Boolean => Type::get_bool(context),
        TypeInfo::B256 => Type::get_b256(context),
        TypeInfo::StringSlice => Type::get_slice(context),
        TypeInfo::StringArray(n) => Type::new_string_array(context, n.val() as u64),
        TypeInfo::Struct(decl_ref) => super::types::get_struct_for_types(
            type_engine,
            decl_engine,
            context,
            decl_engine
                .get_struct(decl_ref)
                .fields
                .iter()
                .map(|field| field.type_argument.type_id)
                .collect::<Vec<_>>()
                .as_slice(),
        )?,
        TypeInfo::Enum(decl_ref) => create_tagged_union_type(
            type_engine,
            decl_engine,
            context,
            &decl_engine.get_enum(decl_ref).variants,
        )?,
        TypeInfo::Array(elem_type, length) => {
            let elem_type = convert_resolved_typeid(
                type_engine,
                decl_engine,
                context,
                &elem_type.type_id,
                span,
            )?;
            Type::new_array(context, elem_type, length.val() as u64)
        }
        TypeInfo::Tuple(fields) => {
            if fields.is_empty() {
                // XXX We've removed Unit from the core compiler, replaced with an empty Tuple.
                // Perhaps the same should be done for the IR, although it would use an empty
                // aggregate which might not make as much sense as a dedicated Unit type.
                Type::get_unit(context)
            } else {
                let new_fields = fields.iter().map(|x| x.type_id).collect();
                create_tuple_aggregate(type_engine, decl_engine, context, new_fields)?
            }
        }
        TypeInfo::RawUntypedPtr => Type::get_uint64(context),
        TypeInfo::RawUntypedSlice => Type::get_slice(context),
        TypeInfo::Ptr(_) => Type::get_uint64(context),
        TypeInfo::Slice(_) => Type::get_slice(context),
        TypeInfo::Alias { ty, .. } => {
            convert_resolved_typeid(type_engine, decl_engine, context, &ty.type_id, span)?
        }

        // Unsupported types which shouldn't exist in the AST after type checking and
        // monomorphisation.
        TypeInfo::Custom { .. } => reject_type!("Custom"),
        TypeInfo::Contract => reject_type!("Contract"),
        TypeInfo::ContractCaller { .. } => reject_type!("ContractCaller"),
        TypeInfo::Unknown => reject_type!("Unknown"),
        TypeInfo::UnknownGeneric { .. } => reject_type!("Generic"),
        TypeInfo::Placeholder(_) => reject_type!("Placeholder"),
        TypeInfo::TypeParam(_) => reject_type!("TypeParam"),
        TypeInfo::ErrorRecovery(_) => reject_type!("Error recovery"),
        TypeInfo::Storage { .. } => reject_type!("Storage"),
        TypeInfo::TraitType { .. } => reject_type!("TraitType"),
    })
}
