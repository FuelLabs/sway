//! Code to validate the IR in a [`Context`].
//!
//! During creation, deserialization and optimization the IR should be verified to be in a
//! consistent valid state, using the functions in this module.

use itertools::Itertools;

use crate::{
    context::Context,
    error::IrError,
    function::Function,
    instruction::{FuelVmInstruction, InstOp, Predicate},
    irtype::Type,
    metadata::{MetadataIndex, Metadatum},
    printer,
    value::{Value, ValueDatum},
    variable::LocalVar,
    AnalysisResult, AnalysisResultT, AnalysisResults, BinaryOpKind, Block, BlockArgument,
    BranchToWithArgs, Doc, GlobalVar, LogEventData, Module, Pass, PassMutability, ScopedPass,
    StorageKey, TypeOption, UnaryOpKind,
};

pub struct ModuleVerifierResult;
impl AnalysisResultT for ModuleVerifierResult {}

/// Verify module
pub fn module_verifier(
    context: &Context,
    _analyses: &AnalysisResults,
    module: Module,
) -> Result<AnalysisResult, IrError> {
    context.verify_module(module)?;
    Ok(Box::new(ModuleVerifierResult))
}

pub const MODULE_VERIFIER_NAME: &str = "module-verifier";

pub fn create_module_verifier_pass() -> Pass {
    Pass {
        name: MODULE_VERIFIER_NAME,
        descr: "Verify module",
        deps: vec![],
        runner: ScopedPass::ModulePass(PassMutability::Analysis(module_verifier)),
    }
}

impl Context<'_> {
    /// Verify the contents of this [`Context`] is valid.
    pub fn verify(&self) -> Result<(), IrError> {
        for (module, _) in &self.modules {
            let module = Module(module);
            self.verify_module(module)?;
        }
        Ok(())
    }

    fn verify_module(&self, module: Module) -> Result<(), IrError> {
        for function in module.function_iter(self) {
            self.verify_function(module, function)?;
        }

        // Check that globals have initializers if they are not mutable.
        for global in &self.modules[module.0].global_variables {
            if !global.1.is_mutable(self) && global.1.get_initializer(self).is_none() {
                let global_name = module.lookup_global_variable_name(self, global.1);
                return Err(IrError::VerifyGlobalMissingInitializer(
                    global_name.unwrap_or_else(|| "<unknown>".to_owned()),
                ));
            }
        }
        Ok(())
    }

    fn verify_function(&self, cur_module: Module, function: Function) -> Result<(), IrError> {
        if function.get_module(self) != cur_module {
            return Err(IrError::InconsistentParent(
                function.get_name(self).into(),
                format!("Module_Index_{:?}", cur_module.0),
                format!("Module_Index_{:?}", function.get_module(self).0),
            ));
        }

        let entry_block = function.get_entry_block(self);

        if entry_block.num_predecessors(self) != 0 {
            return Err(IrError::VerifyEntryBlockHasPredecessors(
                function.get_name(self).to_string(),
                entry_block
                    .pred_iter(self)
                    .map(|block| block.get_label(self))
                    .collect(),
            ));
        }

        // Ensure that the entry block arguments are same as function arguments.
        if function.num_args(self) != entry_block.num_args(self) {
            return Err(IrError::VerifyBlockArgMalformed);
        }
        for ((_, func_arg), block_arg) in function.args_iter(self).zip(entry_block.arg_iter(self)) {
            if func_arg != block_arg {
                return Err(IrError::VerifyBlockArgMalformed);
            }
        }

        // Check that locals have initializers if they aren't mutable.
        // TODO: This check is disabled because we incorrect create
        //       immutable locals without initializers at many places.
        // for local in &self.functions[function.0].local_storage {
        //     if !local.1.is_mutable(self) && local.1.get_initializer(self).is_none() {
        //         return Err(IrError::VerifyLocalMissingInitializer(
        //             local.0.to_string(),
        //             function.get_name(self).to_string(),
        //         ));
        //     }
        // }

        for block in function.block_iter(self) {
            self.verify_block(cur_module, function, block)?;
        }
        self.verify_metadata(function.get_metadata(self))?;
        Ok(())
    }

    fn verify_block(
        &self,
        cur_module: Module,
        cur_function: Function,
        cur_block: Block,
    ) -> Result<(), IrError> {
        if cur_block.get_function(self) != cur_function {
            return Err(IrError::InconsistentParent(
                cur_block.get_label(self),
                cur_function.get_name(self).into(),
                cur_block.get_function(self).get_name(self).into(),
            ));
        }

        if cur_block.num_instructions(self) <= 1 && cur_block.num_predecessors(self) == 0 {
            // Empty unreferenced blocks are a harmless artefact.
            return Ok(());
        }

        for (arg_idx, arg_val) in cur_block.arg_iter(self).enumerate() {
            match self.values[arg_val.0].value {
                ValueDatum::Argument(BlockArgument { idx, .. }) if idx == arg_idx => (),
                _ => return Err(IrError::VerifyBlockArgMalformed),
            }
        }

        let r = InstructionVerifier {
            context: self,
            cur_module,
            cur_function,
            cur_block,
        }
        .verify_instructions();

        // Help to understand the verification failure
        // If the error knows the problematic value, prints everything with the error highlighted,
        // if not, print only the block to help pinpoint the issue
        if let Err(error) = &r {
            println!(
                "Verification failed at {}::{}",
                cur_function.get_name(self),
                cur_block.get_label(self)
            );

            let block = if let Some(problematic_value) = error.get_problematic_value() {
                printer::context_print(self, &|current_value: &Value, doc: Doc| {
                    if *current_value == *problematic_value {
                        doc.append(Doc::text_line(format!("\x1b[0;31m^ {error}\x1b[0m")))
                    } else {
                        doc
                    }
                })
            } else {
                printer::block_print(self, cur_function, cur_block, &|_, doc| doc)
            };

            println!("{block}");
        }

        r?;

        let (last_is_term, num_terms) =
            cur_block
                .instruction_iter(self)
                .fold((false, 0), |(_, n), ins| {
                    if ins.is_terminator(self) {
                        (true, n + 1)
                    } else {
                        (false, n)
                    }
                });
        if !last_is_term {
            Err(IrError::MissingTerminator(
                cur_block.get_label(self).clone(),
            ))
        } else if num_terms != 1 {
            Err(IrError::MisplacedTerminator(
                cur_block.get_label(self).clone(),
            ))
        } else {
            Ok(())
        }
    }

    fn verify_metadata(&self, md_idx: Option<MetadataIndex>) -> Result<(), IrError> {
        // For now we check only that struct tags are valid identifiers.
        if let Some(md_idx) = md_idx {
            match &self.metadata[md_idx.0] {
                Metadatum::List(md_idcs) => {
                    for md_idx in md_idcs {
                        self.verify_metadata(Some(*md_idx))?;
                    }
                }
                Metadatum::Struct(tag, ..) => {
                    // We could import Regex to match it, but it's a simple identifier style pattern:
                    // alpha start char, alphanumeric for the rest, or underscore anywhere.
                    if tag.is_empty() {
                        return Err(IrError::InvalidMetadatum(
                            "Struct has empty tag.".to_owned(),
                        ));
                    }
                    let mut chs = tag.chars();
                    let ch0 = chs.next().unwrap();
                    if !(ch0.is_ascii_alphabetic() || ch0 == '_')
                        || chs.any(|ch| !(ch.is_ascii_alphanumeric() || ch == '_'))
                    {
                        return Err(IrError::InvalidMetadatum(format!(
                            "Invalid struct tag: '{tag}'."
                        )));
                    }
                }
                _otherwise => (),
            }
        }
        Ok(())
    }
}

