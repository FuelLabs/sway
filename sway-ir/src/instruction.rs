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
    /// Address of a non-copy (memory) value
    AddrOf(Value),
    /// An opaque list of ASM instructions passed directly to codegen.
    AsmBlock(AsmBlock, Vec<AsmArg>),
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
    /// Temporary! Cast between 'pointers' to reference types.  At this intermediate stage, where
    /// we have removed the old `get_ptr` instruction, we have no way to do the casting and
    /// offsetting it did, which was used almost exclusively by storage accesses.  When we
    /// reintroduce pointers we can make them untyped/opaque which removes the need for casting,
    /// and we can introduce GEP which will allow indexing into aggregates.
    ///
    /// Until then we can replicate the `get_ptr` functionality with `CastPtr`.
    /// The `Value` here should always be a `get_local` and the types must be non-copy.
    CastPtr(Value, Type, u64),
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
    /// Reading a specific element from an array.
    ExtractElement {
        array: Value,
        ty: Type,
        index_val: Value,
    },
    /// Reading a specific field from (nested) structs.
    ExtractValue {
        aggregate: Value,
        ty: Type,
        indices: Vec<u64>,
    },
    /// Umbrella instruction variant for FuelVM-specific instructions
    FuelVm(FuelVmInstruction),
    /// Return a local variable.
    GetLocal(LocalVar),
    /// Writing a specific value to an array.
    InsertElement {
        array: Value,
        ty: Type,
        value: Value,
        index_val: Value,
    },
    /// Writing a specific value to a (nested) struct field.
    InsertValue {
        aggregate: Value,
        ty: Type,
        value: Value,
        indices: Vec<u64>,
    },
    /// Re-interpret an integer value as pointer of some type
    IntToPtr(Value, Type),
    /// Read a value from a memory pointer.
    Load(Value),
    /// Copy a specified number of bytes between pointers.
    MemCopy {
        dst_val: Value,
        src_val: Value,
        byte_len: u64,
    },
    /// No-op, handy as a placeholder instruction.
    Nop,
    /// Return from a function.
    Ret(Value, Type),
    /// Write a value to a memory pointer.
    Store { dst_val: Value, stored_val: Value },
}

