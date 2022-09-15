// =================================================================================================
// Newer IR code gen.
//
// NOTE:  This is converting IR to Vec<Op> first, and then to finalized VM bytecode much like the
// original code.  This is to keep things simple, and to reuse the current tools like DataSection.
//
// But this is not ideal and needs to be refactored:
// - AsmNamespace is tied to data structures from other stages like Ident and Literal.

use fuel_asm::GTFArgs;
use fuel_crypto::Hasher;
use std::{collections::HashMap, sync::Arc};

use crate::{
    asm_generation::{
        build_contract_abi_switch, build_preamble, checks::check_invalid_opcodes,
        compiler_constants, finalized_asm::FinalizedAsm, register_sequencer::RegisterSequencer,
        AbstractInstructionSet, DataId, DataSection, SwayAsmSet,
    },
    asm_lang::{
        virtual_register::*, Label, Op, VirtualImmediate12, VirtualImmediate18, VirtualImmediate24,
        VirtualOp,
    },
    error::*,
    metadata::MetadataManager,
    parse_tree::Literal,
    BuildConfig,
};

use sway_ir::*;
use sway_types::{span::Span, Spanned};

use either::Either;

pub fn compile_ir_to_asm(
    ir: &Context,
    build_config: Option<&BuildConfig>,
) -> CompileResult<FinalizedAsm> {
    let mut warnings: Vec<CompileWarning> = Vec::new();
    let mut errors: Vec<CompileError> = Vec::new();

    let mut reg_seqr = RegisterSequencer::new();
    let mut bytecode: Vec<Op> = build_preamble(&mut reg_seqr).to_vec();

    // Eventually when we get this 'correct' with no hacks we'll want to compile all the modules
    // separately and then use a linker to connect them.  This way we could also keep binary caches
    // of libraries and link against them, rather than recompile everything each time.
    assert!(ir.module_iter().count() == 1);
    let module = ir.module_iter().next().unwrap();
    let (data_section, mut ops, mut reg_seqr) = check!(
        compile_module_to_asm(reg_seqr, ir, module),
        return err(warnings, errors),
        warnings,
        errors
    );
    bytecode.append(&mut ops);

    let asm = match module.get_kind(ir) {
        Kind::Script => SwayAsmSet::ScriptMain {
            program_section: AbstractInstructionSet { ops: bytecode },
            data_section,
        },
        Kind::Contract => SwayAsmSet::ContractAbi {
            program_section: AbstractInstructionSet { ops: bytecode },
            data_section,
        },
        Kind::Library | Kind::Predicate => todo!("libraries and predicates coming soon!"),
    };

    if build_config
        .map(|cfg| cfg.print_intermediate_asm)
        .unwrap_or(false)
    {
        tracing::info!("{}", asm);
    }

    let finalized_asm = asm
        .remove_unnecessary_jumps()
        .allocate_registers(&mut reg_seqr)
        .optimize();

    if build_config
        .map(|cfg| cfg.print_finalized_asm)
        .unwrap_or(false)
    {
        tracing::info!("{}", finalized_asm);
    }

    check!(
        check_invalid_opcodes(&finalized_asm),
        return err(warnings, errors),
        warnings,
        errors
    );

    ok(finalized_asm, warnings, errors)
}

fn compile_module_to_asm(
    reg_seqr: RegisterSequencer,
    context: &Context,
    module: Module,
) -> CompileResult<(DataSection, Vec<Op>, RegisterSequencer)> {
    let mut builder = AsmBuilder::new(DataSection::default(), reg_seqr, context);
    match module.get_kind(context) {
        Kind::Script => {
            // We can't do function calls yet, so we expect everything to be inlined into `main`.
            let function = module
                .function_iter(context)
                .find(|func| &context.functions[func.0].name == "main")
                .expect("Can't find main function!");
            builder
                .compile_function(function)
                .flat_map(|_| builder.finalize())
        }
        Kind::Contract => {
            let mut warnings = Vec::new();
            let mut errors = Vec::new();

            let mut selectors_and_labels: Vec<([u8; 4], Label)> = Vec::new();

            // Compile only the functions which have selectors and gather the selectors and labels.
            for function in module.function_iter(context) {
                if function.has_selector(context) {
                    let selector = function.get_selector(context).unwrap();
                    let label = builder.add_label();
                    check!(
                        builder.compile_function(function),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    selectors_and_labels.push((selector, label));
                }
            }
            let (mut data_section, mut funcs_bytecode, mut reg_seqr) = check!(
                builder.finalize(),
                return err(warnings, errors),
                warnings,
                errors
            );

            let mut bytecode_with_switch =
                build_contract_abi_switch(&mut reg_seqr, &mut data_section, selectors_and_labels);
            bytecode_with_switch.append(&mut funcs_bytecode);
            ok(
                (data_section, bytecode_with_switch, reg_seqr),
                warnings,
                errors,
            )
        }
        Kind::Library | Kind::Predicate => todo!("libraries and predicates coming soon!"),
    }
}

// -------------------------------------------------------------------------------------------------

macro_rules! size_bytes_in_words {
    ($bytes_expr: expr) => {
        ($bytes_expr + 7) / 8
    };
}

// This is a mouthful...
macro_rules! size_bytes_round_up_to_word_alignment {
    ($bytes_expr: expr) => {
        ($bytes_expr + 7) - (($bytes_expr + 7) % 8)
    };
}

struct AsmBuilder<'ir> {
    // Data section is used by the rest of code gen to layout const memory.
    data_section: DataSection,

    // Register sequencer dishes out new registers and labels.
    reg_seqr: RegisterSequencer,

    // Label map is from IR block to label name.
    label_map: HashMap<Block, Label>,

    // Reg map is tracking IR values to VM values.  Ptr map is tracking IR pointers to local
    // storage types.
    reg_map: HashMap<Value, VirtualRegister>,
    ptr_map: HashMap<Pointer, Storage>,

    // Stack base register, copied from $SP at the start, but only if we have stack storage.
    stack_base_reg: Option<VirtualRegister>,

    // IR context we're compiling.
    context: &'ir Context,

    // Metadata manager for converting metadata to Spans, etc.
    md_mgr: MetadataManager,

    // Final resulting VM bytecode ops.
    bytecode: Vec<Op>,
}

// NOTE: For stack storage we need to be aware:
// - sizes are in bytes; CFEI reserves in bytes.
// - offsets are in 64-bit words; LW/SW reads/writes to word offsets. XXX Wrap in a WordOffset struct.

#[derive(Clone, Debug)]
pub(super) enum Storage {
    Data(DataId), // Const storage in the data section.
    Stack(u64), // Storage in the runtime stack starting at an absolute word offset.  Essentially a global.
}

pub enum StateAccessType {
    Read,
    Write,
}

impl<'ir> AsmBuilder<'ir> {
    fn new(data_section: DataSection, reg_seqr: RegisterSequencer, context: &'ir Context) -> Self {
        AsmBuilder {
            data_section,
            reg_seqr,
            label_map: HashMap::new(),
            reg_map: HashMap::new(),
            ptr_map: HashMap::new(),
            stack_base_reg: None,
            context,
            md_mgr: MetadataManager::default(),
            bytecode: Vec::new(),
        }
    }

    // This is here temporarily for in the case when the IR can't absolutely provide a valid span,
    // until we can improve ASM block parsing and verification mostly. It's where it's needed the
    // most, for returning failure errors.  If we move ASM verification to the parser and semantic
    // analysis then ASM block conversion shouldn't/can't fail and we won't need to provide a
    // guaranteed to be available span.
    fn empty_span() -> Span {
        let msg = "unknown source location";
        Span::new(Arc::from(msg), 0, msg.len(), None).unwrap()
    }

    // Handle loading the arguments of a contract call
    fn compile_fn_args(&mut self, function: Function) {
        // We treat contract methods differently. Contract methods have selectors.
        let is_contract_method = function.has_selector(self.context);

        match function.args_iter(self.context).count() {
            // Nothing to do if there are no arguments
            0 => (),

            // A special case for when there's only a single arg, its value (or address) is placed
            // directly in the base register.
            1 => {
                let (_, val) = function.args_iter(self.context).next().unwrap();
                let single_arg_reg = self.value_to_register(val);

                if is_contract_method {
                    self.read_args_value_from_frame(&single_arg_reg);
                } else {
                    self.read_args_value_from_script_data(&single_arg_reg);

                    if val.get_type(self.context).unwrap().is_copy_type() {
                        self.bytecode.push(Op {
                            opcode: either::Either::Left(VirtualOp::LW(
                                single_arg_reg.clone(),
                                single_arg_reg.clone(),
                                VirtualImmediate12 { value: 0 },
                            )),
                            comment: "Load main fn parameter".into(),
                            owning_span: None,
                        });
                    }
                }
            }

            // Otherwise, the args are bundled together and pointed to by the base register.
            _ => {
                let args_base_reg = self.reg_seqr.next();

                if is_contract_method {
                    self.read_args_value_from_frame(&args_base_reg);
                } else {
                    self.read_args_value_from_script_data(&args_base_reg);
                }

                // Successively load each argument. The asm generated depends on the arg type size
                // and whether the offset fits in a 12-bit immediate.
                let mut arg_word_offset = 0;
                for (name, val) in function.args_iter(self.context) {
                    let current_arg_reg = self.value_to_register(val);
                    let arg_type = val.get_type(self.context).unwrap();
                    let arg_type_size_bytes = ir_type_size_in_bytes(self.context, &arg_type);
                    if arg_type.is_copy_type() {
                        if arg_word_offset > compiler_constants::TWELVE_BITS {
                            let offs_reg = self.reg_seqr.next();
                            self.bytecode.push(Op {
                                opcode: Either::Left(VirtualOp::ADD(
                                    args_base_reg.clone(),
                                    args_base_reg.clone(),
                                    offs_reg.clone(),
                                )),
                                comment: format!("Get offset for arg {}", name),
                                owning_span: None,
                            });
                            self.bytecode.push(Op {
                                opcode: Either::Left(VirtualOp::LW(
                                    current_arg_reg.clone(),
                                    offs_reg,
                                    VirtualImmediate12 { value: 0 },
                                )),
                                comment: format!("Get arg {}", name),
                                owning_span: None,
                            });
                        } else {
                            self.bytecode.push(Op {
                                opcode: Either::Left(VirtualOp::LW(
                                    current_arg_reg.clone(),
                                    args_base_reg.clone(),
                                    VirtualImmediate12 {
                                        value: arg_word_offset as u16,
                                    },
                                )),
                                comment: format!("Get arg {}", name),
                                owning_span: None,
                            });
                        }
                    } else if arg_word_offset * 8 > compiler_constants::TWELVE_BITS {
                        let offs_reg = self.reg_seqr.next();
                        self.number_to_reg(arg_word_offset * 8, &offs_reg, None);
                        self.bytecode.push(Op {
                            opcode: either::Either::Left(VirtualOp::ADD(
                                current_arg_reg.clone(),
                                args_base_reg.clone(),
                                offs_reg,
                            )),
                            comment: format!("Get offset or arg {}", name),
                            owning_span: None,
                        });
                    } else {
                        self.bytecode.push(Op {
                            opcode: either::Either::Left(VirtualOp::ADDI(
                                current_arg_reg.clone(),
                                args_base_reg.clone(),
                                VirtualImmediate12 {
                                    value: (arg_word_offset * 8) as u16,
                                },
                            )),
                            comment: format!("Get address for arg {}", name),
                            owning_span: None,
                        });
                    }

                    arg_word_offset += size_bytes_in_words!(arg_type_size_bytes);
                }
            }
        }
    }