struct InstructionVerifier<'a, 'eng> {
    context: &'a Context<'eng>,
    cur_module: Module,
    cur_function: Function,
    cur_block: Block,
}

impl InstructionVerifier<'_, '_> {
    fn verify_instructions(&self) -> Result<(), IrError> {
        for ins in self.cur_block.instruction_iter(self.context) {
            let value_content = &self.context.values[ins.0];
            let ValueDatum::Instruction(instruction) = &value_content.value else {
                unreachable!("The value must be an instruction, because it is retrieved via block instruction iterator.")
            };

            if instruction.parent != self.cur_block {
                return Err(IrError::InconsistentParent(
                    format!("Instr_{:?}", ins.0),
                    self.cur_block.get_label(self.context),
                    instruction.parent.get_label(self.context),
                ));
            }

            match &instruction.op {
                InstOp::AsmBlock(..) => (),
                InstOp::BitCast(value, ty) => self.verify_bitcast(value, ty)?,
                InstOp::UnaryOp { op, arg } => self.verify_unary_op(op, arg)?,
                InstOp::BinaryOp { op, arg1, arg2 } => self.verify_binary_op(op, arg1, arg2)?,
                InstOp::Branch(block) => self.verify_br(block)?,
                InstOp::Call(func, args) => self.verify_call(func, args)?,
                InstOp::CastPtr(val, ty) => self.verify_cast_ptr(val, ty)?,
                InstOp::Cmp(pred, lhs_value, rhs_value) => {
                    self.verify_cmp(pred, lhs_value, rhs_value)?
                }
                InstOp::ConditionalBranch {
                    cond_value,
                    true_block,
                    false_block,
                } => self.verify_cbr(cond_value, true_block, false_block)?,
                InstOp::ContractCall {
                    params,
                    coins,
                    asset_id,
                    gas,
                    ..
                } => self.verify_contract_call(params, coins, asset_id, gas)?,
                // XXX move the fuelvm verification into a module
                InstOp::FuelVm(fuel_vm_instr) => match fuel_vm_instr {
                    FuelVmInstruction::Gtf { index, tx_field_id } => {
                        self.verify_gtf(index, tx_field_id)?
                    }
                    FuelVmInstruction::Log {
                        log_val,
                        log_ty,
                        log_id,
                        log_data,
                    } => self.verify_log(log_val, log_ty, log_id, log_data)?,
                    FuelVmInstruction::ReadRegister(_) => (),
                    FuelVmInstruction::JmpMem => (),
                    FuelVmInstruction::Revert(val) => self.verify_revert(val)?,
                    FuelVmInstruction::Smo {
                        recipient,
                        message,
                        message_size,
                        coins,
                    } => self.verify_smo(recipient, message, message_size, coins)?,
                    FuelVmInstruction::StateClear {
                        key,
                        number_of_slots,
                    } => self.verify_state_clear(key, number_of_slots)?,
                    FuelVmInstruction::StateLoadWord(key) => self.verify_state_load_word(key)?,
                    FuelVmInstruction::StateLoadQuadWord {
                        load_val: dst_val,
                        key,
                        number_of_slots,
                    }
                    | FuelVmInstruction::StateStoreQuadWord {
                        stored_val: dst_val,
                        key,
                        number_of_slots,
                    } => self.verify_state_access_quad(dst_val, key, number_of_slots)?,
                    FuelVmInstruction::StateStoreWord {
                        stored_val: dst_val,
                        key,
                    } => self.verify_state_store_word(dst_val, key)?,
                    FuelVmInstruction::WideUnaryOp { op, result, arg } => {
                        self.verify_wide_unary_op(op, result, arg)?
                    }
                    FuelVmInstruction::WideBinaryOp {
                        op,
                        result,
                        arg1,
                        arg2,
                    } => self.verify_wide_binary_op(op, result, arg1, arg2)?,
                    FuelVmInstruction::WideModularOp {
                        op,
                        result,
                        arg1,
                        arg2,
                        arg3,
                    } => self.verify_wide_modular_op(op, result, arg1, arg2, arg3)?,
                    FuelVmInstruction::WideCmpOp { op, arg1, arg2 } => {
                        self.verify_wide_cmp(op, arg1, arg2)?
                    }
                    FuelVmInstruction::Retd { .. } => (),
                },
                InstOp::GetElemPtr {
                    base,
                    elem_ptr_ty,
                    indices,
                } => self.verify_get_elem_ptr(&ins, base, elem_ptr_ty, indices)?,
                InstOp::GetLocal(local_var) => self.verify_get_local(local_var)?,
                InstOp::GetGlobal(global_var) => self.verify_get_global(global_var)?,
                InstOp::GetConfig(_, name) => self.verify_get_config(self.cur_module, name)?,
                InstOp::GetStorageKey(storage_key) => self.verify_get_storage_key(storage_key)?,
                InstOp::IntToPtr(value, ty) => self.verify_int_to_ptr(value, ty)?,
                InstOp::Load(ptr) => self.verify_load(ptr)?,
                InstOp::MemCopyBytes {
                    dst_val_ptr,
                    src_val_ptr,
                    byte_len,
                } => self.verify_mem_copy_bytes(dst_val_ptr, src_val_ptr, byte_len)?,
                InstOp::MemCopyVal {
                    dst_val_ptr,
                    src_val_ptr,
                } => self.verify_mem_copy_val(dst_val_ptr, src_val_ptr)?,
                InstOp::MemClearVal { dst_val_ptr } => self.verify_mem_clear_val(dst_val_ptr)?,
                InstOp::Nop => (),
                InstOp::PtrToInt(val, ty) => self.verify_ptr_to_int(val, ty)?,
                InstOp::Ret(val, ty) => self.verify_ret(val, ty)?,
                InstOp::Store {
                    dst_val_ptr,
                    stored_val,
                } => self.verify_store(&ins, dst_val_ptr, stored_val)?,
            };

            // Verify the instruction metadata too.
            self.context.verify_metadata(value_content.metadata)?;
        }

        Ok(())
    }

