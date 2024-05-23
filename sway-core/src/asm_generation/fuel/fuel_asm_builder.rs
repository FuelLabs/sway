use crate::{
    asm_generation::{
        asm_builder::{AsmBuilder, AsmBuilderResult},
        from_ir::{StateAccessType, Storage},
        fuel::{
            abstract_instruction_set::AbstractInstructionSet,
            compiler_constants,
            data_section::{DataId, DataSection, Entry},
            register_sequencer::RegisterSequencer,
        },
        ProgramKind,
    },
    asm_lang::{
        virtual_register::*, Label, Op, VirtualImmediate06, VirtualImmediate12, VirtualImmediate18,
        VirtualOp, WideCmp, WideOperations,
    },
    decl_engine::DeclRefFunction,
    metadata::MetadataManager,
};

use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
    warning::CompileWarning,
    warning::Warning,
};
use sway_ir::*;
use sway_types::{span::Span, Spanned};

use either::Either;
use std::collections::HashMap;

pub struct FuelAsmBuilder<'ir, 'eng> {
    pub(super) program_kind: ProgramKind,

    // Data section is used by the rest of code gen to layout const memory.
    pub(super) data_section: DataSection,

    // Register sequencer dishes out new registers and labels.
    pub(super) reg_seqr: RegisterSequencer,

    // Label maps are from IR functions or blocks to label name.  Functions have a start and end
    // label.
    pub(super) func_label_map: HashMap<Function, (Label, Label)>,
    pub(super) block_label_map: HashMap<Block, Label>,

    // Reg map is tracking IR values to VM values.  Ptr map is tracking IR pointers to local
    // storage types.
    pub(super) reg_map: HashMap<Value, VirtualRegister>,
    pub(super) ptr_map: HashMap<LocalVar, Storage>,
    // PHIs need a register to which predecessor blocks will copy the value to.
    // That VirtualRegister is then copied to another one in the block, mapped by reg_map.
    pub(super) phi_reg_map: HashMap<Value, VirtualRegister>,

    // The currently compiled function has an end label which is at the end of the function body
    // but before the call cleanup, and a copy of the $retv for when the return value is a reference
    // type and must be copied in memory.  Unless we have nested function declarations this vector
    // will usually have 0 or 1 entry.
    pub(super) return_ctxs: Vec<(Label, VirtualRegister)>,

    // Stack size and base register for locals and num_extra_args in any call in the function.
    pub(super) locals_ctxs: Vec<(u64, VirtualRegister, u64)>,

    // IR context we're compiling.
    pub(super) context: &'ir Context<'eng>,

    // Metadata manager for converting metadata to Spans, etc.
    pub(super) md_mgr: MetadataManager,

    // Final resulting VM bytecode ops; entry functions with their function and label, and regular
    // non-entry functions.
    pub(super) entries: Vec<(Function, Label, Vec<Op>, Option<DeclRefFunction>)>,
    pub(super) non_entries: Vec<Vec<Op>>,

    // In progress VM bytecode ops.
    pub(super) cur_bytecode: Vec<Op>,
}

pub type FuelAsmBuilderResult = (
    DataSection,
    RegisterSequencer,
    Vec<(
        Function,
        Label,
        AbstractInstructionSet,
        Option<DeclRefFunction>,
    )>,
    Vec<AbstractInstructionSet>,
);

impl<'ir, 'eng> AsmBuilder for FuelAsmBuilder<'ir, 'eng> {
    fn func_to_labels(&mut self, func: &Function) -> (Label, Label) {
        self.func_to_labels(func)
    }

    fn compile_function(
        &mut self,
        handler: &Handler,
        function: Function,
    ) -> Result<(), ErrorEmitted> {
        self.compile_function(handler, function)
    }

    fn finalize(&self) -> AsmBuilderResult {
        self.finalize()
    }
}

impl<'ir, 'eng> FuelAsmBuilder<'ir, 'eng> {
    pub fn new(
        program_kind: ProgramKind,
        data_section: DataSection,
        reg_seqr: RegisterSequencer,
        context: &'ir Context<'eng>,
    ) -> Self {
        FuelAsmBuilder {
            program_kind,
            data_section,
            reg_seqr,
            func_label_map: HashMap::new(),
            block_label_map: HashMap::new(),
            reg_map: HashMap::new(),
            ptr_map: HashMap::new(),
            phi_reg_map: HashMap::new(),
            return_ctxs: Vec::new(),
            locals_ctxs: Vec::new(),
            context,
            md_mgr: MetadataManager::default(),
            entries: Vec::new(),
            non_entries: Vec::new(),
            cur_bytecode: Vec::new(),
        }
    }

    pub fn finalize(&self) -> AsmBuilderResult {
        AsmBuilderResult::Fuel((
            self.data_section.clone(),
            self.reg_seqr,
            self.entries
                .clone()
                .into_iter()
                .map(|(f, l, ops, test_decl_ref)| {
                    (f, l, AbstractInstructionSet { ops }, test_decl_ref)
                })
                .collect(),
            self.non_entries
                .clone()
                .into_iter()
                .map(|ops| AbstractInstructionSet { ops })
                .collect(),
        ))
    }

    pub(super) fn compile_block(
        &mut self,
        handler: &Handler,
        block: &Block,
        func_is_entry: bool,
    ) -> Result<(), ErrorEmitted> {
        if block
            .get_function(self.context)
            .get_entry_block(self.context)
            != *block
        {
            // If the block has an arg, copy value from its phi_reg_map vreg to a new one.
            for arg in block.arg_iter(self.context) {
                let phi_reg = self.phi_reg_map.entry(*arg).or_insert(self.reg_seqr.next());
                // Associate a new virtual register for this arg and copy phi_reg to it.
                let arg_reg = self.reg_seqr.next();
                self.reg_map.insert(*arg, arg_reg.clone());
                self.cur_bytecode.push(Op::register_move(
                    arg_reg.clone(),
                    phi_reg.clone(),
                    "parameter from branch to block argument",
                    None,
                ));
            }
        }

        for instr_val in block.instruction_iter(self.context) {
            self.compile_instruction(handler, &instr_val, func_is_entry)?;
        }
        Ok(())
    }

