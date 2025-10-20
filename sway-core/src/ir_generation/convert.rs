use crate::{
    ir_generation::function::FnCompiler,
    language::Literal,
    metadata::MetadataManager,
    type_system::{TypeId, TypeInfo},
    Engines,
};

use super::types::{create_tagged_union_type, create_tuple_aggregate};

use sway_error::error::CompileError;
use sway_ir::{module::Module, Constant, ConstantContent, Context, Type, Value};
use sway_types::{integer_bits::IntegerBits, span::Span};

pub(super) fn convert_literal_to_value(context: &mut Context, ast_literal: &Literal) -> Value {
    match ast_literal {
        // In Sway for now we don't have `as` casting and for integers which may be implicitly cast
        // between widths we just emit a warning, and essentially ignore it. We also assume a
        // 'Numeric' integer of undetermined width is 'u64`. The IR would like to be type
        // consistent and doesn't tolerate missing integers of different width, so for now, until we
        // do introduce explicit `as` casting, all integers are `u64` as far as the IR is
        // concerned.
        //
        // XXX The above isn't true for other targets.  We need to improved this.
        // FIXME
        Literal::U8(n) => ConstantContent::get_uint(context, 8, *n as u64),
        Literal::U16(n) => ConstantContent::get_uint(context, 64, *n as u64),
        Literal::U32(n) => ConstantContent::get_uint(context, 64, *n as u64),
        Literal::U64(n) => ConstantContent::get_uint(context, 64, *n),
        Literal::U256(n) => ConstantContent::get_uint256(context, n.clone()),
        Literal::Numeric(_) => unreachable!(),
        Literal::String(s) => ConstantContent::get_string(context, s.as_str().as_bytes().to_vec()),
        Literal::Boolean(b) => ConstantContent::get_bool(context, *b),
        Literal::B256(bs) => ConstantContent::get_b256(context, *bs),
    }
}

pub(super) fn convert_literal_to_constant(
    context: &mut Context,
    ast_literal: &Literal,
) -> Constant {
    let c = match ast_literal {
        // All integers are `u64`.  See comment above.
        Literal::U8(n) => ConstantContent::new_uint(context, 8, *n as u64),
        Literal::U16(n) => ConstantContent::new_uint(context, 64, *n as u64),
        Literal::U32(n) => ConstantContent::new_uint(context, 64, *n as u64),
        Literal::U64(n) => ConstantContent::new_uint(context, 64, *n),
        Literal::U256(n) => ConstantContent::new_uint256(context, n.clone()),
        Literal::Numeric(_) => unreachable!(),
        Literal::String(s) => ConstantContent::new_string(context, s.as_str().as_bytes().to_vec()),
        Literal::Boolean(b) => ConstantContent::new_bool(context, *b),
        Literal::B256(bs) => ConstantContent::new_b256(context, *bs),
    };
    Constant::unique(context, c)
}

pub(super) fn convert_resolved_type_id(
    engines: &Engines,
    context: &mut Context,
    md_mgr: &mut MetadataManager,
    module: Module,
    function_compiler: Option<&FnCompiler>,
    ast_type: TypeId,
    span: &Span,
) -> Result<Type, CompileError> {
    let ast_type = engines.te().get(ast_type);
    convert_resolved_type_info(
        engines,
        context,
        md_mgr,
        module,
        function_compiler,
        &ast_type,
        span,
    )
}

pub(super) fn convert_resolved_typeid_no_span(
    engines: &Engines,
    context: &mut Context,
    md_mgr: &mut MetadataManager,
    module: Module,
    function_compiler: Option<&FnCompiler>,
    ast_type: TypeId,
) -> Result<Type, CompileError> {
    let msg = "unknown source location";
    let span = crate::span::Span::from_string(msg.to_string());
    convert_resolved_type_id(
        engines,
        context,
        md_mgr,
        module,
        function_compiler,
        ast_type,
        &span,
    )
}