#[derive(Debug, Clone, DebugWithContext)]
pub enum FuelVmInstruction {
    /// Generate a unique integer value
    GetStorageKey,
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
    /// a struct with the first field being a `B256` representing the recipient. The rest of the
    /// struct is the message data being sent.
    /// - Assumes the existence of an `OutputMessage` at `output_index`
    /// - `message_size`, `output_index`, and `coins` must be of type `U64`.
    Smo {
        recipient_and_message: Value,
        message_size: Value,
        output_index: Value,
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

#[derive(Debug, Clone, Copy)]
pub enum Predicate {
    /// Equivalence.
    Equal,
    // More soon.  NotEqual, LessThan, LessThanOrEqual, GreaterThan, GreaterThanOrEqual.
}

#[derive(Debug, Clone, Copy)]
pub enum BinaryOpKind {
    Add,
    Sub,
    Mul,
    Div,
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
            Instruction::AddrOf(_) => Some(Type::get_uint64(context)),
            Instruction::AsmBlock(asm_block, _) => Some(asm_block.get_type(context)),
            Instruction::BinaryOp { arg1, .. } => arg1.get_type(context),
            Instruction::BitCast(_, ty) => Some(*ty),
            Instruction::Call(function, _) => Some(context.functions[function.0].return_type),
            Instruction::CastPtr(_val, ty, _offs) => Some(*ty),
            Instruction::Cmp(..) => Some(Type::get_bool(context)),
            Instruction::ContractCall { return_type, .. } => Some(*return_type),
            Instruction::ExtractElement { ty, .. } => ty.get_array_elem_type(context),
            Instruction::ExtractValue { ty, indices, .. } => ty.get_indexed_type(context, indices),
            Instruction::FuelVm(FuelVmInstruction::GetStorageKey) => Some(Type::get_b256(context)),
            Instruction::FuelVm(FuelVmInstruction::Gtf { .. }) => Some(Type::get_uint64(context)),
            Instruction::FuelVm(FuelVmInstruction::Log { .. }) => Some(Type::get_unit(context)),
            Instruction::FuelVm(FuelVmInstruction::ReadRegister(_)) => {
                Some(Type::get_uint64(context))
            }
            Instruction::FuelVm(FuelVmInstruction::StateLoadWord(_)) => {
                Some(Type::get_uint64(context))
            }
            Instruction::InsertElement { array, .. } => array.get_type(context),
            Instruction::InsertValue { aggregate, .. } => aggregate.get_type(context),
            Instruction::Load(ptr_val) => match &context.values[ptr_val.0].value {
                ValueDatum::Argument(arg) => Some(arg.ty),
                ValueDatum::Configurable(conf) => Some(conf.ty),
                ValueDatum::Constant(cons) => Some(cons.ty),
                ValueDatum::Instruction(ins) => ins.get_type(context),
            },

            // These can be recursed to via Load, so we return the pointer type.
            Instruction::GetLocal(local_var) => Some(local_var.get_type(context)),

            // Used to re-interpret an integer as a pointer to some type so return the pointer type.
            Instruction::IntToPtr(_, ty) => Some(*ty),

            // These are all terminators which don't return, essentially.  No type.
            Instruction::Branch(_) => None,
            Instruction::ConditionalBranch { .. } => None,
            Instruction::FuelVm(FuelVmInstruction::Revert(..)) => None,
            Instruction::Ret(..) => None,

            Instruction::FuelVm(FuelVmInstruction::Smo { .. }) => Some(Type::get_unit(context)),
            Instruction::FuelVm(FuelVmInstruction::StateClear { .. }) => {
                Some(Type::get_bool(context))
            }
            Instruction::FuelVm(FuelVmInstruction::StateLoadQuadWord { .. }) => {
                Some(Type::get_bool(context))
            }
            Instruction::FuelVm(FuelVmInstruction::StateStoreQuadWord { .. }) => {
                Some(Type::get_bool(context))
            }
            Instruction::FuelVm(FuelVmInstruction::StateStoreWord { .. }) => {
                Some(Type::get_bool(context))
            }
            Instruction::MemCopy { .. } => Some(Type::get_unit(context)),
            Instruction::Store { .. } => Some(Type::get_unit(context)),

            // No-op is also no-type.
            Instruction::Nop => None,
        }
    }

    /// Some [`Instruction`]s may have struct arguments.  Return it if so for this instruction.
    pub fn get_aggregate(&self, context: &Context) -> Option<Type> {
        let ty = match self {
            Instruction::Call(func, _args) => Some(context.functions[func.0].return_type),
            Instruction::GetLocal(local_var) => Some(local_var.get_type(context)),
            Instruction::ExtractElement { ty, .. } => ty.get_array_elem_type(context),
            Instruction::ExtractValue { ty, indices, .. } =>
            // This array is a field in a struct or element in an array.
            {
                ty.get_indexed_type(context, indices)
            }
            // Unknown aggregate instruction.  Adding these as we come across them...
            _otherwise => None,
        };
        ty.filter(|ty| ty.is_array(context) || ty.is_struct(context))
    }