    pub(super) fn compile_instruction(
        &mut self,
        handler: &Handler,
        instr_val: &Value,
        func_is_entry: bool,
    ) -> Result<(), ErrorEmitted> {
        let Some(instruction) = instr_val.get_instruction(self.context) else {
            return Err(handler.emit_err(CompileError::Internal(
                "Value not an instruction.",
                self.md_mgr
                    .val_to_span(self.context, *instr_val)
                    .unwrap_or_else(Span::dummy),
            )));
        };

        // The only instruction whose compilation returns a Result itself is AsmBlock, which
        // we special-case here.  Ideally, the ASM block verification would happen much sooner,
        // perhaps during parsing.  https://github.com/FuelLabs/sway/issues/801
        if let InstOp::AsmBlock(asm, args) = &instruction.op {
            self.compile_asm_block(handler, instr_val, asm, args)
        } else {
            // These matches all return `Result<(), CompileError>`.
            match &instruction.op {
                InstOp::AsmBlock(..) => unreachable!("Handled immediately above."),
                InstOp::BitCast(val, ty) => self.compile_bitcast(instr_val, val, ty),
                InstOp::UnaryOp { op, arg } => self.compile_unary_op(instr_val, op, arg),
                InstOp::BinaryOp { op, arg1, arg2 } => {
                    self.compile_binary_op(instr_val, op, arg1, arg2)
                }
                InstOp::Branch(to_block) => self.compile_branch(to_block),
                InstOp::Call(func, args) => self.compile_call(instr_val, func, args),
                InstOp::CastPtr(val, _ty) => self.compile_no_op_move(instr_val, val),
                InstOp::Cmp(pred, lhs_value, rhs_value) => {
                    self.compile_cmp(instr_val, pred, lhs_value, rhs_value)
                }
                InstOp::ConditionalBranch {
                    cond_value,
                    true_block,
                    false_block,
                } => self.compile_conditional_branch(cond_value, true_block, false_block),
                InstOp::ContractCall {
                    params,
                    coins,
                    asset_id,
                    gas,
                    ..
                } => self.compile_contract_call(instr_val, params, coins, asset_id, gas),
                InstOp::FuelVm(fuel_vm_instr) => match fuel_vm_instr {
                    FuelVmInstruction::Gtf { index, tx_field_id } => {
                        self.compile_gtf(instr_val, index, *tx_field_id)
                    }
                    FuelVmInstruction::Log {
                        log_val,
                        log_ty,
                        log_id,
                    } => self.compile_log(instr_val, log_val, log_ty, log_id),
                    FuelVmInstruction::ReadRegister(reg) => {
                        self.compile_read_register(instr_val, reg);
                        Ok(())
                    }
                    FuelVmInstruction::Revert(revert_val) => {
                        self.compile_revert(instr_val, revert_val)
                    }
                    FuelVmInstruction::Smo {
                        recipient,
                        message,
                        message_size,
                        coins,
                    } => self.compile_smo(instr_val, recipient, message, message_size, coins),
                    FuelVmInstruction::StateClear {
                        key,
                        number_of_slots,
                    } => self.compile_state_clear(instr_val, key, number_of_slots),
                    FuelVmInstruction::StateLoadQuadWord {
                        load_val,
                        key,
                        number_of_slots,
                    } => self.compile_state_access_quad_word(
                        instr_val,
                        load_val,
                        key,
                        number_of_slots,
                        StateAccessType::Read,
                    ),
                    FuelVmInstruction::StateLoadWord(key) => {
                        self.compile_state_load_word(instr_val, key)
                    }
                    FuelVmInstruction::StateStoreQuadWord {
                        stored_val,
                        key,
                        number_of_slots,
                    } => self.compile_state_access_quad_word(
                        instr_val,
                        stored_val,
                        key,
                        number_of_slots,
                        StateAccessType::Write,
                    ),
                    FuelVmInstruction::StateStoreWord { stored_val, key } => {
                        self.compile_state_store_word(instr_val, stored_val, key)
                    }

                    // Wide operations
                    FuelVmInstruction::WideUnaryOp { op, result, arg } => {
                        self.compile_wide_unary_op(instr_val, op, arg, result)
                    }
                    FuelVmInstruction::WideBinaryOp {
                        op,
                        result,
                        arg1,
                        arg2,
                    } => self.compile_wide_binary_op(instr_val, op, arg1, arg2, result),
                    FuelVmInstruction::WideCmpOp { op, arg1, arg2 } => {
                        self.compile_wide_cmp_op(instr_val, op, arg1, arg2)
                    }
                    FuelVmInstruction::WideModularOp {
                        op,
                        result,
                        arg1,
                        arg2,
                        arg3,
                    } => self.compile_wide_modular_op(instr_val, op, result, arg1, arg2, arg3),
                    FuelVmInstruction::JmpMem => self.compile_jmp_mem(instr_val),
                    FuelVmInstruction::Retd { ptr, len } => self.compile_retd(instr_val, ptr, len),
                },
                InstOp::GetElemPtr {
                    base,
                    elem_ptr_ty,
                    indices,
                } => self.compile_get_elem_ptr(instr_val, base, elem_ptr_ty, indices),
                InstOp::GetLocal(local_var) => self.compile_get_local(instr_val, local_var),
                InstOp::IntToPtr(val, _) => self.compile_no_op_move(instr_val, val),
                InstOp::Load(src_val) => self.compile_load(instr_val, src_val),
                InstOp::MemCopyBytes {
                    dst_val_ptr,
                    src_val_ptr,
                    byte_len,
                } => self.compile_mem_copy_bytes(instr_val, dst_val_ptr, src_val_ptr, *byte_len),
                InstOp::MemCopyVal {
                    dst_val_ptr,
                    src_val_ptr,
                } => self.compile_mem_copy_val(instr_val, dst_val_ptr, src_val_ptr),
                InstOp::Nop => Ok(()),
                InstOp::PtrToInt(ptr_val, _int_ty) => self.compile_no_op_move(instr_val, ptr_val),
                InstOp::Ret(ret_val, ty) => {
                    if func_is_entry {
                        self.compile_ret_from_entry(instr_val, ret_val, ty)
                    } else {
                        self.compile_ret_from_call(instr_val, ret_val)
                    }
                }
                InstOp::Store {
                    dst_val_ptr,
                    stored_val,
                } => self.compile_store(instr_val, dst_val_ptr, stored_val),
            }
            .map_err(|e| handler.emit_err(e))
        }
    }

    fn compile_asm_block(
        &mut self,
        handler: &Handler,
        instr_val: &Value,
        asm: &AsmBlock,
        asm_args: &[AsmArg],
    ) -> Result<(), ErrorEmitted> {
        let mut inline_reg_map = HashMap::new();
        let mut inline_ops = Vec::new();
        for AsmArg { name, initializer } in asm_args {
            if ConstantRegister::parse_register_name(name.as_str()).is_some() {
                handler.emit_warn(CompileWarning {
                    span: name.span().clone(),
                    warning_content: Warning::ShadowingReservedRegister {
                        reg_name: name.clone(),
                    },
                });
            }

            let arg_reg = match initializer {
                Some(init_val) => {
                    let init_val_reg = match self.value_to_register(init_val) {
                        Ok(ivr) => ivr,
                        Err(e) => {
                            return Err(handler.emit_err(e));
                        }
                    };
                    match init_val_reg {
                        VirtualRegister::Virtual(_) => init_val_reg,
                        VirtualRegister::Constant(_) => {
                            let const_copy = self.reg_seqr.next();
                            inline_ops.push(Op {
                                opcode: Either::Left(VirtualOp::MOVE(
                                    const_copy.clone(),
                                    init_val_reg,
                                )),
                                comment: "copy const asm init to GP reg".into(),
                                owning_span: self.md_mgr.val_to_span(self.context, *instr_val),
                            });
                            const_copy
                        }
                    }
                }
                None => self.reg_seqr.next(),
            };
            inline_reg_map.insert(name.as_str(), arg_reg);
        }

        let realize_register = |reg_name: &str| {
            inline_reg_map.get(reg_name).cloned().or_else(|| {
                ConstantRegister::parse_register_name(reg_name).map(VirtualRegister::Constant)
            })
        };

        // For each opcode in the asm expression, attempt to parse it into an opcode and
        // replace references to the above registers with the newly allocated ones.
        let asm_block = asm;
        for op in &asm_block.body {
            let replaced_registers = op
                .args
                .iter()
                .map(|reg_name| -> Result<_, CompileError> {
                    realize_register(reg_name.as_str()).ok_or_else(|| {
                        CompileError::UnknownRegister {
                            span: reg_name.span(),
                            initialized_registers: inline_reg_map
                                .keys()
                                .copied()
                                .collect::<Vec<_>>()
                                .join("\n"),
                        }
                    })
                })
                .filter_map(|res| match res {
                    Err(e) => {
                        handler.emit_err(e);
                        None
                    }
                    Ok(o) => Some(o),
                })
                .collect::<Vec<VirtualRegister>>();

            // Parse the actual op and registers.
            let op_span = self
                .md_mgr
                .md_to_span(self.context, op.metadata)
                .unwrap_or_else(Span::dummy);
            let opcode = Op::parse_opcode(
                handler,
                &op.op_name,
                &replaced_registers,
                &op.immediate,
                op_span.clone(),
            )?;

            inline_ops.push(Op {
                opcode: either::Either::Left(opcode),
                comment: "asm block".into(),
                owning_span: Some(op_span),
            });
        }

        // Now, load the designated asm return register into the desired return register, but only
        // if it was named.
        if let Some(ret_reg_name) = &asm_block.return_name {
            // Lookup and replace the return register.
            let ret_reg = match realize_register(ret_reg_name.as_str()) {
                Some(reg) => reg,
                None => {
                    return Err(handler.emit_err(CompileError::UnknownRegister {
                        initialized_registers: inline_reg_map
                            .keys()
                            .map(|name| name.to_string())
                            .collect::<Vec<_>>()
                            .join("\n"),
                        span: ret_reg_name.span(),
                    }));
                }
            };
            let instr_reg = self.reg_seqr.next();
            inline_ops.push(Op {
                opcode: Either::Left(VirtualOp::MOVE(instr_reg.clone(), ret_reg)),
                comment: "return value from inline asm".into(),
                owning_span: self.md_mgr.val_to_span(self.context, *instr_val),
            });
            self.reg_map.insert(*instr_val, instr_reg);
        }

        self.cur_bytecode.append(&mut inline_ops);

        Ok(())
    }

