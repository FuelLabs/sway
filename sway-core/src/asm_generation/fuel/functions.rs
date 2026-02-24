use crate::{
    asm_generation::{
        from_ir::*,
        fuel::{
            compiler_constants::{self, TWELVE_BITS},
            data_section::Entry,
            fuel_asm_builder::FuelAsmBuilder,
        },
        ProgramKind,
    },
    asm_lang::{
        virtual_register::{self, *},
        ControlFlowOp, JumpType, Op, OrganizationalOp, VirtualImmediate12, VirtualImmediate18,
        VirtualImmediate24, VirtualOp,
    },
    decl_engine::DeclRef,
    fuel_prelude::fuel_asm::GTFArgs,
};

use sway_ir::*;

use either::Either;
use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
};
use sway_types::{Ident, Span};

use super::{compiler_constants::NUM_ARG_REGISTERS, data_section::EntryName};

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
impl FuelAsmBuilder<'_, '_> {
    pub(super) fn compile_call(
        &mut self,
        instr_val: &Value,
        function: &Function,
        args: &[Value],
    ) -> Result<(), CompileError> {
        let fn_name = function.get_name(self.context);

        // Put the args into the args registers.
        if args.len() <= compiler_constants::NUM_ARG_REGISTERS as usize {
            for (idx, arg_val) in args.iter().enumerate() {
                let arg_reg = self.value_to_register(arg_val)?;
                self.cur_bytecode.push(Op::register_move(
                    VirtualRegister::Constant(ConstantRegister::ARG_REGS[idx]),
                    arg_reg,
                    format!("[call: {fn_name}]: pass argument {idx}"),
                    self.md_mgr.val_to_span(self.context, *arg_val),
                ));
            }
        } else {
            // Register ARG_REGS[NUM_ARG_REGISTERS-1] must contain LocalsBase + locals_size
            // so that the callee can index the stack arguments from there.
            // It's also useful for us to save the arguments to the stack next.
            if self.locals_size_bytes() <= TWELVE_BITS {
                self.cur_bytecode.push(Op {
                    opcode: Either::Left(VirtualOp::ADDI(
                        VirtualRegister::Constant(
                            ConstantRegister::ARG_REGS
                                [(compiler_constants::NUM_ARG_REGISTERS - 1) as usize],
                        ),
                        VirtualRegister::Constant(ConstantRegister::LocalsBase),
                        VirtualImmediate12::try_new(self.locals_size_bytes(), Span::dummy())
                            .expect("Stack size too big for these many arguments, cannot handle."),
                    )),
                    comment: format!("[call: {fn_name}]: save address of stack arguments in last argument register"),
                    owning_span: self.md_mgr.val_to_span(self.context, *instr_val),
                });
            } else {
                self.cur_bytecode.push(Op {
                    opcode: Either::Left(VirtualOp::MOVI(
                        VirtualRegister::Constant(
                            ConstantRegister::ARG_REGS
                                [(compiler_constants::NUM_ARG_REGISTERS - 1) as usize],
                        ),
                        VirtualImmediate18::try_new(self.locals_size_bytes(), Span::dummy())
                            .expect("Stack size too big for these many arguments, cannot handle."),
                    )),
                    comment: format!(
                        "[call: {fn_name}]: temporarily save locals size to add up next"
                    ),
                    owning_span: self.md_mgr.val_to_span(self.context, *instr_val),
                });
                self.cur_bytecode.push(Op {
                    opcode: Either::Left(VirtualOp::ADD(
                        VirtualRegister::Constant(
                            ConstantRegister::ARG_REGS
                                [(compiler_constants::NUM_ARG_REGISTERS - 1) as usize],
                        ),
                        VirtualRegister::Constant(ConstantRegister::LocalsBase),
                        VirtualRegister::Constant(
                            ConstantRegister::ARG_REGS
                                [(compiler_constants::NUM_ARG_REGISTERS - 1) as usize],
                        ),
                    )),
                    comment: format!("[call: {fn_name}]: save address of stack arguments in last argument register"),
                    owning_span: self.md_mgr.val_to_span(self.context, *instr_val),
                });
            }

            // Put NUM_ARG_REGISTERS - 1 arguments into arg registers and rest into the stack.
            for (idx, arg_val) in args.iter().enumerate() {
                let arg_reg = self.value_to_register(arg_val)?;
                // Except for the last arg register, the others hold an argument.
                if idx < compiler_constants::NUM_ARG_REGISTERS as usize - 1 {
                    self.cur_bytecode.push(Op::register_move(
                        VirtualRegister::Constant(ConstantRegister::ARG_REGS[idx]),
                        arg_reg,
                        format!("[call: {fn_name}]: pass argument {idx}"),
                        self.md_mgr.val_to_span(self.context, *arg_val),
                    ));
                } else {
                    // All arguments [NUM_ARG_REGISTERS - 1 ..] go into the stack.
                    assert!(
                        self.locals_size_bytes().is_multiple_of(8),
                        "The size of locals is not word aligned"
                    );
                    let stack_offset =
                        (idx as u64 + 1) - compiler_constants::NUM_ARG_REGISTERS as u64;
                    let stack_offset_bytes = self.locals_size_bytes() + (stack_offset * 8);
                    assert!(
                        stack_offset_bytes
                            < self.locals_size_bytes() + (self.max_num_extra_args() * 8)
                    );
                    self.cur_bytecode.push(Op {
                        opcode: Either::Left(VirtualOp::SW(
                            VirtualRegister::Constant(
                                ConstantRegister::ARG_REGS
                                    [compiler_constants::NUM_ARG_REGISTERS as usize - 1],
                            ),
                            arg_reg,
                            VirtualImmediate12::try_new(
                                stack_offset,
                                self.md_mgr
                                    .val_to_span(self.context, *arg_val)
                                    .unwrap_or(Span::dummy()),
                            )
                            .expect("Too many arguments, cannot handle."),
                        )),
                        comment: format!(
                            "[call: {fn_name}]: pass argument {idx} via its stack slot"
                        ),
                        owning_span: self.md_mgr.val_to_span(self.context, *arg_val),
                    });
                }
            }
        }

        // Jump to function and insert return label.
        let (fn_label, _) = self.func_to_labels(function);
        self.cur_bytecode.push(Op {
            opcode: Either::Right(OrganizationalOp::Jump {
                to: fn_label,
                type_: JumpType::Call,
            }),
            comment: format!("[call: {fn_name}]: call function"),
            owning_span: None,
        });

        // Save the return value, if it is not of type unit.
        let ret_reg = self.reg_seqr.next();
        if !function.get_return_type(self.context).is_unit(self.context) {
            self.cur_bytecode.push(Op {
                opcode: Either::Left(VirtualOp::MOVE(
                    ret_reg.clone(),
                    VirtualRegister::Constant(ConstantRegister::CallReturnValue),
                )),
                comment: format!("[call: {fn_name}]: copy returned value"),
                owning_span: None,
            });
        } else {
            self.cur_bytecode.push(Op {
                opcode: Either::Left(VirtualOp::MOVE(
                    ret_reg.clone(),
                    VirtualRegister::Constant(ConstantRegister::Zero),
                )),
                comment: format!("[call: {fn_name}]: copy returned unit value"),
                owning_span: None,
            });
        }
        self.reg_map.insert(*instr_val, ret_reg);

        Ok(())
    }

