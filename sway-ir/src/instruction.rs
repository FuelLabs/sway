//! Instructions for data manipulation, but mostly control flow.
//!
//! Since Sway abstracts most low level operations behind traits they are translated into function
//! calls which contain ASM blocks.
//!
//! Unfortuntely, using opaque ASM blocks limits the effectiveness of certain optimizations and
//! this should be addressed in the future, perhaps by using compiler intrinsic calls instead of
//! the ASM blocks where possible. See: https://github.com/FuelLabs/sway/issues/855,

use rustc_hash::FxHashMap;
use sway_types::ident::Ident;

use crate::{
    asm::{AsmArg, AsmBlock, AsmInstruction},
    block::Block,
    constant::Constant,
    context::Context,
    function::Function,
    irtype::Type,
    local_var::LocalVar,
    pretty::DebugWithContext,
    value::{Value, ValueDatum},
};

#[derive(Debug, Clone, DebugWithContext)]
pub struct BranchToWithArgs {
    pub block: Block,
    pub args: Vec<Value>,
}

#[derive(Debug, Clone, DebugWithContext)]
pub enum Instruction {
    /// An opaque list of ASM instructions passed directly to codegen.
    AsmBlock(AsmBlock, Vec<AsmArg>),
    /// Unary arithmetic operations
    UnaryOp { op: UnaryOpKind, arg: Value },
    /// Binary arithmetic operations
    BinaryOp {
        op: BinaryOpKind,
        arg1: Value,
        arg2: Value,
    },
    /// Cast the type of a value without changing its actual content.
    BitCast(Value, Type),
    /// An unconditional jump.
    Branch(BranchToWithArgs),
    /// A function call with a list of arguments.
    Call(Function, Vec<Value>),
    /// Cast a value's type from one pointer to another.
    CastPtr(Value, Type),
    /// Comparison between two values using various comparators and returning a boolean.
    Cmp(Predicate, Value, Value),
    /// A conditional jump with the boolean condition value and true or false destinations.
    ConditionalBranch {
        cond_value: Value,
        true_block: BranchToWithArgs,
        false_block: BranchToWithArgs,
    },
    /// A contract call with a list of arguments
    ContractCall {
        return_type: Type,
        name: String,
        params: Value,
        coins: Value,
        asset_id: Value,
        gas: Value,
    },
    /// Umbrella instruction variant for FuelVM-specific instructions
    FuelVm(FuelVmInstruction),
    /// Return a local variable.
    GetLocal(LocalVar),
    /// Translate a pointer from a base to a nested element in an aggregate type.
    GetElemPtr {
        base: Value,
        elem_ptr_ty: Type,
        indices: Vec<Value>,
    },
    /// Re-interpret an integer value as pointer of some type
    IntToPtr(Value, Type),
    /// Read a value from a memory pointer.
    Load(Value),
    /// Copy a specified number of bytes between pointers.
    MemCopyBytes {
        dst_val_ptr: Value,
        src_val_ptr: Value,
        byte_len: u64,
    },
    /// Copy a value from one pointer to another.
    MemCopyVal {
        dst_val_ptr: Value,
        src_val_ptr: Value,
    },
    /// No-op, handy as a placeholder instruction.
    Nop,
    /// Cast a pointer to an integer.
    PtrToInt(Value, Type),
    /// Return from a function.
    Ret(Value, Type),
    /// Write a value to a memory pointer.
    Store {
        dst_val_ptr: Value,
        stored_val: Value,
    },
}

#[derive(Debug, Clone, DebugWithContext)]
pub enum FuelVmInstruction {
    Gtf {
        index: Value,
        tx_field_id: u64,
    },
    /// Logs a value along with an identifier.
    Log {
        log_val: Value,
        log_ty: Type,
        log_id: Value,
    },
    /// Reads a special register in the VM.
    ReadRegister(Register),
    /// Revert VM execution.
    Revert(Value),
    /// - Sends a message to an output via the `smo` FuelVM instruction. The first operand must be
    /// a `B256` representing the recipient. The second operand is the message data being sent.
    /// - `message_size` and `coins` must be of type `U64`.
    Smo {
        recipient: Value,
        message: Value,
        message_size: Value,
        coins: Value,
    },
    /// Clears `number_of_slots` storage slots (`b256` each) starting at key `key`.
    StateClear {
        key: Value,
        number_of_slots: Value,
    },
    /// Reads `number_of_slots` slots (`b256` each) from storage starting at key `key` and stores
    /// them in memory starting at address `load_val`.
    StateLoadQuadWord {
        load_val: Value,
        key: Value,
        number_of_slots: Value,
    },
    /// Reads and returns single word from a storage slot.
    StateLoadWord(Value),
    /// Stores `number_of_slots` slots (`b256` each) starting at address `stored_val` in memory into
    /// storage starting at key `key`. `key` must be a `b256`.
    StateStoreQuadWord {
        stored_val: Value,
        key: Value,
        number_of_slots: Value,
    },
    /// Writes a single word to a storage slot. `key` must be a `b256` and the type of `stored_val`
    /// must be a `u64`.
    StateStoreWord {
        stored_val: Value,
        key: Value,
    },
}