    fn compile_bitcast(
        &mut self,
        instr_val: &Value,
        bitcast_val: &Value,
        to_type: &Type,
    ) -> Result<(), CompileError> {
        let val_reg = self.value_to_register(bitcast_val)?;
        let reg = if to_type.is_bool(self.context) {
            // This may not be necessary if we just treat a non-zero value as 'true'.
            let res_reg = self.reg_seqr.next();
            self.cur_bytecode.push(Op {
                opcode: Either::Left(VirtualOp::EQ(
                    res_reg.clone(),
                    val_reg,
                    VirtualRegister::Constant(ConstantRegister::Zero),
                )),
                comment: "convert to inversed boolean".into(),
                owning_span: self.md_mgr.val_to_span(self.context, *instr_val),
            });
            self.cur_bytecode.push(Op {
                opcode: Either::Left(VirtualOp::XORI(
                    res_reg.clone(),
                    res_reg.clone(),
                    VirtualImmediate12 { value: 1 },
                )),
                comment: "invert boolean".into(),
                owning_span: self.md_mgr.val_to_span(self.context, *instr_val),
            });
            res_reg
        } else {
            // This is a no-op, although strictly speaking Unit should probably be compiled as
            // a zero.
            val_reg
        };
        self.reg_map.insert(*instr_val, reg);
        Ok(())
    }

    fn compile_unary_op(
        &mut self,
        instr_val: &Value,
        op: &UnaryOpKind,
        arg: &Value,
    ) -> Result<(), CompileError> {
        let val_reg = self.value_to_register(arg)?;
        let res_reg = self.reg_seqr.next();
        let opcode = match op {
            UnaryOpKind::Not => Either::Left(VirtualOp::NOT(res_reg.clone(), val_reg)),
        };
        self.cur_bytecode.push(Op {
            opcode,
            comment: String::new(),
            owning_span: self.md_mgr.val_to_span(self.context, *instr_val),
        });

        self.reg_map.insert(*instr_val, res_reg);
        Ok(())
    }

    fn compile_wide_unary_op(
        &mut self,
        instr_val: &Value,
        op: &UnaryOpKind,
        arg: &Value,
        result: &Value,
    ) -> Result<(), CompileError> {
        let result_reg = self.value_to_register(result)?;
        let val1_reg = self.value_to_register(arg)?;

        let opcode = match op {
            UnaryOpKind::Not => VirtualOp::WQOP(
                result_reg,
                val1_reg,
                VirtualRegister::Constant(ConstantRegister::Zero),
                VirtualImmediate06::wide_op(crate::asm_lang::WideOperations::Not, false),
            ),
        };

        self.cur_bytecode.push(Op {
            opcode: Either::Left(opcode),
            comment: String::new(),
            owning_span: self.md_mgr.val_to_span(self.context, *instr_val),
        });

        Ok(())
    }

    fn compile_wide_binary_op(
        &mut self,
        instr_val: &Value,
        op: &BinaryOpKind,
        arg1: &Value,
        arg2: &Value,
        result: &Value,
    ) -> Result<(), CompileError> {
        let result_reg = self.value_to_register(result)?;
        let val1_reg = self.value_to_register(arg1)?;
        let val2_reg = self.value_to_register(arg2)?;

        let opcode = match op {
            BinaryOpKind::Add => VirtualOp::WQOP(
                result_reg,
                val1_reg,
                val2_reg,
                VirtualImmediate06::wide_op(WideOperations::Add, true),
            ),
            BinaryOpKind::Sub => VirtualOp::WQOP(
                result_reg,
                val1_reg,
                val2_reg,
                VirtualImmediate06::wide_op(WideOperations::Sub, true),
            ),
            BinaryOpKind::And => VirtualOp::WQOP(
                result_reg,
                val1_reg,
                val2_reg,
                VirtualImmediate06::wide_op(WideOperations::And, true),
            ),
            BinaryOpKind::Or => VirtualOp::WQOP(
                result_reg,
                val1_reg,
                val2_reg,
                VirtualImmediate06::wide_op(WideOperations::Or, true),
            ),
            BinaryOpKind::Xor => VirtualOp::WQOP(
                result_reg,
                val1_reg,
                val2_reg,
                VirtualImmediate06::wide_op(WideOperations::Xor, true),
            ),
            BinaryOpKind::Lsh => VirtualOp::WQOP(
                result_reg,
                val1_reg,
                val2_reg,
                VirtualImmediate06::wide_op(WideOperations::Lsh, false),
            ),
            BinaryOpKind::Rsh => VirtualOp::WQOP(
                result_reg,
                val1_reg,
                val2_reg,
                VirtualImmediate06::wide_op(WideOperations::Rsh, false),
            ),
            BinaryOpKind::Mul => VirtualOp::WQML(
                result_reg,
                val1_reg,
                val2_reg,
                VirtualImmediate06::wide_mul(true, true),
            ),
            BinaryOpKind::Div => VirtualOp::WQDV(
                result_reg,
                val1_reg,
                val2_reg,
                VirtualImmediate06::wide_div(true),
            ),
            _ => todo!(),
        };

        self.cur_bytecode.push(Op {
            opcode: Either::Left(opcode),
            comment: String::new(),
            owning_span: self.md_mgr.val_to_span(self.context, *instr_val),
        });

        Ok(())
    }

    fn compile_wide_modular_op(
        &mut self,
        instr_val: &Value,
        op: &BinaryOpKind,
        result: &Value,
        arg1: &Value,
        arg2: &Value,
        arg3: &Value,
    ) -> Result<(), CompileError> {
        let result_reg = self.value_to_register(result)?;
        let val1_reg = self.value_to_register(arg1)?;
        let val2_reg = self.value_to_register(arg2)?;
        let val3_reg = self.value_to_register(arg3)?;

        let opcode = match op {
            BinaryOpKind::Mod => VirtualOp::WQAM(result_reg, val1_reg, val2_reg, val3_reg),
            _ => todo!(),
        };

        self.cur_bytecode.push(Op {
            opcode: Either::Left(opcode),
            comment: String::new(),
            owning_span: self.md_mgr.val_to_span(self.context, *instr_val),
        });

        Ok(())
    }

    fn compile_retd(
        &mut self,
        instr_val: &Value,
        ptr: &Value,
        len: &Value,
    ) -> Result<(), CompileError> {
        let ptr = self.value_to_register(ptr)?;
        let len = self.value_to_register(len)?;

        self.cur_bytecode.push(Op {
            opcode: Either::Left(VirtualOp::RETD(ptr, len)),
            comment: String::new(),
            owning_span: self.md_mgr.val_to_span(self.context, *instr_val),
        });

        Ok(())
    }

    fn compile_wide_cmp_op(
        &mut self,
        instr_val: &Value,
        op: &Predicate,
        arg1: &Value,
        arg2: &Value,
    ) -> Result<(), CompileError> {
        let res_reg = self.reg_seqr.next();
        let val1_reg = self.value_to_register(arg1)?;
        let val2_reg = self.value_to_register(arg2)?;

        let opcode = match op {
            Predicate::Equal => VirtualOp::WQCM(
                res_reg.clone(),
                val1_reg,
                val2_reg,
                VirtualImmediate06::wide_cmp(WideCmp::Equality, true),
            ),
            Predicate::LessThan => VirtualOp::WQCM(
                res_reg.clone(),
                val1_reg,
                val2_reg,
                VirtualImmediate06::wide_cmp(WideCmp::LessThan, true),
            ),
            Predicate::GreaterThan => VirtualOp::WQCM(
                res_reg.clone(),
                val1_reg,
                val2_reg,
                VirtualImmediate06::wide_cmp(WideCmp::GreaterThan, true),
            ),
        };

        self.cur_bytecode.push(Op {
            opcode: Either::Left(opcode),
            comment: String::new(),
            owning_span: self.md_mgr.val_to_span(self.context, *instr_val),
        });

        self.reg_map.insert(*instr_val, res_reg);

        Ok(())
    }