    pub(super) fn compile_ret_from_call(
        &mut self,
        fn_name: &str,
        instr_val: &Value,
        ret_val: &Value,
    ) -> Result<(), CompileError> {
        // Move the result (if there is one) into the return value register.
        let owning_span = self.md_mgr.val_to_span(self.context, *instr_val);

        if !ret_val
            .get_type(self.context)
            .is_some_and(|t| t.is_unit(self.context))
        {
            let ret_reg = self.value_to_register(ret_val)?;
            self.cur_bytecode.push(Op::register_move(
                VirtualRegister::Constant(ConstantRegister::CallReturnValue),
                ret_reg,
                format!("[fn end: {fn_name}] set return value"),
                owning_span,
            ));
        }

        // Jump to the end of the function.
        let end_label = self
            .return_ctxs
            .last()
            .expect("Calls guaranteed to save return context.");
        self.cur_bytecode.push(Op::jump_to_label(*end_label));

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
                    .fold("".to_string(), |output, b| { format!("{output}{b:02x}") })
            )));
        }

        let is_entry_fn = function.is_entry(self.context);

        // Check function is a leaf fn
        let is_leaf_fn = function.is_leaf_fn(self.context);

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

        self.cur_bytecode.push(match &span {
            Some(span) => Op::jump_label_comment(start_label, span.clone(), comment),
            None => Op::unowned_jump_label_comment(start_label, comment),
        });

        let fn_name = function.get_name(self.context);

        // Manage the call frame.
        if !is_entry_fn {
            // Save any general purpose registers used here on the stack.
            self.cur_bytecode.push(Op {
                opcode: Either::Right(OrganizationalOp::PushAll(start_label)),
                comment: format!("[fn init: {fn_name}]: push all used registers to stack"),
                owning_span: span.clone(),
            });
        }

        let locals_alloc_result = self.alloc_locals(function);

        if is_entry_fn {
            self.compile_external_args(function)
                .map_err(|e| handler.emit_err(e))?
        } else {
            // Make copies of the arg registers, if function is not a leaf fn
            self.compile_fn_call_args(function, is_leaf_fn)
        }

        let reta = self.reg_seqr.next();

        if !is_entry_fn {
            // Store some info describing the call frame.
            self.return_ctxs.push(end_label);
        }

        if !is_leaf_fn && !is_entry_fn {
            self.cur_bytecode.push(Op::register_move(
                reta.clone(),
                VirtualRegister::Constant(ConstantRegister::CallReturnAddress),
                format!("[fn init: {fn_name}]: save return address"),
                None,
            ));
        }

        self.init_locals(locals_alloc_result);

        // Compile instructions. Traverse the IR blocks in reverse post order. This guarantees that
        // each block is processed after all its CFG predecessors have been processed.
        let po = sway_ir::dominator::compute_post_order(self.context, &function);
        for block in po.po_to_block.iter().rev() {
            let label = self.block_to_label(block);
            self.cur_bytecode.push(Op::unowned_jump_label(label));
            self.compile_block(handler, block, is_entry_fn)?;
        }

        // Generate epilogue for non-entry functions.
        // Entry functions will return to the caller via a RET(D),
        // so they don't need an epilogue.
        if !is_entry_fn {
            // Insert the end of function label.
            self.cur_bytecode.push(Op::unowned_jump_label(end_label));

            // Pop the call frame entry.
            self.return_ctxs.pop();

            // Free our stack allocated locals.
            self.drop_locals(fn_name);

            if !is_leaf_fn {
                // Restore $reta.
                self.cur_bytecode.push(Op::register_move(
                    VirtualRegister::Constant(ConstantRegister::CallReturnAddress),
                    reta,
                    format!("[fn end: {fn_name}] restore return address"),
                    None,
                ));
            }

            // Restore general purpose registers.
            self.cur_bytecode.push(Op {
                opcode: Either::Right(OrganizationalOp::PopAll(start_label)),
                comment: format!("[fn end: {fn_name}] restore all used registers"),
                owning_span: None,
            });

            // Jump to the return address.
            self.cur_bytecode.push(Op {
                opcode: Either::Right(ControlFlowOp::ReturnFromCall {
                    zero: VirtualRegister::Constant(ConstantRegister::Zero),
                    reta: VirtualRegister::Constant(ConstantRegister::CallReturnAddress),
                }),
                comment: format!("[fn end: {fn_name}] return from call"),
                owning_span: None,
            });
        }

        // Save this function.
        let mut ops = Vec::new();
        ops.append(&mut self.cur_bytecode);
        if is_entry_fn {
            self.entries
                .push((function, start_label, ops, test_decl_ref));
        } else {
            self.non_entries.push((function, ops));
        }

        Ok(())
    }

    /// Copy all arguments that are passed as registers into new registers. This is done
    /// to allow the current function to call others fns, as the set of function arguments is
    /// always the same.
    ///
    /// This is not required for "leaf fns" as they do not call others.
    ///
    /// We load arguments from the stack on both cases
    fn compile_fn_call_args(&mut self, function: Function, is_leaf_fn: bool) {
        let fn_name = function.get_name(self.context);
        let uses_stack =
            function.num_args(self.context) > compiler_constants::NUM_ARG_REGISTERS as usize;
        for (idx, (arg_name, arg_val)) in function.args_iter(self.context).enumerate() {
            let load_arg =
                uses_stack && (idx >= compiler_constants::NUM_ARG_REGISTERS as usize - 1);
            let arg_reg = if !load_arg {
                let initial_arg_reg = VirtualRegister::Constant(ConstantRegister::ARG_REGS[idx]);
                if !is_leaf_fn {
                    let arg_copy_reg = self.reg_seqr.next();
                    self.cur_bytecode.push(Op::register_move(
                        arg_copy_reg.clone(),
                        initial_arg_reg,
                        format!("[fn init: {fn_name}]: copy argument {idx} ({arg_name})"),
                        self.md_mgr.val_to_span(self.context, *arg_val),
                    ));
                    arg_copy_reg
                } else {
                    initial_arg_reg
                }
            } else {
                let arg_copy_reg = self.reg_seqr.next();

                // All arguments [NUM_ARG_REGISTERS - 1 ..] go into the stack.
                assert!(
                    self.locals_size_bytes().is_multiple_of(8),
                    "The size of locals is not word aligned"
                );

                let stack_offset = (idx as u64 + 1) - compiler_constants::NUM_ARG_REGISTERS as u64;

                self.cur_bytecode.push(Op {
                    opcode: Either::Left(VirtualOp::LW(
                        arg_copy_reg.clone(),
                        VirtualRegister::Constant(
                            ConstantRegister::ARG_REGS
                                [compiler_constants::NUM_ARG_REGISTERS as usize - 1],
                        ),
                        VirtualImmediate12::try_new(
                            stack_offset,
                            self.md_mgr
                                .val_to_span(self.context, *arg_val)
                                .unwrap_or(Span::dummy()),
                        )
                        .expect("Too many arguments, cannot handle."),
                    )),
                    comment: format!("[fn init: {fn_name}]: load argument {idx} ({arg_name}) from its stack slot"),
                    owning_span: self.md_mgr.val_to_span(self.context, *arg_val),
                });

                arg_copy_reg
            };

            // Remember our arg copy.
            self.reg_map.insert(*arg_val, arg_reg);
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
                            .is_some_and(|t| self.is_copy_type(&t))
                        {
                            self.cur_bytecode.push(Op {
                                opcode: either::Either::Left(VirtualOp::LW(
                                    single_arg_reg.clone(),
                                    single_arg_reg.clone(),
                                    VirtualImmediate12::new(0),
                                )),
                                comment: "load main function parameter".into(),
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
                    let arg_type_size = arg_type.size(self.context);
                    if self.is_copy_type(&arg_type) {
                        if arg_word_offset > compiler_constants::TWELVE_BITS {
                            let offs_reg = self.reg_seqr.next();
                            self.cur_bytecode.push(Op {
                                opcode: Either::Left(VirtualOp::ADD(
                                    args_base_reg.clone(),
                                    args_base_reg.clone(),
                                    offs_reg.clone(),
                                )),
                                comment: format!("get offset of argument {name}"),
                                owning_span: None,
                            });

                            if arg_type_size.in_bytes() == 1 {
                                self.cur_bytecode.push(Op {
                                    opcode: Either::Left(VirtualOp::LB(
                                        current_arg_reg.clone(),
                                        offs_reg,
                                        VirtualImmediate12::new(0),
                                    )),
                                    comment: format!("get argument {name}"),
                                    owning_span: None,
                                });
                            } else {
                                self.cur_bytecode.push(Op {
                                    opcode: Either::Left(VirtualOp::LW(
                                        current_arg_reg.clone(),
                                        offs_reg,
                                        VirtualImmediate12::new(0),
                                    )),
                                    comment: format!("get argument {name}"),
                                    owning_span: None,
                                });
                            }
                        } else if arg_type_size.in_bytes() == 1 {
                            self.cur_bytecode.push(Op {
                                opcode: Either::Left(VirtualOp::LB(
                                    current_arg_reg.clone(),
                                    args_base_reg.clone(),
                                    VirtualImmediate12::new(arg_word_offset * 8),
                                )),
                                comment: format!("get argument {name}"),
                                owning_span: None,
                            });
                        } else {
                            self.cur_bytecode.push(Op {
                                opcode: Either::Left(VirtualOp::LW(
                                    current_arg_reg.clone(),
                                    args_base_reg.clone(),
                                    VirtualImmediate12::new(arg_word_offset),
                                )),
                                comment: format!("get argument {name}"),
                                owning_span: None,
                            });
                        }
                    } else {
                        self.immediate_to_reg(
                            arg_word_offset * 8,
                            current_arg_reg.clone(),
                            Some(&args_base_reg),
                            format!("get offset of argument {name}"),
                            None,
                        );
                    }

                    arg_word_offset += arg_type_size.in_words();
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
                VirtualImmediate12::new(74),
            )),
            comment: "get base register for method arguments".into(),
            owning_span: None,
        });
    }

    // Read the argument(s) base from the script data.
    fn read_args_base_from_script_data(&mut self, reg: &VirtualRegister) {
        self.cur_bytecode.push(Op {
            opcode: either::Either::Left(VirtualOp::GTF(
                reg.clone(),
                VirtualRegister::Constant(ConstantRegister::Zero),
                VirtualImmediate12::new(GTFArgs::ScriptData as u64),
            )),
            comment: "get base register for main function arguments".into(),
            owning_span: None,
        });
    }

    /// Read the returns the base pointer for predicate data
    fn read_args_base_from_predicate_data(&mut self, base_reg: &VirtualRegister) {
        // Final label to jump to continue execution, once the predicate data pointer is
        // successfully found
        let success_label = self.reg_seqr.get_label();

        // Use the `gm` instruction to get the index of the predicate. This is the index we're
        // going to use in the subsequent `gtf` instructions.
        let input_index = self.reg_seqr.next();
        self.cur_bytecode.push(Op {
            opcode: either::Either::Left(VirtualOp::GM(
                input_index.clone(),
                VirtualImmediate18::new(3),
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
                VirtualImmediate12::new(GTFArgs::InputType as u64),
            )),
            comment: "get predicate input type".into(),
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
                VirtualImmediate12::new(GTFArgs::InputCoinPredicateData as u64),
            )),
            comment: "get predicate input coin data pointer".into(),
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
            opcode: Either::Left(VirtualOp::MOVI(two.clone(), VirtualImmediate18::new(2))),
            comment:
                "[predicate input is message]: set register to 2 (Input::Message discriminator)"
                    .into(),
            owning_span: None,
        });
        self.cur_bytecode.push(Op {
            opcode: either::Either::Left(VirtualOp::EQ(
                input_type_is_message.clone(),
                input_type,
                two,
            )),
            comment: "[predicate input is message]: check if input type is message".into(),
            owning_span: None,
        });

        // Invert `input_type_is_message` to use in `jnzi`
        let input_type_not_message = self.reg_seqr.next();
        self.cur_bytecode.push(Op {
            opcode: Either::Left(VirtualOp::XORI(
                input_type_not_message.clone(),
                input_type_is_message,
                VirtualImmediate12::new(1),
            )),
            comment: "[predicate input is message]: check if input type is not message".into(),
            owning_span: None,
        });

        // Label to jump to if the input type is *not* 2, i.e. not "message" (and not "coin" since
        // we checked that earlier). Then do the jump.
        let input_type_not_message_label = self.reg_seqr.get_label();
        self.cur_bytecode.push(Op::jump_if_not_zero_comment(
            input_type_not_message,
            input_type_not_message_label,
            "[predicate input is message]: jump to return false from predicate",
        ));

        // If the input is indeed a "message", then use `GTF` to get the "input message predicate
        // data pointer" and store it in `base_reg`
        self.cur_bytecode.push(Op {
            opcode: either::Either::Left(VirtualOp::GTF(
                base_reg.clone(),
                input_index,
                VirtualImmediate12::new(GTFArgs::InputMessagePredicateData as u64),
            )),
            comment: "get predicate input message data pointer".into(),
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
            comment: "return false from predicate".into(),
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
        Vec<InitMutVars>,
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
                assert!(args.iter().all(|arg| arg
                    .get_type(self.context)
                    .unwrap()
                    .size(self.context)
                    .in_words()
                    <= 1));
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
                    match constant.get_content(self.context).value {
                        ConstantValue::Uint(c) if c <= compiler_constants::EIGHTEEN_BITS => {
                            self.ptr_map
                                .insert(*ptr, Storage::Const(VirtualImmediate18::new(c)));
                        }
                        _ => {
                            let data_id =
                                self.data_section.insert_data_value(Entry::from_constant(
                                    self.context,
                                    constant.get_content(self.context),
                                    EntryName::NonConfigurable,
                                    None,
                                ));
                            self.ptr_map.insert(*ptr, Storage::Data(data_id));
                        }
                    }
                    (stack_base_words, init_mut_vars)
                } else {
                    self.ptr_map.insert(*ptr, Storage::Stack(stack_base_words));

                    let ptr_ty = ptr.get_inner_type(self.context);
                    let var_size = ptr_ty.size(self.context);

                    if let Some(constant) = ptr.get_initializer(self.context) {
                        match constant.get_content(self.context).value {
                            ConstantValue::Uint(c) if c <= compiler_constants::EIGHTEEN_BITS => {
                                let imm = VirtualImmediate18::new(c);
                                init_mut_vars.push(InitMutVars {
                                    stack_base_words,
                                    var_size: var_size.clone(),
                                    data: Storage::Const(imm),
                                });
                            }
                            _ => {
                                let data_id =
                                    self.data_section.insert_data_value(Entry::from_constant(
                                        self.context,
                                        constant.get_content(self.context),
                                        EntryName::NonConfigurable,
                                        None,
                                    ));

                                init_mut_vars.push(InitMutVars {
                                    stack_base_words,
                                    var_size: var_size.clone(),
                                    data: Storage::Data(data_id),
                                });
                            }
                        }
                    }

                    (stack_base_words + var_size.in_words(), init_mut_vars)
                }
            },
        );

        // Reserve space on the stack (in bytes) for all our locals which require it.  Firstly save
        // the current $sp.
        let fn_name = function.get_name(self.context);
        let fn_init_prefix = format!(
            "[{} init: {fn_name}]:",
            if function.is_entry(self.context) {
                "entry"
            } else {
                "fn"
            }
        );
        let locals_base_reg = VirtualRegister::Constant(ConstantRegister::LocalsBase);
        self.cur_bytecode.push(Op::register_move(
            locals_base_reg.clone(),
            VirtualRegister::Constant(ConstantRegister::StackPointer),
            format!("{fn_init_prefix} set locals base register"),
            None,
        ));

        let locals_size_bytes = stack_base_words * 8;
        if locals_size_bytes > compiler_constants::TWENTY_FOUR_BITS {
            todo!("Enormous stack usage for locals.");
        }
        self.cur_bytecode.push(Op {
            opcode: Either::Left(VirtualOp::CFEI(
                VirtualRegister::Constant(ConstantRegister::StackPointer),
                VirtualImmediate24::new(locals_size_bytes + (max_num_extra_args * 8),)
            )),
            comment: format!("{fn_init_prefix} allocate: locals {locals_size_bytes} byte(s), call args {max_num_extra_args} slot(s)"),
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
            Vec<InitMutVars>,
            u64,
        ),
    ) {
        // Initialise that stack variables which requires it.
        for InitMutVars {
            stack_base_words,
            var_size,
            data,
        } in init_mut_vars
        {
            if var_size.in_bytes() == 0 {
                // Don't bother initializing zero-sized types.
                continue;
            }
            // Load our initialiser from the data section.
            match data {
                Storage::Data(data_id) => {
                    self.cur_bytecode.push(Op {
                        opcode: Either::Left(VirtualOp::LoadDataId(
                            VirtualRegister::Constant(ConstantRegister::Scratch),
                            data_id,
                        )),
                        comment: "load local variable initializer from data section".to_owned(),
                        owning_span: None,
                    });
                }
                Storage::Stack(_) => panic!("Initializer cannot be on the stack"),
                Storage::Const(c) => {
                    self.cur_bytecode.push(Op {
                        opcode: Either::Left(VirtualOp::MOVI(
                            VirtualRegister::Constant(ConstantRegister::Scratch),
                            c.clone(),
                        )),
                        comment: "load local variable initializer from register".into(),
                        owning_span: None,
                    });
                }
            }

            // Get the stack offset in bytes rather than words.
            let var_stack_off_bytes = stack_base_words * 8;
            let dst_reg = self.reg_seqr.next();
            // Check if we can use the `ADDi` opcode.
            if var_stack_off_bytes <= compiler_constants::TWELVE_BITS {
                // Get the destination on the stack.
                self.cur_bytecode.push(Op {
                    opcode: Either::Left(VirtualOp::ADDI(
                        dst_reg.clone(),
                        locals_base_reg.clone(),
                        VirtualImmediate12::new(var_stack_off_bytes),
                    )),
                    comment: "get local variable address".to_owned(),
                    owning_span: None,
                });
            } else {
                assert!(var_stack_off_bytes <= compiler_constants::EIGHTEEN_BITS);
                // We can't, so load the immediate into a register and then add.
                self.cur_bytecode.push(Op {
                    opcode: Either::Left(VirtualOp::MOVI(
                        dst_reg.clone(),
                        VirtualImmediate18::new(var_stack_off_bytes),
                    )),
                    comment: "move stack offset of local variable into register".to_owned(),
                    owning_span: None,
                });
                self.cur_bytecode.push(Op {
                    opcode: Either::Left(VirtualOp::ADD(
                        dst_reg.clone(),
                        locals_base_reg.clone(),
                        dst_reg.clone(),
                    )),
                    comment: "get local variable address".to_owned(),
                    owning_span: None,
                });
            }

            if var_size.in_words() == 1 {
                // Initialise by value.
                if var_size.in_bytes() == 1 {
                    self.cur_bytecode.push(Op {
                        opcode: Either::Left(VirtualOp::SB(
                            dst_reg,
                            VirtualRegister::Constant(ConstantRegister::Scratch),
                            VirtualImmediate12::new(0),
                        )),
                        comment: "store byte initializer to local variable".to_owned(),
                        owning_span: None,
                    });
                } else {
                    self.cur_bytecode.push(Op {
                        opcode: Either::Left(VirtualOp::SW(
                            dst_reg,
                            VirtualRegister::Constant(ConstantRegister::Scratch),
                            VirtualImmediate12::new(0),
                        )),
                        comment: "store word initializer to local variable".to_owned(),
                        owning_span: None,
                    });
                }
            } else {
                // Initialise by reference.
                assert!(var_size.in_bytes_aligned() <= compiler_constants::TWELVE_BITS);
                self.cur_bytecode.push(Op {
                    opcode: Either::Left(VirtualOp::MCPI(
                        dst_reg,
                        VirtualRegister::Constant(ConstantRegister::Scratch),
                        VirtualImmediate12::new(var_size.in_bytes_aligned()),
                    )),
                    comment: "copy initializer from data section to local variable".to_owned(),
                    owning_span: None,
                });
            }
        }

        self.locals_ctxs
            .push((locals_size_bytes, locals_base_reg, max_num_extra_args));
    }

    /// Free stack allocated locals in non-entry functions.
    pub(super) fn drop_locals(&mut self, fn_name: &str) {
        let (locals_size_bytes, max_num_extra_args) =
            (self.locals_size_bytes(), self.max_num_extra_args());
        if locals_size_bytes > compiler_constants::TWENTY_FOUR_BITS {
            todo!("Enormous stack usage for locals.");
        }
        self.cur_bytecode.push(Op {
            opcode: Either::Left(
                VirtualOp::CFSI(VirtualRegister::Constant(ConstantRegister::StackPointer),
                VirtualImmediate24::new(locals_size_bytes + (max_num_extra_args * 8), ))),
            comment: format!("[fn end: {fn_name}] free: locals {locals_size_bytes} byte(s), call args {max_num_extra_args} slot(s)"),
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
    stack_base_words: u64,
    var_size: TypeSize,
    data: Storage,
}
