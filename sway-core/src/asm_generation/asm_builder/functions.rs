use super::{
    compiler_constants, ir_type_size_in_bytes, size_bytes_in_words,
    size_bytes_round_up_to_word_alignment, AsmBuilder, ProgramKind,
};

use crate::{
    asm_generation::{from_ir::*, Entry},
    asm_lang::{
        virtual_register::*, Op, OrganizationalOp, VirtualImmediate12, VirtualImmediate18,
        VirtualImmediate24, VirtualOp,
    },
    declaration_engine::DeclarationId,
    error::*,
    fuel_prelude::fuel_asm::GTFArgs,
};

use sway_ir::*;

use either::Either;

/// A summary of the adopted calling convention:
///
/// - Function arguments are passed left to right in the reserved registers.  Extra args are passed
///   on the stack.
/// - The return value is returned in $retv.
/// - The return address is passed in $reta.
/// - All other general purpose registers must be preserved.
///
/// If the return value has a copy-type it can be returned in $retv directly.  If the return
/// value is a ref-type its space must be allocated by the caller and its address passed into
/// (and out of) the callee using $retv.
///
/// The general process for a call is therefore the following.  Not all steps are necessary,
/// depending on how many args and local variables the callee has, and whether the callee makes
/// its own calls.
///
/// - Caller:
///   - Place function args into $rarg0 - $rargN and if necessary the stack.
///   - Allocate the return value on the stack if it's a reference type.
///   - Place the return address into $reta
///   - Jump to function address.
///   - If necessary restore the stack to free args.
/// - Callee:
///   - Save general purpose registers to the stack.
///   - Save the args registers, return value pointer and return address.
///   - Save room on the stack for locals.
///   - (Do work.)
///   - Put the result in return value.
///   - Restore the stack to free locals.
///   - Restore the return address.
///   - Restore the general purpose registers from the stack.
///   - Jump to the return address.

impl<'ir> AsmBuilder<'ir> {
    pub(super) fn compile_call(&mut self, instr_val: &Value, function: &Function, args: &[Value]) {
        // Put the args into the args registers.
        for (idx, arg_val) in args.iter().enumerate() {
            if idx < compiler_constants::NUM_ARG_REGISTERS as usize {
                let arg_reg = self.value_to_register(arg_val);
                self.cur_bytecode.push(Op::register_move(
                    VirtualRegister::Constant(ConstantRegister::ARG_REGS[idx]),
                    arg_reg,
                    format!("pass arg {idx}"),
                    self.md_mgr.val_to_span(self.context, *arg_val),
                ));
            } else {
                todo!(
                    "can't do more than {} args yet",
                    compiler_constants::NUM_ARG_REGISTERS
                );
            }
        }

        // Set a new return address.
        let ret_label = self.reg_seqr.get_label();
        self.cur_bytecode.push(Op::move_address(
            VirtualRegister::Constant(ConstantRegister::CallReturnAddress),
            ret_label,
            "set new return addr",
            None,
        ));

        // Jump to function and insert return label.
        let (fn_label, _) = self.func_to_labels(function);
        self.cur_bytecode.push(Op {
            opcode: Either::Right(OrganizationalOp::Call(fn_label)),
            comment: format!("call {}", function.get_name(self.context)),
            owning_span: None,
        });
        self.cur_bytecode.push(Op::unowned_jump_label(ret_label));

        // Save the return value.
        let ret_reg = self.reg_seqr.next();
        self.cur_bytecode.push(Op {
            opcode: Either::Left(VirtualOp::MOVE(
                ret_reg.clone(),
                VirtualRegister::Constant(ConstantRegister::CallReturnValue),
            )),
            comment: "copy the return value".into(),
            owning_span: None,
        });
        self.reg_map.insert(*instr_val, ret_reg);
    }

    pub(super) fn compile_ret_from_call(&mut self, instr_val: &Value, ret_val: &Value) {
        // Move the result into the return value register.
        let owning_span = self.md_mgr.val_to_span(self.context, *instr_val);
        let ret_reg = self.value_to_register(ret_val);
        self.cur_bytecode.push(Op::register_move(
            VirtualRegister::Constant(ConstantRegister::CallReturnValue),
            ret_reg,
            "set return value",
            owning_span,
        ));

        // Jump to the end of the function.
        let end_label = self
            .return_ctxs
            .last()
            .expect("Calls guaranteed to save return context.")
            .0;
        self.cur_bytecode.push(Op::jump_to_label(end_label));
    }