    fn compile_binary_op(
        &mut self,
        instr_val: &Value,
        op: &BinaryOpKind,
        arg1: &Value,
        arg2: &Value,
    ) -> Result<(), CompileError> {
        let val1_reg = self.value_to_register(arg1)?;
        let val2_reg = self.value_to_register(arg2)?;
        let res_reg = self.reg_seqr.next();
        let opcode = match op {
            BinaryOpKind::Add => Either::Left(VirtualOp::ADD(res_reg.clone(), val1_reg, val2_reg)),
            BinaryOpKind::Sub => Either::Left(VirtualOp::SUB(res_reg.clone(), val1_reg, val2_reg)),
            BinaryOpKind::Mul => Either::Left(VirtualOp::MUL(res_reg.clone(), val1_reg, val2_reg)),
            BinaryOpKind::Div => Either::Left(VirtualOp::DIV(res_reg.clone(), val1_reg, val2_reg)),
            BinaryOpKind::And => Either::Left(VirtualOp::AND(res_reg.clone(), val1_reg, val2_reg)),
            BinaryOpKind::Or => Either::Left(VirtualOp::OR(res_reg.clone(), val1_reg, val2_reg)),
            BinaryOpKind::Xor => Either::Left(VirtualOp::XOR(res_reg.clone(), val1_reg, val2_reg)),
            BinaryOpKind::Mod => Either::Left(VirtualOp::MOD(res_reg.clone(), val1_reg, val2_reg)),
            BinaryOpKind::Rsh => Either::Left(VirtualOp::SRL(res_reg.clone(), val1_reg, val2_reg)),
            BinaryOpKind::Lsh => Either::Left(VirtualOp::SLL(res_reg.clone(), val1_reg, val2_reg)),
        };
        self.cur_bytecode.push(Op {
            opcode,
            comment: String::new(),
            owning_span: self.md_mgr.val_to_span(self.context, *instr_val),
        });

        self.reg_map.insert(*instr_val, res_reg);
        Ok(())
    }

    fn compile_branch(&mut self, to_block: &BranchToWithArgs) -> Result<(), CompileError> {
        self.compile_branch_to_phi_value(to_block)?;

        let label = self.block_to_label(&to_block.block);
        self.cur_bytecode.push(Op::jump_to_label(label));

        Ok(())
    }

    fn compile_cmp(
        &mut self,
        instr_val: &Value,
        pred: &Predicate,
        lhs_value: &Value,
        rhs_value: &Value,
    ) -> Result<(), CompileError> {
        let lhs_reg = self.value_to_register(lhs_value)?;
        let rhs_reg = self.value_to_register(rhs_value)?;
        let res_reg = self.reg_seqr.next();
        let comment = String::new();
        let owning_span = self.md_mgr.val_to_span(self.context, *instr_val);
        match pred {
            Predicate::Equal => {
                self.cur_bytecode.push(Op {
                    opcode: Either::Left(VirtualOp::EQ(res_reg.clone(), lhs_reg, rhs_reg)),
                    comment,
                    owning_span,
                });
            }
            Predicate::LessThan => {
                self.cur_bytecode.push(Op {
                    opcode: Either::Left(VirtualOp::LT(res_reg.clone(), lhs_reg, rhs_reg)),
                    comment,
                    owning_span,
                });
            }
            Predicate::GreaterThan => {
                self.cur_bytecode.push(Op {
                    opcode: Either::Left(VirtualOp::GT(res_reg.clone(), lhs_reg, rhs_reg)),
                    comment,
                    owning_span,
                });
            }
        }
        self.reg_map.insert(*instr_val, res_reg);
        Ok(())
    }

    fn compile_conditional_branch(
        &mut self,
        cond_value: &Value,
        true_block: &BranchToWithArgs,
        false_block: &BranchToWithArgs,
    ) -> Result<(), CompileError> {
        if true_block.block == false_block.block && true_block.block.num_args(self.context) > 0 {
            return Err(CompileError::Internal(
                "Cannot compile CBR with both branches going to same dest block",
                self.md_mgr
                    .val_to_span(self.context, *cond_value)
                    .unwrap_or_else(Span::dummy),
            ));
        }
        self.compile_branch_to_phi_value(true_block)?;
        self.compile_branch_to_phi_value(false_block)?;

        let cond_reg = self.value_to_register(cond_value)?;

        let true_label = self.block_to_label(&true_block.block);
        self.cur_bytecode
            .push(Op::jump_if_not_zero(cond_reg, true_label));

        let false_label = self.block_to_label(&false_block.block);
        self.cur_bytecode.push(Op::jump_to_label(false_label));

        Ok(())
    }