/// Comparison operations.
#[derive(Debug, Clone, Copy)]
pub enum Predicate {
    Equal,
    LessThan,
    GreaterThan,
}

#[derive(Debug, Clone, Copy)]
pub enum UnaryOpKind {
    Not,
}

#[derive(Debug, Clone, Copy)]
pub enum BinaryOpKind {
    Add,
    Sub,
    Mul,
    Div,
    And,
    Or,
    Xor,
    Mod,
    Rsh,
    Lsh,
}

/// Special registers in the Fuel Virtual Machine.
#[derive(Debug, Clone, Copy)]
pub enum Register {
    /// Contains overflow/underflow of addition, subtraction, and multiplication.
    Of,
    /// The program counter. Memory address of the current instruction.
    Pc,
    /// Memory address of bottom of current writable stack area.
    Ssp,
    /// Memory address on top of current writable stack area (points to free memory).
    Sp,
    /// Memory address of beginning of current call frame.
    Fp,
    /// Memory address below the current bottom of the heap (points to free memory).
    Hp,
    /// Error codes for particular operations.
    Error,
    /// Remaining gas globally.
    Ggas,
    /// Remaining gas in the context.
    Cgas,
    /// Received balance for this context.
    Bal,
    /// Pointer to the start of the currently-executing code.
    Is,
    /// Return value or pointer.
    Ret,
    /// Return value length in bytes.
    Retl,
    /// Flags register.
    Flag,
}

impl Instruction {
    /// Some [`Instruction`]s can return a value, but for some a return value doesn't make sense.
    ///
    /// Those which perform side effects such as writing to memory and also terminators such as
    /// `Ret` do not have a type.
    pub fn get_type(&self, context: &Context) -> Option<Type> {
        match self {
            // These all return something in particular.
            Instruction::AsmBlock(asm_block, _) => Some(asm_block.get_type(context)),
            Instruction::UnaryOp { arg, .. } => arg.get_type(context),
            Instruction::BinaryOp { arg1, .. } => arg1.get_type(context),
            Instruction::BitCast(_, ty) => Some(*ty),
            Instruction::Call(function, _) => Some(context.functions[function.0].return_type),
            Instruction::CastPtr(_val, ty) => Some(*ty),
            Instruction::Cmp(..) => Some(Type::get_bool(context)),
            Instruction::ContractCall { return_type, .. } => Some(*return_type),
            Instruction::FuelVm(FuelVmInstruction::Gtf { .. }) => Some(Type::get_uint64(context)),
            Instruction::FuelVm(FuelVmInstruction::Log { .. }) => Some(Type::get_unit(context)),
            Instruction::FuelVm(FuelVmInstruction::ReadRegister(_)) => {
                Some(Type::get_uint64(context))
            }
            Instruction::FuelVm(FuelVmInstruction::Smo { .. }) => Some(Type::get_unit(context)),

            // Load needs to strip the pointer from the source type.
            Instruction::Load(ptr_val) => match &context.values[ptr_val.0].value {
                ValueDatum::Argument(arg) => arg.ty.get_pointee_type(context),
                ValueDatum::Configurable(conf) => conf.ty.get_pointee_type(context),
                ValueDatum::Constant(cons) => cons.ty.get_pointee_type(context),
                ValueDatum::Instruction(ins) => ins
                    .get_type(context)
                    .and_then(|ty| ty.get_pointee_type(context)),
            },

            // These return pointer types.
            Instruction::GetElemPtr { elem_ptr_ty, .. } => Some(*elem_ptr_ty),
            Instruction::GetLocal(local_var) => Some(local_var.get_type(context)),

            // Use for casting between pointers and pointer-width integers.
            Instruction::IntToPtr(_, ptr_ty) => Some(*ptr_ty),
            Instruction::PtrToInt(_, int_ty) => Some(*int_ty),

            // These are all terminators which don't return, essentially.  No type.
            Instruction::Branch(_)
            | Instruction::ConditionalBranch { .. }
            | Instruction::FuelVm(FuelVmInstruction::Revert(..))
            | Instruction::Ret(..) => None,

            // No-op is also no-type.
            Instruction::Nop => None,

            // State load returns a u64, other state ops return a bool.
            Instruction::FuelVm(FuelVmInstruction::StateLoadWord(_)) => {
                Some(Type::get_uint64(context))
            }
            Instruction::FuelVm(FuelVmInstruction::StateClear { .. })
            | Instruction::FuelVm(FuelVmInstruction::StateLoadQuadWord { .. })
            | Instruction::FuelVm(FuelVmInstruction::StateStoreQuadWord { .. })
            | Instruction::FuelVm(FuelVmInstruction::StateStoreWord { .. }) => {
                Some(Type::get_bool(context))
            }

            // Memory writes return unit.
            Instruction::MemCopyBytes { .. }
            | Instruction::MemCopyVal { .. }
            | Instruction::Store { .. } => Some(Type::get_unit(context)),
        }
    }

