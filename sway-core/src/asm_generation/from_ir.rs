// =================================================================================================
// Newer IR code gen.
//
// NOTE:  This is converting IR to Vec<Op> first, and then to finalized VM bytecode much like the
// original code.  This is to keep things simple, and to reuse the current tools like DataSection.
//
// But this is not ideal and needs to be refactored:
// - AsmNamespace is tied to data structures from other stages like Ident and Literal.

use std::collections::HashMap;

use crate::{
    asm_generation::{
        build_contract_abi_switch, build_preamble, finalized_asm::FinalizedAsm,
        register_sequencer::RegisterSequencer, AbstractInstructionSet, DataId, DataSection,
        SwayAsmSet,
    },
    asm_lang::{virtual_register::*, Label, Op, VirtualImmediate12, VirtualImmediate24, VirtualOp},
    error::*,
    parse_tree::Literal,
    BuildConfig,
};

use sway_ir::*;
use sway_types::span::Span;

use either::Either;

pub fn compile_ir_to_asm(ir: &Context, build_config: &BuildConfig) -> CompileResult<FinalizedAsm> {
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

    if build_config.print_intermediate_asm {
        println!("{}", asm);
    }

    let finalized_asm = asm
        .remove_unnecessary_jumps()
        .allocate_registers(&mut reg_seqr)
        .optimize();

    if build_config.print_finalized_asm {
        println!("{}", finalized_asm);
    }

    check!(
        crate::checks::check_invalid_opcodes(&finalized_asm),
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

    // Reg map, const map and var map are all tracking IR values to VM values.  Var map has an
    // optional (None) register until its first assignment.
    reg_map: HashMap<Value, VirtualRegister>,
    ptr_map: HashMap<Pointer, Storage>,

    // Stack base register, copied from $SP at the start, but only if we have stack storage.
    stack_base_reg: Option<VirtualRegister>,

    // The layouts of each aggregate; their whole size in bytes and field offsets in words.
    aggregate_layouts: HashMap<Aggregate, (u64, Vec<FieldLayout>)>,

    // IR context we're compiling.
    context: &'ir Context,

    // Final resulting VM bytecode ops.
    bytecode: Vec<Op>,
}

struct FieldLayout {
    offset_in_words: u64, // Use words because LW/SW do.
    size_in_bytes: u64,   // Use bytes because CFEI/MCP do.
}

// NOTE: For stack storage we need to be aware:
// - sizes are in bytes; CFEI reserves in bytes.
// - offsets are in 64-bit words; LW/SW reads/writes to word offsets. XXX Wrap in a WordOffset struct.

#[derive(Clone, Debug)]
pub(super) enum Storage {
    Data(DataId),              // Const storage in the data section.
    Register(VirtualRegister), // Storage in a register.
    Stack(u64), // Storage in the runtime stack starting at an absolute word offset.  Essentially a global.
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
            aggregate_layouts: HashMap::new(),
            context,
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
        Span {
            span: pest::Span::new(std::sync::Arc::from(msg), 0, msg.len()).unwrap(),
            path: None,
        }
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
                    Type::Unit | Type::Bool | Type::Uint(_) => {
                        let reg = self.reg_seqr.next();
                        self.ptr_map.insert(*ptr, Storage::Register(reg));
                    }
                    Type::B256 => {
                        self.ptr_map.insert(*ptr, Storage::Stack(stack_base));
                        stack_base += 4;
                    }
                    Type::String(count) => {
                        self.ptr_map.insert(*ptr, Storage::Stack(stack_base));

                        // XXX `count` is a CHAR count, not BYTE count.  We need to count the size
                        // of the string before allocating.  For now assuming CHAR == BYTE.
                        stack_base += size_bytes_in_words!(count);
                    }
                    Type::Array(aggregate) => {
                        // Store this aggregate at the current stack base.
                        self.ptr_map.insert(*ptr, Storage::Stack(stack_base));

                        // Reserve space by incrementing the base.
                        stack_base += size_bytes_in_words!(self.aggregate_size(&aggregate));
                    }
                    Type::Struct(aggregate) => {
                        // Store this aggregate at the current stack base.
                        self.ptr_map.insert(*ptr, Storage::Stack(stack_base));

                        // Reserve space by incrementing the base.
                        stack_base += size_bytes_in_words!(self.aggregate_size(&aggregate));
                    }
                    Type::Union(aggregate) => {
                        // Store this aggregate AND a 64bit tag at the current stack base.
                        self.ptr_map.insert(*ptr, Storage::Stack(stack_base));

                        // Reserve space by incrementing the base.
                        stack_base +=
                            size_bytes_in_words!(self.aggregate_max_field_size(&aggregate));
                    }
                    Type::ContractCaller(_) => {
                        self.ptr_map.insert(*ptr, Storage::Stack(stack_base));

                        // Reserve space for the contract address only.
                        stack_base += 4;
                    }
                    Type::Contract => {
                        unimplemented!("contract on the stack?")
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
                if stack_base * 8 > crate::asm_generation::compiler_constants::TWENTY_FOUR_BITS {
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
        // Compile instructions.
        self.add_locals(function);

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
                Instruction::AsmBlock(asm, args) => {
                    check!(
                        self.compile_asm_block(instr_val, asm, args),
                        return err(warnings, errors),
                        warnings,
                        errors
                    )
                }
                Instruction::Branch(to_block) => self.compile_branch(block, to_block),
                Instruction::Call(..) => {
                    errors.push(CompileError::Internal(
                        "Calls are not yet supported.",
                        instr_val
                            .get_span(self.context)
                            .unwrap_or_else(Self::empty_span),
                    ));
                    return err(warnings, errors);
                }
                Instruction::ConditionalBranch {
                    cond_value,
                    true_block,
                    false_block,
                } => self.compile_conditional_branch(cond_value, block, true_block, false_block),
                Instruction::ExtractElement {
                    array,
                    ty,
                    index_val,
                } => self.compile_extract_element(instr_val, array, ty, index_val),
                Instruction::ExtractValue {
                    aggregate,
                    ty,
                    indices,
                } => self.compile_extract_value(instr_val, aggregate, ty, indices),
                Instruction::GetPointer(ptr) => self.compile_get_pointer(instr_val, ptr),
                Instruction::InsertElement {
                    array,
                    ty,
                    value,
                    index_val,
                } => self.compile_insert_element(instr_val, array, ty, value, index_val),
                Instruction::InsertValue {
                    aggregate,
                    ty,
                    value,
                    indices,
                } => self.compile_insert_value(instr_val, aggregate, ty, value, indices),
                Instruction::Load(ptr) => self.compile_load(instr_val, ptr),
                Instruction::Nop => (),
                Instruction::Phi(_) => (), // Managing the phi value is done in br and cbr compilation.
                Instruction::Ret(ret_val, ty) => self.compile_ret(instr_val, ret_val, ty),
                Instruction::Store { ptr, stored_val } => {
                    self.compile_store(instr_val, ptr, stored_val)
                }
            }
        } else {
            errors.push(CompileError::Internal(
                "Value not an instruction.",
                instr_val
                    .get_span(self.context)
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
                            span: reg_name.span().clone(),
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
            let op_span = match op.span_md_idx {
                None => {
                    // XXX This sucks.  We have two options: not needing a span to parse the opcode
                    // (which is used for the error) or force a span from the IR somehow, maybe by
                    // using a .ir file?  OK, we have a third and best option: do the parsing of
                    // asm blocks in the parser itself, so we can verify them all the way back then
                    // and not have to worry about them being malformed all the way down here in
                    // codegen.
                    Self::empty_span()
                }
                Some(span_md_idx) => match span_md_idx.to_span(self.context) {
                    Ok(span) => span,
                    Err(ir_error) => {
                        errors.push(CompileError::InternalOwned(
                            ir_error.to_string(),
                            instr_val
                                .get_span(self.context)
                                .unwrap_or_else(Self::empty_span),
                        ));
                        return err(warnings, errors);
                    }
                },
            };
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
                        span: ret_reg_name.span().clone(),
                    });
                    return err(warnings, errors);
                }
            };
            let instr_reg = self.reg_seqr.next();
            inline_ops.push(Op {
                opcode: Either::Left(VirtualOp::MOVE(instr_reg.clone(), ret_reg)),
                comment: "return value from inline asm".into(),
                owning_span: instr_val.get_span(self.context),
            });
            self.reg_map.insert(*instr_val, instr_reg);
        }

        self.bytecode.append(&mut inline_ops);

        ok((), warnings, errors)
    }

    fn compile_branch(&mut self, from_block: &Block, to_block: &Block) {
        self.compile_branch_to_phi_value(from_block, to_block);

        let label = self.block_to_label(to_block);
        self.bytecode.push(Op::jump_to_label(label));
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

        let false_label = self.block_to_label(false_block);
        self.bytecode.push(Op::jump_if_not_equal(
            cond_reg,
            VirtualRegister::Constant(ConstantRegister::One),
            false_label,
        ));

        let true_label = self.block_to_label(true_block);
        self.bytecode.push(Op::jump_to_label(true_label));
    }

    fn compile_branch_to_phi_value(&mut self, from_block: &Block, to_block: &Block) {
        if let Some(local_val) = to_block.get_phi_val_coming_from(self.context, from_block) {
            let local_reg = self.value_to_register(&local_val);
            let phi_reg = self.value_to_register(&to_block.get_phi(self.context));
            self.bytecode
                .push(Op::unowned_register_move(phi_reg, local_reg));
        }
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

        // We could put the OOB check here, though I'm now thinking it would be too wasteful.
        // See compile_bounds_assertion() in expression/array.rs (or look in Git history).

        let instr_reg = self.reg_seqr.next();
        let elem_size = self.ir_type_size_in_bytes(&ty.get_elem_type(self.context).unwrap());
        if elem_size <= 8 {
            self.bytecode.push(Op {
                opcode: Either::Left(VirtualOp::MULI(
                    index_reg.clone(),
                    index_reg.clone(),
                    VirtualImmediate12 { value: 8 },
                )),
                comment: "extract_element relative offset".into(),
                owning_span: instr_val.get_span(self.context),
            });
            let elem_offs_reg = self.reg_seqr.next();
            self.bytecode.push(Op {
                opcode: Either::Left(VirtualOp::ADD(elem_offs_reg.clone(), base_reg, index_reg)),
                comment: "extract_element absolute offset".into(),
                owning_span: instr_val.get_span(self.context),
            });
            self.bytecode.push(Op {
                opcode: Either::Left(VirtualOp::LW(
                    instr_reg.clone(),
                    elem_offs_reg,
                    VirtualImmediate12 { value: 0 },
                )),
                comment: "extract_element".into(),
                owning_span: instr_val.get_span(self.context),
            });
        } else {
            // Value too big for a register, so we return the memory offset.
            if elem_size > crate::asm_generation::compiler_constants::TWELVE_BITS {
                let size_data_id = self
                    .data_section
                    .insert_data_value(&Literal::U64(elem_size));
                let size_reg = self.reg_seqr.next();
                self.bytecode.push(Op {
                    opcode: Either::Left(VirtualOp::LWDataId(size_reg.clone(), size_data_id)),
                    owning_span: instr_val.get_span(self.context),
                    comment: "loading element size for relative offset".into(),
                });
                self.bytecode.push(Op {
                    opcode: Either::Left(VirtualOp::MUL(instr_reg.clone(), index_reg, size_reg)),
                    comment: "extract_element relative offset".into(),
                    owning_span: instr_val.get_span(self.context),
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
                    owning_span: instr_val.get_span(self.context),
                });
            }
            self.bytecode.push(Op {
                opcode: Either::Left(VirtualOp::ADD(
                    instr_reg.clone(),
                    base_reg,
                    instr_reg.clone(),
                )),
                comment: "extract_element absolute offset".into(),
                owning_span: instr_val.get_span(self.context),
            });
        }

        self.reg_map.insert(*instr_val, instr_reg);
    }

    fn compile_extract_value(
        &mut self,
        instr_val: &Value,
        aggregate: &Value,
        ty: &Aggregate,
        indices: &[u64],
    ) {
        // Base register should pointer to some stack allocated memory.
        let base_reg = self.value_to_register(aggregate);
        let (extract_offset, value_size) = self.aggregate_idcs_to_field_layout(ty, indices);

        let instr_reg = self.reg_seqr.next();
        if value_size <= 8 {
            if extract_offset > crate::asm_generation::compiler_constants::TWELVE_BITS {
                let offset_reg = self.reg_seqr.next();
                self.number_to_reg(
                    extract_offset,
                    &offset_reg,
                    instr_val.get_span(self.context),
                );
                self.bytecode.push(Op {
                    opcode: Either::Left(VirtualOp::ADD(
                        offset_reg.clone(),
                        base_reg.clone(),
                        base_reg,
                    )),
                    comment: "add array base to offset".into(),
                    owning_span: instr_val.get_span(self.context),
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
                    owning_span: instr_val.get_span(self.context),
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
                    owning_span: instr_val.get_span(self.context),
                });
            }
        } else {
            // Value too big for a register, so we return the memory offset.
            if extract_offset * 8 > crate::asm_generation::compiler_constants::TWELVE_BITS {
                let offset_reg = self.reg_seqr.next();
                self.number_to_reg(
                    extract_offset * 8,
                    &offset_reg,
                    instr_val.get_span(self.context),
                );
                self.bytecode.push(Op {
                    opcode: either::Either::Left(VirtualOp::ADD(
                        instr_reg.clone(),
                        base_reg,
                        offset_reg,
                    )),
                    comment: "extract address".into(),
                    owning_span: instr_val.get_span(self.context),
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
                    owning_span: instr_val.get_span(self.context),
                });
            }
        }

        self.reg_map.insert(*instr_val, instr_reg);
    }

    fn compile_get_pointer(&mut self, instr_val: &Value, ptr: &Pointer) {
        // `get_ptr` is like a `load` except the value isn't dereferenced.
        match self.ptr_map.get(ptr) {
            None => unimplemented!("BUG? Uninitialised pointer."),
            Some(storage) => match storage.clone() {
                Storage::Data(_data_id) => {
                    // Not sure if we'll ever need this.
                    unimplemented!("TODO get_ptr() into the data section.");
                }
                Storage::Register(var_reg) => {
                    self.reg_map.insert(*instr_val, var_reg);
                }
                Storage::Stack(word_offs) => {
                    let word_offs = word_offs * 8;
                    let instr_reg = self.reg_seqr.next();
                    if word_offs > crate::asm_generation::compiler_constants::TWELVE_BITS {
                        self.number_to_reg(word_offs, &instr_reg, instr_val.get_span(self.context));
                    } else {
                        self.bytecode.push(Op {
                            opcode: either::Either::Left(VirtualOp::ADDI(
                                instr_reg.clone(),
                                self.stack_base_reg.as_ref().unwrap().clone(),
                                VirtualImmediate12 {
                                    value: (word_offs) as u16,
                                },
                            )),
                            comment: "get_ptr".into(),
                            owning_span: instr_val.get_span(self.context),
                        });
                    }
                    self.reg_map.insert(*instr_val, instr_reg);
                }
            },
        }
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

        let elem_size = self.ir_type_size_in_bytes(&ty.get_elem_type(self.context).unwrap());
        if elem_size <= 8 {
            self.bytecode.push(Op {
                opcode: Either::Left(VirtualOp::MULI(
                    index_reg.clone(),
                    index_reg.clone(),
                    VirtualImmediate12 { value: 8 },
                )),
                comment: "insert_element relative offset".into(),
                owning_span: instr_val.get_span(self.context),
            });
            let elem_offs_reg = self.reg_seqr.next();
            self.bytecode.push(Op {
                opcode: Either::Left(VirtualOp::ADD(
                    elem_offs_reg.clone(),
                    base_reg.clone(),
                    index_reg,
                )),
                comment: "insert_element absolute offset".into(),
                owning_span: instr_val.get_span(self.context),
            });
            self.bytecode.push(Op {
                opcode: Either::Left(VirtualOp::SW(
                    elem_offs_reg,
                    insert_reg,
                    VirtualImmediate12 { value: 0 },
                )),
                comment: "insert_element".into(),
                owning_span: instr_val.get_span(self.context),
            });
        } else {
            // Element size is larger than 8; we switch to bytewise offsets and sizes and use MCP.
            if elem_size > crate::asm_generation::compiler_constants::TWELVE_BITS {
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
                    owning_span: instr_val.get_span(self.context),
                });
                self.bytecode.push(Op {
                    opcode: Either::Left(VirtualOp::ADD(
                        elem_index_offs_reg.clone(),
                        base_reg.clone(),
                        elem_index_offs_reg.clone(),
                    )),
                    comment: "insert_element absolute offset".into(),
                    owning_span: instr_val.get_span(self.context),
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
                    owning_span: instr_val.get_span(self.context),
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
        aggregate: &Value,
        ty: &Aggregate,
        value: &Value,
        indices: &[u64],
    ) {
        // Base register should point to some stack allocated memory.
        let base_reg = self.value_to_register(aggregate);

        let insert_reg = self.value_to_register(value);
        let (insert_offs, value_size) = self.aggregate_idcs_to_field_layout(ty, indices);

        let indices_str = indices
            .iter()
            .map(|idx| format!("{}", idx))
            .collect::<Vec<String>>()
            .join(",");
        if value_size <= 8 {
            if insert_offs > crate::asm_generation::compiler_constants::TWELVE_BITS {
                let insert_offs_reg = self.reg_seqr.next();
                self.number_to_reg(
                    insert_offs,
                    &insert_offs_reg,
                    instr_val.get_span(self.context),
                );
                self.bytecode.push(Op {
                    opcode: Either::Left(VirtualOp::ADD(
                        base_reg.clone(),
                        base_reg.clone(),
                        insert_offs_reg,
                    )),
                    comment: "insert_value absolute offset".into(),
                    owning_span: instr_val.get_span(self.context),
                });
                self.bytecode.push(Op {
                    opcode: Either::Left(VirtualOp::SW(
                        base_reg.clone(),
                        insert_reg,
                        VirtualImmediate12 { value: 0 },
                    )),
                    comment: format!("insert_value @ {}", indices_str),
                    owning_span: instr_val.get_span(self.context),
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
                    owning_span: instr_val.get_span(self.context),
                });
            }
        } else {
            let offs_reg = self.reg_seqr.next();
            if insert_offs * 8 > crate::asm_generation::compiler_constants::TWELVE_BITS {
                self.number_to_reg(insert_offs * 8, &offs_reg, instr_val.get_span(self.context));
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
                    owning_span: instr_val.get_span(self.context),
                });
            }
            if value_size > crate::asm_generation::compiler_constants::TWELVE_BITS {
                let size_reg = self.reg_seqr.next();
                self.number_to_reg(value_size, &size_reg, instr_val.get_span(self.context));
                self.bytecode.push(Op {
                    opcode: Either::Left(VirtualOp::MCP(offs_reg, insert_reg, size_reg)),
                    comment: "store struct field value".into(),
                    owning_span: instr_val.get_span(self.context),
                });
            } else {
                self.bytecode.push(Op {
                    opcode: Either::Left(VirtualOp::MCPI(
                        offs_reg,
                        insert_reg,
                        VirtualImmediate12 {
                            value: value_size as u16,
                        },
                    )),
                    comment: "store struct field value".into(),
                    owning_span: instr_val.get_span(self.context),
                });
            }
        }

        // We set the 'instruction' register to the base register, so that cascading inserts will
        // work.
        self.reg_map.insert(*instr_val, base_reg);
    }

    fn compile_load(&mut self, instr_val: &Value, ptr: &Pointer) {
        let load_size_in_words =
            size_bytes_in_words!(self.ir_type_size_in_bytes(ptr.get_type(self.context)));
        let instr_reg = self.reg_seqr.next();
        match self.ptr_map.get(ptr) {
            None => unimplemented!("BUG? Uninitialised pointer."),
            Some(storage) => match storage.clone() {
                Storage::Data(data_id) => {
                    self.bytecode.push(Op {
                        opcode: Either::Left(VirtualOp::LWDataId(instr_reg.clone(), data_id)),
                        comment: "load constant".into(),
                        owning_span: instr_val.get_span(self.context),
                    });
                }
                Storage::Register(var_reg) => {
                    self.bytecode.push(Op {
                        opcode: Either::Left(VirtualOp::MOVE(instr_reg.clone(), var_reg)),
                        comment: String::new(),
                        owning_span: instr_val.get_span(self.context),
                    });
                }
                Storage::Stack(word_offs) => {
                    let base_reg = self.stack_base_reg.as_ref().unwrap().clone();
                    // XXX Need to check for zero sized types?
                    if load_size_in_words == 1 {
                        // Value can fit in a register, so we load the value.
                        if word_offs > crate::asm_generation::compiler_constants::TWELVE_BITS {
                            let offs_reg = self.reg_seqr.next();
                            self.bytecode.push(Op {
                                opcode: Either::Left(VirtualOp::ADD(
                                    base_reg.clone(),
                                    base_reg,
                                    offs_reg.clone(),
                                )),
                                comment: "insert_value absolute offset".into(),
                                owning_span: instr_val.get_span(self.context),
                            });
                            self.bytecode.push(Op {
                                opcode: Either::Left(VirtualOp::LW(
                                    instr_reg.clone(),
                                    offs_reg,
                                    VirtualImmediate12 { value: 0 },
                                )),
                                comment: "load value".into(),
                                owning_span: instr_val.get_span(self.context),
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
                                owning_span: instr_val.get_span(self.context),
                            });
                        }
                    } else {
                        // Value too big for a register, so we return the memory offset.  This is
                        // what LW to the data section does, via LWDataId.
                        let word_offs = word_offs * 8;
                        if word_offs > crate::asm_generation::compiler_constants::TWELVE_BITS {
                            let offs_reg = self.reg_seqr.next();
                            self.number_to_reg(
                                word_offs,
                                &offs_reg,
                                instr_val.get_span(self.context),
                            );
                            self.bytecode.push(Op {
                                opcode: either::Either::Left(VirtualOp::ADD(
                                    instr_reg.clone(),
                                    base_reg,
                                    offs_reg,
                                )),
                                comment: "load address".into(),
                                owning_span: instr_val.get_span(self.context),
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
                                owning_span: instr_val.get_span(self.context),
                            });
                        }
                    }
                }
            },
        }
        self.reg_map.insert(*instr_val, instr_reg);
    }

    fn compile_ret(&mut self, instr_val: &Value, ret_val: &Value, ret_type: &Type) {
        if ret_type == &Type::Unit {
            // Unit returns should always be zero, although because they can be omitted from
            // functions, the register is sometimes uninitialized. Manually return zero in this
            // case.
            self.bytecode.push(Op {
                opcode: Either::Left(VirtualOp::RET(VirtualRegister::Constant(
                    ConstantRegister::Zero,
                ))),
                owning_span: instr_val.get_span(self.context),
                comment: "returning unit as zero".into(),
            });
        } else {
            let ret_reg = self.value_to_register(ret_val);
            let size_in_bytes = self.ir_type_size_in_bytes(ret_type);

            if size_in_bytes <= 8 {
                self.bytecode.push(Op {
                    owning_span: instr_val.get_span(self.context),
                    opcode: Either::Left(VirtualOp::RET(ret_reg)),
                    comment: "".into(),
                });
            } else {
                // If the type is larger than one word, then we use RETD to return data.  First put
                // the size into the data section, then add a LW to get it, then add a RETD which
                // uses it.
                let size_reg = self.reg_seqr.next();
                let size_data_id = self
                    .data_section
                    .insert_data_value(&Literal::U64(size_in_bytes));

                self.bytecode.push(Op {
                    opcode: Either::Left(VirtualOp::LWDataId(size_reg.clone(), size_data_id)),
                    owning_span: instr_val.get_span(self.context),
                    comment: "loading size for RETD".into(),
                });
                self.bytecode.push(Op {
                    owning_span: instr_val.get_span(self.context),
                    opcode: Either::Left(VirtualOp::RETD(ret_reg, size_reg)),
                    comment: "".into(),
                });
            }
        }
    }

    fn compile_store(&mut self, instr_val: &Value, ptr: &Pointer, stored_val: &Value) {
        let stored_reg = self.value_to_register(stored_val);
        let is_struct_ptr = ptr.is_struct_ptr(self.context);
        match self.ptr_map.get(ptr) {
            None => unreachable!("Bug! Trying to store to an unknown pointer."),
            Some(storage) => match storage {
                Storage::Data(_) => unreachable!("BUG! Trying to store to the data section."),
                Storage::Register(reg) => {
                    self.bytecode.push(Op {
                        opcode: Either::Left(VirtualOp::MOVE(reg.clone(), stored_reg)),
                        comment: String::new(),
                        owning_span: instr_val.get_span(self.context),
                    });
                }
                Storage::Stack(word_offs) => {
                    let word_offs = *word_offs;
                    let store_size_in_words = size_bytes_in_words!(
                        self.ir_type_size_in_bytes(ptr.get_type(self.context))
                    );
                    match store_size_in_words {
                        // We can have empty sized types which we can ignore.
                        0 => (),
                        1 => {
                            let base_reg = self.stack_base_reg.as_ref().unwrap().clone();

                            // A single word can be stored with SW.
                            let stored_reg = if !is_struct_ptr {
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
                                    owning_span: instr_val.get_span(self.context),
                                });
                                tmp_reg
                            };
                            if word_offs > crate::asm_generation::compiler_constants::TWELVE_BITS {
                                let offs_reg = self.reg_seqr.next();
                                self.number_to_reg(
                                    word_offs,
                                    &offs_reg,
                                    instr_val.get_span(self.context),
                                );
                                self.bytecode.push(Op {
                                    opcode: Either::Left(VirtualOp::ADD(
                                        base_reg.clone(),
                                        base_reg,
                                        offs_reg.clone(),
                                    )),
                                    comment: "store absolute offset".into(),
                                    owning_span: instr_val.get_span(self.context),
                                });
                                self.bytecode.push(Op {
                                    opcode: Either::Left(VirtualOp::SW(
                                        offs_reg,
                                        stored_reg,
                                        VirtualImmediate12 { value: 0 },
                                    )),
                                    comment: "store value".into(),
                                    owning_span: instr_val.get_span(self.context),
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
                                    owning_span: instr_val.get_span(self.context),
                                });
                            }
                        }
                        _ => {
                            let base_reg = self.stack_base_reg.as_ref().unwrap().clone();

                            // Bigger than 1 word needs a MCPI.  XXX Or MCP if it's huge.
                            let dest_offs_reg = self.reg_seqr.next();
                            if word_offs * 8
                                > crate::asm_generation::compiler_constants::TWELVE_BITS
                            {
                                self.number_to_reg(
                                    word_offs * 8,
                                    &dest_offs_reg,
                                    instr_val.get_span(self.context),
                                );
                                self.bytecode.push(Op {
                                    opcode: either::Either::Left(VirtualOp::ADD(
                                        dest_offs_reg.clone(),
                                        base_reg,
                                        dest_offs_reg.clone(),
                                    )),
                                    comment: "get store offset".into(),
                                    owning_span: instr_val.get_span(self.context),
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
                                    owning_span: instr_val.get_span(self.context),
                                });
                            }

                            if store_size_in_words * 8
                                > crate::asm_generation::compiler_constants::TWELVE_BITS
                            {
                                let size_reg = self.reg_seqr.next();
                                self.number_to_reg(
                                    store_size_in_words * 8,
                                    &size_reg,
                                    instr_val.get_span(self.context),
                                );
                                self.bytecode.push(Op {
                                    opcode: Either::Left(VirtualOp::MCP(
                                        dest_offs_reg,
                                        stored_reg,
                                        size_reg,
                                    )),
                                    comment: "store value".into(),
                                    owning_span: instr_val.get_span(self.context),
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
                                    owning_span: instr_val.get_span(self.context),
                                });
                            }
                        }
                    }
                }
            },
        };
    }

    fn value_to_register(&mut self, value: &Value) -> VirtualRegister {
        match self.reg_map.get(value) {
            Some(reg) => reg.clone(),
            None => {
                match &self.context.values[value.0].value {
                    // Handle constants.
                    ValueDatum::Constant(constant) => {
                        match &constant.value {
                            ConstantValue::Struct(_) | ConstantValue::Array(_) => {
                                // A constant struct or array.  We still allocate space for it on
                                // the stack, but create the field or element initialisers
                                // recursively.

                                // Get the total size.
                                let total_size = size_bytes_round_up_to_word_alignment!(
                                    self.constant_size_in_bytes(constant)
                                );
                                if total_size
                                    > crate::asm_generation::compiler_constants::TWENTY_FOUR_BITS
                                {
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

                                    let mut alloc_op =
                                        Op::unowned_stack_allocate_memory(VirtualImmediate24 {
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
                                    self.initialise_constant_memory(
                                        constant,
                                        &start_reg,
                                        0,
                                        value.get_span(self.context),
                                    );
                                }

                                // Return the start ptr.
                                start_reg
                            }

                            ConstantValue::Undef
                            | ConstantValue::Unit
                            | ConstantValue::Bool(_)
                            | ConstantValue::Uint(_)
                            | ConstantValue::B256(_)
                            | ConstantValue::String(_) => {
                                // Get the constant into the namespace.
                                let lit = ir_constant_to_ast_literal(constant);
                                let data_id = self.data_section.insert_data_value(&lit);

                                // Allocate a register for it, and a load instruction.
                                let reg = self.reg_seqr.next();
                                self.bytecode.push(Op {
                                    opcode: either::Either::Left(VirtualOp::LWDataId(
                                        reg.clone(),
                                        data_id,
                                    )),
                                    comment: "literal instantiation".into(),
                                    owning_span: value.get_span(self.context),
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
                        }
                    }

                    _otherwise => {
                        // Just make a new register for this value.
                        let reg = self.reg_seqr.next();
                        self.reg_map.insert(*value, reg.clone());
                        reg
                    }
                }
            }
        }
    }

    fn number_to_reg(&mut self, offset: u64, offset_reg: &VirtualRegister, span: Option<Span>) {
        if offset > crate::asm_generation::compiler_constants::TWENTY_FOUR_BITS {
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

    fn constant_size_in_bytes(&mut self, constant: &Constant) -> u64 {
        match &constant.value {
            ConstantValue::Undef => self.ir_type_size_in_bytes(&constant.ty),
            ConstantValue::Unit => 8,
            ConstantValue::Bool(_) => 8,
            ConstantValue::Uint(_) => 8,
            ConstantValue::B256(_) => 32,
            ConstantValue::String(s) => s.len() as u64, // String::len() returns the byte size, not char count.
            ConstantValue::Array(elems) => {
                if elems.is_empty() {
                    0
                } else {
                    self.constant_size_in_bytes(&elems[0]) * elems.len() as u64
                }
            }
            ConstantValue::Struct(fields) => fields
                .iter()
                .fold(0, |acc, field| acc + self.constant_size_in_bytes(field)),
        }
    }

    fn initialise_constant_memory(
        &mut self,
        constant: &Constant,
        start_reg: &VirtualRegister,
        offs_in_words: u64,
        span: Option<Span>,
    ) -> u64 {
        match &constant.value {
            ConstantValue::Undef => {
                // We don't need to actually create an initialiser, but we do need to return the
                // field size in words.
                size_bytes_in_words!(self.ir_type_size_in_bytes(&constant.ty))
            }
            ConstantValue::Unit
            | ConstantValue::Bool(_)
            | ConstantValue::Uint(_)
            | ConstantValue::B256(_) => {
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
                if matches!(lit, Literal::B256(_)) {
                    let offs_reg = self.reg_seqr.next();
                    if offs_in_words * 8 > crate::asm_generation::compiler_constants::TWELVE_BITS {
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
                            VirtualImmediate12 { value: 32 },
                        )),
                        comment: "initialise aggregate field".into(),
                        owning_span: span,
                    });

                    4 // 32 bytes is 4 words.
                } else {
                    if offs_in_words > crate::asm_generation::compiler_constants::TWELVE_BITS {
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
            }

            ConstantValue::String(_) => {
                // These are still not properly implemented until we refactor for spans!  There's
                // an issue on GitHub for it.
                self.bytecode.push(Op {
                    opcode: Either::Left(VirtualOp::NOOP),
                    comment: "strings aren't implemented!".into(),
                    owning_span: span,
                });

                0
            }

            ConstantValue::Array(items) | ConstantValue::Struct(items) => {
                let mut cur_offs = offs_in_words;
                for item in items {
                    let item_size =
                        self.initialise_constant_memory(item, start_reg, cur_offs, span.clone());
                    cur_offs += item_size;
                }
                cur_offs
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

    // Aggregate size in bytes.
    fn aggregate_size(&mut self, aggregate: &Aggregate) -> u64 {
        self.analyze_aggregate(aggregate);
        self.aggregate_layouts.get(aggregate).unwrap().0
    }

    // Size of largest aggregate field in bytes.
    fn aggregate_max_field_size(&mut self, aggregate: &Aggregate) -> u64 {
        self.analyze_aggregate(aggregate);
        self.aggregate_layouts
            .get(aggregate)
            .unwrap()
            .1
            .iter()
            .map(|layout| layout.size_in_bytes)
            .max()
            .unwrap_or(0)
    }

    // Aggregate (nested) field offset in words and size in bytes.
    fn aggregate_idcs_to_field_layout(
        &mut self,
        aggregate: &Aggregate,
        idcs: &[u64],
    ) -> (u64, u64) {
        self.analyze_aggregate(aggregate);

        idcs.iter()
            .fold(
                ((0, 0), Type::Struct(*aggregate)),
                |((offs, _), ty), idx| match ty {
                    Type::Struct(aggregate) => {
                        let agg_content = &self.context.aggregates[aggregate.0];
                        let field_type = agg_content.field_types()[*idx as usize];

                        let field_layout =
                            &self.aggregate_layouts.get(&aggregate).unwrap().1[*idx as usize];

                        (
                            (
                                offs + field_layout.offset_in_words,
                                field_layout.size_in_bytes,
                            ),
                            field_type,
                        )
                    }
                    _otherwise => panic!("Attempt to access field in non-aggregate."),
                },
            )
            .0
    }

    fn analyze_aggregate(&mut self, aggregate: &Aggregate) {
        if self.aggregate_layouts.contains_key(aggregate) {
            return;
        }

        match &self.context.aggregates[aggregate.0] {
            AggregateContent::FieldTypes(field_types) => {
                let (total_in_words, offsets) =
                    field_types
                        .iter()
                        .fold((0, Vec::new()), |(cur_offset, mut layouts), ty| {
                            let field_size_in_bytes = self.ir_type_size_in_bytes(ty);
                            layouts.push(FieldLayout {
                                offset_in_words: cur_offset,
                                size_in_bytes: field_size_in_bytes,
                            });
                            (
                                cur_offset + size_bytes_in_words!(field_size_in_bytes),
                                layouts,
                            )
                        });
                self.aggregate_layouts
                    .insert(*aggregate, (total_in_words * 8, offsets));
            }
            AggregateContent::ArrayType(el_type, count) => {
                // Careful!  We *could* wrap the aggregate in Type::Array and call
                // ir_type_size_in_bytes() BUT we'd then enter a recursive loop.
                let el_size = self.ir_type_size_in_bytes(el_type);
                self.aggregate_layouts
                    .insert(*aggregate, (count * el_size, Vec::new()));
            }
        }
    }

    fn ir_type_size_in_bytes(&mut self, ty: &Type) -> u64 {
        match ty {
            Type::Unit | Type::Bool | Type::Uint(_) => 8,
            Type::B256 => 32,
            Type::String(n) => *n,
            Type::Array(aggregate) | Type::Struct(aggregate) => {
                self.analyze_aggregate(aggregate);
                self.aggregate_size(aggregate)
            }
            Type::Union(aggregate) => {
                self.analyze_aggregate(aggregate);
                self.aggregate_max_field_size(aggregate)
            }
            Type::ContractCaller(_) => {
                // We only store the address.
                32
            }
            Type::Contract => {
                unimplemented!("do contract/contract caller have/need a size?")
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
        ConstantValue::String(_) => Literal::String(crate::span::Span {
            span: pest::Span::new(
                "STRINGS ARE UNIMPLEMENTED UNTIL WE REDO DATASECTION".into(),
                0,
                51,
            )
            .unwrap(),
            path: None,
        }),
        ConstantValue::Array(_) => unimplemented!(),
        ConstantValue::Struct(_) => unimplemented!(),
    }
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
                    println!("---- IR To ASM: {:?} ----", path);
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
        let asm_result = compile_ir_to_asm(
            &ir,
            &BuildConfig {
                file_name: std::sync::Arc::new("".into()),
                dir_of_code: std::sync::Arc::new("".into()),
                manifest_path: std::sync::Arc::new("".into()),
                use_ir: false,
                print_intermediate_asm: false,
                print_finalized_asm: false,
                print_ir: false,
                generated_names: std::sync::Arc::new(std::sync::Mutex::new(vec![])),
            },
        );

        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let asm = asm_result.unwrap(&mut warnings, &mut errors);
        assert!(warnings.is_empty() && errors.is_empty());

        let asm_script = format!("{}", asm);
        if asm_script != expected {
            println!("{}", prettydiff::diff_lines(&expected, &asm_script));
            panic!();
        }
    }
}

// =================================================================================================