    fn compile_branch_to_phi_value(
        &mut self,
        to_block: &BranchToWithArgs,
    ) -> Result<(), CompileError> {
        for (i, param) in to_block.args.iter().enumerate() {
            // We only need a MOVE here if param is actually assigned to a register
            if let Ok(local_reg) = self.value_to_register(param) {
                let phi_val = to_block.block.get_arg(self.context, i).unwrap();
                let phi_reg = self
                    .phi_reg_map
                    .entry(phi_val)
                    .or_insert(self.reg_seqr.next());
                self.cur_bytecode.push(Op::register_move(
                    phi_reg.clone(),
                    local_reg,
                    "parameter from branch to block argument",
                    None,
                ));
            }
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn compile_contract_call(
        &mut self,
        instr_val: &Value,
        params: &Value,
        coins: &Value,
        asset_id: &Value,
        gas: &Value,
    ) -> Result<(), CompileError> {
        let ra_pointer = self.value_to_register(params)?;
        let coins_register = self.value_to_register(coins)?;
        let asset_id_register = self.value_to_register(asset_id)?;
        let gas_register = self.value_to_register(gas)?;

        self.cur_bytecode.push(Op {
            opcode: Either::Left(VirtualOp::CALL(
                ra_pointer,
                coins_register,
                asset_id_register,
                gas_register,
            )),
            comment: "call external contract".into(),
            owning_span: self.md_mgr.val_to_span(self.context, *instr_val),
        });

        // now, move the return value of the contract call to the return register.
        // TODO validate RETL matches the expected type (this is a comment from the old codegen)
        let instr_reg = self.reg_seqr.next();
        self.cur_bytecode.push(Op::register_move(
            instr_reg.clone(),
            VirtualRegister::Constant(ConstantRegister::ReturnValue),
            "save call result",
            None,
        ));
        self.reg_map.insert(*instr_val, instr_reg);
        Ok(())
    }

    fn compile_get_elem_ptr(
        &mut self,
        instr_val: &Value,
        base_val: &Value,
        _elem_ty: &Type,
        indices: &[Value],
    ) -> Result<(), CompileError> {
        let owning_span = self.md_mgr.val_to_span(self.context, *instr_val);
        let base_type = base_val
            .get_type(self.context)
            .and_then(|ty| ty.get_pointee_type(self.context))
            .ok_or_else(|| {
                CompileError::Internal(
                    "Failed to get type of base value for GEP.",
                    owning_span.as_ref().cloned().unwrap_or_else(Span::dummy),
                )
            })?;

        // A utility lambda to unwrap Values which must be constant uints.
        let unwrap_constant_uint = |idx_val: &Value| {
            idx_val
                .get_constant(self.context)
                .and_then(|idx_const| {
                    if let ConstantValue::Uint(idx) = idx_const.value {
                        Some(idx as usize)
                    } else {
                        None
                    }
                })
                .ok_or_else(|| {
                    CompileError::Internal(
                        "Failed to convert struct or union index from constant to integer.",
                        owning_span.as_ref().cloned().unwrap_or_else(Span::dummy),
                    )
                })
        };

        // The indices for a GEP are Values.  For structs and unions they are always constant
        // uints.  For arrays they may be any value expression.  So we need to take all the
        // individual offsets and add them up.
        //
        // Ideally, most of the time, there will only be a single constant struct index.  And often
        // they will be zero, making the GEP a no-op.  But if not we need to add the non-constant
        // values together.
        //
        // Eventually this can be optimised with an ASM opt pass which can combine constant
        // ADD/ADDIs together.  Then we could just emit an ADD for every index at this stage.  But
        // until then we can keep track of the constant values and add them once.

        let base_reg = self.value_to_register(base_val)?;
        let (base_reg, const_offs, _) = indices.iter().try_fold(
            (base_reg, 0, base_type),
            |(reg, offs, elem_ty), idx_val| {
                // So we're folding to a Result, as unwrapping the constants can fail.
                // If we find a constant index then we add its offset to `offs`.  Otherwise we grab
                // its value, which should be compiled already, and add it to reg.
                if elem_ty.is_struct(self.context) {
                    // For structs the index must be a const uint.
                    unwrap_constant_uint(idx_val).map(|idx| {
                        let (field_offs_in_bytes, field_type) = elem_ty
                            .get_struct_field_offset_and_type(self.context, idx as u64)
                            .expect("Element is a struct.");
                        (reg, offs + field_offs_in_bytes, field_type)
                    })
                } else if elem_ty.is_union(self.context) {
                    // For unions the index must also be a const uint.
                    unwrap_constant_uint(idx_val).map(|idx| {
                        let (field_offs_in_bytes, field_type) = elem_ty
                            .get_union_field_offset_and_type(self.context, idx as u64)
                            .expect("Element is a union.");
                        (reg, offs + field_offs_in_bytes, field_type)
                    })
                } else if elem_ty.is_array(self.context) {
                    // For arrays the index is a value.  We need to fetch it and add it to
                    // the base.
                    let array_elem_ty =
                        elem_ty.get_array_elem_type(self.context).ok_or_else(|| {
                            CompileError::Internal(
                                "Failed to get elem type for known array.",
                                owning_span.clone().unwrap_or_else(Span::dummy),
                            )
                        })?;
                    let array_elem_size = array_elem_ty.size(self.context).in_bytes();
                    let size_reg = self.reg_seqr.next();
                    self.immediate_to_reg(
                        array_elem_size,
                        size_reg.clone(),
                        None,
                        "get size of element",
                        owning_span.clone(),
                    );

                    let index_reg = self.value_to_register(idx_val)?;
                    let offset_reg = self.reg_seqr.next();

                    self.cur_bytecode.push(Op {
                        opcode: Either::Left(VirtualOp::MUL(
                            offset_reg.clone(),
                            index_reg,
                            size_reg,
                        )),
                        comment: "get offset to array element".into(),
                        owning_span: owning_span.clone(),
                    });
                    self.cur_bytecode.push(Op {
                        opcode: Either::Left(VirtualOp::ADD(
                            offset_reg.clone(),
                            reg,
                            offset_reg.clone(),
                        )),
                        comment: "add to array base".into(),
                        owning_span: owning_span.clone(),
                    });

                    Ok((offset_reg, offs, array_elem_ty))
                } else {
                    Err(CompileError::Internal(
                        "Cannot get element offset in non-aggregate.",
                        sway_types::span::Span::dummy(),
                    ))
                }
            },
        )?;

        if const_offs == 0 {
            // No need to add anything.
            self.reg_map.insert(*instr_val, base_reg);
        } else {
            let instr_reg = self.reg_seqr.next();
            self.immediate_to_reg(
                const_offs,
                instr_reg.clone(),
                Some(&base_reg),
                "get offset to element",
                owning_span.clone(),
            );
            self.reg_map.insert(*instr_val, instr_reg);
        }

        Ok(())
    }

    fn compile_get_local(
        &mut self,
        instr_val: &Value,
        local_var: &LocalVar,
    ) -> Result<(), CompileError> {
        let owning_span = self.md_mgr.val_to_span(self.context, *instr_val);
        match self.ptr_map.get(local_var) {
            Some(Storage::Stack(word_offs)) => {
                if *word_offs == 0 {
                    self.reg_map
                        .insert(*instr_val, self.locals_base_reg().clone());
                } else {
                    let instr_reg = self.reg_seqr.next();
                    let base_reg = self.locals_base_reg().clone();
                    let byte_offs = *word_offs * 8;

                    // If the byte offset requires a data section entry, then convert the word
                    // offset to a register first (without any base). Then, multiply the result by
                    // 8 to get the byte offset. The result can then be manually added to
                    // `base_reg`.
                    //
                    // Otherwise, just convert the byte offset directly to a register.
                    if byte_offs > compiler_constants::EIGHTEEN_BITS {
                        self.immediate_to_reg(
                            *word_offs,
                            instr_reg.clone(),
                            None,
                            "get word offset to local from base",
                            owning_span.clone(),
                        );
                        self.cur_bytecode.push(Op {
                            opcode: Either::Left(VirtualOp::MULI(
                                instr_reg.clone(),
                                instr_reg.clone(),
                                VirtualImmediate12 { value: 8u16 },
                            )),
                            comment: "get byte offset to local from base".into(),
                            owning_span: owning_span.clone(),
                        });
                        self.cur_bytecode.push(Op {
                            opcode: Either::Left(VirtualOp::ADD(
                                instr_reg.clone(),
                                base_reg.clone(),
                                instr_reg.clone(),
                            )),
                            comment: "get absolute byte offset to local".into(),
                            owning_span,
                        });
                    } else {
                        self.immediate_to_reg(
                            byte_offs,
                            instr_reg.clone(),
                            Some(&base_reg),
                            "get offset to local",
                            owning_span,
                        );
                    }
                    self.reg_map.insert(*instr_val, instr_reg);
                }
                Ok(())
            }
            Some(Storage::Data(data_id)) => {
                let instr_reg = self.reg_seqr.next();
                self.cur_bytecode.push(Op {
                    opcode: Either::Left(VirtualOp::LoadDataId(instr_reg.clone(), data_id.clone())),
                    comment: "get local constant".into(),
                    owning_span,
                });
                self.reg_map.insert(*instr_val, instr_reg);

                Ok(())
            }
            _ => Err(CompileError::Internal(
                "Malformed storage for local var found.",
                self.md_mgr
                    .val_to_span(self.context, *instr_val)
                    .unwrap_or_else(Span::dummy),
            )),
        }
    }

    fn compile_gtf(
        &mut self,
        instr_val: &Value,
        index: &Value,
        tx_field_id: u64,
    ) -> Result<(), CompileError> {
        let instr_reg = self.reg_seqr.next();
        let index_reg = self.value_to_register(index)?;
        self.cur_bytecode.push(Op {
            opcode: either::Either::Left(VirtualOp::GTF(
                instr_reg.clone(),
                index_reg,
                VirtualImmediate12 {
                    value: tx_field_id as u16,
                },
            )),
            comment: "get transaction field".into(),
            owning_span: self.md_mgr.val_to_span(self.context, *instr_val),
        });
        self.reg_map.insert(*instr_val, instr_reg);
        Ok(())
    }

    fn compile_load(&mut self, instr_val: &Value, src_val: &Value) -> Result<(), CompileError> {
        let owning_span = self.md_mgr.val_to_span(self.context, *instr_val);
        let src_ty = src_val
            .get_type(self.context)
            .and_then(|src_ty| src_ty.get_pointee_type(self.context))
            .filter(|inner_ty| self.is_copy_type(inner_ty))
            .ok_or_else(|| {
                CompileError::Internal(
                    "Attempt to load from non-copy type.",
                    owning_span.clone().unwrap_or_else(Span::dummy),
                )
            })?;
        let byte_len = src_ty.size(self.context).in_bytes();

        let src_reg = self.value_to_register(src_val)?;
        let instr_reg = self.reg_seqr.next();

        match byte_len {
            1 => {
                self.cur_bytecode.push(Op {
                    opcode: Either::Left(VirtualOp::LB(
                        instr_reg.clone(),
                        src_reg,
                        VirtualImmediate12 { value: 0 },
                    )),
                    comment: "load value".into(),
                    owning_span,
                });
            }
            8.. => {
                self.cur_bytecode.push(Op {
                    opcode: Either::Left(VirtualOp::LW(
                        instr_reg.clone(),
                        src_reg,
                        VirtualImmediate12 { value: 0 },
                    )),
                    comment: "load value".into(),
                    owning_span,
                });
            }
            _ => {
                return Err(CompileError::Internal(
                    "Attempt to load {byte_len} bytes sized value.",
                    owning_span.unwrap_or_else(Span::dummy),
                ));
            }
        }

        self.reg_map.insert(*instr_val, instr_reg);
        Ok(())
    }

    fn compile_mem_copy_bytes(
        &mut self,
        instr_val: &Value,
        dst_val_ptr: &Value,
        src_val_ptr: &Value,
        byte_len: u64,
    ) -> Result<(), CompileError> {
        if byte_len == 0 {
            // A zero length MCP will revert.
            return Ok(());
        }

        let owning_span = self.md_mgr.val_to_span(self.context, *instr_val);

        let dst_reg = self.value_to_register(dst_val_ptr)?;
        let src_reg = self.value_to_register(src_val_ptr)?;

        let len_reg = self.reg_seqr.next();
        self.cur_bytecode.push(Op {
            opcode: Either::Left(VirtualOp::MOVI(
                len_reg.clone(),
                VirtualImmediate18 {
                    value: byte_len as u32,
                },
            )),
            comment: "get length for mcp".into(),
            owning_span: owning_span.clone(),
        });

        self.cur_bytecode.push(Op {
            opcode: Either::Left(VirtualOp::MCP(dst_reg, src_reg, len_reg)),
            comment: "copy memory with mem_copy".into(),
            owning_span,
        });

        Ok(())
    }

    fn compile_mem_copy_val(
        &mut self,
        instr_val: &Value,
        dst_val_ptr: &Value,
        src_val_ptr: &Value,
    ) -> Result<(), CompileError> {
        let dst_ty = dst_val_ptr
            .get_type(self.context)
            .and_then(|ptr_ty| ptr_ty.get_pointee_type(self.context))
            .ok_or_else(|| {
                CompileError::Internal(
                    "mem_copy dst type must be known and a pointer.",
                    self.md_mgr
                        .val_to_span(self.context, *instr_val)
                        .unwrap_or_else(Span::dummy),
                )
            })?;
        let byte_len = dst_ty.size(self.context).in_bytes();
        self.compile_mem_copy_bytes(instr_val, dst_val_ptr, src_val_ptr, byte_len)
    }

    fn compile_log(
        &mut self,
        instr_val: &Value,
        log_val: &Value,
        log_ty: &Type,
        log_id: &Value,
    ) -> Result<(), CompileError> {
        let owning_span = self.md_mgr.val_to_span(self.context, *instr_val);
        let log_val_reg = self.value_to_register(log_val)?;
        let log_id_reg = self.value_to_register(log_id)?;

        if !log_ty.is_ptr(self.context) {
            self.cur_bytecode.push(Op {
                owning_span,
                opcode: Either::Left(VirtualOp::LOG(
                    log_val_reg,
                    log_id_reg,
                    VirtualRegister::Constant(ConstantRegister::Zero),
                    VirtualRegister::Constant(ConstantRegister::Zero),
                )),
                comment: "".into(),
            });
        } else {
            // If the type is a pointer then we use LOGD to log the data. First put the size into
            // the data section, then add a LW to get it, then add a LOGD which uses it.
            let log_ty = log_ty.get_pointee_type(self.context).unwrap();

            // Slices arrive here as "ptr slice" because they are demoted. (see fn log_demotion)
            let is_slice = log_ty.is_slice(self.context);

            if is_slice {
                let ptr_reg = self.reg_seqr.next();
                let size_reg = self.reg_seqr.next();
                self.cur_bytecode.push(Op {
                    opcode: Either::Left(VirtualOp::LW(
                        ptr_reg.clone(),
                        log_val_reg.clone(),
                        VirtualImmediate12 { value: 0 },
                    )),
                    owning_span: owning_span.clone(),
                    comment: "load slice ptr".into(),
                });
                self.cur_bytecode.push(Op {
                    opcode: Either::Left(VirtualOp::LW(
                        size_reg.clone(),
                        log_val_reg.clone(),
                        VirtualImmediate12 { value: 1 },
                    )),
                    owning_span: owning_span.clone(),
                    comment: "load slice size".into(),
                });
                self.cur_bytecode.push(Op {
                    owning_span,
                    opcode: Either::Left(VirtualOp::LOGD(
                        VirtualRegister::Constant(ConstantRegister::Zero),
                        log_id_reg,
                        ptr_reg,
                        size_reg,
                    )),
                    comment: "log slice".into(),
                });
            } else {
                let size_in_bytes = log_ty.size(self.context).in_bytes();

                let size_reg = self.reg_seqr.next();
                self.immediate_to_reg(
                    size_in_bytes,
                    size_reg.clone(),
                    None,
                    "loading size for LOGD",
                    owning_span.clone(),
                );

                self.cur_bytecode.push(Op {
                    owning_span: owning_span.clone(),
                    opcode: Either::Left(VirtualOp::LOGD(
                        VirtualRegister::Constant(ConstantRegister::Zero),
                        log_id_reg.clone(),
                        log_val_reg.clone(),
                        size_reg,
                    )),
                    comment: "log ptr".into(),
                });
            }
        }

        Ok(())
    }

    fn compile_read_register(&mut self, instr_val: &Value, reg: &sway_ir::Register) {
        let instr_reg = self.reg_seqr.next();
        self.cur_bytecode.push(Op {
            opcode: Either::Left(VirtualOp::MOVE(
                instr_reg.clone(),
                VirtualRegister::Constant(match reg {
                    sway_ir::Register::Of => ConstantRegister::Overflow,
                    sway_ir::Register::Pc => ConstantRegister::ProgramCounter,
                    sway_ir::Register::Ssp => ConstantRegister::StackStartPointer,
                    sway_ir::Register::Sp => ConstantRegister::StackPointer,
                    sway_ir::Register::Fp => ConstantRegister::FramePointer,
                    sway_ir::Register::Hp => ConstantRegister::HeapPointer,
                    sway_ir::Register::Error => ConstantRegister::Error,
                    sway_ir::Register::Ggas => ConstantRegister::GlobalGas,
                    sway_ir::Register::Cgas => ConstantRegister::ContextGas,
                    sway_ir::Register::Bal => ConstantRegister::Balance,
                    sway_ir::Register::Is => ConstantRegister::InstructionStart,
                    sway_ir::Register::Ret => ConstantRegister::ReturnValue,
                    sway_ir::Register::Retl => ConstantRegister::ReturnLength,
                    sway_ir::Register::Flag => ConstantRegister::Flags,
                }),
            )),
            comment: "move register into abi function".to_owned(),
            owning_span: self.md_mgr.val_to_span(self.context, *instr_val),
        });

        self.reg_map.insert(*instr_val, instr_reg);
    }

    fn compile_ret_from_entry(
        &mut self,
        instr_val: &Value,
        ret_val: &Value,
        ret_type: &Type,
    ) -> Result<(), CompileError> {
        let owning_span = self.md_mgr.val_to_span(self.context, *instr_val);
        if ret_type.is_unit(self.context) {
            // Unit returns should always be zero, although because they can be omitted from
            // functions, the register is sometimes uninitialized. Manually return zero in this
            // case.
            self.cur_bytecode.push(Op {
                opcode: Either::Left(VirtualOp::RET(VirtualRegister::Constant(
                    ConstantRegister::Zero,
                ))),
                owning_span,
                comment: "returning unit as zero".into(),
            });
        } else {
            let ret_reg = self.value_to_register(ret_val)?;

            if !ret_type.is_ptr(self.context) && !ret_type.is_slice(self.context) {
                self.cur_bytecode.push(Op {
                    owning_span,
                    opcode: Either::Left(VirtualOp::RET(ret_reg)),
                    comment: "".into(),
                });
            } else {
                // Sometimes (all the time?) a slice type will be `ptr slice`.
                let ret_type = ret_type.get_pointee_type(self.context).unwrap_or(*ret_type);

                // If the type is a pointer then we use RETD to return data.
                let size_reg = self.reg_seqr.next();
                if ret_type.is_slice(self.context) {
                    // If this is a slice then return what it points to.
                    self.cur_bytecode.push(Op {
                        opcode: Either::Left(VirtualOp::LW(
                            size_reg.clone(),
                            ret_reg.clone(),
                            VirtualImmediate12 { value: 1 },
                        )),
                        owning_span: owning_span.clone(),
                        comment: "load size of returned slice".into(),
                    });
                    self.cur_bytecode.push(Op {
                        opcode: Either::Left(VirtualOp::LW(
                            ret_reg.clone(),
                            ret_reg.clone(),
                            VirtualImmediate12 { value: 0 },
                        )),
                        owning_span: owning_span.clone(),
                        comment: "load ptr of returned slice".into(),
                    });
                } else {
                    let size_in_bytes = ret_type
                        .get_pointee_type(self.context)
                        .unwrap_or(ret_type)
                        .size(self.context)
                        .in_bytes();
                    self.immediate_to_reg(
                        size_in_bytes,
                        size_reg.clone(),
                        None,
                        "get size of returned ref",
                        owning_span.clone(),
                    );
                }
                self.cur_bytecode.push(Op {
                    owning_span,
                    opcode: Either::Left(VirtualOp::RETD(ret_reg, size_reg)),
                    comment: "".into(),
                });
            }
        }

        Ok(())
    }

    fn compile_revert(
        &mut self,
        instr_val: &Value,
        revert_val: &Value,
    ) -> Result<(), CompileError> {
        let owning_span = self.md_mgr.val_to_span(self.context, *instr_val);
        let revert_reg = self.value_to_register(revert_val)?;

        self.cur_bytecode.push(Op {
            owning_span,
            opcode: Either::Left(VirtualOp::RVRT(revert_reg)),
            comment: "".into(),
        });

        Ok(())
    }

    fn compile_jmp_mem(&mut self, instr_val: &Value) -> Result<(), CompileError> {
        let owning_span = self.md_mgr.val_to_span(self.context, *instr_val);
        let target_reg = self.reg_seqr.next();
        let is_target_reg = self.reg_seqr.next();
        let by4_reg = self.reg_seqr.next();

        self.cur_bytecode.push(Op {
            owning_span: owning_span.clone(),
            opcode: Either::Left(VirtualOp::LW(
                target_reg.clone(),
                VirtualRegister::Constant(ConstantRegister::HeapPointer),
                VirtualImmediate12::new(0, Span::dummy()).unwrap(),
            )),
            comment: "jmp_mem: Load MEM[$hp]".into(),
        });
        self.cur_bytecode.push(Op {
            owning_span: owning_span.clone(),
            opcode: Either::Left(VirtualOp::SUB(
                is_target_reg.clone(),
                target_reg,
                VirtualRegister::Constant(ConstantRegister::InstructionStart),
            )),
            comment: "jmp_mem: Subtract $is since Jmp adds it back.".into(),
        });
        self.cur_bytecode.push(Op {
            owning_span: owning_span.clone(),
            opcode: Either::Left(VirtualOp::DIVI(
                by4_reg.clone(),
                is_target_reg.clone(),
                VirtualImmediate12::new(4, Span::dummy()).unwrap(),
            )),
            comment: "jmp_mem: Divide by 4 since Jmp multiplies by 4.".into(),
        });

        self.cur_bytecode.push(Op {
            owning_span,
            opcode: Either::Left(VirtualOp::JMP(by4_reg)),
            comment: "jmp_mem: Jump to computed value".into(),
        });

        Ok(())
    }

    fn compile_smo(
        &mut self,
        instr_val: &Value,
        recipient: &Value,
        message: &Value,
        message_size: &Value,
        coins: &Value,
    ) -> Result<(), CompileError> {
        let owning_span = self.md_mgr.val_to_span(self.context, *instr_val);
        let recipient_reg = self.value_to_register(recipient)?;
        let message_reg = self.value_to_register(message)?;
        let message_size_reg = self.value_to_register(message_size)?;
        let coins_reg = self.value_to_register(coins)?;

        self.cur_bytecode.push(Op {
            owning_span,
            opcode: Either::Left(VirtualOp::SMO(
                recipient_reg,
                message_reg,
                message_size_reg,
                coins_reg,
            )),
            comment: "".into(),
        });

        Ok(())
    }

    fn compile_state_clear(
        &mut self,
        instr_val: &Value,
        key: &Value,
        number_of_slots: &Value,
    ) -> Result<(), CompileError> {
        let owning_span = self.md_mgr.val_to_span(self.context, *instr_val);

        // XXX not required after we have FuelVM specific verifier.
        if !key
            .get_type(self.context)
            .map_or(true, |key_ty| key_ty.is_ptr(self.context))
        {
            return Err(CompileError::Internal(
                "Key value for state clear is not a pointer.",
                owning_span.unwrap_or_else(Span::dummy),
            ));
        }

        // Get the key pointer.
        let key_reg = self.value_to_register(key)?;

        // Capture the status of whether the slot was set before calling this instruction.
        let was_slot_set_reg = self.reg_seqr.next();

        // Number of slots to be cleared
        let number_of_slots_reg = self.value_to_register(number_of_slots)?;

        self.cur_bytecode.push(Op {
            opcode: Either::Left(VirtualOp::SCWQ(
                key_reg,
                was_slot_set_reg.clone(),
                number_of_slots_reg,
            )),
            comment: "clear a sequence of storage slots".into(),
            owning_span,
        });

        self.reg_map.insert(*instr_val, was_slot_set_reg);

        Ok(())
    }

    fn compile_state_access_quad_word(
        &mut self,
        instr_val: &Value,
        val: &Value,
        key: &Value,
        number_of_slots: &Value,
        access_type: StateAccessType,
    ) -> Result<(), CompileError> {
        let owning_span = self.md_mgr.val_to_span(self.context, *instr_val);

        // Make sure that both val and key are pointers to B256.
        // XXX not required after we have FuelVM specific verifier.
        if !val
            .get_type(self.context)
            .and_then(|val_ty| key.get_type(self.context).map(|key_ty| (val_ty, key_ty)))
            .map_or(false, |(val_ty, key_ty)| {
                val_ty.is_ptr(self.context) && key_ty.is_ptr(self.context)
            })
        {
            return Err(CompileError::Internal(
                "Val or key value for state access quad word is not a pointer.",
                owning_span.unwrap_or_else(Span::dummy),
            ));
        }

        let val_reg = self.value_to_register(val)?;
        let key_reg = self.value_to_register(key)?;
        let was_slot_set_reg = self.reg_seqr.next();
        let number_of_slots_reg = self.value_to_register(number_of_slots)?;

        self.cur_bytecode.push(Op {
            opcode: Either::Left(match access_type {
                StateAccessType::Read => VirtualOp::SRWQ(
                    val_reg,
                    was_slot_set_reg.clone(),
                    key_reg,
                    number_of_slots_reg,
                ),
                StateAccessType::Write => VirtualOp::SWWQ(
                    key_reg,
                    was_slot_set_reg.clone(),
                    val_reg,
                    number_of_slots_reg,
                ),
            }),
            comment: "access a sequence of storage slots".into(),
            owning_span,
        });

        self.reg_map.insert(*instr_val, was_slot_set_reg);

        Ok(())
    }

    fn compile_state_load_word(
        &mut self,
        instr_val: &Value,
        key: &Value,
    ) -> Result<(), CompileError> {
        let owning_span = self.md_mgr.val_to_span(self.context, *instr_val);

        // XXX not required after we have FuelVM specific verifier.
        if !key
            .get_type(self.context)
            .map_or(true, |key_ty| key_ty.is_ptr(self.context))
        {
            return Err(CompileError::Internal(
                "Key value for state load word is not a pointer.",
                owning_span.unwrap_or_else(Span::dummy),
            ));
        }

        let key_reg = self.value_to_register(key)?;
        let was_slot_set_reg = self.reg_seqr.next();
        let load_reg = self.reg_seqr.next();

        self.cur_bytecode.push(Op {
            opcode: Either::Left(VirtualOp::SRW(load_reg.clone(), was_slot_set_reg, key_reg)),
            comment: "single word state access".into(),
            owning_span,
        });

        self.reg_map.insert(*instr_val, load_reg);

        Ok(())
    }

    fn compile_state_store_word(
        &mut self,
        instr_val: &Value,
        store_val: &Value,
        key: &Value,
    ) -> Result<(), CompileError> {
        let owning_span = self.md_mgr.val_to_span(self.context, *instr_val);

        // XXX not required after we have FuelVM specific verifier.
        if !store_val
            .get_type(self.context)
            .and_then(|val_ty| key.get_type(self.context).map(|key_ty| (val_ty, key_ty)))
            .map_or(false, |(val_ty, key_ty)| {
                val_ty.is_uint64(self.context) && key_ty.is_ptr(self.context)
            })
        {
            return Err(CompileError::Internal(
                "Val or key value for state store word is not a pointer.",
                owning_span.unwrap_or_else(Span::dummy),
            ));
        }

        let store_reg = self.value_to_register(store_val)?;
        let key_reg = self.value_to_register(key)?;
        let was_slot_set_reg = self.reg_seqr.next();

        self.cur_bytecode.push(Op {
            opcode: Either::Left(VirtualOp::SWW(key_reg, was_slot_set_reg.clone(), store_reg)),
            comment: "single word state access".into(),
            owning_span,
        });

        self.reg_map.insert(*instr_val, was_slot_set_reg);

        Ok(())
    }

    fn compile_store(
        &mut self,
        instr_val: &Value,
        dst_val: &Value,
        stored_val: &Value,
    ) -> Result<(), CompileError> {
        let owning_span = self.md_mgr.val_to_span(self.context, *instr_val);

        if stored_val
            .get_type(self.context)
            .map_or(true, |ty| !self.is_copy_type(&ty))
        {
            // NOTE: Very hacky special case here which must be fixed.  We've been given a
            // configurable constant which doesn't have a pointer type and shouldn't still be using
            // `store`.
            if stored_val.is_configurable(self.context) {
                // So we know it's not a copy type so we actually need a MCP.
                self.compile_mem_copy_val(instr_val, dst_val, stored_val)
            } else {
                Err(CompileError::Internal(
                    "Attempt to store a non-copy type.",
                    owning_span.unwrap_or_else(Span::dummy),
                ))
            }
        } else {
            let stored_ty = stored_val.get_type(self.context).unwrap();
            let byte_len = stored_ty.size(self.context).in_bytes();

            let dst_reg = self.value_to_register(dst_val)?;
            let val_reg = self.value_to_register(stored_val)?;

            match byte_len {
                1 => {
                    self.cur_bytecode.push(Op {
                        opcode: Either::Left(VirtualOp::SB(
                            dst_reg,
                            val_reg,
                            VirtualImmediate12 { value: 0 },
                        )),
                        comment: "store value".into(),
                        owning_span,
                    });
                }
                8.. => {
                    self.cur_bytecode.push(Op {
                        opcode: Either::Left(VirtualOp::SW(
                            dst_reg,
                            val_reg,
                            VirtualImmediate12 { value: 0 },
                        )),
                        comment: "store value".into(),
                        owning_span,
                    });
                }
                _ => {
                    return Err(CompileError::Internal(
                        "Attempt to load {byte_len} bytes sized value.",
                        owning_span.unwrap_or_else(Span::dummy),
                    ));
                }
            }

            Ok(())
        }
    }

    fn compile_no_op_move(
        &mut self,
        instr_val: &Value,
        rhs_val: &Value,
    ) -> Result<(), CompileError> {
        // For cast_ptr, int_to_ptr, ptr_to_int, etc. these are NOPs and just need updates to the
        // register map.
        self.value_to_register(rhs_val).map(|val_reg| {
            self.reg_map.insert(*instr_val, val_reg);
        })
    }

    // ---------------------------------------------------------------------------------------------

    // TODO-IG: Reassess all the places we use `is_copy_type`.
    pub(crate) fn is_copy_type(&self, ty: &Type) -> bool {
        ty.is_unit(self.context)
            || ty.is_never(self.context)
            || ty.is_bool(self.context)
            || ty
                .get_uint_width(self.context)
                .map(|x| x < 256)
                .unwrap_or(false)
    }

    fn initialise_constant(
        &mut self,
        constant: &Constant,
        config_name: Option<String>,
        span: Option<Span>,
    ) -> (VirtualRegister, Option<DataId>) {
        match &constant.value {
            // Use cheaper $zero or $one registers if possible.
            ConstantValue::Unit | ConstantValue::Bool(false) | ConstantValue::Uint(0)
                if config_name.is_none() =>
            {
                (VirtualRegister::Constant(ConstantRegister::Zero), None)
            }

            ConstantValue::Uint(1) if config_name.is_none() => {
                (VirtualRegister::Constant(ConstantRegister::One), None)
            }

            _otherwise => {
                // Get the constant into the namespace.
                let entry = Entry::from_constant(self.context, constant, config_name, None);
                let data_id = self.data_section.insert_data_value(entry);

                // Allocate a register for it, and a load instruction.
                let reg = self.reg_seqr.next();
                self.cur_bytecode.push(Op {
                    opcode: either::Either::Left(VirtualOp::LoadDataId(
                        reg.clone(),
                        data_id.clone(),
                    )),
                    comment: "literal instantiation".into(),
                    owning_span: span,
                });
                (reg, Some(data_id))
            }
        }

        // Insert the value into the map.
        //self.reg_map.insert(*value, reg.clone());
        //
        // Actually, no, don't.  It's possible for constant values to be
        // reused in the IR, especially with transforms which copy blocks
        // around, like inlining.  The `LW`/`LoadDataId` instruction above
        // initialises that constant value but it may be in a conditional
        // block and not actually get evaluated for every possible
        // execution. So using the register later on by pulling it from
        // `self.reg_map` will have a potentially uninitialised register.
        //
        // By not putting it in the map we recreate the `LW` each time it's
        // used, which also isn't ideal.  A better solution is to put this
        // initialisation into the IR itself, and allow for analysis there
        // to determine when it may be initialised and/or reused.
    }

    // Get the reg corresponding to `value`. Returns an ICE if the value is not in reg_map or is
    // not a constant.
    pub(super) fn value_to_register(
        &mut self,
        value: &Value,
    ) -> Result<VirtualRegister, CompileError> {
        self.reg_map
            .get(value)
            .cloned()
            .or_else(|| {
                value.get_constant(self.context).map(|constant| {
                    let span = self.md_mgr.val_to_span(self.context, *value);
                    self.initialise_constant(constant, None, span).0
                })
            })
            .or_else(|| {
                value.get_configurable(self.context).map(|constant| {
                    let span = self.md_mgr.val_to_span(self.context, *value);
                    let config_name = self
                        .md_mgr
                        .md_to_config_const_name(self.context, value.get_metadata(self.context))
                        .unwrap()
                        .to_string();

                    let initialized =
                        self.initialise_constant(constant, Some(config_name.clone()), span);
                    if let Some(data_id) = initialized.1 {
                        self.data_section.config_map.insert(config_name, data_id.0);
                    }
                    initialized.0
                })
            })
            .ok_or_else(|| {
                CompileError::Internal(
                    "An attempt to get register for unknown Value.",
                    Span::dummy(),
                )
            })
    }

    pub(super) fn immediate_to_reg<S: Into<String>>(
        &mut self,
        imm: u64,
        reg: VirtualRegister,
        base: Option<&VirtualRegister>,
        comment: S,
        span: Option<Span>,
    ) {
        // We have a few different options here.
        // - If we're given a base to add to and the immediate is small enough we can use ADDI.
        // - If the immediate is too big for that then we need to MOVI and ADD.
        // - If the immediate is very big then we LW and ADD.
        // XXX This can be done with peephole optimisations when we get them.
        if imm <= compiler_constants::TWELVE_BITS && base.is_some() {
            self.cur_bytecode.push(Op {
                opcode: Either::Left(VirtualOp::ADDI(
                    reg,
                    #[allow(clippy::unnecessary_unwrap)]
                    base.unwrap().clone(),
                    VirtualImmediate12 { value: imm as u16 },
                )),
                comment: comment.into(),
                owning_span: span,
            });
        } else if imm <= compiler_constants::EIGHTEEN_BITS {
            let comment = comment.into();
            self.cur_bytecode.push(Op {
                opcode: Either::Left(VirtualOp::MOVI(
                    reg.clone(),
                    VirtualImmediate18 { value: imm as u32 },
                )),
                comment: comment.clone(),
                owning_span: span.clone(),
            });
            if let Some(base_reg) = base {
                self.cur_bytecode.push(Op {
                    opcode: Either::Left(VirtualOp::ADD(reg.clone(), base_reg.clone(), reg)),
                    comment,
                    owning_span: span,
                });
            }
        } else {
            let comment = comment.into();
            let data_id = self
                .data_section
                .insert_data_value(Entry::new_word(imm, None, None));
            self.cur_bytecode.push(Op {
                opcode: Either::Left(VirtualOp::LoadDataId(reg.clone(), data_id)),
                owning_span: span.clone(),
                comment: comment.clone(),
            });
            if let Some(base_reg) = base {
                self.cur_bytecode.push(Op {
                    opcode: Either::Left(VirtualOp::ADD(reg.clone(), base_reg.clone(), reg)),
                    comment,
                    owning_span: span,
                });
            }
        }
    }

    pub(super) fn func_to_labels(&mut self, func: &Function) -> (Label, Label) {
        self.func_label_map.get(func).cloned().unwrap_or_else(|| {
            let labels = (self.reg_seqr.get_label(), self.reg_seqr.get_label());
            self.func_label_map.insert(*func, labels);
            labels
        })
    }

    pub(super) fn block_to_label(&mut self, block: &Block) -> Label {
        self.block_label_map.get(block).cloned().unwrap_or_else(|| {
            let label = self.reg_seqr.get_label();
            self.block_label_map.insert(*block, label);
            label
        })
    }
}