    pub fn get_operands(&self) -> Vec<Value> {
        match self {
            Instruction::AsmBlock(_, args) => args.iter().filter_map(|aa| aa.initializer).collect(),
            Instruction::BitCast(v, _) => vec![*v],
            Instruction::UnaryOp { op: _, arg } => vec![*arg],
            Instruction::BinaryOp { op: _, arg1, arg2 } => vec![*arg1, *arg2],
            Instruction::Branch(BranchToWithArgs { args, .. }) => args.clone(),
            Instruction::Call(_, vs) => vs.clone(),
            Instruction::CastPtr(val, _ty) => vec![*val],
            Instruction::Cmp(_, lhs, rhs) => vec![*lhs, *rhs],
            Instruction::ConditionalBranch {
                cond_value,
                true_block,
                false_block,
            } => {
                let mut v = vec![*cond_value];
                v.extend_from_slice(&true_block.args);
                v.extend_from_slice(&false_block.args);
                v
            }
            Instruction::ContractCall {
                return_type: _,
                name: _,
                params,
                coins,
                asset_id,
                gas,
            } => vec![*params, *coins, *asset_id, *gas],
            Instruction::GetElemPtr {
                base,
                elem_ptr_ty: _,
                indices,
            } => {
                let mut vals = indices.clone();
                vals.push(*base);
                vals
            }
            Instruction::GetLocal(_local_var) => {
                // TODO: Not sure.
                vec![]
            }
            Instruction::IntToPtr(v, _) => vec![*v],
            Instruction::Load(v) => vec![*v],
            Instruction::MemCopyBytes {
                dst_val_ptr,
                src_val_ptr,
                byte_len: _,
            } => {
                vec![*dst_val_ptr, *src_val_ptr]
            }
            Instruction::MemCopyVal {
                dst_val_ptr,
                src_val_ptr,
            } => {
                vec![*dst_val_ptr, *src_val_ptr]
            }
            Instruction::Nop => vec![],
            Instruction::PtrToInt(v, _) => vec![*v],
            Instruction::Ret(v, _) => vec![*v],
            Instruction::Store {
                dst_val_ptr,
                stored_val,
            } => {
                vec![*dst_val_ptr, *stored_val]
            }

            Instruction::FuelVm(fuel_vm_instr) => match fuel_vm_instr {
                FuelVmInstruction::Gtf {
                    index,
                    tx_field_id: _,
                } => vec![*index],
                FuelVmInstruction::Log {
                    log_val, log_id, ..
                } => vec![*log_val, *log_id],
                FuelVmInstruction::ReadRegister(_) => vec![],
                FuelVmInstruction::Revert(v) => vec![*v],
                FuelVmInstruction::Smo {
                    recipient,
                    message,
                    message_size,
                    coins,
                } => vec![*recipient, *message, *message_size, *coins],
                FuelVmInstruction::StateClear {
                    key,
                    number_of_slots,
                } => vec![*key, *number_of_slots],
                FuelVmInstruction::StateLoadQuadWord {
                    load_val,
                    key,
                    number_of_slots,
                } => vec![*load_val, *key, *number_of_slots],
                FuelVmInstruction::StateLoadWord(key) => vec![*key],
                FuelVmInstruction::StateStoreQuadWord {
                    stored_val,
                    key,
                    number_of_slots,
                } => {
                    vec![*stored_val, *key, *number_of_slots]
                }
                FuelVmInstruction::StateStoreWord { stored_val, key } => vec![*stored_val, *key],
            },
        }
    }