fn convert_resolved_type_info(
    engines: &Engines,
    context: &mut Context,
    md_mgr: &mut MetadataManager,
    module: Module,
    function_compiler: Option<&FnCompiler>,
    ast_type: &TypeInfo,
    span: &Span,
) -> Result<Type, CompileError> {
    // A handy macro for rejecting unsupported types.
    macro_rules! reject_type {
        ($name_str:literal) => {{
            return Err(CompileError::TypeMustBeKnownAtThisPoint {
                span: span.clone(),
                internal: $name_str.into(),
            });
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
        TypeInfo::StringArray(length) if length.expr().as_literal_val().is_some() => {
            Type::new_string_array(context, length.expr().as_literal_val().unwrap() as u64)
        }
        TypeInfo::Struct(decl_ref) => super::types::get_struct_for_types(
            engines,
            context,
            md_mgr,
            module,
            engines
                .de()
                .get_struct(decl_ref)
                .fields
                .iter()
                .map(|field| field.type_argument.type_id)
                .collect::<Vec<_>>()
                .as_slice(),
        )?,
        TypeInfo::Enum(decl_ref) => create_tagged_union_type(
            engines,
            context,
            md_mgr,
            module,
            &engines.de().get_enum(decl_ref).variants,
        )?,
        TypeInfo::Array(elem_type, length) => {
            let const_expr = length.expr().to_ty_expression(engines);

            let constant_evaluated =
                crate::ir_generation::const_eval::compile_constant_expression_to_constant(
                    engines,
                    context,
                    md_mgr,
                    module,
                    None,
                    function_compiler,
                    &const_expr,
                )
                .unwrap();
            let len = constant_evaluated.get_content(context).as_uint().unwrap();

            let elem_type = convert_resolved_type_id(
                engines,
                context,
                md_mgr,
                module,
                function_compiler,
                elem_type.type_id,
                span,
            )?;
            Type::new_array(context, elem_type, len as u64)
        }

        TypeInfo::Tuple(fields) => {
            if fields.is_empty() {
                // XXX We've removed Unit from the core compiler, replaced with an empty Tuple.
                // Perhaps the same should be done for the IR, although it would use an empty
                // aggregate which might not make as much sense as a dedicated Unit type.
                Type::get_unit(context)
            } else {
                let new_fields: Vec<_> = fields.iter().map(|x| x.type_id).collect();
                create_tuple_aggregate(engines, context, md_mgr, module, &new_fields)?
            }
        }
        TypeInfo::RawUntypedPtr => Type::get_ptr(context),
        TypeInfo::RawUntypedSlice => Type::get_slice(context),
        TypeInfo::Ptr(pointee_ty) => {
            let pointee_ty = convert_resolved_type_id(
                engines,
                context,
                md_mgr,
                module,
                function_compiler,
                pointee_ty.type_id,
                span,
            )?;
            Type::new_typed_pointer(context, pointee_ty)
        }
        TypeInfo::Alias { ty, .. } => convert_resolved_type_id(
            engines,
            context,
            md_mgr,
            module,
            function_compiler,
            ty.type_id,
            span,
        )?,
        // refs to slice are actually fat pointers,
        // all others refs are thin pointers.
        TypeInfo::Ref {
            referenced_type, ..
        } => {
            if let Some(slice_elem) = engines.te().get(referenced_type.type_id).as_slice() {
                let elem_ir_type = convert_resolved_type_id(
                    engines,
                    context,
                    md_mgr,
                    module,
                    function_compiler,
                    slice_elem.type_id,
                    span,
                )?;
                Type::get_typed_slice(context, elem_ir_type)
            } else {
                let referenced_ir_type = convert_resolved_type_id(
                    engines,
                    context,
                    md_mgr,
                    module,
                    function_compiler,
                    referenced_type.type_id,
                    span,
                )?;
                Type::new_typed_pointer(context, referenced_ir_type)
            }
        }
        TypeInfo::Never => Type::get_never(context),

        // Unsized types
        TypeInfo::Slice(_) => reject_type!("unsized"),

        // Unsupported types which shouldn't exist in the AST after type checking and
        // monomorphisation.
        TypeInfo::Custom { .. } => reject_type!("Custom"),
        TypeInfo::Contract => reject_type!("Contract"),
        TypeInfo::ContractCaller { .. } => reject_type!("ContractCaller"),
        TypeInfo::UntypedEnum(_) => reject_type!("UntypedEnum"),
        TypeInfo::UntypedStruct(_) => reject_type!("UntypedStruct"),
        TypeInfo::Unknown => reject_type!("Unknown"),
        TypeInfo::UnknownGeneric { .. } => reject_type!("Generic"),
        TypeInfo::Placeholder(_) => reject_type!("Placeholder"),
        TypeInfo::TypeParam(_) => reject_type!("TypeParam"),
        TypeInfo::ErrorRecovery(_) => reject_type!("Error recovery"),
        TypeInfo::TraitType { .. } => reject_type!("TraitType"),
        TypeInfo::StringArray(..) => reject_type!("String Array with non literal length"),
    })
}