    // Read the argument(s) base from the call frame.
    fn read_args_value_from_frame(&mut self, reg: &VirtualRegister) {
        self.bytecode.push(Op {
            opcode: Either::Left(VirtualOp::LW(
                reg.clone(),
                VirtualRegister::Constant(ConstantRegister::FramePointer),
                // see https://github.com/FuelLabs/fuel-specs/pull/193#issuecomment-876496372
                VirtualImmediate12 { value: 74 },
            )),
            comment: "Base register for method parameter".into(),
            owning_span: None,
        });
    }

    // Read the argument(s) base from the script data.
    fn read_args_value_from_script_data(&mut self, reg: &VirtualRegister) {
        self.bytecode.push(Op {
            opcode: either::Either::Left(VirtualOp::GTF(
                reg.clone(),
                VirtualRegister::Constant(ConstantRegister::Zero),
                VirtualImmediate12 {
                    value: GTFArgs::ScriptData as u16,
                },
            )),
            comment: "Base register for main fn parameter".into(),
            owning_span: None,
        });
    }

    fn add_locals(&mut self, function: Function) {
        // If they're immutable and have a constant initialiser then they go in the data section.
        // Otherwise they go in runtime allocated space, either a register or on the stack.
        //
        // Stack offsets are in words to both enforce alignment and simplify use with LW/SW.
        let mut stack_base = 0_u64;
        for (_name, ptr) in function.locals_iter(self.context) {
            let ptr_content = &self.context.pointers[ptr.0];
            if !ptr_content.is_mutable && ptr_content.initializer.is_some() {
                let constant = ptr_content.initializer.as_ref().unwrap();
                let lit = ir_constant_to_ast_literal(constant);
                let data_id = self.data_section.insert_data_value(&lit);
                self.ptr_map.insert(*ptr, Storage::Data(data_id));
            } else {
                match ptr_content.ty {
                    Type::Unit | Type::Bool | Type::Uint(_) | Type::Pointer(_) => {
                        self.ptr_map.insert(*ptr, Storage::Stack(stack_base));
                        stack_base += 1;
                    }
                    Type::B256 => {
                        // XXX Like strings, should we just reserve space for a pointer?
                        self.ptr_map.insert(*ptr, Storage::Stack(stack_base));
                        stack_base += 4;
                    }
                    Type::String(n) => {
                        // Strings are always constant and used by reference, so we only store the
                        // pointer on the stack.
                        self.ptr_map.insert(*ptr, Storage::Stack(stack_base));
                        stack_base += size_bytes_round_up_to_word_alignment!(n)
                    }
                    Type::Array(_) | Type::Struct(_) | Type::Union(_) => {
                        // Store this aggregate at the current stack base.
                        self.ptr_map.insert(*ptr, Storage::Stack(stack_base));

                        // Reserve space by incrementing the base.
                        stack_base += size_bytes_in_words!(ir_type_size_in_bytes(
                            self.context,
                            &ptr_content.ty
                        ));
                    }
                };
            }
        }

        // Reserve space on the stack for ALL our locals which require it.
        if !self.ptr_map.is_empty() {
            let base_reg = self.reg_seqr.next();
            self.bytecode.push(Op::unowned_register_move_comment(
                base_reg.clone(),
                VirtualRegister::Constant(ConstantRegister::StackPointer),
                "save locals base register",
            ));

            // It's possible (though undesirable) to have empty local data structures only.
            if stack_base != 0 {
                if stack_base * 8 > compiler_constants::TWENTY_FOUR_BITS {
                    todo!("Enormous stack usage for locals.");
                }
                let mut alloc_op = Op::unowned_stack_allocate_memory(VirtualImmediate24 {
                    value: (stack_base * 8) as u32,
                });
                alloc_op.comment = format!("allocate {} bytes for all locals", stack_base * 8);
                self.bytecode.push(alloc_op);
            }
            self.stack_base_reg = Some(base_reg);
        }
    }

    fn add_block_label(&mut self, block: Block) {
        if &block.get_label(self.context) != "entry" {
            let label = self.block_to_label(&block);
            self.bytecode.push(Op::unowned_jump_label(label))
        }
    }

    fn add_label(&mut self) -> Label {
        let label = self.reg_seqr.get_label();
        self.bytecode.push(Op::unowned_jump_label(label.clone()));
        label
    }

    fn finalize(self) -> CompileResult<(DataSection, Vec<Op>, RegisterSequencer)> {
        // XXX Assuming no warnings...
        ok(
            (self.data_section, self.bytecode, self.reg_seqr),
            Vec::new(),
            Vec::new(),
        )
    }

    fn compile_function(&mut self, function: Function) -> CompileResult<()> {
        if function.has_selector(self.context) {
            // Add a comment noting that this is a named contract method.
            self.bytecode.push(Op::new_comment(format!(
                "contract method: {}, selector: 0x{}",
                function.get_name(self.context),
                function
                    .get_selector(self.context)
                    .unwrap()
                    .into_iter()
                    .map(|b| format!("{b:02x}"))
                    .collect::<String>()
            )));
        }

        // Compile instructions.
        self.add_locals(function);
        self.compile_fn_args(function);
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        for block in function.block_iter(self.context) {
            self.add_block_label(block);
            for instr_val in block.instruction_iter(self.context) {
                check!(
                    self.compile_instruction(&block, &instr_val),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
            }
        }
        ok((), warnings, errors)
    }

    fn compile_instruction(&mut self, block: &Block, instr_val: &Value) -> CompileResult<()> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        if let ValueDatum::Instruction(instruction) = &self.context.values[instr_val.0].value {
            match instruction {
                Instruction::AddrOf(arg) => self.compile_addr_of(instr_val, arg),
                Instruction::AsmBlock(asm, args) => {
                    check!(
                        self.compile_asm_block(instr_val, asm, args),
                        return err(warnings, errors),
                        warnings,
                        errors
                    )
                }
                Instruction::BitCast(val, ty) => self.compile_bitcast(instr_val, val, ty),
                Instruction::BinaryOp { op, arg1, arg2 } => {
                    self.compile_binary_op(instr_val, op, arg1, arg2)
                }
                Instruction::Branch(to_block) => self.compile_branch(block, to_block),
                Instruction::Call(..) => {
                    errors.push(CompileError::Internal(
                        "Calls are not yet supported.",
                        self.md_mgr
                            .val_to_span(self.context, *instr_val)
                            .unwrap_or_else(Self::empty_span),
                    ));
                    return err(warnings, errors);
                }
                Instruction::Cmp(pred, lhs_value, rhs_value) => {
                    self.compile_cmp(instr_val, pred, lhs_value, rhs_value)
                }
                Instruction::ConditionalBranch {
                    cond_value,
                    true_block,
                    false_block,
                } => self.compile_conditional_branch(cond_value, block, true_block, false_block),
                Instruction::ContractCall {
                    params,
                    coins,
                    asset_id,
                    gas,
                    ..
                } => self.compile_contract_call(instr_val, params, coins, asset_id, gas),
                Instruction::ExtractElement {
                    array,
                    ty,
                    index_val,
                } => self.compile_extract_element(instr_val, array, ty, index_val),
                Instruction::ExtractValue {
                    aggregate, indices, ..
                } => self.compile_extract_value(instr_val, aggregate, indices),
                Instruction::GetStorageKey => {
                    check!(
                        self.compile_get_storage_key(instr_val),
                        return err(warnings, errors),
                        warnings,
                        errors
                    )
                }
                Instruction::GetPointer {
                    base_ptr,
                    ptr_ty,
                    offset,
                } => self.compile_get_pointer(instr_val, base_ptr, ptr_ty, *offset),
                Instruction::Gtf { index, tx_field_id } => {
                    self.compile_gtf(instr_val, index, *tx_field_id)
                }
                Instruction::InsertElement {
                    array,
                    ty,
                    value,
                    index_val,
                } => self.compile_insert_element(instr_val, array, ty, value, index_val),
                Instruction::InsertValue {
                    aggregate,
                    value,
                    indices,
                    ..
                } => self.compile_insert_value(instr_val, aggregate, value, indices),
                Instruction::IntToPtr(val, _) => self.compile_int_to_ptr(instr_val, val),
                Instruction::Load(src_val) => check!(
                    self.compile_load(instr_val, src_val),
                    return err(warnings, errors),
                    warnings,
                    errors
                ),
                Instruction::Log {
                    log_val,
                    log_ty,
                    log_id,
                } => self.compile_log(instr_val, log_val, log_ty, log_id),
                Instruction::Nop => (),
                Instruction::Phi(_) => (), // Managing the phi value is done in br and cbr compilation.
                Instruction::ReadRegister(reg) => self.compile_read_register(instr_val, reg),
                Instruction::Ret(ret_val, ty) => self.compile_ret(instr_val, ret_val, ty),
                Instruction::StateLoadQuadWord { load_val, key } => check!(
                    self.compile_state_access_quad_word(
                        instr_val,
                        load_val,
                        key,
                        StateAccessType::Read
                    ),
                    return err(warnings, errors),
                    warnings,
                    errors
                ),
                Instruction::StateLoadWord(key) => check!(
                    self.compile_state_load_word(instr_val, key),
                    return err(warnings, errors),
                    warnings,
                    errors
                ),
                Instruction::StateStoreQuadWord { stored_val, key } => check!(
                    self.compile_state_access_quad_word(
                        instr_val,
                        stored_val,
                        key,
                        StateAccessType::Write
                    ),
                    return err(warnings, errors),
                    warnings,
                    errors
                ),
                Instruction::StateStoreWord { stored_val, key } => check!(
                    self.compile_state_store_word(instr_val, stored_val, key),
                    return err(warnings, errors),
                    warnings,
                    errors
                ),
                Instruction::Store {
                    dst_val,
                    stored_val,
                } => check!(
                    self.compile_store(instr_val, dst_val, stored_val),
                    return err(warnings, errors),
                    warnings,
                    errors
                ),
            }
        } else {
            errors.push(CompileError::Internal(
                "Value not an instruction.",
                self.md_mgr
                    .val_to_span(self.context, *instr_val)
                    .unwrap_or_else(Self::empty_span),
            ));
        }
        ok((), warnings, errors)
    }

