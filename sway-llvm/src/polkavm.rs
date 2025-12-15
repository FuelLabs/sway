use super::{LlvmError, ModuleLowerer, Result, TargetVm};
use inkwell::{
    types::{AnyType, AnyTypeEnum, BasicMetadataTypeEnum, BasicType},
    values::{BasicMetadataValueEnum, BasicValue, BasicValueEnum, PointerValue, ValueKind},
    AddressSpace,
};
use sway_ir::{irtype::TypeContent, value::Value};

impl<'ctx, 'ir, 'eng> ModuleLowerer<'ctx, 'ir, 'eng> {
    pub(super) fn lower_polkavm_log(
        &mut self,
        log_val: Value,
        log_ty: sway_ir::irtype::Type,
        log_id: Value,
        log_data: Option<sway_ir::instruction::LogEventData>,
    ) -> Result<()> {
        if self.opts.target_vm != TargetVm::PolkaVm {
            return Err(super::LlvmError::Lowering(
                "FuelVM log intrinsic is not supported for this backend".into(),
            ));
        }

        let i64_ty = self.llvm.custom_width_int_type(64);
        let metadata = log_data.map(|data| data.encoded()).unwrap_or_default();
        let metadata_val = i64_ty.const_int(metadata, false);
        let log_id_basic = self.get_basic_value(log_id)?;
        let log_id_val = self.ensure_int_value(log_id_basic)?;
        let log_basic = self.get_basic_value(log_val)?;

        let is_slice_ty = matches!(
            log_ty.get_content(self.ir),
            TypeContent::Slice | TypeContent::TypedSlice(_) | TypeContent::StringSlice
        );

        if log_ty.is_ptr(self.ir) {
            let (ptr_val, len_val) = self.lower_polkavm_log_pointer(log_basic, log_ty)?;
            let args = vec![
                metadata_val.as_basic_value_enum(),
                log_id_val.as_basic_value_enum(),
                ptr_val,
                len_val,
            ]
            .into_iter()
            .map(BasicMetadataValueEnum::from)
            .collect::<Vec<_>>();
            let function = self.ensure_polkavm_import(
                "logd",
                None,
                &[i64_ty.into(), i64_ty.into(), i64_ty.into(), i64_ty.into()],
            )?;
            self.handle_builder_result(self.builder.build_call(function, &args, "call_logd"))?;
        } else if is_slice_ty {
            let (ptr_val, len_val) = self.lower_polkavm_log_slice_value(log_basic)?;
            let args = vec![
                metadata_val.as_basic_value_enum(),
                log_id_val.as_basic_value_enum(),
                ptr_val,
                len_val,
            ]
            .into_iter()
            .map(BasicMetadataValueEnum::from)
            .collect::<Vec<_>>();
            let function = self.ensure_polkavm_import(
                "logd",
                None,
                &[i64_ty.into(), i64_ty.into(), i64_ty.into(), i64_ty.into()],
            )?;
            self.handle_builder_result(self.builder.build_call(function, &args, "call_logd"))?;
        } else {
            let value = self.ensure_int_value(log_basic)?;
            let zero = i64_ty.const_zero();
            let args = vec![
                value.as_basic_value_enum(),
                log_id_val.as_basic_value_enum(),
                zero.as_basic_value_enum(),
                zero.as_basic_value_enum(),
            ]
            .into_iter()
            .map(BasicMetadataValueEnum::from)
            .collect::<Vec<_>>();
            let function = self.ensure_polkavm_import(
                "log",
                None,
                &[i64_ty.into(), i64_ty.into(), i64_ty.into(), i64_ty.into()],
            )?;
            self.handle_builder_result(self.builder.build_call(function, &args, "call_log"))?;
        }

        Ok(())
    }