    fn verify_bitcast(&self, value: &Value, ty: &Type) -> Result<(), IrError> {
        // The bitsize of bools and unit is 1 which obviously won't match a typical uint.  LLVM
        // would use `trunc` or `zext` to make types match sizes before casting.  Until we have
        // similar we'll just make sure the sizes are <= 64 bits.
        let val_ty = value
            .get_type(self.context)
            .ok_or(IrError::VerifyBitcastUnknownSourceType)?;
        if self.type_bit_size(&val_ty).is_some_and(|sz| sz > 64)
            || self.type_bit_size(ty).is_some_and(|sz| sz > 64)
        {
            Err(IrError::VerifyBitcastBetweenInvalidTypes(
                val_ty.as_string(self.context),
                ty.as_string(self.context),
            ))
        } else {
            Ok(())
        }
    }

    fn verify_unary_op(&self, op: &UnaryOpKind, arg: &Value) -> Result<(), IrError> {
        let arg_ty = arg
            .get_type(self.context)
            .ok_or(IrError::VerifyUnaryOpIncorrectArgType)?;
        match op {
            UnaryOpKind::Not => {
                if !arg_ty.is_uint(self.context) && !arg_ty.is_b256(self.context) {
                    return Err(IrError::VerifyUnaryOpIncorrectArgType);
                }
            }
        }

        Ok(())
    }

    fn verify_wide_cmp(&self, _: &Predicate, arg1: &Value, arg2: &Value) -> Result<(), IrError> {
        let arg1_ty = arg1
            .get_type(self.context)
            .ok_or(IrError::VerifyBinaryOpIncorrectArgType)?;
        let arg2_ty = arg2
            .get_type(self.context)
            .ok_or(IrError::VerifyBinaryOpIncorrectArgType)?;

        if arg1_ty.is_ptr(self.context) && arg2_ty.is_ptr(self.context) {
            Ok(())
        } else {
            Err(IrError::VerifyBinaryOpIncorrectArgType)
        }
    }

