use crate::{
    asm_generation::{
        from_ir::*,
        fuel::{compiler_constants, data_section::Entry, fuel_asm_builder::FuelAsmBuilder},
        ProgramKind,
    },
    asm_lang::{
        virtual_register::{self, *},
        Op, OrganizationalOp, VirtualImmediate12, VirtualImmediate18, VirtualImmediate24,
        VirtualOp,
    },
    decl_engine::DeclRef,
    fuel_prelude::fuel_asm::GTFArgs,
    size_bytes_in_words, size_bytes_round_up_to_word_alignment,
};

use sway_ir::*;

use either::Either;
use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
};
use sway_types::{Ident, Span};

use super::{compiler_constants::NUM_ARG_REGISTERS, data_section::DataId};

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
///
/// When a function has more than NUM_ARG_REGISTERS, the last arg register
/// is used to point to the stack location of the remaining arguments.
/// Stack space for the extra arguments is allocated in the caller when
/// locals of the caller are allocated.
impl<'ir, 'eng> FuelAsmBuilder<'ir, 'eng> {
    pub(super) fn compile_call(
        &mut self,
        instr_val: &Value,
        function: &Function,
        args: &[Value],
    ) -> Result<(), CompileError> {
        // Put the args into the args registers.
        if args.len() <= compiler_constants::NUM_ARG_REGISTERS as usize {
            for (idx, arg_val) in args.iter().enumerate() {
                let arg_reg = self.value_to_register(arg_val)?;
                self.cur_bytecode.push(Op::register_move(
                    VirtualRegister::Constant(ConstantRegister::ARG_REGS[idx]),
                    arg_reg,
                    format!("pass arg {idx}"),
                    self.md_mgr.val_to_span(self.context, *arg_val),
                ));
            }
        } else {
            // Put NUM_ARG_REGISTERS - 1 arguments into arg registers and rest into the stack.
            for (idx, arg_val) in args.iter().enumerate() {
                let arg_reg = self.value_to_register(arg_val)?;
                // Except for the last arg register, the others hold an argument.
                if idx < compiler_constants::NUM_ARG_REGISTERS as usize - 1 {
                    self.cur_bytecode.push(Op::register_move(
                        VirtualRegister::Constant(ConstantRegister::ARG_REGS[idx]),
                        arg_reg,
                        format!("pass arg {idx}"),
                        self.md_mgr.val_to_span(self.context, *arg_val),
                    ));
                } else {
                    // All arguments [NUM_ARG_REGISTERS - 1 ..] go into the stack.
                    assert!(
                        self.locals_size_bytes() % 8 == 0,
                        "The size of locals is not word aligned"
                    );
                    let stack_offset_bytes = self.locals_size_bytes()
                        + (((idx as u64 + 1) - compiler_constants::NUM_ARG_REGISTERS as u64) * 8);
                    assert!(
                        stack_offset_bytes
                            < self.locals_size_bytes() + (self.max_num_extra_args() * 8)
                    );
                    self.cur_bytecode.push(Op {
                        opcode: Either::Left(VirtualOp::SW(
                            VirtualRegister::Constant(ConstantRegister::LocalsBase),
                            arg_reg,
                            VirtualImmediate12::new(
                                // The VM multiples the offset by 8, so we divide it by 8.
                                stack_offset_bytes / 8,
                                self.md_mgr
                                    .val_to_span(self.context, *arg_val)
                                    .unwrap_or(Span::dummy()),
                            )
                            .expect("Too many arguments, cannot handle."),
                        )),
                        comment: format!("Pass arg {idx} via its stack slot"),
                        owning_span: self.md_mgr.val_to_span(self.context, *arg_val),
                    });
                }
            }
            // Register ARG_REGS[NUM_ARG_REGISTERS-1] must contain LocalsBase + locals_size
            // so that the callee can index the stack arguments from there.
            self.cur_bytecode.push(Op {
                opcode: Either::Left(VirtualOp::ADDI(
                    VirtualRegister::Constant(
                        ConstantRegister::ARG_REGS
                            [(compiler_constants::NUM_ARG_REGISTERS - 1) as usize],
                    ),
                    VirtualRegister::Constant(ConstantRegister::LocalsBase),
                    VirtualImmediate12::new(self.locals_size_bytes(), Span::dummy())
                        .expect("Too many arguments, cannot handle."),
                )),
                comment: "Save address of stack arguments in last arg register".to_string(),
                owning_span: self.md_mgr.val_to_span(self.context, *instr_val),
            });
        }

        // Set a new return address.
        let ret_label = self.reg_seqr.get_label();
        self.cur_bytecode.push(Op::save_ret_addr(
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

        Ok(())
    }

    pub(super) fn compile_ret_from_call(
        &mut self,
        instr_val: &Value,
        ret_val: &Value,
    ) -> Result<(), CompileError> {
        // Move the result into the return value register.
        let owning_span = self.md_mgr.val_to_span(self.context, *instr_val);
        let ret_reg = self.value_to_register(ret_val)?;
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

        Ok(())
    }

    pub fn compile_function(
        &mut self,
        handler: &Handler,
        function: Function,
    ) -> Result<(), ErrorEmitted> {
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
        let test_decl_ref = match (&span, &test_decl_index) {
            (Some(span), Some(decl_index)) => Some(DeclRef::new(
                Ident::new(span.clone()),
                *decl_index,
                span.clone(),
            )),
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

        let locals_alloc_result = self.alloc_locals(function);

        if func_is_entry {
            self.compile_external_args(function)
                .map_err(|e| handler.emit_err(e))?
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

        self.init_locals(locals_alloc_result);

        // Compile instructions. Traverse the IR blocks in reverse post order. This guarantees that
        // each block is processed after all its CFG predecessors have been processed.
        let po = sway_ir::dominator::compute_post_order(self.context, &function);
        for block in po.po_to_block.iter().rev() {
            let label = self.block_to_label(block);
            self.cur_bytecode.push(Op::unowned_jump_label(label));
            self.compile_block(handler, block, func_is_entry)?;
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
                .push((function, start_label, ops, test_decl_ref));
        } else {
            self.non_entries.push(ops);
        }

        Ok(())
    }

    fn compile_fn_call_args(&mut self, function: Function) {
        if function.num_args(self.context) <= compiler_constants::NUM_ARG_REGISTERS as usize {
            // All arguments are passed through registers.
            for (idx, (_, arg_val)) in function.args_iter(self.context).enumerate() {
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
            }
        } else {
            // Get NUM_ARG_REGISTERS - 1 arguments from arg registers and rest from the stack.
            for (idx, (_, arg_val)) in function.args_iter(self.context).enumerate() {
                let arg_copy_reg = self.reg_seqr.next();
                // Except for the last arg register, the others hold an argument.
                if idx < compiler_constants::NUM_ARG_REGISTERS as usize - 1 {
                    // Make a copy of the args in case we make calls and need to use the arg registers.
                    self.cur_bytecode.push(Op::register_move(
                        arg_copy_reg.clone(),
                        VirtualRegister::Constant(ConstantRegister::ARG_REGS[idx]),
                        format!("save arg {idx}"),
                        self.md_mgr.val_to_span(self.context, *arg_val),
                    ));
                } else {
                    // All arguments [NUM_ARG_REGISTERS - 1 ..] go into the stack.
                    assert!(
                        self.locals_size_bytes() % 8 == 0,
                        "The size of locals is not word aligned"
                    );
                    let stack_offset =
                        (idx as u64 + 1) - compiler_constants::NUM_ARG_REGISTERS as u64;
                    self.cur_bytecode.push(Op {
                        opcode: Either::Left(VirtualOp::LW(
                            arg_copy_reg.clone(),
                            VirtualRegister::Constant(
                                ConstantRegister::ARG_REGS
                                    [compiler_constants::NUM_ARG_REGISTERS as usize - 1],
                            ),
                            VirtualImmediate12::new(
                                stack_offset,
                                self.md_mgr
                                    .val_to_span(self.context, *arg_val)
                                    .unwrap_or(Span::dummy()),
                            )
                            .expect("Too many arguments, cannot handle."),
                        )),
                        comment: format!("Load arg {idx} from its stack slot"),
                        owning_span: self.md_mgr.val_to_span(self.context, *arg_val),
                    });
                }
                // Remember our arg copy.
                self.reg_map.insert(*arg_val, arg_copy_reg);
            }
        }
    }

    // Handle loading the arguments of a contract call
    fn compile_external_args(&mut self, function: Function) -> Result<(), CompileError> {
        match function.args_iter(self.context).count() {
            // Nothing to do if there are no arguments
            0 => Ok(()),

            // A special case for when there's only a single arg, its value (or address) is placed
            // directly in the base register.
            1 => {
                let (_, val) = function.args_iter(self.context).next().unwrap();
                let single_arg_reg = self.reg_seqr.next();
                match self.program_kind {
                    ProgramKind::Contract => {
                        self.read_args_base_from_frame(&single_arg_reg);
                    }
                    ProgramKind::Library => {} // Nothing to do here
                    ProgramKind::Script | ProgramKind::Predicate => {
                        if let ProgramKind::Predicate = self.program_kind {
                            self.read_args_base_from_predicate_data(&single_arg_reg);
                        } else {
                            self.read_args_base_from_script_data(&single_arg_reg);
                        }

                        // The base is an offset.  Dereference it.
                        // XXX val.get_type() should be a pointer if it's not meant to be loaded.
                        if val
                            .get_type(self.context)
                            .map_or(false, |t| self.is_copy_type(&t))
                        {
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
                self.reg_map.insert(*val, single_arg_reg);
                Ok(())
            }

            // Otherwise, the args are bundled together and pointed to by the base register.
            _ => {
                let args_base_reg = self.reg_seqr.next();
                match self.program_kind {
                    ProgramKind::Contract => self.read_args_base_from_frame(&args_base_reg),
                    ProgramKind::Library => return Ok(()), // Nothing to do here
                    ProgramKind::Predicate => {
                        self.read_args_base_from_predicate_data(&args_base_reg)
                    }
                    ProgramKind::Script => self.read_args_base_from_script_data(&args_base_reg),
                }

                // Successively load each argument. The asm generated depends on the arg type size
                // and whether the offset fits in a 12-bit immediate.
                let mut arg_word_offset = 0;
                for (name, val) in function.args_iter(self.context) {
                    let current_arg_reg = self.reg_seqr.next();

                    // The function arg type might be a pointer, but the value in the struct will
                    // be of the pointed to type.  So strip the pointer if necessary.
                    let arg_type = val
                        .get_type(self.context)
                        .map(|ty| ty.get_pointee_type(self.context).unwrap_or(ty))
                        .unwrap();
                    let arg_type_size_bytes = ir_type_size_in_bytes(self.context, &arg_type);
                    if self.is_copy_type(&arg_type) {
                        if arg_word_offset > compiler_constants::TWELVE_BITS {
                            let offs_reg = self.reg_seqr.next();
                            self.cur_bytecode.push(Op {
                                opcode: Either::Left(VirtualOp::ADD(
                                    args_base_reg.clone(),
                                    args_base_reg.clone(),
                                    offs_reg.clone(),
                                )),
                                comment: format!("get offset for arg {name}"),
                                owning_span: None,
                            });

                            if arg_type.size_in_bytes(self.context) == 1 {
                                self.cur_bytecode.push(Op {
                                    opcode: Either::Left(VirtualOp::LB(
                                        current_arg_reg.clone(),
                                        offs_reg,
                                        VirtualImmediate12 { value: 0 },
                                    )),
                                    comment: format!("get arg {name}"),
                                    owning_span: None,
                                });
                            } else {
                                self.cur_bytecode.push(Op {
                                    opcode: Either::Left(VirtualOp::LW(
                                        current_arg_reg.clone(),
                                        offs_reg,
                                        VirtualImmediate12 { value: 0 },
                                    )),
                                    comment: format!("get arg {name}"),
                                    owning_span: None,
                                });
                            }
                        } else if arg_type.size_in_bytes(self.context) == 1 {
                            self.cur_bytecode.push(Op {
                                opcode: Either::Left(VirtualOp::LB(
                                    current_arg_reg.clone(),
                                    args_base_reg.clone(),
                                    VirtualImmediate12 {
                                        value: arg_word_offset as u16 * 8,
                                    },
                                )),
                                comment: format!("get arg {name}"),
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
                                comment: format!("get arg {name}"),
                                owning_span: None,
                            });
                        }
                    } else {
                        self.immediate_to_reg(
                            arg_word_offset * 8,
                            current_arg_reg.clone(),
                            Some(&args_base_reg),
                            format!("get offset or arg {name}"),
                            None,
                        );
                    }

                    arg_word_offset += size_bytes_in_words!(arg_type_size_bytes);
                    self.reg_map.insert(*val, current_arg_reg);
                }

                Ok(())
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

        // Use the `gm` instruction to get the index of the predicate. This is the index we're
        // going to use in the subsequent `gtf` instructions.
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

        // If the input is indeed a "coin", then use `GTF` to get the "input coin predicate data
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

        // Otherwise, insert the label to jump to if the input type is not a "coin".
        self.cur_bytecode
            .push(Op::unowned_jump_label(input_type_not_coin_label));

        // Check if the input type is "message" by comparing the input type to a register
        // containing 2.
        let input_type_is_message = self.reg_seqr.next();
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
                input_type_is_message.clone(),
                input_type,
                two,
            )),
            comment: "input type is message(2)".into(),
            owning_span: None,
        });

        // Invert `input_type_is_message` to use in `jnzi`
        let input_type_not_message = self.reg_seqr.next();
        self.cur_bytecode.push(Op {
            opcode: Either::Left(VirtualOp::XORI(
                input_type_not_message.clone(),
                input_type_is_message,
                VirtualImmediate12 { value: 1 },
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

    fn alloc_locals(
        &mut self,
        function: Function,
    ) -> (
        u64,
        virtual_register::VirtualRegister,
        Vec<(u64, u64, u64, DataId)>,
        u64,
    ) {
        // Scan the function to see if there are any calls to functions with more than
        // NUM_ARG_REGISTERS. The extra args will need stack allocation too.
        let mut max_num_extra_args = 0u64;
        for (_block, inst) in function.instruction_iter(self.context) {
            if let Some(Instruction {
                op: InstOp::Call(_, args),
                ..
            }) = inst.get_instruction(self.context)
            {
                if args.len() > NUM_ARG_REGISTERS as usize {
                    // When we have more than NUM_ARG_REGISTERS, the last arg register
                    // is used to point to the stack location of extra args. So we'll
                    // only have NUM_ARG_REGISTERS - 1 arguments passed in registers.
                    max_num_extra_args = std::cmp::max(
                        max_num_extra_args,
                        args.len() as u64 - NUM_ARG_REGISTERS as u64 + 1,
                    );
                }
                // All arguments must fit in the register (thanks to the demotion passes).
                assert!(args.iter().all(|arg| ir_type_size_in_bytes(
                    self.context,
                    &arg.get_type(self.context).unwrap()
                ) <= 8));
            }
        }

        // If they're immutable and have a constant initialiser then they go in the data section.
        //
        // Otherwise they go in runtime allocated space, either a register or on the stack.
        //
        // Stack offsets are in words to both enforce alignment and simplify use with LW/SW.
        let (stack_base_words, init_mut_vars) = function.locals_iter(self.context).fold(
            (0, Vec::new()),
            |(stack_base_words, mut init_mut_vars), (_name, ptr)| {
                if let (false, Some(constant)) = (
                    ptr.is_mutable(self.context),
                    ptr.get_initializer(self.context),
                ) {
                    let data_id = self.data_section.insert_data_value(Entry::from_constant(
                        self.context,
                        constant,
                        None,
                        None,
                    ));
                    self.ptr_map.insert(*ptr, Storage::Data(data_id));
                    (stack_base_words, init_mut_vars)
                } else {
                    self.ptr_map.insert(*ptr, Storage::Stack(stack_base_words));

                    let ptr_ty = ptr.get_inner_type(self.context);
                    let var_byte_size = ir_type_size_in_bytes(self.context, &ptr_ty);
                    let var_word_size = match ptr_ty.get_content(self.context) {
                        TypeContent::Uint(256) => 4,
                        TypeContent::Unit
                        | TypeContent::Bool
                        | TypeContent::Uint(_)
                        | TypeContent::Pointer(_) => 1,
                        TypeContent::Slice => 2,
                        TypeContent::B256 => 4,
                        TypeContent::StringSlice => 2,
                        TypeContent::StringArray(n) => size_bytes_round_up_to_word_alignment!(n),
                        TypeContent::Array(..) | TypeContent::Struct(_) | TypeContent::Union(_) => {
                            size_bytes_in_words!(ir_type_size_in_bytes(self.context, &ptr_ty))
                        }
                    };

                    if let Some(constant) = ptr.get_initializer(self.context) {
                        let data_id = self.data_section.insert_data_value(Entry::from_constant(
                            self.context,
                            constant,
                            None,
                            None,
                        ));

                        init_mut_vars.push((stack_base_words, var_word_size, var_byte_size, data_id));
                    }

                    (stack_base_words + var_word_size, init_mut_vars)
                }
            },
        );

        // Reserve space on the stack (in bytes) for all our locals which require it.  Firstly save
        // the current $sp.
        let locals_base_reg = VirtualRegister::Constant(ConstantRegister::LocalsBase);
        self.cur_bytecode.push(Op::register_move(
            locals_base_reg.clone(),
            VirtualRegister::Constant(ConstantRegister::StackPointer),
            "save locals base register",
            None,
        ));

        let locals_size_bytes = stack_base_words * 8;
        if locals_size_bytes > compiler_constants::TWENTY_FOUR_BITS {
            todo!("Enormous stack usage for locals.");
        }
        self.cur_bytecode.push(Op {
            opcode: Either::Left(VirtualOp::CFEI(VirtualImmediate24 {
                value: locals_size_bytes as u32 + (max_num_extra_args * 8) as u32,
            })),
            comment: format!("allocate {locals_size_bytes} bytes for locals and {max_num_extra_args} slots for call arguments."),
            owning_span: None,
        });
        (
            locals_size_bytes,
            locals_base_reg,
            init_mut_vars,
            max_num_extra_args,
        )
    }

    fn init_locals(
        &mut self,
        (locals_size_bytes, locals_base_reg, init_mut_vars, max_num_extra_args): (
            u64,
            virtual_register::VirtualRegister,
            Vec<(u64, u64, u64, DataId)>,
            u64,
        ),
    ) {
        // Initialise that stack variables which requires it.
        for (
            var_stack_offs,
            var_word_size,
            var_byte_size,
            var_data_id,
        ) in init_mut_vars
        {
            // Load our initialiser from the data section.
            self.cur_bytecode.push(Op {
                opcode: Either::Left(VirtualOp::LoadDataId(
                    VirtualRegister::Constant(ConstantRegister::Scratch),
                    var_data_id,
                )),
                comment: "load initializer from data section".to_owned(),
                owning_span: None,
            });

            // Get the stack offset in bytes rather than words.
            let var_stack_off_bytes = var_stack_offs * 8;
            let dst_reg = self.reg_seqr.next();
            // Check if we can use the `ADDi` opcode.
            if var_stack_off_bytes <= compiler_constants::TWELVE_BITS {
                // Get the destination on the stack.
                self.cur_bytecode.push(Op {
                    opcode: Either::Left(VirtualOp::ADDI(
                        dst_reg.clone(),
                        locals_base_reg.clone(),
                        VirtualImmediate12 {
                            value: var_stack_off_bytes as u16,
                        },
                    )),
                    comment: "calc local variable address".to_owned(),
                    owning_span: None,
                });
            } else {
                assert!(var_stack_off_bytes <= compiler_constants::EIGHTEEN_BITS);
                // We can't, so load the immediate into a register and then add.
                self.cur_bytecode.push(Op {
                    opcode: Either::Left(VirtualOp::MOVI(
                        dst_reg.clone(),
                        VirtualImmediate18 {
                            value: var_stack_off_bytes as u32,
                        },
                    )),
                    comment: "stack offset of local variable into register".to_owned(),
                    owning_span: None,
                });
                self.cur_bytecode.push(Op {
                    opcode: Either::Left(VirtualOp::ADD(
                        dst_reg.clone(),
                        locals_base_reg.clone(),
                        dst_reg.clone(),
                    )),
                    comment: "calc local variable address".to_owned(),
                    owning_span: None,
                });
            }

            if var_word_size == 1 {
                // Initialise by value.
                if var_byte_size == 1 {
                    self.cur_bytecode.push(Op {
                        opcode: Either::Left(VirtualOp::SB(
                            dst_reg,
                            VirtualRegister::Constant(ConstantRegister::Scratch),
                            VirtualImmediate12 { value: 0 },
                        )),
                        comment: "store initializer to local variable".to_owned(),
                        owning_span: None,
                    });
                } else {
                    self.cur_bytecode.push(Op {
                        opcode: Either::Left(VirtualOp::SW(
                            dst_reg,
                            VirtualRegister::Constant(ConstantRegister::Scratch),
                            VirtualImmediate12 { value: 0 },
                        )),
                        comment: "store initializer to local variable".to_owned(),
                        owning_span: None,
                    });
                }
            } else {
                // Initialise by reference.
                let var_byte_size = var_word_size * 8;
                assert!(var_byte_size <= compiler_constants::TWELVE_BITS);
                self.cur_bytecode.push(Op {
                    opcode: Either::Left(VirtualOp::MCPI(
                        dst_reg,
                        VirtualRegister::Constant(ConstantRegister::Scratch),
                        VirtualImmediate12 {
                            value: var_byte_size as u16,
                        },
                    )),
                    comment: "copy initializer from data section to local variable".to_owned(),
                    owning_span: None,
                });
            }
        }

        self.locals_ctxs
            .push((locals_size_bytes, locals_base_reg, max_num_extra_args));
    }

    fn drop_locals(&mut self, _function: Function) {
        let (locals_size_bytes, max_num_extra_args) =
            (self.locals_size_bytes(), self.max_num_extra_args());
        if locals_size_bytes > compiler_constants::TWENTY_FOUR_BITS {
            todo!("Enormous stack usage for locals.");
        }
        self.cur_bytecode.push(Op {
            opcode: Either::Left(VirtualOp::CFSI(VirtualImmediate24 {
                value: u32::try_from(locals_size_bytes + (max_num_extra_args * 8)).unwrap(),
            })),
            comment: format!("free {locals_size_bytes} bytes for locals and {max_num_extra_args} slots for extra call arguments."),
            owning_span: None,
        });
    }

    pub(super) fn locals_base_reg(&self) -> &VirtualRegister {
        &self.locals_ctxs.last().expect("No locals").1
    }

    pub(super) fn locals_size_bytes(&self) -> u64 {
        self.locals_ctxs.last().expect("No locals").0
    }

    pub(super) fn max_num_extra_args(&self) -> u64 {
        self.locals_ctxs.last().expect("No locals").2
    }
}

struct InitMutVars {
    stack_base: u64,
    var_word_size: u64,
    var_byte_size: u64,
    data_id: DataId,
}