    fn lower_polkavm_log_pointer(
        &mut self,
        log_basic: BasicValueEnum<'ctx>,
        log_ty: sway_ir::irtype::Type,
    ) -> Result<(BasicValueEnum<'ctx>, BasicValueEnum<'ctx>)> {
        let log_ptr = self.to_pointer_value(log_basic)?;
        if let Some(pointee_ty) = log_ty.get_pointee_type(self.ir) {
            let size_in_bytes = pointee_ty.size(self.ir).in_bytes();
            let size_const = self
                .llvm
                .custom_width_int_type(64)
                .const_int(size_in_bytes, false)
                .as_basic_value_enum();
            return self.lower_polkavm_log_data_ptr_with_len(log_ptr, size_const);
        }
        Err(super::LlvmError::Lowering(
            "pointer log has no pointee type".into(),
        ))
    }

    fn lower_polkavm_log_data_ptr_with_len(
        &mut self,
        ptr_value: PointerValue<'ctx>,
        len_value: BasicValueEnum<'ctx>,
    ) -> Result<(BasicValueEnum<'ctx>, BasicValueEnum<'ctx>)> {
        let ptr_int_ty = self.llvm.ptr_sized_int_type(&self.target_data, None);
        let data_int = self.handle_builder_result(self.builder.build_ptr_to_int(
            ptr_value,
            ptr_int_ty,
            "log_data_ptr_int",
        ))?;
        let len_int = self.ensure_int_value(len_value)?;
        Ok((
            data_int.as_basic_value_enum(),
            len_int.as_basic_value_enum(),
        ))
    }

    fn lower_polkavm_log_slice_value(
        &mut self,
        slice_val: BasicValueEnum<'ctx>,
    ) -> Result<(BasicValueEnum<'ctx>, BasicValueEnum<'ctx>)> {
        let struct_val = slice_val.into_struct_value();
        let data_field = self.handle_builder_result(self.builder.build_extract_value(
            struct_val,
            0,
            "slice_data_ptr",
        ))?;
        let len_field = self.handle_builder_result(self.builder.build_extract_value(
            struct_val,
            1,
            "slice_len",
        ))?;
        let data_ptr = data_field.into_pointer_value();
        self.lower_polkavm_log_data_ptr_with_len(data_ptr, len_field)
    }

    pub(super) fn lower_polkavm_revert(&mut self, val: Value) -> Result<()> {
        if self.opts.target_vm != TargetVm::PolkaVm {
            return Err(super::LlvmError::Lowering(
                "FuelVM revert intrinsic is not supported for this backend".into(),
            ));
        }
        let i64_ty = self.llvm.custom_width_int_type(64);
        let arg_val = {
            let basic = self.get_basic_value(val)?;
            self.ensure_int_value(basic)?
        };
        let func = self.ensure_polkavm_import("revert", None, &[i64_ty.into()])?;
        self.handle_builder_result(self.builder.build_call(
            func,
            &[arg_val.into()],
            "call_revert",
        ))?;
        // Revert never returns; terminate the block to keep the IR well-formed.
        let _ = self.builder.build_unreachable();
        self.current_block = None;
        Ok(())
    }

    pub(super) fn lower_polkavm_read_register(
        &mut self,
        reg: sway_ir::instruction::Register,
        inst_value: Value,
    ) -> Result<()> {
        if self.opts.target_vm != TargetVm::PolkaVm {
            return Err(super::LlvmError::Lowering(
                "FuelVM read_register intrinsic is not supported for this backend".into(),
            ));
        }
        let i64_ty = self.llvm.custom_width_int_type(64);
        let reg_id = match reg {
            sway_ir::instruction::Register::Of => 0u64,
            sway_ir::instruction::Register::Pc => 1,
            sway_ir::instruction::Register::Ssp => 2,
            sway_ir::instruction::Register::Sp => 3,
            sway_ir::instruction::Register::Fp => 4,
            sway_ir::instruction::Register::Hp => 5,
            sway_ir::instruction::Register::Error => 6,
            sway_ir::instruction::Register::Ggas => 7,
            sway_ir::instruction::Register::Cgas => 8,
            sway_ir::instruction::Register::Bal => 9,
            sway_ir::instruction::Register::Is => 10,
            sway_ir::instruction::Register::Ret => 11,
            sway_ir::instruction::Register::Retl => 12,
            sway_ir::instruction::Register::Flag => 13,
        };
        let reg_val = i64_ty.const_int(reg_id, false);
        let func =
            self.ensure_polkavm_import("read_register", Some(i64_ty.into()), &[i64_ty.into()])?;
        let call = self.handle_builder_result(self.builder.build_call(
            func,
            &[reg_val.into()],
            "call_rr",
        ))?;
        let ret = match call.try_as_basic_value() {
            ValueKind::Basic(val) => val,
            _ => {
                return Err(super::LlvmError::Lowering(
                    "read_register did not return value".into(),
                ))
            }
        };
        self.value_map.insert(inst_value, ret);
        Ok(())
    }