    fn verify_wide_modular_op(
        &self,
        _op: &BinaryOpKind,
        result: &Value,
        arg1: &Value,
        arg2: &Value,
        arg3: &Value,
    ) -> Result<(), IrError> {
        let result_ty = result
            .get_type(self.context)
            .ok_or(IrError::VerifyBinaryOpIncorrectArgType)?;
        let arg1_ty = arg1
            .get_type(self.context)
            .ok_or(IrError::VerifyBinaryOpIncorrectArgType)?;
        let arg2_ty = arg2
            .get_type(self.context)
            .ok_or(IrError::VerifyBinaryOpIncorrectArgType)?;
        let arg3_ty = arg3
            .get_type(self.context)
            .ok_or(IrError::VerifyBinaryOpIncorrectArgType)?;

        if !arg1_ty.is_ptr(self.context)
            || !arg2_ty.is_ptr(self.context)
            || !arg3_ty.is_ptr(self.context)
            || !result_ty.is_ptr(self.context)
        {
            return Err(IrError::VerifyBinaryOpIncorrectArgType);
        }

        Ok(())
    }

    fn verify_wide_binary_op(
        &self,
        op: &BinaryOpKind,
        result: &Value,
        arg1: &Value,
        arg2: &Value,
    ) -> Result<(), IrError> {
        let result_ty = result
            .get_type(self.context)
            .ok_or(IrError::VerifyBinaryOpIncorrectArgType)?;
        let arg1_ty = arg1
            .get_type(self.context)
            .ok_or(IrError::VerifyBinaryOpIncorrectArgType)?;
        let arg2_ty = arg2
            .get_type(self.context)
            .ok_or(IrError::VerifyBinaryOpIncorrectArgType)?;

        match op {
            // Shifts rhs are 64 bits
            BinaryOpKind::Lsh | BinaryOpKind::Rsh => {
                if !arg1_ty.is_ptr(self.context)
                    || !arg2_ty.is_uint64(self.context)
                    || !result_ty.is_ptr(self.context)
                {
                    return Err(IrError::VerifyBinaryOpIncorrectArgType);
                }
            }
            BinaryOpKind::Add
            | BinaryOpKind::Sub
            | BinaryOpKind::Mul
            | BinaryOpKind::Div
            | BinaryOpKind::And
            | BinaryOpKind::Or
            | BinaryOpKind::Xor
            | BinaryOpKind::Mod => {
                if !arg1_ty.is_ptr(self.context)
                    || !arg2_ty.is_ptr(self.context)
                    || !result_ty.is_ptr(self.context)
                {
                    return Err(IrError::VerifyBinaryOpIncorrectArgType);
                }
            }
        }

        Ok(())
    }

    fn verify_wide_unary_op(
        &self,
        _op: &UnaryOpKind,
        result: &Value,
        arg: &Value,
    ) -> Result<(), IrError> {
        let result_ty = result
            .get_type(self.context)
            .ok_or(IrError::VerifyBinaryOpIncorrectArgType)?;
        let arg_ty = arg
            .get_type(self.context)
            .ok_or(IrError::VerifyBinaryOpIncorrectArgType)?;

        if !arg_ty.is_ptr(self.context) || !result_ty.is_ptr(self.context) {
            return Err(IrError::VerifyBinaryOpIncorrectArgType);
        }

        Ok(())
    }

    fn verify_binary_op(
        &self,
        op: &BinaryOpKind,
        arg1: &Value,
        arg2: &Value,
    ) -> Result<(), IrError> {
        let arg1_ty = arg1
            .get_type(self.context)
            .ok_or(IrError::VerifyBinaryOpIncorrectArgType)?;
        let arg2_ty = arg2
            .get_type(self.context)
            .ok_or(IrError::VerifyBinaryOpIncorrectArgType)?;

        match op {
            // Shifts can have the rhs with different type
            BinaryOpKind::Lsh | BinaryOpKind::Rsh => {
                let is_lhs_ok = arg1_ty.is_uint(self.context) || arg1_ty.is_b256(self.context);
                if !is_lhs_ok || !arg2_ty.is_uint(self.context) {
                    return Err(IrError::VerifyBinaryOpIncorrectArgType);
                }
            }
            BinaryOpKind::Add | BinaryOpKind::Sub => {
                if !(arg1_ty.eq(self.context, &arg2_ty) && arg1_ty.is_uint(self.context)
                    || arg1_ty.is_ptr(self.context) && arg2_ty.is_uint64(self.context))
                {
                    return Err(IrError::VerifyBinaryOpIncorrectArgType);
                }
            }
            BinaryOpKind::Mul | BinaryOpKind::Div | BinaryOpKind::Mod => {
                if !arg1_ty.eq(self.context, &arg2_ty) || !arg1_ty.is_uint(self.context) {
                    return Err(IrError::VerifyBinaryOpIncorrectArgType);
                }
            }
            BinaryOpKind::And | BinaryOpKind::Or | BinaryOpKind::Xor => {
                if !arg1_ty.eq(self.context, &arg2_ty)
                    || !(arg1_ty.is_uint(self.context) || arg1_ty.is_b256(self.context))
                {
                    return Err(IrError::VerifyBinaryOpIncorrectArgType);
                }
            }
        }

        Ok(())
    }