    /// Replace `old_val` with `new_val` if it is referenced by this instruction's arguments.
    pub fn replace_values(&mut self, replace_map: &FxHashMap<Value, Value>) {
        let replace = |val: &mut Value| {
            while let Some(new_val) = replace_map.get(val) {
                *val = *new_val;
            }
        };
        match self {
            Instruction::AsmBlock(_, args) => args.iter_mut().for_each(|asm_arg| {
                asm_arg
                    .initializer
                    .iter_mut()
                    .for_each(|init_val| replace(init_val))
            }),
            Instruction::BitCast(value, _) => replace(value),
            Instruction::UnaryOp { op: _, arg } => {
                replace(arg);
            }
            Instruction::BinaryOp { op: _, arg1, arg2 } => {
                replace(arg1);
                replace(arg2);
            }
            Instruction::Branch(block) => {
                block.args.iter_mut().for_each(replace);
            }
            Instruction::Call(_, args) => args.iter_mut().for_each(replace),
            Instruction::CastPtr(val, _ty) => replace(val),
            Instruction::Cmp(_, lhs_val, rhs_val) => {
                replace(lhs_val);
                replace(rhs_val);
            }
            Instruction::ConditionalBranch {
                cond_value,
                true_block,
                false_block,
            } => {
                replace(cond_value);
                true_block.args.iter_mut().for_each(replace);
                false_block.args.iter_mut().for_each(replace);
            }
            Instruction::ContractCall {
                params,
                coins,
                asset_id,
                gas,
                ..
            } => {
                replace(params);
                replace(coins);
                replace(asset_id);
                replace(gas);
            }
            Instruction::GetLocal(_) => (),
            Instruction::GetElemPtr {
                base,
                elem_ptr_ty: _,
                indices,
            } => {
                replace(base);
                indices.iter_mut().for_each(replace);
            }
            Instruction::IntToPtr(value, _) => replace(value),
            Instruction::Load(ptr) => replace(ptr),
            Instruction::MemCopyBytes {
                dst_val_ptr,
                src_val_ptr,
                ..
            } => {
                replace(dst_val_ptr);
                replace(src_val_ptr);
            }
            Instruction::MemCopyVal {
                dst_val_ptr,
                src_val_ptr,
            } => {
                replace(dst_val_ptr);
                replace(src_val_ptr);
            }
            Instruction::Nop => (),
            Instruction::PtrToInt(value, _) => replace(value),
            Instruction::Ret(ret_val, _) => replace(ret_val),
            Instruction::Store {
                stored_val,
                dst_val_ptr,
            } => {
                replace(stored_val);
                replace(dst_val_ptr);
            }

            Instruction::FuelVm(fuel_vm_instr) => match fuel_vm_instr {
                FuelVmInstruction::Gtf { index, .. } => replace(index),
                FuelVmInstruction::Log {
                    log_val, log_id, ..
                } => {
                    replace(log_val);
                    replace(log_id);
                }
                FuelVmInstruction::ReadRegister { .. } => (),
                FuelVmInstruction::Revert(revert_val) => replace(revert_val),
                FuelVmInstruction::Smo {
                    recipient,
                    message,
                    message_size,
                    coins,
                } => {
                    replace(recipient);
                    replace(message);
                    replace(message_size);
                    replace(coins);
                }
                FuelVmInstruction::StateClear {
                    key,
                    number_of_slots,
                } => {
                    replace(key);
                    replace(number_of_slots);
                }
                FuelVmInstruction::StateLoadQuadWord {
                    load_val,
                    key,
                    number_of_slots,
                } => {
                    replace(load_val);
                    replace(key);
                    replace(number_of_slots);
                }
                FuelVmInstruction::StateLoadWord(key) => {
                    replace(key);
                }
                FuelVmInstruction::StateStoreQuadWord {
                    stored_val,
                    key,
                    number_of_slots,
                } => {
                    replace(key);
                    replace(stored_val);
                    replace(number_of_slots);
                }
                FuelVmInstruction::StateStoreWord { stored_val, key } => {
                    replace(key);
                    replace(stored_val);
                }
            },
        }
    }