    pub(crate) fn compile_function(&mut self, function: Function) -> CompileResult<()> {
        assert!(
            self.cur_bytecode.is_empty(),
            "can't do nested functions yet"
        );

        if function.has_selector(self.context) {
            // Add a comment noting that this is a named contract method.
            self.cur_bytecode.push(Op::new_comment(format!(
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

        let func_is_entry = function.is_entry(self.context);

        // Insert a function label.
        let (start_label, end_label) = self.func_to_labels(&function);
        let md = function.get_metadata(self.context);
        let span = self.md_mgr.md_to_span(self.context, md);
        let test_decl_index = self.md_mgr.md_to_test_decl_index(self.context, md);
        let test_decl_id = match (&span, &test_decl_index) {
            (Some(span), Some(decl_index)) => Some(DeclarationId::new(*decl_index, span.clone())),
            _ => None,
        };
        let comment = format!(
            "--- start of function: {} ---",
            function.get_name(self.context)
        );
        self.cur_bytecode.push(match span {
            Some(span) => Op::jump_label_comment(start_label, span, comment),
            None => Op::unowned_jump_label_comment(start_label, comment),
        });

        // Manage the call frame.
        if !func_is_entry {
            // Save any general purpose registers used here on the stack.
            self.cur_bytecode.push(Op {
                opcode: Either::Right(OrganizationalOp::PushAll(start_label)),
                comment: "save all regs".to_owned(),
                owning_span: None,
            });
        }

        if func_is_entry {
            self.compile_external_args(function)
        } else {
            // Make copies of the arg registers.
            self.compile_fn_call_args(function)
        }

        let reta = self.reg_seqr.next(); // XXX only do this if this function makes calls
        if !func_is_entry {
            // Save $reta and $retv
            self.cur_bytecode.push(Op::register_move(
                reta.clone(),
                VirtualRegister::Constant(ConstantRegister::CallReturnAddress),
                "save reta",
                None,
            ));
            let retv = self.reg_seqr.next();
            self.cur_bytecode.push(Op::register_move(
                retv.clone(),
                VirtualRegister::Constant(ConstantRegister::CallReturnValue),
                "save retv",
                None,
            ));

            // Store some info describing the call frame.
            self.return_ctxs.push((end_label, retv));
        }

        self.init_locals(function);

        // Compile instructions.
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        for block in function.block_iter(self.context) {
            self.insert_block_label(block);
            for instr_val in block.instruction_iter(self.context) {
                check!(
                    self.compile_instruction(&instr_val, func_is_entry),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
            }
        }

        if !func_is_entry {
            // Insert the end of function label.
            self.cur_bytecode.push(Op::unowned_jump_label(end_label));

            // Pop the call frame entry.
            self.return_ctxs.pop();

            // Free our stack allocated locals.  This is unneeded for entries since they will have
            // actually returned to the calling context via a VM RET.
            self.drop_locals(function);

            // Restore $reta.
            self.cur_bytecode.push(Op::register_move(
                VirtualRegister::Constant(ConstantRegister::CallReturnAddress),
                reta,
                "restore reta",
                None,
            ));

            // Restore GP regs.
            self.cur_bytecode.push(Op {
                opcode: Either::Right(OrganizationalOp::PopAll(start_label)),
                comment: "restore all regs".to_owned(),
                owning_span: None,
            });

            // Jump to the return address.
            self.cur_bytecode.push(Op::jump_to_register(
                VirtualRegister::Constant(ConstantRegister::CallReturnAddress),
                "return from call",
                None,
            ));
        }

        // Save this function.
        let mut ops = Vec::new();
        ops.append(&mut self.cur_bytecode);
        if func_is_entry {
            self.entries
                .push((function, start_label, ops, test_decl_id));
        } else {
            self.non_entries.push(ops);
        }

        ok((), warnings, errors)
    }

    fn compile_fn_call_args(&mut self, function: Function) {
        // The first n args are passed in registers, but the rest arrive on the stack.
        for (idx, (_, arg_val)) in function.args_iter(self.context).enumerate() {
            if idx < compiler_constants::NUM_ARG_REGISTERS as usize {
                // Make a copy of the args in case we make calls and need to use the arg registers.
                let arg_copy_reg = self.reg_seqr.next();
                self.cur_bytecode.push(Op::register_move(
                    arg_copy_reg.clone(),
                    VirtualRegister::Constant(ConstantRegister::ARG_REGS[idx]),
                    format!("save arg {idx}"),
                    self.md_mgr.val_to_span(self.context, *arg_val),
                ));

                // Remember our arg copy.
                self.reg_map.insert(*arg_val, arg_copy_reg);
            } else {
                todo!(
                    "can't do more than {} args yet",
                    compiler_constants::NUM_ARG_REGISTERS
                );
            }
        }
    }

    // Handle loading the arguments of a contract call
    fn compile_external_args(&mut self, function: Function) {
        match function.args_iter(self.context).count() {
            // Nothing to do if there are no arguments
            0 => (),

            // A special case for when there's only a single arg, its value (or address) is placed
            // directly in the base register.
            1 => {
                let (_, val) = function.args_iter(self.context).next().unwrap();
                let single_arg_reg = self.value_to_register(val);
                match self.program_kind {
                    ProgramKind::Contract => self.read_args_base_from_frame(&single_arg_reg),
                    ProgramKind::Library => (), // Nothing to do here
                    ProgramKind::Script | ProgramKind::Predicate => {
                        if let ProgramKind::Predicate = self.program_kind {
                            self.read_args_base_from_predicate_data(&single_arg_reg);
                        } else {
                            self.read_args_base_from_script_data(&single_arg_reg);
                        }

                        // The base is an offset.  Dereference it.
                        if val.get_type(self.context).unwrap().is_copy_type() {
                            self.cur_bytecode.push(Op {
                                opcode: either::Either::Left(VirtualOp::LW(
                                    single_arg_reg.clone(),
                                    single_arg_reg.clone(),
                                    VirtualImmediate12 { value: 0 },
                                )),
                                comment: "load main fn parameter".into(),
                                owning_span: None,
                            });
                        }
                    }
                }
            }

            // Otherwise, the args are bundled together and pointed to by the base register.
            _ => {
                let args_base_reg = self.reg_seqr.next();
                match self.program_kind {
                    ProgramKind::Contract => self.read_args_base_from_frame(&args_base_reg),
                    ProgramKind::Library => return, // Nothing to do here
                    ProgramKind::Predicate => {
                        self.read_args_base_from_predicate_data(&args_base_reg)
                    }
                    ProgramKind::Script => self.read_args_base_from_script_data(&args_base_reg),
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
                            self.cur_bytecode.push(Op {
                                opcode: Either::Left(VirtualOp::ADD(
                                    args_base_reg.clone(),
                                    args_base_reg.clone(),
                                    offs_reg.clone(),
                                )),
                                comment: format!("get offset for arg {}", name),
                                owning_span: None,
                            });
                            self.cur_bytecode.push(Op {
                                opcode: Either::Left(VirtualOp::LW(
                                    current_arg_reg.clone(),
                                    offs_reg,
                                    VirtualImmediate12 { value: 0 },
                                )),
                                comment: format!("get arg {}", name),
                                owning_span: None,
                            });
                        } else {
                            self.cur_bytecode.push(Op {
                                opcode: Either::Left(VirtualOp::LW(
                                    current_arg_reg.clone(),
                                    args_base_reg.clone(),
                                    VirtualImmediate12 {
                                        value: arg_word_offset as u16,
                                    },
                                )),
                                comment: format!("get arg {}", name),
                                owning_span: None,
                            });
                        }
                    } else if arg_word_offset * 8 > compiler_constants::TWELVE_BITS {
                        let offs_reg = self.reg_seqr.next();
                        self.number_to_reg(arg_word_offset * 8, &offs_reg, None);
                        self.cur_bytecode.push(Op {
                            opcode: either::Either::Left(VirtualOp::ADD(
                                current_arg_reg.clone(),
                                args_base_reg.clone(),
                                offs_reg,
                            )),
                            comment: format!("get offset or arg {}", name),
                            owning_span: None,
                        });
                    } else {
                        self.cur_bytecode.push(Op {
                            opcode: either::Either::Left(VirtualOp::ADDI(
                                current_arg_reg.clone(),
                                args_base_reg.clone(),
                                VirtualImmediate12 {
                                    value: (arg_word_offset * 8) as u16,
                                },
                            )),
                            comment: format!("get address for arg {}", name),
                            owning_span: None,
                        });
                    }

                    arg_word_offset += size_bytes_in_words!(arg_type_size_bytes);
                }
            }
        }
    }

    // Read the argument(s) base from the call frame.
    fn read_args_base_from_frame(&mut self, reg: &VirtualRegister) {
        self.cur_bytecode.push(Op {
            opcode: Either::Left(VirtualOp::LW(
                reg.clone(),
                VirtualRegister::Constant(ConstantRegister::FramePointer),
                // see https://github.com/FuelLabs/fuel-specs/pull/193#issuecomment-876496372
                VirtualImmediate12 { value: 74 },
            )),
            comment: "base register for method parameter".into(),
            owning_span: None,
        });
    }

    // Read the argument(s) base from the script data.
    fn read_args_base_from_script_data(&mut self, reg: &VirtualRegister) {
        self.cur_bytecode.push(Op {
            opcode: either::Either::Left(VirtualOp::GTF(
                reg.clone(),
                VirtualRegister::Constant(ConstantRegister::Zero),
                VirtualImmediate12 {
                    value: GTFArgs::ScriptData as u16,
                },
            )),
            comment: "base register for main fn parameter".into(),
            owning_span: None,
        });
    }

    /// Read the returns the base pointer for predicate data
    fn read_args_base_from_predicate_data(&mut self, base_reg: &VirtualRegister) {
        // Final label to jump to to continue execution, once the predicate data pointer is
        // successfully found
        let success_label = self.reg_seqr.get_label();

        let input_index = self.reg_seqr.next();
        self.cur_bytecode.push(Op {
            opcode: either::Either::Left(VirtualOp::GM(
                input_index.clone(),
                VirtualImmediate18 { value: 3_u32 },
            )),
            comment: "get predicate index".into(),
            owning_span: None,
        });

        // Find the type of the "Input" using `GTF`. The returned value is one of three possible
        // ones:
        // 0 -> Input Coin = 0,
        // 1 -> Input Contract,
        // 2 -> Input Message
        // We only care about input coins and input message.
        let input_type = self.reg_seqr.next();
        self.cur_bytecode.push(Op {
            opcode: either::Either::Left(VirtualOp::GTF(
                input_type.clone(),
                input_index.clone(),
                VirtualImmediate12 {
                    value: GTFArgs::InputType as u16,
                },
            )),
            comment: "get input type".into(),
            owning_span: None,
        });

        // Label to jump to if the input type is *not* zero, i.e. not "coin". Then do the jump.
        let input_type_not_coin_label = self.reg_seqr.get_label();
        self.cur_bytecode.push(Op::jump_if_not_zero(
            input_type.clone(),
            input_type_not_coin_label,
        ));

        // If the input is indeed a "message", then use `GTF` to get the "input coin predicate data
        // pointer" and store in the `base_reg`
        self.cur_bytecode.push(Op {
            opcode: either::Either::Left(VirtualOp::GTF(
                base_reg.clone(),
                input_index.clone(),
                VirtualImmediate12 {
                    value: GTFArgs::InputCoinPredicateData as u16,
                },
            )),
            comment: "get input coin predicate data pointer".into(),
            owning_span: None,
        });

        // Now that we have the actual pointer, we can jump to the success label to continue
        // execution.
        self.cur_bytecode.push(Op::jump_to_label(success_label));

        // Otherwise, insert the label to jump to if the input type is not "coin".
        self.cur_bytecode
            .push(Op::unowned_jump_label(input_type_not_coin_label));

        // Check if the input type is "message" by comparing the input type to a register
        // containing 2.
        let input_type_not_message = self.reg_seqr.next();
        let two = self.reg_seqr.next();
        self.cur_bytecode.push(Op {
            opcode: Either::Left(VirtualOp::MOVI(
                two.clone(),
                VirtualImmediate18 { value: 2u32 },
            )),
            comment: "register containing 2".into(),
            owning_span: None,
        });
        self.cur_bytecode.push(Op {
            opcode: either::Either::Left(VirtualOp::EQ(
                input_type_not_message.clone(),
                input_type,
                two,
            )),
            comment: "input type is not message(2)".into(),
            owning_span: None,
        });

        // Label to jump to if the input type is *not* 2, i.e. not "message" (and not "coin" since
        // we checked that earlier). Then do the jump.
        let input_type_not_message_label = self.reg_seqr.get_label();
        self.cur_bytecode.push(Op::jump_if_not_zero(
            input_type_not_message,
            input_type_not_message_label,
        ));

        // If the input is indeed a "message", then use `GTF` to get the "input message predicate
        // data pointer" and store it in `base_reg`
        self.cur_bytecode.push(Op {
            opcode: either::Either::Left(VirtualOp::GTF(
                base_reg.clone(),
                input_index,
                VirtualImmediate12 {
                    value: GTFArgs::InputMessagePredicateData as u16,
                },
            )),
            comment: "input message predicate data pointer".into(),
            owning_span: None,
        });
        self.cur_bytecode.push(Op::jump_to_label(success_label));

        // Otherwise, insert the label to jump to if the input type is not "message".
        self.cur_bytecode
            .push(Op::unowned_jump_label(input_type_not_message_label));

        // If we got here, then the input type is neither a coin nor a message. In this case, the
        // predicate should just fail to verify and should return `false`.
        self.cur_bytecode.push(Op {
            opcode: Either::Left(VirtualOp::RET(VirtualRegister::Constant(
                ConstantRegister::Zero,
            ))),
            owning_span: None,
            comment: "return false".into(),
        });

        // Final success label to continue execution at if we successfully obtained the predicate
        // data pointer
        self.cur_bytecode
            .push(Op::unowned_jump_label(success_label));
    }

    fn init_locals(&mut self, function: Function) {
        // If they're immutable and have a constant initialiser then they go in the data section.
        // Otherwise they go in runtime allocated space, either a register or on the stack.
        //
        // Stack offsets are in words to both enforce alignment and simplify use with LW/SW.
        let mut stack_base = 0_u64;
        for (_name, ptr) in function.locals_iter(self.context) {
            if !ptr.is_mutable(self.context) && ptr.get_initializer(self.context).is_some() {
                let constant = ptr.get_initializer(self.context).unwrap();
                let data_id = self
                    .data_section
                    .insert_data_value(Entry::from_constant(self.context, constant));
                self.ptr_map.insert(*ptr, Storage::Data(data_id));
            } else {
                match ptr.get_type(self.context) {
                    Type::Unit | Type::Bool | Type::Uint(_) | Type::Pointer(_) => {
                        self.ptr_map.insert(*ptr, Storage::Stack(stack_base));
                        stack_base += 1;
                    }
                    Type::Slice => {
                        self.ptr_map.insert(*ptr, Storage::Stack(stack_base));
                        stack_base += 2;
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
                    ty @ (Type::Array(_) | Type::Struct(_) | Type::Union(_)) => {
                        // Store this aggregate at the current stack base.
                        self.ptr_map.insert(*ptr, Storage::Stack(stack_base));

                        // Reserve space by incrementing the base.
                        stack_base += size_bytes_in_words!(ir_type_size_in_bytes(self.context, ty));
                    }
                };
            }
        }

        // Reserve space on the stack (in bytes) for all our locals which require it.  Firstly save
        // the current $sp.
        let locals_base_reg = self.reg_seqr.next();
        self.cur_bytecode.push(Op::register_move(
            locals_base_reg.clone(),
            VirtualRegister::Constant(ConstantRegister::StackPointer),
            "save locals base register",
            None,
        ));

        let locals_size = stack_base * 8;
        if locals_size != 0 {
            if locals_size > compiler_constants::TWENTY_FOUR_BITS {
                todo!("Enormous stack usage for locals.");
            }
            self.cur_bytecode.push(Op {
                opcode: Either::Left(VirtualOp::CFEI(VirtualImmediate24 {
                    value: locals_size as u32,
                })),
                comment: format!("allocate {} bytes for locals", locals_size),
                owning_span: None,
            });
        }
        self.locals_ctxs.push((locals_size, locals_base_reg));
    }

    fn drop_locals(&mut self, _function: Function) {
        let (locals_size, _locals_base_reg) = self
            .locals_ctxs
            .pop()
            .expect("Calls guaranteed to save locals context.");
        if locals_size != 0 {
            if locals_size > compiler_constants::TWENTY_FOUR_BITS {
                todo!("Enormous stack usage for locals.");
            }
            self.cur_bytecode.push(Op {
                opcode: Either::Left(VirtualOp::CFSI(VirtualImmediate24 {
                    value: locals_size as u32,
                })),
                comment: format!("free {} bytes for locals", locals_size),
                owning_span: None,
            });
        }
    }

    pub(super) fn locals_base_reg(&self) -> &VirtualRegister {
        &self.locals_ctxs.last().expect("No locals").1
    }
}