    fn verify_br(&self, dest_block: &BranchToWithArgs) -> Result<(), IrError> {
        if !self
            .cur_function
            .block_iter(self.context)
            .contains(&dest_block.block)
        {
            Err(IrError::VerifyBranchToMissingBlock(
                self.context.blocks[dest_block.block.0].label.clone(),
            ))
        } else {
            self.verify_dest_args(dest_block)
        }
    }

    fn verify_call(&self, callee: &Function, args: &[Value]) -> Result<(), IrError> {
        let callee_content = &self.context.functions[callee.0];
        if !self.cur_module.function_iter(self.context).contains(callee) {
            return Err(IrError::VerifyCallToMissingFunction(
                callee_content.name.clone(),
            ));
        }

        let callee_arg_types = callee_content
            .arguments
            .iter()
            .map(|(_, arg_val)| {
                if let ValueDatum::Argument(BlockArgument { ty, .. }) =
                    &self.context.values[arg_val.0].value
                {
                    Ok(*ty)
                } else {
                    Err(IrError::VerifyArgumentValueIsNotArgument(
                        callee_content.name.clone(),
                    ))
                }
            })
            .collect::<Result<Vec<Type>, IrError>>()?;

        for (opt_caller_arg_type, callee_arg_type) in args
            .iter()
            .map(|val| val.get_type(self.context))
            .zip(callee_arg_types.iter())
        {
            if opt_caller_arg_type.is_none() {
                return Err(IrError::VerifyUntypedValuePassedToFunction);
            }

            let caller_arg_type = opt_caller_arg_type.as_ref().unwrap();
            if !caller_arg_type.eq(self.context, callee_arg_type) {
                return Err(IrError::VerifyCallArgTypeMismatch(
                    callee_content.name.clone(),
                    caller_arg_type.as_string(self.context),
                    callee_arg_type.as_string(self.context),
                ));
            }
        }

        Ok(())
    }

    fn verify_cast_ptr(&self, val: &Value, ty: &Type) -> Result<(), IrError> {
        if !(val
            .get_type(self.context)
            .is_some_and(|ty| ty.is_ptr(self.context)))
        {
            let ty = val
                .get_type(self.context)
                .map(|ty| ty.as_string(self.context))
                .unwrap_or("Unknown".into());
            return Err(IrError::VerifyPtrCastFromNonPointer(ty));
        }

        if !ty.is_ptr(self.context) {
            Err(IrError::VerifyPtrCastToNonPointer(
                ty.as_string(self.context),
            ))
        } else {
            Ok(())
        }
    }

    fn verify_dest_args(&self, dest: &BranchToWithArgs) -> Result<(), IrError> {
        if dest.block.num_args(self.context) != dest.args.len() {
            return Err(IrError::VerifyBranchParamsMismatch);
        }
        for (arg_idx, dest_param) in dest.block.arg_iter(self.context).enumerate() {
            match dest.args.get(arg_idx) {
                Some(actual)
                    if dest_param
                        .get_type(self.context)
                        .unwrap()
                        .eq(self.context, &actual.get_type(self.context).unwrap()) => {}
                _ =>
                // TODO: https://github.com/FuelLabs/sway/pull/2880
                {
                    // return Err(IrError::VerifyBranchParamsMismatch)
                }
            }
        }
        Ok(())
    }

    fn verify_cbr(
        &self,
        cond_val: &Value,
        true_block: &BranchToWithArgs,
        false_block: &BranchToWithArgs,
    ) -> Result<(), IrError> {
        if !cond_val
            .get_type(self.context)
            .is(Type::is_bool, self.context)
        {
            Err(IrError::VerifyConditionExprNotABool)
        } else if !self
            .cur_function
            .block_iter(self.context)
            .contains(&true_block.block)
        {
            Err(IrError::VerifyBranchToMissingBlock(
                self.context.blocks[true_block.block.0].label.clone(),
            ))
        } else if !self
            .cur_function
            .block_iter(self.context)
            .contains(&false_block.block)
        {
            Err(IrError::VerifyBranchToMissingBlock(
                self.context.blocks[false_block.block.0].label.clone(),
            ))
        } else {
            self.verify_dest_args(true_block)
                .and_then(|()| self.verify_dest_args(false_block))
        }
    }

    fn verify_cmp(
        &self,
        _pred: &Predicate,
        lhs_value: &Value,
        rhs_value: &Value,
    ) -> Result<(), IrError> {
        // Comparisons must be between integers or equivalent pointers at this stage.
        match (
            lhs_value.get_type(self.context),
            rhs_value.get_type(self.context),
        ) {
            (Some(lhs_ty), Some(rhs_ty)) => {
                if !lhs_ty.eq(self.context, &rhs_ty) {
                    Err(IrError::VerifyCmpTypeMismatch(
                        lhs_ty.as_string(self.context),
                        rhs_ty.as_string(self.context),
                    ))
                } else if lhs_ty.is_bool(self.context)
                    || lhs_ty.is_uint(self.context)
                    || lhs_ty.is_ptr(self.context)
                    || lhs_ty.is_b256(self.context)
                {
                    Ok(())
                } else {
                    Err(IrError::VerifyCmpBadTypes(
                        lhs_ty.as_string(self.context),
                        rhs_ty.as_string(self.context),
                    ))
                }
            }
            _otherwise => Err(IrError::VerifyCmpUnknownTypes),
        }
    }