    pub fn get_operands(&self) -> Vec<Value> {
        match self {
            Instruction::AddrOf(v) => vec![*v],
            Instruction::AsmBlock(_, args) => args.iter().filter_map(|aa| aa.initializer).collect(),
            Instruction::BitCast(v, _) => vec![*v],
            Instruction::BinaryOp { op: _, arg1, arg2 } => vec![*arg1, *arg2],
            Instruction::Branch(BranchToWithArgs { args, .. }) => args.clone(),
            Instruction::Call(_, vs) => vs.clone(),
            Instruction::CastPtr(val, _ty, _offs) => vec![*val],
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
            Instruction::MemCopy {
                dst_val,
                src_val,
                byte_len: _,
            } => {
                vec![*dst_val, *src_val]
            }
            Instruction::ContractCall {
                return_type: _,
                name: _,
                params,
                coins,
                asset_id,
                gas,
            } => vec![*params, *coins, *asset_id, *gas],
            Instruction::ExtractElement {
                array,
                ty: _,
                index_val,
            } => vec![*array, *index_val],
            Instruction::ExtractValue {
                aggregate,
                ty: _,
                indices: _,
            } => vec![*aggregate],
            Instruction::FuelVm(fuel_vm_instr) => match fuel_vm_instr {
                FuelVmInstruction::GetStorageKey => vec![],
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
                    recipient_and_message,
                    message_size,
                    output_index,
                    coins,
                } => vec![*recipient_and_message, *message_size, *output_index, *coins],
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
            Instruction::GetLocal(_local_var) => {
                // TODO: Not sure.
                vec![]
            }
            Instruction::InsertElement {
                array,
                ty: _,
                value,
                index_val,
            } => vec![*array, *value, *index_val],
            Instruction::InsertValue {
                aggregate,
                ty: _,
                value,
                indices: _,
            } => vec![*aggregate, *value],
            Instruction::IntToPtr(v, _) => vec![*v],
            Instruction::Load(v) => vec![*v],
            Instruction::Nop => vec![],
            Instruction::Ret(v, _) => vec![*v],
            Instruction::Store {
                dst_val,
                stored_val,
            } => {
                vec![*dst_val, *stored_val]
            }
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
            Instruction::AddrOf(arg) => replace(arg),
            Instruction::AsmBlock(_, args) => args.iter_mut().for_each(|asm_arg| {
                asm_arg
                    .initializer
                    .iter_mut()
                    .for_each(|init_val| replace(init_val))
            }),
            Instruction::BitCast(value, _) => replace(value),
            Instruction::BinaryOp { op: _, arg1, arg2 } => {
                replace(arg1);
                replace(arg2);
            }
            Instruction::Branch(block) => {
                block.args.iter_mut().for_each(replace);
            }
            Instruction::Call(_, args) => args.iter_mut().for_each(replace),
            Instruction::CastPtr(val, _ty, _offs) => replace(val),
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
            Instruction::InsertElement {
                array,
                value,
                index_val,
                ..
            } => {
                replace(array);
                replace(value);
                replace(index_val);
            }
            Instruction::InsertValue {
                aggregate, value, ..
            } => {
                replace(aggregate);
                replace(value);
            }
            Instruction::ExtractElement {
                array, index_val, ..
            } => {
                replace(array);
                replace(index_val);
            }
            Instruction::ExtractValue { aggregate, .. } => replace(aggregate),
            Instruction::FuelVm(fuel_vm_instr) => match fuel_vm_instr {
                FuelVmInstruction::GetStorageKey => (),
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
                    recipient_and_message,
                    message_size,
                    output_index,
                    coins,
                } => {
                    replace(recipient_and_message);
                    replace(message_size);
                    replace(output_index);
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
            Instruction::IntToPtr(value, _) => replace(value),
            Instruction::Load(_) => (),
            Instruction::MemCopy {
                dst_val, src_val, ..
            } => {
                replace(dst_val);
                replace(src_val);
            }
            Instruction::Nop => (),
            Instruction::Ret(ret_val, _) => replace(ret_val),
            Instruction::Store { stored_val, .. } => {
                replace(stored_val);
            }
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
                | Instruction::MemCopy { .. }
                | Instruction::Store { .. }
                // Insert(Element/Value), unlike those in LLVM
                // do not have SSA semantics. They are like stores.
                | Instruction::InsertElement { .. }
                | Instruction::InsertValue { .. } => true,
                | Instruction::AddrOf(_)
                | Instruction::BitCast(..)
                | Instruction::BinaryOp { .. }
                    | Instruction::CastPtr(..)
                | Instruction::Cmp(..)
                | Instruction::ExtractElement {  .. }
                | Instruction::ExtractValue { .. }
                | Instruction::FuelVm(FuelVmInstruction::GetStorageKey)
                | Instruction::FuelVm(FuelVmInstruction::Gtf { .. })
                | Instruction::FuelVm(FuelVmInstruction::ReadRegister(_))
                | Instruction::FuelVm(FuelVmInstruction::Revert(..))
                | Instruction::FuelVm(FuelVmInstruction::StateLoadWord(_))
                | Instruction::Load(_)
                | Instruction::GetLocal(_)
                | Instruction::IntToPtr(..)
                | Instruction::Branch(_)
                | Instruction::ConditionalBranch { .. }
                | Instruction::Ret(..)
                | Instruction::Nop => false,
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
pub struct InstructionInserter<'a> {
    context: &'a mut Context,
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

impl<'a> InstructionInserter<'a> {
    /// Return a new [`InstructionInserter`] context for `block`.
    pub fn new(context: &'a mut Context, block: Block) -> InstructionInserter<'a> {
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

    pub fn addr_of(self, value: Value) -> Value {
        make_instruction!(self, Instruction::AddrOf(value))
    }

    pub fn bitcast(self, value: Value, ty: Type) -> Value {
        make_instruction!(self, Instruction::BitCast(value, ty))
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

    pub fn cast_ptr(self, val: Value, ty: Type, offs: u64) -> Value {
        make_instruction!(self, Instruction::CastPtr(val, ty, offs))
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

    pub fn extract_element(self, array: Value, ty: Type, index_val: Value) -> Value {
        make_instruction!(
            self,
            Instruction::ExtractElement {
                array,
                ty,
                index_val,
            }
        )
    }

    pub fn extract_value(self, aggregate: Value, ty: Type, indices: Vec<u64>) -> Value {
        make_instruction!(
            self,
            Instruction::ExtractValue {
                aggregate,
                ty,
                indices,
            }
        )
    }

    pub fn get_storage_key(self) -> Value {
        make_instruction!(self, Instruction::FuelVm(FuelVmInstruction::GetStorageKey))
    }

    pub fn gtf(self, index: Value, tx_field_id: u64) -> Value {
        make_instruction!(
            self,
            Instruction::FuelVm(FuelVmInstruction::Gtf { index, tx_field_id })
        )
    }

    pub fn get_local(self, local_var: LocalVar) -> Value {
        make_instruction!(self, Instruction::GetLocal(local_var))
    }

    pub fn insert_element(self, array: Value, ty: Type, value: Value, index_val: Value) -> Value {
        make_instruction!(
            self,
            Instruction::InsertElement {
                array,
                ty,
                value,
                index_val,
            }
        )
    }

    pub fn insert_value(
        self,
        aggregate: Value,
        ty: Type,
        value: Value,
        indices: Vec<u64>,
    ) -> Value {
        make_instruction!(
            self,
            Instruction::InsertValue {
                aggregate,
                ty,
                value,
                indices,
            }
        )
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

    pub fn mem_copy(self, dst_val: Value, src_val: Value, byte_len: u64) -> Value {
        make_instruction!(
            self,
            Instruction::MemCopy {
                dst_val,
                src_val,
                byte_len
            }
        )
    }

    pub fn nop(self) -> Value {
        make_instruction!(self, Instruction::Nop)
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

    pub fn smo(
        self,
        recipient_and_message: Value,
        message_size: Value,
        output_index: Value,
        coins: Value,
    ) -> Value {
        make_instruction!(
            self,
            Instruction::FuelVm(FuelVmInstruction::Smo {
                recipient_and_message,
                message_size,
                output_index,
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

    pub fn store(self, dst_val: Value, stored_val: Value) -> Value {
        make_instruction!(
            self,
            Instruction::Store {
                dst_val,
                stored_val,
            }
        )
    }
}