    fn metadata_to_any_type(ty: BasicMetadataTypeEnum<'ctx>) -> AnyTypeEnum<'ctx> {
        match ty {
            BasicMetadataTypeEnum::ArrayType(t) => t.as_any_type_enum(),
            BasicMetadataTypeEnum::FloatType(t) => t.as_any_type_enum(),
            BasicMetadataTypeEnum::IntType(t) => t.as_any_type_enum(),
            BasicMetadataTypeEnum::PointerType(t) => t.as_any_type_enum(),
            BasicMetadataTypeEnum::StructType(t) => t.as_any_type_enum(),
            BasicMetadataTypeEnum::VectorType(t) => t.as_any_type_enum(),
            BasicMetadataTypeEnum::ScalableVectorType(t) => t.as_any_type_enum(),
            BasicMetadataTypeEnum::MetadataType(_) => {
                panic!("metadata type is not supported for polkavm imports")
            }
        }
    }

    pub(super) fn ensure_polkavm_import(
        &mut self,
        name: &str,
        ret: Option<inkwell::types::BasicTypeEnum<'ctx>>,
        args: &[BasicMetadataTypeEnum<'ctx>],
    ) -> Result<inkwell::values::FunctionValue<'ctx>> {
        if let Some(func) = self.polkavm_imports.get(name) {
            let fn_ty = func.get_type();
            let expected_ret = ret.map(|t| t.as_any_type_enum());
            let actual_ret = fn_ty.get_return_type().map(|t| t.as_any_type_enum());
            let actual_params = fn_ty.get_param_types();
            let actual_any: Vec<_> = actual_params
                .iter()
                .map(|t| Self::metadata_to_any_type(*t))
                .collect();
            let expected_any: Vec<_> = args
                .iter()
                .map(|t| Self::metadata_to_any_type(*t))
                .collect();
            if expected_ret != actual_ret
                || actual_any.len() != expected_any.len()
                || actual_any
                    .iter()
                    .zip(expected_any.iter())
                    .any(|(a, b)| a != b)
            {
                return Err(LlvmError::Lowering(format!(
                    "polkavm import `{name}` requested with conflicting signature"
                )));
            }
            return Ok(*func);
        }

        let import_ty = match ret {
            Some(ret_ty) => ret_ty.fn_type(args, false),
            None => self.llvm.void_type().fn_type(args, false),
        };
        let function = self.llvm_module.add_function(name, import_ty, None);
        let metadata_symbol = self.emit_polkavm_import_metadata(name, args.len() as u8, 0)?;

        let asm = format!(
            ".section .text.polkavm_import,\"ax\"\n\
             .globl {name}\n\
             .align 2\n\
             .type {name}, @function\n\
             {name}:\n\
             .insn r 0xb, 0, 0, zero, zero, zero\n\
             auipc zero, %pcrel_hi({metadata_sym})\n\
             ret\n",
            name = name,
            metadata_sym = metadata_symbol,
        );
        self.append_module_inline_asm(&asm);

        self.polkavm_imports.insert(name.to_string(), function);
        Ok(function)
    }

    fn emit_polkavm_import_metadata(
        &mut self,
        name: &str,
        input_regs: u8,
        output_regs: u8,
    ) -> Result<String> {
        let i8 = self.llvm.i8_type();
        let ptr_ty = self.llvm.ptr_type(AddressSpace::default());

        let metadata_symbol = format!("polkavm_import_metadata_{name}");
        let metadata_symbol_symbol = format!("polkavm_import_metadata_symbol_{name}");
        let name_symbol = format!("polkavm_import_name_{name}");
        let name_bytes = format!("{name}\0");
        let name_array_ty = i8.array_type(name_bytes.len() as u32);
        let name_global = self
            .llvm_module
            .add_global(name_array_ty, None, &name_symbol);
        let name_inits: Vec<_> = name_bytes
            .as_bytes()
            .iter()
            .map(|b| i8.const_int(*b as u64, false))
            .collect();
        name_global.set_initializer(&i8.const_array(&name_inits));
        name_global.set_constant(true);
        name_global.set_alignment(1);
        name_global.set_unnamed_address(inkwell::values::UnnamedAddress::Global);

        // Mirror rustc's polkavm_import! layout:
        // - A helper record `{ ptr name, u32 name_len }` in `.polkavm_metadata`, align 4.
        // - A main record `{ u8 kind=2 (import), u32 flags=0, u32 name_len, ptr name, u8 in_regs, u8 out_regs, [5] pad }`
        //   materialized as <{ [9 x i8], ptr, [7 x i8] }> in `.polkavm_metadata`, align 1.
        // - The trampoline carries a `%pcrel_hi` relocation to the main record.
        let meta_symbol_ty = self.llvm.struct_type(
            &[ptr_ty.as_basic_type_enum(), i8.array_type(4).into()],
            false,
        );
        let name_ptr = name_global.as_pointer_value();
        let cast_name = name_ptr.const_cast(ptr_ty);
        let name_len = (name_bytes.len() - 1) as u64;
        let len_bytes = [
            (name_len & 0xFF) as u8,
            ((name_len >> 8) & 0xFF) as u8,
            ((name_len >> 16) & 0xFF) as u8,
            ((name_len >> 24) & 0xFF) as u8,
        ]
        .map(|b| i8.const_int(b as u64, false));
        let meta_symbol_vals: Vec<BasicValueEnum> = vec![
            cast_name.as_basic_value_enum(),
            i8.const_array(&len_bytes).as_basic_value_enum(),
        ];
        let metadata_symbol_global =
            self.llvm_module
                .add_global(meta_symbol_ty, None, &metadata_symbol_symbol);
        metadata_symbol_global
            .set_initializer(&meta_symbol_ty.const_named_struct(&meta_symbol_vals));
        metadata_symbol_global.set_constant(true);
        metadata_symbol_global.set_alignment(4);
        metadata_symbol_global.set_section(Some(".polkavm_metadata"));
        metadata_symbol_global.set_unnamed_address(inkwell::values::UnnamedAddress::Global);

        let header_bytes = [
            i8.const_int(2, false), // kind = import
            i8.const_zero(),
            i8.const_zero(),
            i8.const_zero(),
            i8.const_zero(), // flags = 0 (le u32)
            len_bytes[0],
            len_bytes[1],
            len_bytes[2],
            len_bytes[3], // name_len (le u32)
        ];
        let trailer_bytes = [
            i8.const_int(input_regs as u64, false),
            i8.const_int(output_regs as u64, false),
            i8.const_zero(),
            i8.const_zero(),
            i8.const_zero(),
            i8.const_zero(),
            i8.const_zero(),
        ];
        let metadata_ty = self.llvm.struct_type(
            &[
                i8.array_type(9).into(),
                ptr_ty.as_basic_type_enum(),
                i8.array_type(7).into(),
            ],
            true,
        );
        let metadata_vals: Vec<BasicValueEnum> = vec![
            i8.const_array(&header_bytes).as_basic_value_enum(),
            cast_name.as_basic_value_enum(),
            i8.const_array(&trailer_bytes).as_basic_value_enum(),
        ];
        let metadata_global = self
            .llvm_module
            .add_global(metadata_ty, None, &metadata_symbol);
        metadata_global.set_initializer(&metadata_ty.const_named_struct(&metadata_vals));
        metadata_global.set_constant(true);
        metadata_global.set_alignment(1);
        metadata_global.set_section(Some(".polkavm_metadata"));
        metadata_global.set_unnamed_address(inkwell::values::UnnamedAddress::Global);

        Ok(metadata_symbol)
    }
}