    fn verify_contract_call(
        &self,
        params: &Value,
        coins: &Value,
        asset_id: &Value,
        gas: &Value,
    ) -> Result<(), IrError> {
        if !self.context.experimental.new_encoding {
            // - The params must be a struct with the B256 address, u64 selector and u64 address to
            //   user args.
            // - The coins and gas must be u64s.
            // - The asset_id must be a B256
            let fields = params
                .get_type(self.context)
                .and_then(|ty| ty.get_pointee_type(self.context))
                .map_or_else(std::vec::Vec::new, |ty| ty.get_field_types(self.context));
            if fields.len() != 3
                || !fields[0].is_b256(self.context)
                || !fields[1].is_uint64(self.context)
                || !fields[2].is_uint64(self.context)
            {
                Err(IrError::VerifyContractCallBadTypes("params".to_owned()))
            } else {
                Ok(())
            }
            .and_then(|_| {
                if coins
                    .get_type(self.context)
                    .is(Type::is_uint64, self.context)
                {
                    Ok(())
                } else {
                    Err(IrError::VerifyContractCallBadTypes("coins".to_owned()))
                }
            })
            .and_then(|_| {
                if asset_id
                    .get_type(self.context)
                    .and_then(|ty| ty.get_pointee_type(self.context))
                    .is(Type::is_b256, self.context)
                {
                    Ok(())
                } else {
                    Err(IrError::VerifyContractCallBadTypes("asset_id".to_owned()))
                }
            })
            .and_then(|_| {
                if gas.get_type(self.context).is(Type::is_uint64, self.context) {
                    Ok(())
                } else {
                    Err(IrError::VerifyContractCallBadTypes("gas".to_owned()))
                }
            })
        } else {
            Ok(())
        }
    }

    fn verify_get_elem_ptr(
        &self,
        ins: &Value,
        base: &Value,
        elem_ptr_ty: &Type,
        indices: &[Value],
    ) -> Result<(), IrError> {
        let base_ty =
            self.get_ptr_type(base, |s| IrError::VerifyGepFromNonPointer(s, Some(*ins)))?;
        if !base_ty.is_aggregate(self.context) {
            return Err(IrError::VerifyGepOnNonAggregate);
        }

        let Some(elem_inner_ty) = elem_ptr_ty.get_pointee_type(self.context) else {
            return Err(IrError::VerifyGepElementTypeNonPointer);
        };

        if indices.is_empty() {
            return Err(IrError::VerifyGepInconsistentTypes(
                "Empty Indices".into(),
                Some(*base),
            ));
        }

        let index_ty = base_ty.get_value_indexed_type(self.context, indices);

        if self.opt_ty_not_eq(&Some(elem_inner_ty), &index_ty) {
            return Err(IrError::VerifyGepInconsistentTypes(
                format!(
                    "Element type \"{}\" versus index type {:?}",
                    elem_inner_ty.as_string(self.context),
                    index_ty.map(|x| x.as_string(self.context))
                ),
                Some(*ins),
            ));
        }

        Ok(())
    }

    fn verify_get_local(&self, local_var: &LocalVar) -> Result<(), IrError> {
        if !self.context.functions[self.cur_function.0]
            .local_storage
            .values()
            .any(|var| var == local_var)
        {
            Err(IrError::VerifyGetNonExistentLocalVarPointer)
        } else {
            Ok(())
        }
    }

    fn verify_get_global(&self, global_var: &GlobalVar) -> Result<(), IrError> {
        if !self.context.modules[self.cur_module.0]
            .global_variables
            .values()
            .any(|var| var == global_var)
        {
            Err(IrError::VerifyGetNonExistentGlobalVarPointer)
        } else {
            Ok(())
        }
    }

    fn verify_get_config(&self, module: Module, name: &str) -> Result<(), IrError> {
        if !self.context.modules[module.0].configs.contains_key(name) {
            Err(IrError::VerifyGetNonExistentConfigPointer)
        } else {
            Ok(())
        }
    }

    fn verify_get_storage_key(&self, storage_key: &StorageKey) -> Result<(), IrError> {
        if !self.context.modules[self.cur_module.0]
            .storage_keys
            .values()
            .any(|key| key == storage_key)
        {
            Err(IrError::VerifyGetNonExistentStorageKeyPointer)
        } else {
            Ok(())
        }
    }

    fn verify_gtf(&self, index: &Value, _tx_field_id: &u64) -> Result<(), IrError> {
        // We should perhaps verify that _tx_field_id fits in a twelve bit immediate
        if !index.get_type(self.context).is(Type::is_uint, self.context) {
            Err(IrError::VerifyInvalidGtfIndexType)
        } else {
            Ok(())
        }
    }

    fn verify_int_to_ptr(&self, value: &Value, ty: &Type) -> Result<(), IrError> {
        // We want the source value to be an integer and the destination type to be a pointer.
        let val_ty = value
            .get_type(self.context)
            .ok_or(IrError::VerifyIntToPtrUnknownSourceType)?;
        if !val_ty.is_uint(self.context) {
            return Err(IrError::VerifyIntToPtrFromNonIntegerType(
                val_ty.as_string(self.context),
            ));
        }
        if !ty.is_ptr(self.context) {
            return Err(IrError::VerifyIntToPtrToNonPointer(
                ty.as_string(self.context),
            ));
        }

        Ok(())
    }

