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
use sway_types::Ident;

use super::data_section::DataId;

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

impl<'ir, 'eng> FuelAsmBuilder<'ir, 'eng> {
    pub(super) fn compile_call(
        &mut self,
        instr_val: &Value,
        function: &Function,
        args: &[Value],
    ) -> Result<(), CompileError> {
        // Put the args into the args registers.
        for (idx, arg_val) in args.iter().enumerate() {
            if idx < compiler_constants::NUM_ARG_REGISTERS as usize {
                let arg_reg = self.value_to_register(arg_val)?;
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

            for instr_val in block.instruction_iter(self.context) {
                self.compile_instruction(handler, &instr_val, func_is_entry)?;
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
                .push((function, start_label, ops, test_decl_ref));
        } else {
            self.non_entries.push(ops);
        }

        Ok(())
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
                            self.cur_bytecode.push(Op {
                                opcode: Either::Left(VirtualOp::LW(
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
        Vec<(u64, u64, DataId)>,
    ) {
        // If they're immutable and have a constant initialiser then they go in the data section.
        //
        // Otherwise they go in runtime allocated space, either a register or on the stack.
        //
        // Stack offsets are in words to both enforce alignment and simplify use with LW/SW.
        let (stack_base, init_mut_vars) = function.locals_iter(self.context).fold(
            (0, Vec::new()),
            |(stack_base, mut init_mut_vars), (_name, ptr)| {
                if let (false, Some(constant)) = (
                    ptr.is_mutable(self.context),
                    ptr.get_initializer(self.context),
                ) {
                    let data_id = self.data_section.insert_data_value(Entry::from_constant(
                        self.context,
                        constant,
                        None,
                    ));
                    self.ptr_map.insert(*ptr, Storage::Data(data_id));
                    (stack_base, init_mut_vars)
                } else {
                    self.ptr_map.insert(*ptr, Storage::Stack(stack_base));

                    let ptr_ty = ptr.get_inner_type(self.context);
                    let var_size = match ptr_ty.get_content(self.context) {
                        TypeContent::Uint(256) => 4,
                        TypeContent::Unit
                        | TypeContent::Bool
                        | TypeContent::Uint(_)
                        | TypeContent::Pointer(_) => 1,
                        TypeContent::Slice => 2,
                        TypeContent::B256 => 4,
                        TypeContent::String(n) => size_bytes_round_up_to_word_alignment!(n),
                        TypeContent::Array(..) | TypeContent::Struct(_) | TypeContent::Union(_) => {
                            size_bytes_in_words!(ir_type_size_in_bytes(self.context, &ptr_ty))
                        }
                    };

                    if let Some(constant) = ptr.get_initializer(self.context) {
                        let data_id = self.data_section.insert_data_value(Entry::from_constant(
                            self.context,
                            constant,
                            None,
                        ));

                        init_mut_vars.push((stack_base, var_size, data_id));
                    }

                    (stack_base + var_size, init_mut_vars)
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

        let locals_size = stack_base * 8;
        if locals_size > compiler_constants::TWENTY_FOUR_BITS {
            todo!("Enormous stack usage for locals.");
        }
        self.cur_bytecode.push(Op {
            opcode: Either::Left(VirtualOp::CFEI(VirtualImmediate24 {
                value: locals_size as u32,
            })),
            comment: format!("allocate {locals_size} bytes for locals"),
            owning_span: None,
        });
        (locals_size, locals_base_reg, init_mut_vars)
    }

    fn init_locals(
        &mut self,
        (locals_size, locals_base_reg, init_mut_vars): (
            u64,
            virtual_register::VirtualRegister,
            Vec<(u64, u64, DataId)>,
        ),
    ) {
        // Initialise that stack variables which require it.
        for (var_stack_offs, var_word_size, var_data_id) in init_mut_vars {
            // Load our initialiser from the data section.
            self.cur_bytecode.push(Op {
                opcode: Either::Left(VirtualOp::LWDataId(
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
                self.cur_bytecode.push(Op {
                    opcode: Either::Left(VirtualOp::SW(
                        dst_reg,
                        VirtualRegister::Constant(ConstantRegister::Scratch),
                        VirtualImmediate12 { value: 0 },
                    )),
                    comment: "store initializer to local variable".to_owned(),
                    owning_span: None,
                });
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

        self.locals_ctxs.push((locals_size, locals_base_reg));
    }

    fn drop_locals(&mut self, _function: Function) {
        let (locals_size, _locals_base_reg) = self
            .locals_ctxs
            .pop()
            .expect("Calls guaranteed to save locals context.");
        if locals_size > compiler_constants::TWENTY_FOUR_BITS {
            todo!("Enormous stack usage for locals.");
        }
        self.cur_bytecode.push(Op {
            opcode: Either::Left(VirtualOp::CFSI(VirtualImmediate24 {
                value: u32::try_from(locals_size).unwrap(),
            })),
            comment: format!("free {locals_size} bytes for locals"),
            owning_span: None,
        });
    }

    pub(super) fn locals_base_reg(&self) -> &VirtualRegister {
        &self.locals_ctxs.last().expect("No locals").1
    }
}