    pub fn may_have_side_effect(&self) -> bool {
        match self {
            Instruction::AsmBlock(_, _)
            | Instruction::Call(..)
            | Instruction::ContractCall { .. }
            | Instruction::FuelVm(FuelVmInstruction::Log { .. })
            | Instruction::FuelVm(FuelVmInstruction::Smo { .. })
            | Instruction::FuelVm(FuelVmInstruction::StateClear { .. })
            | Instruction::FuelVm(FuelVmInstruction::StateLoadQuadWord { .. })
            | Instruction::FuelVm(FuelVmInstruction::StateStoreQuadWord { .. })
            | Instruction::FuelVm(FuelVmInstruction::StateStoreWord { .. })
            | Instruction::FuelVm(FuelVmInstruction::Revert(..))
            | Instruction::MemCopyBytes { .. }
            | Instruction::MemCopyVal { .. }
            | Instruction::Store { .. }
            | Instruction::Ret(..) => true,

            Instruction::UnaryOp { .. }
            | Instruction::BinaryOp { .. }
            | Instruction::BitCast(..)
            | Instruction::Branch(_)
            | Instruction::CastPtr { .. }
            | Instruction::Cmp(..)
            | Instruction::ConditionalBranch { .. }
            | Instruction::FuelVm(FuelVmInstruction::Gtf { .. })
            | Instruction::FuelVm(FuelVmInstruction::ReadRegister(_))
            | Instruction::FuelVm(FuelVmInstruction::StateLoadWord(_))
            | Instruction::GetElemPtr { .. }
            | Instruction::GetLocal(_)
            | Instruction::IntToPtr(..)
            | Instruction::Load(_)
            | Instruction::Nop
            | Instruction::PtrToInt(..) => false,
        }
    }

    pub fn is_terminator(&self) -> bool {
        matches!(
            self,
            Instruction::Branch(_)
                | Instruction::ConditionalBranch { .. }
                | Instruction::Ret(..)
                | Instruction::FuelVm(FuelVmInstruction::Revert(..))
        )
    }
}

/// Iterate over all [`Instruction`]s in a specific [`Block`].
pub struct InstructionIterator {
    instructions: Vec<generational_arena::Index>,
    next: usize,
    next_back: isize,
}

impl InstructionIterator {
    pub fn new(context: &Context, block: &Block) -> Self {
        // Copy all the current instruction indices, so they may be modified in the context during
        // iteration.
        InstructionIterator {
            instructions: context.blocks[block.0]
                .instructions
                .iter()
                .map(|val| val.0)
                .collect(),
            next: 0,
            next_back: context.blocks[block.0].instructions.len() as isize - 1,
        }
    }
}

impl Iterator for InstructionIterator {
    type Item = Value;

    fn next(&mut self) -> Option<Value> {
        if self.next < self.instructions.len() {
            let idx = self.next;
            self.next += 1;
            Some(Value(self.instructions[idx]))
        } else {
            None
        }
    }
}

impl DoubleEndedIterator for InstructionIterator {
    fn next_back(&mut self) -> Option<Value> {
        if self.next_back >= 0 {
            let idx = self.next_back;
            self.next_back -= 1;
            Some(Value(self.instructions[idx as usize]))
        } else {
            None
        }
    }
}

/// Provide a context for appending new [`Instruction`]s to a [`Block`].
pub struct InstructionInserter<'a, 'eng> {
    context: &'a mut Context<'eng>,
    block: Block,
}

macro_rules! make_instruction {
    ($self: ident, $ctor: expr) => {{
        let instruction_val = Value::new_instruction($self.context, $ctor);
        $self.context.blocks[$self.block.0]
            .instructions
            .push(instruction_val);
        instruction_val
    }};
}

impl<'a, 'eng> InstructionInserter<'a, 'eng> {
    /// Return a new [`InstructionInserter`] context for `block`.
    pub fn new(context: &'a mut Context<'eng>, block: Block) -> InstructionInserter<'a, 'eng> {
        InstructionInserter { context, block }
    }