    // OK, I began by trying to translate the IR ASM block data structures back into AST data
    // structures which I could feed to the code in asm_generation/expression/mod.rs where it
    // compiles the inline ASM.  But it's more work to do that than to just re-implement that
    // algorithm with the IR data here.

    fn compile_asm_block(
        &mut self,
        instr_val: &Value,
        asm: &AsmBlock,
        asm_args: &[AsmArg],
    ) -> CompileResult<()> {
        let mut warnings: Vec<CompileWarning> = Vec::new();
        let mut errors: Vec<CompileError> = Vec::new();
        let mut inline_reg_map = HashMap::new();
        let mut inline_ops = Vec::new();
        for AsmArg { name, initializer } in asm_args {
            assert_or_warn!(
                ConstantRegister::parse_register_name(name.as_str()).is_none(),
                warnings,
                name.span().clone(),
                Warning::ShadowingReservedRegister {
                    reg_name: name.clone()
                }
            );
            let arg_reg = initializer
                .map(|init_val| self.value_to_register(&init_val))
                .unwrap_or_else(|| self.reg_seqr.next());
            inline_reg_map.insert(name.as_str(), arg_reg);
        }

        let realize_register = |reg_name: &str| {
            inline_reg_map.get(reg_name).cloned().or_else(|| {
                ConstantRegister::parse_register_name(reg_name).map(&VirtualRegister::Constant)
            })
        };

        // For each opcode in the asm expression, attempt to parse it into an opcode and
        // replace references to the above registers with the newly allocated ones.
        let asm_block = &self.context.asm_blocks[asm.0];
        for op in &asm_block.body {
            let replaced_registers = op
                .args
                .iter()
                .map(|reg_name| -> Result<_, CompileError> {
                    realize_register(reg_name.as_str()).ok_or_else(|| {
                        CompileError::UnknownRegister {
                            span: reg_name.span(),
                            initialized_registers: inline_reg_map
                                .iter()
                                .map(|(name, _)| *name)
                                .collect::<Vec<_>>()
                                .join("\n"),
                        }
                    })
                })
                .filter_map(|res| match res {
                    Err(e) => {
                        errors.push(e);
                        None
                    }
                    Ok(o) => Some(o),
                })
                .collect::<Vec<VirtualRegister>>();

            // Parse the actual op and registers.
            let op_span = self
                .md_mgr
                .md_to_span(self.context, op.metadata)
                .unwrap_or_else(Self::empty_span);
            let opcode = check!(
                Op::parse_opcode(
                    &op.name,
                    &replaced_registers,
                    &op.immediate,
                    op_span.clone(),
                ),
                return err(warnings, errors),
                warnings,
                errors
            );

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
                    errors.push(CompileError::UnknownRegister {
                        initialized_registers: inline_reg_map
                            .iter()
                            .map(|(name, _)| name.to_string())
                            .collect::<Vec<_>>()
                            .join("\n"),
                        span: ret_reg_name.span(),
                    });
                    return err(warnings, errors);
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

        self.bytecode.append(&mut inline_ops);

        ok((), warnings, errors)
    }

    fn compile_addr_of(&mut self, instr_val: &Value, arg: &Value) {
        let reg = self.value_to_register(arg);
        self.reg_map.insert(*instr_val, reg);
    }

    fn compile_bitcast(&mut self, instr_val: &Value, bitcast_val: &Value, to_type: &Type) {
        let val_reg = self.value_to_register(bitcast_val);
        let reg = if let Type::Bool = to_type {
            // This may not be necessary if we just treat a non-zero value as 'true'.
            let res_reg = self.reg_seqr.next();
            self.bytecode.push(Op {
                opcode: Either::Left(VirtualOp::EQ(
                    res_reg.clone(),
                    val_reg,
                    VirtualRegister::Constant(ConstantRegister::Zero),
                )),
                comment: "convert to inversed boolean".into(),
                owning_span: self.md_mgr.val_to_span(self.context, *instr_val),
            });
            self.bytecode.push(Op {
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
    }

    fn compile_binary_op(
        &mut self,
        instr_val: &Value,
        op: &BinaryOpKind,
        arg1: &Value,
        arg2: &Value,
    ) {
        let val1_reg = self.value_to_register(arg1);
        let val2_reg = self.value_to_register(arg2);
        let res_reg = self.reg_seqr.next();
        let opcode = match op {
            BinaryOpKind::Add => Either::Left(VirtualOp::ADD(res_reg.clone(), val1_reg, val2_reg)),
            BinaryOpKind::Sub => Either::Left(VirtualOp::SUB(res_reg.clone(), val1_reg, val2_reg)),
            BinaryOpKind::Mul => Either::Left(VirtualOp::MUL(res_reg.clone(), val1_reg, val2_reg)),
            BinaryOpKind::Div => Either::Left(VirtualOp::DIV(res_reg.clone(), val1_reg, val2_reg)),
        };
        self.bytecode.push(Op {
            opcode,
            comment: String::new(),
            owning_span: self.md_mgr.val_to_span(self.context, *instr_val),
        });

        self.reg_map.insert(*instr_val, res_reg);
    }

    fn compile_branch(&mut self, from_block: &Block, to_block: &Block) {
        self.compile_branch_to_phi_value(from_block, to_block);

        let label = self.block_to_label(to_block);
        self.bytecode.push(Op::jump_to_label(label));
    }

    fn compile_cmp(
        &mut self,
        instr_val: &Value,
        pred: &Predicate,
        lhs_value: &Value,
        rhs_value: &Value,
    ) {
        let lhs_reg = self.value_to_register(lhs_value);
        let rhs_reg = self.value_to_register(rhs_value);
        let res_reg = self.reg_seqr.next();
        match pred {
            Predicate::Equal => {
                self.bytecode.push(Op {
                    opcode: Either::Left(VirtualOp::EQ(res_reg.clone(), lhs_reg, rhs_reg)),
                    comment: String::new(),
                    owning_span: self.md_mgr.val_to_span(self.context, *instr_val),
                });
            }
        }
        self.reg_map.insert(*instr_val, res_reg);
    }

    fn compile_conditional_branch(
        &mut self,
        cond_value: &Value,
        from_block: &Block,
        true_block: &Block,
        false_block: &Block,
    ) {
        self.compile_branch_to_phi_value(from_block, true_block);
        self.compile_branch_to_phi_value(from_block, false_block);

        let cond_reg = self.value_to_register(cond_value);

        let true_label = self.block_to_label(true_block);
        self.bytecode
            .push(Op::jump_if_not_zero(cond_reg, true_label));

        let false_label = self.block_to_label(false_block);
        self.bytecode.push(Op::jump_to_label(false_label));
    }

    fn compile_branch_to_phi_value(&mut self, from_block: &Block, to_block: &Block) {
        if let Some(local_val) = to_block.get_phi_val_coming_from(self.context, from_block) {
            // We only need a MOVE here if get_phi_val_coming_from() is actually assigned to a
            // register
            if let Some(local_reg) = self.value_to_register_or_none(&local_val) {
                let phi_reg = self.value_to_register(&to_block.get_phi(self.context));
                self.bytecode.push(Op::unowned_register_move_comment(
                    phi_reg,
                    local_reg,
                    "branch to phi value",
                ));
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn compile_contract_call(
        &mut self,
        instr_val: &Value,
        params: &Value,
        coins: &Value,
        asset_id: &Value,
        gas: &Value,
    ) {
        let ra_pointer = self.value_to_register(params);
        let coins_register = self.value_to_register(coins);
        let asset_id_register = self.value_to_register(asset_id);
        let gas_register = self.value_to_register(gas);

        self.bytecode.push(Op {
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
        self.bytecode.push(Op::unowned_register_move(
            instr_reg.clone(),
            VirtualRegister::Constant(ConstantRegister::ReturnValue),
        ));
        self.reg_map.insert(*instr_val, instr_reg);
    }

    fn compile_extract_element(
        &mut self,
        instr_val: &Value,
        array: &Value,
        ty: &Aggregate,
        index_val: &Value,
    ) {
        // Base register should pointer to some stack allocated memory.
        let base_reg = self.value_to_register(array);

        // Index value is the array element index, not byte nor word offset.
        let index_reg = self.value_to_register(index_val);
        let rel_offset_reg = match index_reg {
            VirtualRegister::Virtual(_) => {
                // We can reuse the register.
                index_reg.clone()
            }
            VirtualRegister::Constant(_) => {
                // We have a constant register, cannot reuse it.
                self.reg_seqr.next()
            }
        };

        // We could put the OOB check here, though I'm now thinking it would be too wasteful.
        // See compile_bounds_assertion() in expression/array.rs (or look in Git history).

        let instr_reg = self.reg_seqr.next();
        let owning_span = self.md_mgr.val_to_span(self.context, *instr_val);
        let elem_type = ty.get_elem_type(self.context).unwrap();
        let elem_size = ir_type_size_in_bytes(self.context, &elem_type);
        if elem_type.is_copy_type() {
            self.bytecode.push(Op {
                opcode: Either::Left(VirtualOp::MULI(
                    rel_offset_reg.clone(),
                    index_reg,
                    VirtualImmediate12 { value: 8 },
                )),
                comment: "extract_element relative offset".into(),
                owning_span: owning_span.clone(),
            });
            let elem_offs_reg = self.reg_seqr.next();
            self.bytecode.push(Op {
                opcode: Either::Left(VirtualOp::ADD(
                    elem_offs_reg.clone(),
                    base_reg,
                    rel_offset_reg,
                )),
                comment: "extract_element absolute offset".into(),
                owning_span: owning_span.clone(),
            });
            self.bytecode.push(Op {
                opcode: Either::Left(VirtualOp::LW(
                    instr_reg.clone(),
                    elem_offs_reg,
                    VirtualImmediate12 { value: 0 },
                )),
                comment: "extract_element".into(),
                owning_span,
            });
        } else {
            // Value too big for a register, so we return the memory offset.
            if elem_size > compiler_constants::TWELVE_BITS {
                let size_data_id = self
                    .data_section
                    .insert_data_value(&Literal::U64(elem_size));
                let size_reg = self.reg_seqr.next();
                self.bytecode.push(Op {
                    opcode: Either::Left(VirtualOp::LWDataId(size_reg.clone(), size_data_id)),
                    owning_span: owning_span.clone(),
                    comment: "loading element size for relative offset".into(),
                });
                self.bytecode.push(Op {
                    opcode: Either::Left(VirtualOp::MUL(instr_reg.clone(), index_reg, size_reg)),
                    comment: "extract_element relative offset".into(),
                    owning_span: owning_span.clone(),
                });
            } else {
                self.bytecode.push(Op {
                    opcode: Either::Left(VirtualOp::MULI(
                        instr_reg.clone(),
                        index_reg,
                        VirtualImmediate12 {
                            value: elem_size as u16,
                        },
                    )),
                    comment: "extract_element relative offset".into(),
                    owning_span: owning_span.clone(),
                });
            }
            self.bytecode.push(Op {
                opcode: Either::Left(VirtualOp::ADD(
                    instr_reg.clone(),
                    base_reg,
                    instr_reg.clone(),
                )),
                comment: "extract_element absolute offset".into(),
                owning_span,
            });
        }

        self.reg_map.insert(*instr_val, instr_reg);
    }

    fn compile_extract_value(&mut self, instr_val: &Value, aggregate_val: &Value, indices: &[u64]) {
        // Base register should pointer to some stack allocated memory.
        let base_reg = self.value_to_register(aggregate_val);
        let ((extract_offset, _), field_type) = aggregate_idcs_to_field_layout(
            self.context,
            &aggregate_val.get_stripped_ptr_type(self.context).unwrap(),
            indices,
        );

        let instr_reg = self.reg_seqr.next();
        let owning_span = self.md_mgr.val_to_span(self.context, *instr_val);
        if field_type.is_copy_type() {
            if extract_offset > compiler_constants::TWELVE_BITS {
                let offset_reg = self.reg_seqr.next();
                self.number_to_reg(extract_offset, &offset_reg, owning_span.clone());
                self.bytecode.push(Op {
                    opcode: Either::Left(VirtualOp::ADD(
                        offset_reg.clone(),
                        base_reg.clone(),
                        base_reg,
                    )),
                    comment: "add array base to offset".into(),
                    owning_span: owning_span.clone(),
                });
                self.bytecode.push(Op {
                    opcode: Either::Left(VirtualOp::LW(
                        instr_reg.clone(),
                        offset_reg,
                        VirtualImmediate12 { value: 0 },
                    )),
                    comment: format!(
                        "extract_value @ {}",
                        indices
                            .iter()
                            .map(|idx| format!("{}", idx))
                            .collect::<Vec<String>>()
                            .join(",")
                    ),
                    owning_span,
                });
            } else {
                self.bytecode.push(Op {
                    opcode: Either::Left(VirtualOp::LW(
                        instr_reg.clone(),
                        base_reg,
                        VirtualImmediate12 {
                            value: extract_offset as u16,
                        },
                    )),
                    comment: format!(
                        "extract_value @ {}",
                        indices
                            .iter()
                            .map(|idx| format!("{}", idx))
                            .collect::<Vec<String>>()
                            .join(",")
                    ),
                    owning_span,
                });
            }
        } else {
            // Value too big for a register, so we return the memory offset.
            if extract_offset * 8 > compiler_constants::TWELVE_BITS {
                let offset_reg = self.reg_seqr.next();
                self.number_to_reg(extract_offset * 8, &offset_reg, owning_span.clone());
                self.bytecode.push(Op {
                    opcode: either::Either::Left(VirtualOp::ADD(
                        instr_reg.clone(),
                        base_reg,
                        offset_reg,
                    )),
                    comment: "extract address".into(),
                    owning_span,
                });
            } else {
                self.bytecode.push(Op {
                    opcode: either::Either::Left(VirtualOp::ADDI(
                        instr_reg.clone(),
                        base_reg,
                        VirtualImmediate12 {
                            value: (extract_offset * 8) as u16,
                        },
                    )),
                    comment: "extract address".into(),
                    owning_span,
                });
            }
        }

        self.reg_map.insert(*instr_val, instr_reg);
    }

    fn compile_get_storage_key(&mut self, instr_val: &Value) -> CompileResult<()> {
        let warnings: Vec<CompileWarning> = Vec::new();
        let mut errors: Vec<CompileError> = Vec::new();

        let state_idx = self.md_mgr.val_to_storage_key(self.context, *instr_val);
        let instr_span = self.md_mgr.val_to_span(self.context, *instr_val);

        let storage_slot_to_hash = match state_idx {
            Some(state_idx) => {
                format!(
                    "{}{}",
                    sway_utils::constants::STORAGE_DOMAIN_SEPARATOR,
                    state_idx
                )
            }
            None => {
                errors.push(CompileError::Internal(
                    "State index for __get_storage_key is not available as a metadata",
                    instr_span.unwrap_or_else(Self::empty_span),
                ));
                return err(warnings, errors);
            }
        };

        let hashed_storage_slot = Hasher::hash(storage_slot_to_hash);

        let data_id = self
            .data_section
            .insert_data_value(&Literal::B256(hashed_storage_slot.into()));

        // Allocate a register for it, and a load instruction.
        let reg = self.reg_seqr.next();

        self.bytecode.push(Op {
            opcode: either::Either::Left(VirtualOp::LWDataId(reg.clone(), data_id)),
            comment: "literal instantiation".into(),
            owning_span: instr_span,
        });
        self.reg_map.insert(*instr_val, reg);
        ok((), warnings, errors)
    }

    fn compile_get_pointer(
        &mut self,
        instr_val: &Value,
        base_ptr: &Pointer,
        ptr_ty: &Pointer,
        offset: u64,
    ) {
        // `get_ptr` is like a `load` except the value isn't dereferenced.
        let owning_span = self.md_mgr.val_to_span(self.context, *instr_val);
        match self.ptr_map.get(base_ptr) {
            None => unimplemented!("BUG? Uninitialised pointer."),
            Some(storage) => match storage.clone() {
                Storage::Data(_data_id) => {
                    // Not sure if we'll ever need this.
                    unimplemented!("TODO get_ptr() into the data section.");
                }
                Storage::Stack(word_offs) => {
                    let ptr_ty_size_in_bytes =
                        ir_type_size_in_bytes(self.context, ptr_ty.get_type(self.context));

                    let offset_in_bytes = word_offs * 8 + ptr_ty_size_in_bytes * offset;
                    let instr_reg = self.reg_seqr.next();
                    if offset_in_bytes > compiler_constants::TWELVE_BITS {
                        self.number_to_reg(offset_in_bytes, &instr_reg, owning_span.clone());
                        self.bytecode.push(Op {
                            opcode: either::Either::Left(VirtualOp::ADD(
                                instr_reg.clone(),
                                self.stack_base_reg.as_ref().unwrap().clone(),
                                instr_reg.clone(),
                            )),
                            comment: "get offset reg for get_ptr".into(),
                            owning_span,
                        });
                    } else {
                        self.bytecode.push(Op {
                            opcode: either::Either::Left(VirtualOp::ADDI(
                                instr_reg.clone(),
                                self.stack_base_reg.as_ref().unwrap().clone(),
                                VirtualImmediate12 {
                                    value: (offset_in_bytes) as u16,
                                },
                            )),
                            comment: "get offset reg for get_ptr".into(),
                            owning_span,
                        });
                    }
                    self.reg_map.insert(*instr_val, instr_reg);
                }
            },
        }
    }

    fn compile_gtf(&mut self, instr_val: &Value, index: &Value, tx_field_id: u64) {
        let instr_reg = self.reg_seqr.next();
        let index_reg = self.value_to_register(index);
        self.bytecode.push(Op {
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
    }

    fn compile_insert_element(
        &mut self,
        instr_val: &Value,
        array: &Value,
        ty: &Aggregate,
        value: &Value,
        index_val: &Value,
    ) {
        // Base register should point to some stack allocated memory.
        let base_reg = self.value_to_register(array);
        let insert_reg = self.value_to_register(value);

        // Index value is the array element index, not byte nor word offset.
        let index_reg = self.value_to_register(index_val);
        let rel_offset_reg = match index_reg {
            VirtualRegister::Virtual(_) => {
                // We can reuse the register.
                index_reg.clone()
            }
            VirtualRegister::Constant(_) => {
                // We have a constant register, cannot reuse it.
                self.reg_seqr.next()
            }
        };

        let owning_span = self.md_mgr.val_to_span(self.context, *instr_val);

        let elem_type = ty.get_elem_type(self.context).unwrap();
        let elem_size = ir_type_size_in_bytes(self.context, &elem_type);
        if elem_type.is_copy_type() {
            self.bytecode.push(Op {
                opcode: Either::Left(VirtualOp::MULI(
                    rel_offset_reg.clone(),
                    index_reg,
                    VirtualImmediate12 { value: 8 },
                )),
                comment: "insert_element relative offset".into(),
                owning_span: owning_span.clone(),
            });
            let elem_offs_reg = self.reg_seqr.next();
            self.bytecode.push(Op {
                opcode: Either::Left(VirtualOp::ADD(
                    elem_offs_reg.clone(),
                    base_reg.clone(),
                    rel_offset_reg,
                )),
                comment: "insert_element absolute offset".into(),
                owning_span: owning_span.clone(),
            });
            self.bytecode.push(Op {
                opcode: Either::Left(VirtualOp::SW(
                    elem_offs_reg,
                    insert_reg,
                    VirtualImmediate12 { value: 0 },
                )),
                comment: "insert_element".into(),
                owning_span,
            });
        } else {
            // Element size is larger than 8; we switch to bytewise offsets and sizes and use MCP.
            if elem_size > compiler_constants::TWELVE_BITS {
                todo!("array element size bigger than 4k")
            } else {
                let elem_index_offs_reg = self.reg_seqr.next();
                self.bytecode.push(Op {
                    opcode: Either::Left(VirtualOp::MULI(
                        elem_index_offs_reg.clone(),
                        index_reg,
                        VirtualImmediate12 {
                            value: elem_size as u16,
                        },
                    )),
                    comment: "insert_element relative offset".into(),
                    owning_span: owning_span.clone(),
                });
                self.bytecode.push(Op {
                    opcode: Either::Left(VirtualOp::ADD(
                        elem_index_offs_reg.clone(),
                        base_reg.clone(),
                        elem_index_offs_reg.clone(),
                    )),
                    comment: "insert_element absolute offset".into(),
                    owning_span: owning_span.clone(),
                });
                self.bytecode.push(Op {
                    opcode: Either::Left(VirtualOp::MCPI(
                        elem_index_offs_reg,
                        insert_reg,
                        VirtualImmediate12 {
                            value: elem_size as u16,
                        },
                    )),
                    comment: "insert_element store value".into(),
                    owning_span,
                });
            }
        }

        // We set the 'instruction' register to the base register, so that cascading inserts will
        // work.
        self.reg_map.insert(*instr_val, base_reg);
    }

    fn compile_insert_value(
        &mut self,
        instr_val: &Value,
        aggregate_val: &Value,
        value: &Value,
        indices: &[u64],
    ) {
        // Base register should point to some stack allocated memory.
        let base_reg = self.value_to_register(aggregate_val);

        let insert_reg = self.value_to_register(value);
        let ((mut insert_offs, field_size_in_bytes), field_type) = aggregate_idcs_to_field_layout(
            self.context,
            &aggregate_val.get_stripped_ptr_type(self.context).unwrap(),
            indices,
        );

        let value_type = value.get_stripped_ptr_type(self.context).unwrap();
        let value_size_in_bytes = ir_type_size_in_bytes(self.context, &value_type);
        let value_size_in_words = size_bytes_in_words!(value_size_in_bytes);

        // Account for the padding if the final field type is a union and the value we're trying to
        // insert is smaller than the size of the union (i.e. we're inserting a small variant).
        if matches!(field_type, Type::Union(_)) {
            let field_size_in_words = size_bytes_in_words!(field_size_in_bytes);
            assert!(field_size_in_words >= value_size_in_words);
            insert_offs += field_size_in_words - value_size_in_words;
        }

        let indices_str = indices
            .iter()
            .map(|idx| format!("{}", idx))
            .collect::<Vec<String>>()
            .join(",");

        let owning_span = self.md_mgr.val_to_span(self.context, *instr_val);

        if value_type.is_copy_type() {
            if insert_offs > compiler_constants::TWELVE_BITS {
                let insert_offs_reg = self.reg_seqr.next();
                self.number_to_reg(insert_offs, &insert_offs_reg, owning_span.clone());
                self.bytecode.push(Op {
                    opcode: Either::Left(VirtualOp::ADD(
                        base_reg.clone(),
                        base_reg.clone(),
                        insert_offs_reg,
                    )),
                    comment: "insert_value absolute offset".into(),
                    owning_span: owning_span.clone(),
                });
                self.bytecode.push(Op {
                    opcode: Either::Left(VirtualOp::SW(
                        base_reg.clone(),
                        insert_reg,
                        VirtualImmediate12 { value: 0 },
                    )),
                    comment: format!("insert_value @ {}", indices_str),
                    owning_span,
                });
            } else {
                self.bytecode.push(Op {
                    opcode: Either::Left(VirtualOp::SW(
                        base_reg.clone(),
                        insert_reg,
                        VirtualImmediate12 {
                            value: insert_offs as u16,
                        },
                    )),
                    comment: format!("insert_value @ {}", indices_str),
                    owning_span,
                });
            }
        } else {
            let offs_reg = self.reg_seqr.next();
            if insert_offs * 8 > compiler_constants::TWELVE_BITS {
                self.number_to_reg(insert_offs * 8, &offs_reg, owning_span.clone());
            } else {
                self.bytecode.push(Op {
                    opcode: either::Either::Left(VirtualOp::ADDI(
                        offs_reg.clone(),
                        base_reg.clone(),
                        VirtualImmediate12 {
                            value: (insert_offs * 8) as u16,
                        },
                    )),
                    comment: format!("get struct field(s) {} offset", indices_str),
                    owning_span: owning_span.clone(),
                });
            }
            if value_size_in_bytes > compiler_constants::TWELVE_BITS {
                let size_reg = self.reg_seqr.next();
                self.number_to_reg(value_size_in_bytes, &size_reg, owning_span.clone());
                self.bytecode.push(Op {
                    opcode: Either::Left(VirtualOp::MCP(offs_reg, insert_reg, size_reg)),
                    comment: "store struct field value".into(),
                    owning_span,
                });
            } else {
                self.bytecode.push(Op {
                    opcode: Either::Left(VirtualOp::MCPI(
                        offs_reg,
                        insert_reg,
                        VirtualImmediate12 {
                            value: value_size_in_bytes as u16,
                        },
                    )),
                    comment: "store struct field value".into(),
                    owning_span,
                });
            }
        }

        // We set the 'instruction' register to the base register, so that cascading inserts will
        // work.
        self.reg_map.insert(*instr_val, base_reg);
    }

    fn compile_int_to_ptr(&mut self, instr_val: &Value, int_to_ptr_val: &Value) {
        let val_reg = self.value_to_register(int_to_ptr_val);
        self.reg_map.insert(*instr_val, val_reg);
    }

    fn compile_load(&mut self, instr_val: &Value, src_val: &Value) -> CompileResult<()> {
        let ptr = self.resolve_ptr(src_val);
        if ptr.value.is_none() {
            return ptr.map(|_| ());
        }
        let (ptr, _ptr_ty, _offset) = ptr.value.unwrap();
        let instr_reg = self.reg_seqr.next();
        let owning_span = self.md_mgr.val_to_span(self.context, *instr_val);
        match self.ptr_map.get(&ptr) {
            None => unimplemented!("BUG? Uninitialised pointer."),
            Some(storage) => match storage.clone() {
                Storage::Data(data_id) => {
                    self.bytecode.push(Op {
                        opcode: Either::Left(VirtualOp::LWDataId(instr_reg.clone(), data_id)),
                        comment: "load constant".into(),
                        owning_span,
                    });
                }
                Storage::Stack(word_offs) => {
                    let base_reg = self.stack_base_reg.as_ref().unwrap().clone();
                    if ptr.get_type(self.context).is_copy_type() {
                        // Value can fit in a register, so we load the value.
                        if word_offs > compiler_constants::TWELVE_BITS {
                            let offs_reg = self.reg_seqr.next();
                            self.number_to_reg(
                                word_offs * 8, // Base reg for LW is in bytes
                                &offs_reg,
                                owning_span.clone(),
                            );
                            self.bytecode.push(Op {
                                opcode: Either::Left(VirtualOp::ADD(
                                    offs_reg.clone(),
                                    base_reg,
                                    offs_reg.clone(),
                                )),
                                comment: "absolute offset for load".into(),
                                owning_span: owning_span.clone(),
                            });
                            self.bytecode.push(Op {
                                opcode: Either::Left(VirtualOp::LW(
                                    instr_reg.clone(),
                                    offs_reg.clone(),
                                    VirtualImmediate12 { value: 0 },
                                )),
                                comment: "load value".into(),
                                owning_span,
                            });
                        } else {
                            self.bytecode.push(Op {
                                opcode: Either::Left(VirtualOp::LW(
                                    instr_reg.clone(),
                                    base_reg,
                                    VirtualImmediate12 {
                                        value: word_offs as u16,
                                    },
                                )),
                                comment: "load value".into(),
                                owning_span,
                            });
                        }
                    } else {
                        // Value too big for a register, so we return the memory offset.  This is
                        // what LW to the data section does, via LWDataId.
                        let word_offs = word_offs * 8;
                        if word_offs > compiler_constants::TWELVE_BITS {
                            let offs_reg = self.reg_seqr.next();
                            self.number_to_reg(word_offs, &offs_reg, owning_span.clone());
                            self.bytecode.push(Op {
                                opcode: either::Either::Left(VirtualOp::ADD(
                                    instr_reg.clone(),
                                    base_reg,
                                    offs_reg,
                                )),
                                comment: "load address".into(),
                                owning_span,
                            });
                        } else {
                            self.bytecode.push(Op {
                                opcode: either::Either::Left(VirtualOp::ADDI(
                                    instr_reg.clone(),
                                    base_reg,
                                    VirtualImmediate12 {
                                        value: word_offs as u16,
                                    },
                                )),
                                comment: "load address".into(),
                                owning_span,
                            });
                        }
                    }
                }
            },
        }
        self.reg_map.insert(*instr_val, instr_reg);
        ok((), Vec::new(), Vec::new())
    }

    fn compile_log(&mut self, instr_val: &Value, log_val: &Value, log_ty: &Type, log_id: &Value) {
        let owning_span = self.md_mgr.val_to_span(self.context, *instr_val);
        let log_val_reg = self.value_to_register(log_val);
        let log_id_reg = self.value_to_register(log_id);

        if log_ty.is_copy_type() {
            self.bytecode.push(Op {
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
            // If the type not a reference type then we use LOGD to log the data. First put the
            // size into the data section, then add a LW to get it, then add a LOGD which uses
            // it.
            let size_reg = self.reg_seqr.next();
            let size_in_bytes = ir_type_size_in_bytes(self.context, log_ty);
            let size_data_id = self
                .data_section
                .insert_data_value(&Literal::U64(size_in_bytes));

            self.bytecode.push(Op {
                opcode: Either::Left(VirtualOp::LWDataId(size_reg.clone(), size_data_id)),
                owning_span: owning_span.clone(),
                comment: "loading size for LOGD".into(),
            });
            self.bytecode.push(Op {
                owning_span,
                opcode: Either::Left(VirtualOp::LOGD(
                    VirtualRegister::Constant(ConstantRegister::Zero),
                    log_id_reg,
                    log_val_reg,
                    size_reg,
                )),
                comment: "".into(),
            });
        }
    }

    fn compile_read_register(&mut self, instr_val: &Value, reg: &sway_ir::Register) {
        let instr_reg = self.reg_seqr.next();
        self.bytecode.push(Op {
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

    fn compile_ret(&mut self, instr_val: &Value, ret_val: &Value, ret_type: &Type) {
        let owning_span = self.md_mgr.val_to_span(self.context, *instr_val);
        if ret_type.eq(self.context, &Type::Unit) {
            // Unit returns should always be zero, although because they can be omitted from
            // functions, the register is sometimes uninitialized. Manually return zero in this
            // case.
            self.bytecode.push(Op {
                opcode: Either::Left(VirtualOp::RET(VirtualRegister::Constant(
                    ConstantRegister::Zero,
                ))),
                owning_span,
                comment: "returning unit as zero".into(),
            });
        } else {
            let ret_reg = self.value_to_register(ret_val);

            if ret_type.is_copy_type() {
                self.bytecode.push(Op {
                    owning_span,
                    opcode: Either::Left(VirtualOp::RET(ret_reg)),
                    comment: "".into(),
                });
            } else {
                // If the type not a reference type then we use RETD to return data.  First put the
                // size into the data section, then add a LW to get it, then add a RETD which uses
                // it.
                let size_reg = self.reg_seqr.next();
                let size_in_bytes = ir_type_size_in_bytes(self.context, ret_type);
                let size_data_id = self
                    .data_section
                    .insert_data_value(&Literal::U64(size_in_bytes));

                self.bytecode.push(Op {
                    opcode: Either::Left(VirtualOp::LWDataId(size_reg.clone(), size_data_id)),
                    owning_span: owning_span.clone(),
                    comment: "loading size for RETD".into(),
                });
                self.bytecode.push(Op {
                    owning_span,
                    opcode: Either::Left(VirtualOp::RETD(ret_reg, size_reg)),
                    comment: "".into(),
                });
            }
        }
    }

    fn offset_reg(
        &mut self,
        base_reg: &VirtualRegister,
        offset_in_bytes: u64,
        span: Option<Span>,
    ) -> VirtualRegister {
        let offset_reg = self.reg_seqr.next();
        if offset_in_bytes > compiler_constants::TWELVE_BITS {
            let offs_reg = self.reg_seqr.next();
            self.number_to_reg(offset_in_bytes, &offs_reg, span.clone());
            self.bytecode.push(Op {
                opcode: either::Either::Left(VirtualOp::ADD(
                    offset_reg.clone(),
                    base_reg.clone(),
                    offs_reg,
                )),
                comment: "get offset".into(),
                owning_span: span,
            });
        } else {
            self.bytecode.push(Op {
                opcode: either::Either::Left(VirtualOp::ADDI(
                    offset_reg.clone(),
                    base_reg.clone(),
                    VirtualImmediate12 {
                        value: offset_in_bytes as u16,
                    },
                )),
                comment: "get offset".into(),
                owning_span: span,
            });
        }

        offset_reg
    }

    fn compile_state_access_quad_word(
        &mut self,
        instr_val: &Value,
        val: &Value,
        key: &Value,
        access_type: StateAccessType,
    ) -> CompileResult<()> {
        // Make sure that both val and key are pointers to B256.
        assert!(matches!(
            val.get_stripped_ptr_type(self.context),
            Some(Type::B256)
        ));
        assert!(matches!(
            key.get_stripped_ptr_type(self.context),
            Some(Type::B256)
        ));
        let owning_span = self.md_mgr.val_to_span(self.context, *instr_val);

        let key_ptr = self.resolve_ptr(key);
        if key_ptr.value.is_none() {
            return key_ptr.map(|_| ());
        }
        let (key_ptr, ptr_ty, offset) = key_ptr.value.unwrap();

        // Not expecting an offset here nor a pointer cast
        assert!(offset == 0);
        assert!(ptr_ty.get_type(self.context).eq(self.context, &Type::B256));

        let val_reg = if matches!(
            &self.context.values[val.0].value,
            ValueDatum::Instruction(Instruction::IntToPtr(..))
        ) {
            match self.reg_map.get(val) {
                Some(vreg) => vreg.clone(),
                None => unreachable!("int_to_ptr instruction doesn't have vreg mapped"),
            }
        } else {
            // Expect ptr_ty here to also be b256 and offset to be whatever...
            let val_ptr = self.resolve_ptr(val);
            if val_ptr.value.is_none() {
                return val_ptr.map(|_| ());
            }
            let (val_ptr, ptr_ty, offset) = val_ptr.value.unwrap();
            // Expect the ptr_ty for val to also be B256
            assert!(ptr_ty.get_type(self.context).eq(self.context, &Type::B256));
            match self.ptr_map.get(&val_ptr) {
                Some(Storage::Stack(val_offset)) => {
                    let base_reg = self.stack_base_reg.as_ref().unwrap().clone();
                    let val_offset_in_bytes = val_offset * 8 + offset * 32;
                    self.offset_reg(&base_reg, val_offset_in_bytes, owning_span.clone())
                }
                _ => unreachable!("Unexpected storage locations for key and val"),
            }
        };

        let key_reg = match self.ptr_map.get(&key_ptr) {
            Some(Storage::Stack(key_offset)) => {
                let base_reg = self.stack_base_reg.as_ref().unwrap().clone();
                let key_offset_in_bytes = key_offset * 8;
                self.offset_reg(&base_reg, key_offset_in_bytes, owning_span.clone())
            }
            _ => unreachable!("Unexpected storage locations for key and val"),
        };

        self.bytecode.push(Op {
            opcode: Either::Left(match access_type {
                StateAccessType::Read => VirtualOp::SRWQ(val_reg, key_reg),
                StateAccessType::Write => VirtualOp::SWWQ(key_reg, val_reg),
            }),
            comment: "quad word state access".into(),
            owning_span,
        });
        ok((), Vec::new(), Vec::new())
    }

    fn compile_state_load_word(&mut self, instr_val: &Value, key: &Value) -> CompileResult<()> {
        // Make sure that the key is a pointers to B256.
        assert!(matches!(
            key.get_stripped_ptr_type(self.context),
            Some(Type::B256)
        ));

        let key_ptr = self.resolve_ptr(key);
        if key_ptr.value.is_none() {
            return key_ptr.map(|_| ());
        }
        let (key_ptr, ptr_ty, offset) = key_ptr.value.unwrap();

        // Not expecting an offset here nor a pointer cast
        assert!(offset == 0);
        assert!(ptr_ty.get_type(self.context).eq(self.context, &Type::B256));

        let load_reg = self.reg_seqr.next();
        let owning_span = self.md_mgr.val_to_span(self.context, *instr_val);
        match self.ptr_map.get(&key_ptr) {
            Some(Storage::Stack(key_offset)) => {
                let base_reg = self.stack_base_reg.as_ref().unwrap().clone();
                let key_offset_in_bytes = key_offset * 8;

                let key_reg = self.offset_reg(&base_reg, key_offset_in_bytes, owning_span.clone());

                self.bytecode.push(Op {
                    opcode: Either::Left(VirtualOp::SRW(load_reg.clone(), key_reg)),
                    comment: "single word state access".into(),
                    owning_span,
                });
            }
            _ => unreachable!("Unexpected storage location for key"),
        }

        self.reg_map.insert(*instr_val, load_reg);
        ok((), Vec::new(), Vec::new())
    }

    fn compile_state_store_word(
        &mut self,
        instr_val: &Value,
        store_val: &Value,
        key: &Value,
    ) -> CompileResult<()> {
        // Make sure that key is a pointer to B256.
        assert!(matches!(
            key.get_stripped_ptr_type(self.context),
            Some(Type::B256)
        ));

        // Make sure that store_val is a U64 value.
        assert!(matches!(
            store_val.get_type(self.context),
            Some(Type::Uint(64))
        ));
        let store_reg = self.value_to_register(store_val);

        // Expect the get_ptr here to have type b256 and offset = 0???
        let key_ptr = self.resolve_ptr(key);
        if key_ptr.value.is_none() {
            return key_ptr.map(|_| ());
        }
        let (key_ptr, ptr_ty, offset) = key_ptr.value.unwrap();

        // Not expecting an offset here nor a pointer cast
        assert!(offset == 0);
        assert!(ptr_ty.get_type(self.context).eq(self.context, &Type::B256));

        let owning_span = self.md_mgr.val_to_span(self.context, *instr_val);
        match self.ptr_map.get(&key_ptr) {
            Some(Storage::Stack(key_offset)) => {
                let base_reg = self.stack_base_reg.as_ref().unwrap().clone();
                let key_offset_in_bytes = key_offset * 8;

                let key_reg = self.offset_reg(&base_reg, key_offset_in_bytes, owning_span.clone());

                self.bytecode.push(Op {
                    opcode: Either::Left(VirtualOp::SWW(key_reg, store_reg)),
                    comment: "single word state access".into(),
                    owning_span,
                });
            }
            _ => unreachable!("Unexpected storage locations for key and store_val"),
        }

        ok((), Vec::new(), Vec::new())
    }

    fn compile_store(
        &mut self,
        instr_val: &Value,
        dst_val: &Value,
        stored_val: &Value,
    ) -> CompileResult<()> {
        let ptr = self.resolve_ptr(dst_val);
        if ptr.value.is_none() {
            return ptr.map(|_| ());
        }
        let (ptr, _ptr_ty, _offset) = ptr.value.unwrap();
        let stored_reg = self.value_to_register(stored_val);
        let is_aggregate_ptr = ptr.is_aggregate_ptr(self.context);
        let owning_span = self.md_mgr.val_to_span(self.context, *instr_val);
        match self.ptr_map.get(&ptr) {
            None => unreachable!("Bug! Trying to store to an unknown pointer."),
            Some(storage) => match storage {
                Storage::Data(_) => unreachable!("BUG! Trying to store to the data section."),
                Storage::Stack(word_offs) => {
                    let word_offs = *word_offs;
                    let store_type = ptr.get_type(self.context);
                    let store_size_in_words =
                        size_bytes_in_words!(ir_type_size_in_bytes(self.context, store_type));
                    if store_type.is_copy_type() {
                        let base_reg = self.stack_base_reg.as_ref().unwrap().clone();

                        // A single word can be stored with SW.
                        let stored_reg = if !is_aggregate_ptr {
                            // stored_reg is a value.
                            stored_reg
                        } else {
                            // stored_reg is a pointer, even though size is 1.  We need to load it.
                            let tmp_reg = self.reg_seqr.next();
                            self.bytecode.push(Op {
                                opcode: Either::Left(VirtualOp::LW(
                                    tmp_reg.clone(),
                                    stored_reg,
                                    VirtualImmediate12 { value: 0 },
                                )),
                                comment: "load for store".into(),
                                owning_span: owning_span.clone(),
                            });
                            tmp_reg
                        };
                        if word_offs > compiler_constants::TWELVE_BITS {
                            let offs_reg = self.reg_seqr.next();
                            self.number_to_reg(
                                word_offs * 8, // Base reg for SW is in bytes
                                &offs_reg,
                                owning_span.clone(),
                            );
                            self.bytecode.push(Op {
                                opcode: Either::Left(VirtualOp::ADD(
                                    offs_reg.clone(),
                                    base_reg,
                                    offs_reg.clone(),
                                )),
                                comment: "store absolute offset".into(),
                                owning_span: owning_span.clone(),
                            });
                            self.bytecode.push(Op {
                                opcode: Either::Left(VirtualOp::SW(
                                    offs_reg,
                                    stored_reg,
                                    VirtualImmediate12 { value: 0 },
                                )),
                                comment: "store value".into(),
                                owning_span,
                            });
                        } else {
                            self.bytecode.push(Op {
                                opcode: Either::Left(VirtualOp::SW(
                                    base_reg,
                                    stored_reg,
                                    VirtualImmediate12 {
                                        value: word_offs as u16,
                                    },
                                )),
                                comment: "store value".into(),
                                owning_span,
                            });
                        }
                    } else {
                        let base_reg = self.stack_base_reg.as_ref().unwrap().clone();

                        // Bigger than 1 word needs a MCPI.  XXX Or MCP if it's huge.
                        let dest_offs_reg = self.reg_seqr.next();
                        if word_offs * 8 > compiler_constants::TWELVE_BITS {
                            self.number_to_reg(word_offs * 8, &dest_offs_reg, owning_span.clone());
                            self.bytecode.push(Op {
                                opcode: either::Either::Left(VirtualOp::ADD(
                                    dest_offs_reg.clone(),
                                    base_reg,
                                    dest_offs_reg.clone(),
                                )),
                                comment: "get store offset".into(),
                                owning_span: owning_span.clone(),
                            });
                        } else {
                            self.bytecode.push(Op {
                                opcode: either::Either::Left(VirtualOp::ADDI(
                                    dest_offs_reg.clone(),
                                    base_reg,
                                    VirtualImmediate12 {
                                        value: (word_offs * 8) as u16,
                                    },
                                )),
                                comment: "get store offset".into(),
                                owning_span: owning_span.clone(),
                            });
                        }

                        if store_size_in_words * 8 > compiler_constants::TWELVE_BITS {
                            let size_reg = self.reg_seqr.next();
                            self.number_to_reg(
                                store_size_in_words * 8,
                                &size_reg,
                                owning_span.clone(),
                            );
                            self.bytecode.push(Op {
                                opcode: Either::Left(VirtualOp::MCP(
                                    dest_offs_reg,
                                    stored_reg,
                                    size_reg,
                                )),
                                comment: "store value".into(),
                                owning_span,
                            });
                        } else {
                            self.bytecode.push(Op {
                                opcode: Either::Left(VirtualOp::MCPI(
                                    dest_offs_reg,
                                    stored_reg,
                                    VirtualImmediate12 {
                                        value: (store_size_in_words * 8) as u16,
                                    },
                                )),
                                comment: "store value".into(),
                                owning_span,
                            });
                        }
                    }
                }
            },
        };
        ok((), Vec::new(), Vec::new())
    }

    fn resolve_ptr(&mut self, ptr_val: &Value) -> CompileResult<(Pointer, Pointer, u64)> {
        match &self.context.values[ptr_val.0].value {
            ValueDatum::Instruction(Instruction::GetPointer {
                base_ptr,
                ptr_ty,
                offset,
            }) => ok((*base_ptr, *ptr_ty, *offset), Vec::new(), Vec::new()),
            _otherwise => err(
                Vec::new(),
                vec![CompileError::Internal(
                    "Pointer arg for load/store is not a get_ptr instruction.",
                    self.md_mgr
                        .val_to_span(self.context, *ptr_val)
                        .unwrap_or_else(Self::empty_span),
                )],
            ),
        }
    }

    fn initialise_non_aggregate_type(
        &mut self,
        constant: &Constant,
        span: Option<Span>,
    ) -> VirtualRegister {
        let value_size = ir_type_size_in_bytes(self.context, &constant.ty);
        if size_bytes_in_words!(value_size) == 1 {
            match constant.value {
                ConstantValue::Unit | ConstantValue::Bool(false) | ConstantValue::Uint(0) => {
                    return VirtualRegister::Constant(ConstantRegister::Zero)
                }

                ConstantValue::Bool(true) | ConstantValue::Uint(1) => {
                    return VirtualRegister::Constant(ConstantRegister::One)
                }
                _ => (),
            }
        }

        // Get the constant into the namespace.
        let lit = ir_constant_to_ast_literal(constant);
        let data_id = self.data_section.insert_data_value(&lit);

        // Allocate a register for it, and a load instruction.
        let reg = self.reg_seqr.next();
        self.bytecode.push(Op {
            opcode: either::Either::Left(VirtualOp::LWDataId(reg.clone(), data_id)),
            comment: "literal instantiation".into(),
            owning_span: span,
        });

        // Insert the value into the map.
        //self.reg_map.insert(*value, reg.clone());
        //
        // Actually, no, don't.  It's possible for constant values to be
        // reused in the IR, especially with transforms which copy blocks
        // around, like inlining.  The `LW`/`LWDataId` instruction above
        // initialises that constant value but it may be in a conditional
        // block and not actually get evaluated for every possible
        // execution. So using the register later on by pulling it from
        // `self.reg_map` will have a potentially uninitialised register.
        //
        // By not putting it in the map we recreate the `LW` each time it's
        // used, which also isn't ideal.  A better solution is to put this
        // initialisation into the IR itself, and allow for analysis there
        // to determine when it may be initialised and/or reused.

        // Return register.
        reg
    }

    fn initialise_aggregate_type(
        &mut self,
        constant: &Constant,
        value_type: &Type,
        span: Option<Span>,
    ) -> VirtualRegister {
        // A constant struct or array.  We still allocate space for it on
        // the stack, but create the field or element initialisers
        // recursively.

        // Get the total size using the value type. We shouldn't use constant.ty here because
        // the actual type might containt unions which constant.ty doesn't account for.
        let total_size = size_bytes_round_up_to_word_alignment!(ir_type_size_in_bytes(
            self.context,
            value_type,
        ));
        if total_size > compiler_constants::TWENTY_FOUR_BITS {
            todo!("Enormous stack usage for locals.");
        }

        let start_reg = self.reg_seqr.next();

        // We can have zero sized structs and maybe arrays?
        if total_size > 0 {
            // Save the stack pointer.
            self.bytecode.push(Op::unowned_register_move_comment(
                start_reg.clone(),
                VirtualRegister::Constant(ConstantRegister::StackPointer),
                "save register for temporary stack value",
            ));

            let mut alloc_op = Op::unowned_stack_allocate_memory(VirtualImmediate24 {
                value: total_size as u32,
            });
            alloc_op.comment = format!(
                "allocate {} bytes for temporary {}",
                total_size,
                if matches!(&constant.value, ConstantValue::Struct(_)) {
                    "struct"
                } else {
                    "array"
                },
            );
            self.bytecode.push(alloc_op);

            // Fill in the fields.
            self.initialise_constant_memory(constant, value_type, &start_reg, 0, span);
        }

        // Return the start ptr.
        start_reg
    }

    // Get the reg corresponding to `value`. Returns None if the value is not in reg_map or is not
    // a constant.
    fn value_to_register_or_none(&mut self, value: &Value) -> Option<VirtualRegister> {
        let value_type = value.get_type(self.context).unwrap();
        match self.reg_map.get(value) {
            Some(reg) => Some(reg.clone()),
            None => {
                match &self.context.values[value.0].value {
                    // Handle constants.
                    ValueDatum::Constant(constant) => {
                        let span = self.md_mgr.val_to_span(self.context, *value);
                        match &value_type {
                            Type::Unit
                            | Type::Bool
                            | Type::Uint(_)
                            | Type::B256
                            | Type::String(_)
                            | Type::Pointer(_) => {
                                Some(self.initialise_non_aggregate_type(constant, span))
                            }
                            Type::Array(_) | Type::Struct(_) | Type::Union(_) => {
                                Some(self.initialise_aggregate_type(constant, &value_type, span))
                            }
                        }
                    }
                    _otherwise => None,
                }
            }
        }
    }

    // Same as `value_to_register_or_none` but returns a new register if no register is found or if
    // `value` is not a constant.
    fn value_to_register(&mut self, value: &Value) -> VirtualRegister {
        match self.value_to_register_or_none(value) {
            Some(reg) => reg,
            None => {
                // Just make a new register for this value.
                let reg = self.reg_seqr.next();
                self.reg_map.insert(*value, reg.clone());
                reg
            }
        }
    }

    fn number_to_reg(&mut self, offset: u64, offset_reg: &VirtualRegister, span: Option<Span>) {
        if offset > compiler_constants::TWENTY_FOUR_BITS {
            todo!("Absolutely giant arrays.");
        }

        // Use bitwise ORs and SHIFTs to crate a 24 bit value in a register.
        self.bytecode.push(Op {
            opcode: either::Either::Left(VirtualOp::ORI(
                offset_reg.clone(),
                VirtualRegister::Constant(ConstantRegister::Zero),
                VirtualImmediate12 {
                    value: (offset >> 12) as u16,
                },
            )),
            comment: "get extract offset high bits".into(),
            owning_span: span.clone(),
        });
        self.bytecode.push(Op {
            opcode: either::Either::Left(VirtualOp::SLLI(
                offset_reg.clone(),
                offset_reg.clone(),
                VirtualImmediate12 { value: 12 },
            )),
            comment: "shift extract offset high bits".into(),
            owning_span: span.clone(),
        });
        self.bytecode.push(Op {
            opcode: either::Either::Left(VirtualOp::ORI(
                offset_reg.clone(),
                offset_reg.clone(),
                VirtualImmediate12 {
                    value: (offset & 0xfff) as u16,
                },
            )),
            comment: "get extract offset low bits".into(),
            owning_span: span,
        });
    }

    // Insert asm instructions to initialise a stack variable of type `value_type` with a Constant
    // `constant`. Here, `value_type` accounts for the fact that the stack variable might include
    // unions.
    //
    // If the initialiser is smaller than `value_type` (e.g. initialising a union with one of
    // its small variants), add zero padding.
    fn initialise_constant_memory(
        &mut self,
        constant: &Constant,
        value_type: &Type,
        start_reg: &VirtualRegister,
        offs_in_words: u64,
        span: Option<Span>,
    ) -> u64 {
        let value_size = ir_type_size_in_bytes(self.context, value_type);
        let value_size_in_words = size_bytes_in_words!(value_size);

        if matches!(constant.value, ConstantValue::Undef) {
            // We don't need to actually create an initialiser, but we do need to return the
            // field size in words.
            return size_bytes_in_words!(value_size);
        }

        match &value_type {
            Type::Unit | Type::Bool | Type::Uint(_) | Type::Pointer(_) => {
                // Get the constant into the namespace.
                let lit = ir_constant_to_ast_literal(constant);
                let data_id = self.data_section.insert_data_value(&lit);

                // Load the initialiser value.
                let init_reg = self.reg_seqr.next();
                self.bytecode.push(Op {
                    opcode: either::Either::Left(VirtualOp::LWDataId(init_reg.clone(), data_id)),
                    comment: "literal instantiation for aggregate field".into(),
                    owning_span: span.clone(),
                });

                if offs_in_words > compiler_constants::TWELVE_BITS {
                    let offs_reg = self.reg_seqr.next();
                    self.number_to_reg(offs_in_words, &offs_reg, span.clone());
                    self.bytecode.push(Op {
                        opcode: either::Either::Left(VirtualOp::ADD(
                            start_reg.clone(),
                            start_reg.clone(),
                            offs_reg.clone(),
                        )),
                        comment: "calculate byte offset to aggregate field".into(),
                        owning_span: span.clone(),
                    });
                    self.bytecode.push(Op {
                        opcode: Either::Left(VirtualOp::SW(
                            start_reg.clone(),
                            init_reg,
                            VirtualImmediate12 { value: 0 },
                        )),
                        comment: "initialise aggregate field".into(),
                        owning_span: span,
                    });
                } else {
                    self.bytecode.push(Op {
                        opcode: Either::Left(VirtualOp::SW(
                            start_reg.clone(),
                            init_reg,
                            VirtualImmediate12 {
                                value: offs_in_words as u16,
                            },
                        )),
                        comment: "initialise aggregate field".into(),
                        owning_span: span,
                    });
                }

                1
            }
            Type::B256 | Type::String(_) => {
                // Get the constant into the namespace.
                let lit = ir_constant_to_ast_literal(constant);
                let data_id = self.data_section.insert_data_value(&lit);

                // Load the initialiser value.
                let init_reg = self.reg_seqr.next();
                self.bytecode.push(Op {
                    opcode: either::Either::Left(VirtualOp::LWDataId(init_reg.clone(), data_id)),
                    comment: "literal instantiation for aggregate field".into(),
                    owning_span: span.clone(),
                });

                // Write the initialiser to memory.  Most Literals are 1 word, B256 is 32 bytes and
                // needs to use a MCP instruction.
                let offs_reg = self.reg_seqr.next();
                if offs_in_words * 8 > compiler_constants::TWELVE_BITS {
                    self.number_to_reg(offs_in_words * 8, &offs_reg, span.clone());
                    self.bytecode.push(Op {
                        opcode: either::Either::Left(VirtualOp::ADD(
                            offs_reg.clone(),
                            start_reg.clone(),
                            offs_reg.clone(),
                        )),
                        comment: "calculate byte offset to aggregate field".into(),
                        owning_span: span.clone(),
                    });
                } else {
                    self.bytecode.push(Op {
                        opcode: either::Either::Left(VirtualOp::ADDI(
                            offs_reg.clone(),
                            start_reg.clone(),
                            VirtualImmediate12 {
                                value: (offs_in_words * 8) as u16,
                            },
                        )),
                        comment: "calculate byte offset to aggregate field".into(),
                        owning_span: span.clone(),
                    });
                }
                self.bytecode.push(Op {
                    opcode: Either::Left(VirtualOp::MCPI(
                        offs_reg,
                        init_reg,
                        VirtualImmediate12 {
                            value: value_size as u16,
                        },
                    )),
                    comment: "initialise aggregate field".into(),
                    owning_span: span,
                });

                value_size_in_words
            }
            Type::Array(aggregate) => {
                match (&constant.value, &self.context.aggregates[aggregate.0]) {
                    (ConstantValue::Array(items), AggregateContent::ArrayType(element_type, _)) => {
                        // Recurse for each item, accumulating the field offset and the final size.
                        items.iter().fold(0, |local_offs, item| {
                            local_offs
                                + self.initialise_constant_memory(
                                    item,
                                    element_type,
                                    start_reg,
                                    offs_in_words + local_offs,
                                    span.clone(),
                                )
                        })
                    }
                    _ => unreachable!("Inconsistent types for constant initialisation"),
                }
            }
            Type::Struct(aggregate) => {
                match (&constant.value, &self.context.aggregates[aggregate.0]) {
                    (ConstantValue::Struct(items), AggregateContent::FieldTypes(field_tys)) => {
                        // Recurse for each item, accumulating the field offset and the final size.
                        items.iter().zip(field_tys.iter()).fold(
                            0,
                            |local_offs, (item, field_tys)| {
                                local_offs
                                    + self.initialise_constant_memory(
                                        item,
                                        field_tys,
                                        start_reg,
                                        offs_in_words + local_offs,
                                        span.clone(),
                                    )
                            },
                        )
                    }
                    _ => unreachable!("Inconsistent types for constant initialisation"),
                }
            }
            Type::Union(_) => {
                // If the literal we're trying to initialise with is smaller than than the actual
                // size of the union, then a padding of zeros is required. Calculate the size of
                // the padding and set the appropriate bytes to zero.
                let constant_size = ir_type_size_in_bytes(self.context, &constant.ty);
                assert!(constant_size <= value_size);
                let padding_size = value_size - constant_size;
                let padding_size_in_words = size_bytes_in_words!(padding_size);

                if padding_size > 0 {
                    // Store padding of zeros and then store the value itself
                    let union_base_reg = self.reg_seqr.next();
                    if offs_in_words * 8 > compiler_constants::TWELVE_BITS {
                        let offs_reg = self.reg_seqr.next();
                        self.number_to_reg(offs_in_words * 8, &offs_reg, span.clone());
                        self.bytecode.push(Op {
                            opcode: either::Either::Left(VirtualOp::ADD(
                                union_base_reg.clone(),
                                start_reg.clone(),
                                offs_reg.clone(),
                            )),
                            comment: "get base pointer for union".into(),
                            owning_span: span.clone(),
                        });
                    } else {
                        self.bytecode.push(Op {
                            opcode: either::Either::Left(VirtualOp::ADDI(
                                union_base_reg.clone(),
                                start_reg.clone(),
                                VirtualImmediate12 {
                                    value: (offs_in_words * 8) as u16,
                                },
                            )),
                            comment: "get base pointer for union".into(),
                            owning_span: span.clone(),
                        });
                    }
                    self.bytecode.push(Op {
                        opcode: Either::Left(VirtualOp::MCLI(
                            union_base_reg,
                            VirtualImmediate18 {
                                value: padding_size as u32,
                            },
                        )),
                        comment: "clear padding for union initialisation".into(),
                        owning_span: span.clone(),
                    });
                }

                // Now do the actual initialisation
                self.initialise_constant_memory(
                    constant,
                    &constant.ty,
                    start_reg,
                    offs_in_words + padding_size_in_words,
                    span,
                );

                value_size_in_words
            }
        }
    }

    fn block_to_label(&mut self, block: &Block) -> Label {
        match self.label_map.get(block) {
            Some(label) => label.clone(),
            None => {
                let label = self.reg_seqr.get_label();
                self.label_map.insert(*block, label.clone());
                label
            }
        }
    }
}

fn ir_constant_to_ast_literal(constant: &Constant) -> Literal {
    match &constant.value {
        ConstantValue::Undef => unreachable!("Cannot convert 'undef' to a literal."),
        ConstantValue::Unit => Literal::U64(0), // No unit.
        ConstantValue::Bool(b) => Literal::Boolean(*b),
        ConstantValue::Uint(n) => Literal::U64(*n),
        ConstantValue::B256(bs) => Literal::B256(*bs),
        ConstantValue::String(bs) => {
            // ConstantValue::String bytes are guaranteed to be valid UTF8.
            let s = std::str::from_utf8(bs).unwrap();
            Literal::String(Span::new(std::sync::Arc::from(s), 0, s.len(), None).unwrap())
        }
        ConstantValue::Array(_) | ConstantValue::Struct(_) => {
            unreachable!("Cannot convert aggregates to a literal.")
        }
    }
}

// -------------------------------------------------------------------------------------------------

pub fn ir_type_size_in_bytes(context: &Context, ty: &Type) -> u64 {
    match ty {
        Type::Unit | Type::Bool | Type::Uint(_) | Type::Pointer(_) => 8,
        Type::B256 => 32,
        Type::String(n) => size_bytes_round_up_to_word_alignment!(n),
        Type::Array(aggregate) => {
            if let AggregateContent::ArrayType(el_ty, cnt) = &context.aggregates[aggregate.0] {
                cnt * ir_type_size_in_bytes(context, el_ty)
            } else {
                unreachable!("Wrong content for array.")
            }
        }
        Type::Struct(aggregate) => {
            if let AggregateContent::FieldTypes(field_tys) = &context.aggregates[aggregate.0] {
                // Sum up all the field sizes.
                field_tys
                    .iter()
                    .map(|field_ty| ir_type_size_in_bytes(context, field_ty))
                    .sum()
            } else {
                unreachable!("Wrong content for struct.")
            }
        }
        Type::Union(aggregate) => {
            if let AggregateContent::FieldTypes(field_tys) = &context.aggregates[aggregate.0] {
                // Find the max size for field sizes.
                field_tys
                    .iter()
                    .map(|field_ty| ir_type_size_in_bytes(context, field_ty))
                    .max()
                    .unwrap_or(0)
            } else {
                unreachable!("Wrong content for union.")
            }
        }
    }
}

// Aggregate (nested) field offset in words and size in bytes.
pub fn aggregate_idcs_to_field_layout(
    context: &Context,
    ty: &Type,
    idcs: &[u64],
) -> ((u64, u64), Type) {
    idcs.iter()
        .fold(((0, 0), *ty), |((offs, _), ty), idx| match ty {
            Type::Struct(aggregate) => {
                let idx = *idx as usize;
                let field_types = &context.aggregates[aggregate.0].field_types();
                let field_type = field_types[idx];
                let field_offs_in_bytes = field_types
                    .iter()
                    .take(idx)
                    .map(|field_ty| ir_type_size_in_bytes(context, field_ty))
                    .sum::<u64>();
                let field_size_in_bytes = ir_type_size_in_bytes(context, &field_type);

                (
                    (
                        offs + size_bytes_in_words!(field_offs_in_bytes),
                        field_size_in_bytes,
                    ),
                    field_type,
                )
            }

            Type::Union(aggregate) => {
                let idx = *idx as usize;
                let field_type = context.aggregates[aggregate.0].field_types()[idx];
                let union_size_in_bytes = ir_type_size_in_bytes(context, &ty);
                let field_size_in_bytes = ir_type_size_in_bytes(context, &field_type);

                // The union fields are at offset (union_size - variant_size) due to left padding.
                (
                    (
                        offs + size_bytes_in_words!(union_size_in_bytes - field_size_in_bytes),
                        field_size_in_bytes,
                    ),
                    field_type,
                )
            }

            _otherwise => panic!("Attempt to access field in non-aggregate."),
        })
}

// -------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use sway_ir::parser::parse;

    use std::path::PathBuf;

    #[test]
    fn ir_to_asm_tests() {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let dir: PathBuf = format!("{}/tests/ir_to_asm", manifest_dir).into();
        for entry in std::fs::read_dir(dir).unwrap() {
            // We're only interested in the `.sw` files here.
            let path = entry.unwrap().path();
            match path.extension().unwrap().to_str() {
                Some("ir") => {
                    //
                    // Run the tests!
                    //
                    tracing::info!("---- IR To ASM: {:?} ----", path);
                    test_ir_to_asm(path);
                }
                Some("asm") | Some("disabled") => (),
                _ => panic!(
                    "File with invalid extension in tests dir: {:?}",
                    path.file_name().unwrap_or(path.as_os_str())
                ),
            }
        }
    }

    fn test_ir_to_asm(mut path: PathBuf) {
        let input_bytes = std::fs::read(&path).unwrap();
        let input = String::from_utf8_lossy(&input_bytes);

        path.set_extension("asm");

        let expected_bytes = std::fs::read(&path).unwrap();
        let expected = String::from_utf8_lossy(&expected_bytes);

        let ir = parse(&input).expect("parsed ir");
        let asm_result = compile_ir_to_asm(&ir, None);

        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let asm = asm_result.unwrap(&mut warnings, &mut errors);
        assert!(warnings.is_empty() && errors.is_empty());

        let asm_script = format!("{}", asm);
        if asm_script != expected {
            tracing::info!("{}", prettydiff::diff_lines(&expected, &asm_script));
            panic!();
        }
    }
}

// =================================================================================================