    fn verify_load(&self, src_val: &Value) -> Result<(), IrError> {
        // Just confirm `src_val` is a pointer.
        self.get_ptr_type(src_val, IrError::VerifyLoadFromNonPointer)
            .map(|_| ())
    }

    fn verify_log(
        &self,
        log_val: &Value,
        log_ty: &Type,
        log_id: &Value,
        log_data: &Option<LogEventData>,
    ) -> Result<(), IrError> {
        if !log_id
            .get_type(self.context)
            .is(Type::is_uint64, self.context)
        {
            return Err(IrError::VerifyLogId);
        }

        if self.opt_ty_not_eq(&log_val.get_type(self.context), &Some(*log_ty)) {
            return Err(IrError::VerifyLogMismatchedTypes);
        }

        if let Some(log_data) = log_data {
            if log_data.version() != LogEventData::CURRENT_VERSION {
                return Err(IrError::VerifyLogEventDataVersion(log_data.version()));
            }

            if !log_data.is_event() {
                return Err(IrError::VerifyLogEventDataInvalid(
                    "log metadata must describe an event".into(),
                ));
            }

            if log_data.is_indexed() {
                if log_data.num_elements() == 0 || log_data.event_type_size() == 0 {
                    return Err(IrError::VerifyLogEventDataInvalid(
                        "indexed event metadata requires non-zero element count and size".into(),
                    ));
                }
            } else if log_data.num_elements() != 0 || log_data.event_type_size() != 0 {
                return Err(IrError::VerifyLogEventDataInvalid(
                    "non-indexed event metadata must not include element size or count".into(),
                ));
            }
        }

        Ok(())
    }

    fn verify_mem_copy_bytes(
        &self,
        dst_val_ptr: &Value,
        src_val_ptr: &Value,
        _byte_len: &u64,
    ) -> Result<(), IrError> {
        // Just confirm both values are pointers.
        self.get_ptr_type(dst_val_ptr, IrError::VerifyMemcopyNonPointer)
            .and_then(|_| self.get_ptr_type(src_val_ptr, IrError::VerifyMemcopyNonPointer))
            .map(|_| ())
    }

    fn verify_mem_copy_val(&self, dst_val_ptr: &Value, src_val_ptr: &Value) -> Result<(), IrError> {
        // Check both types are pointers and to the same type.
        self.get_ptr_type(dst_val_ptr, IrError::VerifyMemcopyNonPointer)
            .and_then(|dst_ty| {
                self.get_ptr_type(src_val_ptr, IrError::VerifyMemcopyNonPointer)
                    .map(|src_ty| (dst_ty, src_ty))
            })
            .and_then(|(dst_ty, src_ty)| {
                dst_ty
                    .eq(self.context, &src_ty)
                    .then_some(())
                    .ok_or_else(|| {
                        IrError::VerifyMemcopyMismatchedTypes(
                            dst_ty.as_string(self.context),
                            src_ty.as_string(self.context),
                        )
                    })
            })
    }

    // dst_val_ptr must be a a pointer.
    fn verify_mem_clear_val(&self, dst_val_ptr: &Value) -> Result<(), IrError> {
        let _ = self.get_ptr_type(dst_val_ptr, IrError::VerifyMemClearValNonPointer)?;
        Ok(())
    }

    fn verify_ptr_to_int(&self, val: &Value, ty: &Type) -> Result<(), IrError> {
        // XXX Casting pointers to integers is a low level operation which needs to be verified in
        // the target specific verifier.  e.g., for Fuel it is assumed that b256s are 'reference
        // types' and you can to a ptr_to_int on them, but for target agnostic IR this isn't true.
        if !(val
            .get_type(self.context)
            .is_some_and(|ty| ty.is_ptr(self.context)))
        {
            let ty = val
                .get_type(self.context)
                .map(|ty| ty.as_string(self.context))
                .unwrap_or("Unknown".into());
            return Err(IrError::VerifyPtrCastFromNonPointer(ty));
        }
        if !ty.is_uint(self.context) {
            Err(IrError::VerifyPtrToIntToNonInteger(
                ty.as_string(self.context),
            ))
        } else {
            Ok(())
        }
    }

    fn verify_ret(&self, val: &Value, ty: &Type) -> Result<(), IrError> {
        if !self
            .cur_function
            .get_return_type(self.context)
            .eq(self.context, ty)
            || self.opt_ty_not_eq(&val.get_type(self.context), &Some(*ty))
        {
            Err(IrError::VerifyReturnMismatchedTypes(
                self.cur_function.get_name(self.context).to_string(),
            ))
        } else {
            Ok(())
        }
    }

    fn verify_revert(&self, val: &Value) -> Result<(), IrError> {
        if !val.get_type(self.context).is(Type::is_uint64, self.context) {
            Err(IrError::VerifyRevertCodeBadType)
        } else {
            Ok(())
        }
    }