    //
    // XXX Maybe these should return result, in case they get bad args?
    //

    /// Append a new [`Instruction::AsmBlock`] from `args` and a `body`.
    pub fn asm_block(
        self,
        args: Vec<AsmArg>,
        body: Vec<AsmInstruction>,
        return_type: Type,
        return_name: Option<Ident>,
    ) -> Value {
        let asm = AsmBlock::new(
            self.context,
            args.iter().map(|arg| arg.name.clone()).collect(),
            body,
            return_type,
            return_name,
        );
        self.asm_block_from_asm(asm, args)
    }

    pub fn asm_block_from_asm(self, asm: AsmBlock, args: Vec<AsmArg>) -> Value {
        make_instruction!(self, Instruction::AsmBlock(asm, args))
    }

    pub fn bitcast(self, value: Value, ty: Type) -> Value {
        make_instruction!(self, Instruction::BitCast(value, ty))
    }

    pub fn unary_op(self, op: UnaryOpKind, arg: Value) -> Value {
        make_instruction!(self, Instruction::UnaryOp { op, arg })
    }

    pub fn binary_op(self, op: BinaryOpKind, arg1: Value, arg2: Value) -> Value {
        make_instruction!(self, Instruction::BinaryOp { op, arg1, arg2 })
    }

    pub fn branch(self, to_block: Block, dest_params: Vec<Value>) -> Value {
        let br_val = Value::new_instruction(
            self.context,
            Instruction::Branch(BranchToWithArgs {
                block: to_block,
                args: dest_params,
            }),
        );
        to_block.add_pred(self.context, &self.block);
        self.context.blocks[self.block.0].instructions.push(br_val);
        br_val
    }

    pub fn call(self, function: Function, args: &[Value]) -> Value {
        make_instruction!(self, Instruction::Call(function, args.to_vec()))
    }

    pub fn cast_ptr(self, val: Value, ty: Type) -> Value {
        make_instruction!(self, Instruction::CastPtr(val, ty))
    }

    pub fn cmp(self, pred: Predicate, lhs_value: Value, rhs_value: Value) -> Value {
        make_instruction!(self, Instruction::Cmp(pred, lhs_value, rhs_value))
    }

    pub fn conditional_branch(
        self,
        cond_value: Value,
        true_block: Block,
        false_block: Block,
        true_dest_params: Vec<Value>,
        false_dest_params: Vec<Value>,
    ) -> Value {
        let cbr_val = Value::new_instruction(
            self.context,
            Instruction::ConditionalBranch {
                cond_value,
                true_block: BranchToWithArgs {
                    block: true_block,
                    args: true_dest_params,
                },
                false_block: BranchToWithArgs {
                    block: false_block,
                    args: false_dest_params,
                },
            },
        );
        true_block.add_pred(self.context, &self.block);
        false_block.add_pred(self.context, &self.block);
        self.context.blocks[self.block.0].instructions.push(cbr_val);
        cbr_val
    }

    pub fn contract_call(
        self,
        return_type: Type,
        name: String,
        params: Value,
        coins: Value,    // amount of coins to forward
        asset_id: Value, // b256 asset ID of the coint being forwarded
        gas: Value,      // amount of gas to forward
    ) -> Value {
        make_instruction!(
            self,
            Instruction::ContractCall {
                return_type,
                name,
                params,
                coins,
                asset_id,
                gas,
            }
        )
    }

    pub fn gtf(self, index: Value, tx_field_id: u64) -> Value {
        make_instruction!(
            self,
            Instruction::FuelVm(FuelVmInstruction::Gtf { index, tx_field_id })
        )
    }

    // get_elem_ptr() and get_elem_ptr_*() all take the element type and will store the pointer to
    // that type in the instruction, which is later returned by Instruction::get_type().
    pub fn get_elem_ptr(self, base: Value, elem_ty: Type, indices: Vec<Value>) -> Value {
        let elem_ptr_ty = Type::new_ptr(self.context, elem_ty);
        make_instruction!(
            self,
            Instruction::GetElemPtr {
                base,
                elem_ptr_ty,
                indices
            }
        )
    }

    pub fn get_elem_ptr_with_idx(self, base: Value, elem_ty: Type, index: u64) -> Value {
        let idx_val = Constant::get_uint(self.context, 64, index);
        self.get_elem_ptr(base, elem_ty, vec![idx_val])
    }

    pub fn get_elem_ptr_with_idcs(self, base: Value, elem_ty: Type, indices: &[u64]) -> Value {
        let idx_vals = indices
            .iter()
            .map(|idx| Constant::get_uint(self.context, 64, *idx))
            .collect();
        self.get_elem_ptr(base, elem_ty, idx_vals)
    }

    pub fn get_local(self, local_var: LocalVar) -> Value {
        make_instruction!(self, Instruction::GetLocal(local_var))
    }

    pub fn int_to_ptr(self, value: Value, ty: Type) -> Value {
        make_instruction!(self, Instruction::IntToPtr(value, ty))
    }

    pub fn load(self, src_val: Value) -> Value {
        make_instruction!(self, Instruction::Load(src_val))
    }

    pub fn log(self, log_val: Value, log_ty: Type, log_id: Value) -> Value {
        make_instruction!(
            self,
            Instruction::FuelVm(FuelVmInstruction::Log {
                log_val,
                log_ty,
                log_id
            })
        )
    }

    pub fn mem_copy_bytes(self, dst_val_ptr: Value, src_val_ptr: Value, byte_len: u64) -> Value {
        make_instruction!(
            self,
            Instruction::MemCopyBytes {
                dst_val_ptr,
                src_val_ptr,
                byte_len
            }
        )
    }

    pub fn mem_copy_val(self, dst_val_ptr: Value, src_val_ptr: Value) -> Value {
        make_instruction!(
            self,
            Instruction::MemCopyVal {
                dst_val_ptr,
                src_val_ptr,
            }
        )
    }

    pub fn nop(self) -> Value {
        make_instruction!(self, Instruction::Nop)
    }

    pub fn ptr_to_int(self, value: Value, ty: Type) -> Value {
        make_instruction!(self, Instruction::PtrToInt(value, ty))
    }

    pub fn read_register(self, reg: Register) -> Value {
        make_instruction!(
            self,
            Instruction::FuelVm(FuelVmInstruction::ReadRegister(reg))
        )
    }

    pub fn ret(self, value: Value, ty: Type) -> Value {
        make_instruction!(self, Instruction::Ret(value, ty))
    }

    pub fn revert(self, value: Value) -> Value {
        let revert_val = Value::new_instruction(
            self.context,
            Instruction::FuelVm(FuelVmInstruction::Revert(value)),
        );
        self.context.blocks[self.block.0]
            .instructions
            .push(revert_val);
        revert_val
    }

    pub fn smo(self, recipient: Value, message: Value, message_size: Value, coins: Value) -> Value {
        make_instruction!(
            self,
            Instruction::FuelVm(FuelVmInstruction::Smo {
                recipient,
                message,
                message_size,
                coins,
            })
        )
    }

    pub fn state_clear(self, key: Value, number_of_slots: Value) -> Value {
        make_instruction!(
            self,
            Instruction::FuelVm(FuelVmInstruction::StateClear {
                key,
                number_of_slots
            })
        )
    }

    pub fn state_load_quad_word(
        self,
        load_val: Value,
        key: Value,
        number_of_slots: Value,
    ) -> Value {
        make_instruction!(
            self,
            Instruction::FuelVm(FuelVmInstruction::StateLoadQuadWord {
                load_val,
                key,
                number_of_slots
            })
        )
    }

    pub fn state_load_word(self, key: Value) -> Value {
        make_instruction!(
            self,
            Instruction::FuelVm(FuelVmInstruction::StateLoadWord(key))
        )
    }

    pub fn state_store_quad_word(
        self,
        stored_val: Value,
        key: Value,
        number_of_slots: Value,
    ) -> Value {
        make_instruction!(
            self,
            Instruction::FuelVm(FuelVmInstruction::StateStoreQuadWord {
                stored_val,
                key,
                number_of_slots
            })
        )
    }

    pub fn state_store_word(self, stored_val: Value, key: Value) -> Value {
        make_instruction!(
            self,
            Instruction::FuelVm(FuelVmInstruction::StateStoreWord { stored_val, key })
        )
    }

    pub fn store(self, dst_val_ptr: Value, stored_val: Value) -> Value {
        make_instruction!(
            self,
            Instruction::Store {
                dst_val_ptr,
                stored_val,
            }
        )
    }
}