    fn verify_smo(
        &self,
        recipient: &Value,
        message: &Value,
        message_size: &Value,
        coins: &Value,
    ) -> Result<(), IrError> {
        // Check that the first operand is a `b256` representing the recipient address.
        let recipient = self.get_ptr_type(recipient, IrError::VerifySmoRecipientNonPointer)?;
        if !recipient.is_b256(self.context) {
            return Err(IrError::VerifySmoRecipientBadType);
        }

        // Check that the second operand is a struct with two fields
        let struct_ty = self.get_ptr_type(message, IrError::VerifySmoMessageNonPointer)?;

        if !struct_ty.is_struct(self.context) {
            return Err(IrError::VerifySmoBadMessageType);
        }
        let fields = struct_ty.get_field_types(self.context);
        if fields.len() != 2 {
            return Err(IrError::VerifySmoBadMessageType);
        }

        // Check that the second operand is a `u64` representing the message size.
        if !message_size
            .get_type(self.context)
            .is(Type::is_uint64, self.context)
        {
            return Err(IrError::VerifySmoMessageSize);
        }

        // Check that the third operand is a `u64` representing the amount of coins being sent.
        if !coins
            .get_type(self.context)
            .is(Type::is_uint64, self.context)
        {
            return Err(IrError::VerifySmoCoins);
        }

        Ok(())
    }

    fn verify_state_clear(&self, key: &Value, number_of_slots: &Value) -> Result<(), IrError> {
        let key_type = self.get_ptr_type(key, IrError::VerifyStateKeyNonPointer)?;
        if !key_type.is_b256(self.context) {
            Err(IrError::VerifyStateKeyBadType)
        } else if !number_of_slots
            .get_type(self.context)
            .is(Type::is_uint, self.context)
        {
            Err(IrError::VerifyStateAccessNumOfSlots)
        } else {
            Ok(())
        }
    }

    fn verify_state_access_quad(
        &self,
        dst_val: &Value,
        key: &Value,
        number_of_slots: &Value,
    ) -> Result<(), IrError> {
        let dst_ty = self.get_ptr_type(dst_val, IrError::VerifyStateAccessQuadNonPointer)?;
        if !dst_ty.is_b256(self.context) {
            return Err(IrError::VerifyStateDestBadType(
                dst_ty.as_string(self.context),
            ));
        }
        let key_type = self.get_ptr_type(key, IrError::VerifyStateKeyNonPointer)?;
        if !key_type.is_b256(self.context) {
            return Err(IrError::VerifyStateKeyBadType);
        }
        if !number_of_slots
            .get_type(self.context)
            .is(Type::is_uint, self.context)
        {
            return Err(IrError::VerifyStateAccessNumOfSlots);
        }
        Ok(())
    }

    fn verify_state_load_word(&self, key: &Value) -> Result<(), IrError> {
        let key_type = self.get_ptr_type(key, IrError::VerifyStateKeyNonPointer)?;
        if !key_type.is_b256(self.context) {
            Err(IrError::VerifyStateKeyBadType)
        } else {
            Ok(())
        }
    }

    fn verify_state_store_word(&self, dst_val: &Value, key: &Value) -> Result<(), IrError> {
        let key_type = self.get_ptr_type(key, IrError::VerifyStateKeyNonPointer)?;
        if !key_type.is_b256(self.context) {
            Err(IrError::VerifyStateKeyBadType)
        } else if !dst_val
            .get_type(self.context)
            .is(Type::is_uint, self.context)
        {
            Err(IrError::VerifyStateDestBadType(
                Type::get_uint64(self.context).as_string(self.context),
            ))
        } else {
            Ok(())
        }
    }

    fn verify_store(
        &self,
        ins: &Value,
        dst_val: &Value,
        stored_val: &Value,
    ) -> Result<(), IrError> {
        let dst_ty = self.get_ptr_type(dst_val, IrError::VerifyStoreToNonPointer)?;
        let stored_ty = stored_val.get_type(self.context);
        if self.opt_ty_not_eq(&Some(dst_ty), &stored_ty) {
            Err(IrError::VerifyStoreMismatchedTypes(Some(*ins)))
        } else {
            Ok(())
        }
    }

    //----------------------------------------------------------------------------------------------

    // This is a really common operation above... calling `Value::get_type()` and then failing when
    // two don't match.
    fn opt_ty_not_eq(&self, l_ty: &Option<Type>, r_ty: &Option<Type>) -> bool {
        l_ty.is_none() || r_ty.is_none() || !l_ty.unwrap().eq(self.context, r_ty.as_ref().unwrap())
    }

    fn get_ptr_type<F: FnOnce(String) -> IrError>(
        &self,
        val: &Value,
        errfn: F,
    ) -> Result<Type, IrError> {
        val.get_type(self.context)
            .ok_or_else(|| "unknown".to_owned())
            .and_then(|ptr_ty| {
                ptr_ty
                    .get_pointee_type(self.context)
                    .ok_or_else(|| ptr_ty.as_string(self.context))
            })
            .map_err(errfn)
    }

    // Get the bit size for fixed atomic types, or None for other types.
    fn type_bit_size(&self, ty: &Type) -> Option<usize> {
        // Typically we don't want to make assumptions about the size of types in the IR.  This is
        // here until we reintroduce pointers and don't need to care about type sizes (and whether
        // they'd fit in a 64 bit register).
        if ty.is_unit(self.context) || ty.is_bool(self.context) {
            Some(1)
        } else if ty.is_uint(self.context) {
            Some(ty.get_uint_width(self.context).unwrap() as usize)
        } else if ty.is_b256(self.context) {
            Some(256)
        } else {
            None
        }
    }
}
