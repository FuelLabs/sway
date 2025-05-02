//! Instructions for data manipulation, but mostly control flow.
//!
//! Since Sway abstracts most low level operations behind traits they are translated into function
//! calls which contain ASM blocks.
//!
//! Unfortunately, using opaque ASM blocks limits the effectiveness of certain optimizations and
//! this should be addressed in the future, perhaps by using compiler intrinsic calls instead of
//! the ASM blocks where possible. See: https://github.com/FuelLabs/sway/issues/855,

use rustc_hash::FxHashMap;
use sway_types::Ident;

use crate::{
    asm::{AsmArg, AsmBlock},
    block::Block,
    context::Context,
    function::Function,
    irtype::Type,
    pretty::DebugWithContext,
    value::{Value, ValueDatum},
    variable::LocalVar,
    AsmInstruction, ConstantContent, GlobalVar, Module,
};

#[derive(Debug, Clone, DebugWithContext)]
pub struct BranchToWithArgs {
    pub block: Block,
    pub args: Vec<Value>,
}

#[derive(Debug, Clone, DebugWithContext)]
pub struct Instruction {
    pub parent: Block,
    pub op: InstOp,
}

impl Instruction {
    pub fn get_type(&self, context: &Context) -> Option<Type> {
        self.op.get_type(context)
    }
    /// Replace `old_val` with `new_val` if it is referenced by this instruction's arguments.
    pub fn replace_values(&mut self, replace_map: &FxHashMap<Value, Value>) {
        self.op.replace_values(replace_map)
    }
    /// Get the function containing this instruction
    pub fn get_function(&self, context: &Context) -> Function {
        context.blocks[self.parent.0].function
    }
}

#[derive(Debug, Clone, DebugWithContext)]
pub enum InstOp {
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
        name: Option<String>,
        params: Value,
        coins: Value,
        asset_id: Value,
        gas: Value,
    },
    /// Umbrella instruction variant for FuelVM-specific instructions
    FuelVm(FuelVmInstruction),
    /// Return a local variable.
    GetLocal(LocalVar),
    /// Return a global variable.
    GetGlobal(GlobalVar),
    /// Return a ptr to a config
    GetConfig(Module, String),
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
    /// - Sends a message to an output via the `smo` FuelVM instruction.
    /// - The first operand must be a `B256` representing the recipient.
    /// - The second operand is the message data being sent.
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
    WideUnaryOp {
        op: UnaryOpKind,
        result: Value,
        arg: Value,
    },
    WideBinaryOp {
        op: BinaryOpKind,
        result: Value,
        arg1: Value,
        arg2: Value,
    },
    WideModularOp {
        op: BinaryOpKind,
        result: Value,
        arg1: Value,
        arg2: Value,
        arg3: Value,
    },
    WideCmpOp {
        op: Predicate,
        arg1: Value,
        arg2: Value,
    },
    JmpMem,
    Retd {
        ptr: Value,
        len: Value,
    },
}

/// Comparison operations.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Predicate {
    Equal,
    LessThan,
    GreaterThan,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum UnaryOpKind {
    Not,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
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
#[derive(Debug, Clone, Copy, Hash)]
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

impl InstOp {
    /// Some [`Instruction`]s can return a value, but for some a return value doesn't make sense.
    ///
    /// Those which perform side effects such as writing to memory and also terminators such as
    /// `Ret` do not have a type.
    pub fn get_type(&self, context: &Context) -> Option<Type> {
        match self {
            // These all return something in particular.
            InstOp::AsmBlock(asm_block, _) => Some(asm_block.return_type),
            InstOp::UnaryOp { arg, .. } => arg.get_type(context),
            InstOp::BinaryOp { arg1, .. } => arg1.get_type(context),
            InstOp::BitCast(_, ty) => Some(*ty),
            InstOp::Call(function, _) => Some(context.functions[function.0].return_type),
            InstOp::CastPtr(_val, ty) => Some(*ty),
            InstOp::Cmp(..) => Some(Type::get_bool(context)),
            InstOp::ContractCall { return_type, .. } => Some(*return_type),
            InstOp::FuelVm(FuelVmInstruction::Gtf { .. }) => Some(Type::get_uint64(context)),
            InstOp::FuelVm(FuelVmInstruction::Log { .. }) => Some(Type::get_unit(context)),
            InstOp::FuelVm(FuelVmInstruction::ReadRegister(_)) => Some(Type::get_uint64(context)),
            InstOp::FuelVm(FuelVmInstruction::Smo { .. }) => Some(Type::get_unit(context)),

            // Load needs to strip the pointer from the source type.
            InstOp::Load(ptr_val) => match &context.values[ptr_val.0].value {
                ValueDatum::Argument(arg) => arg.ty.get_pointee_type(context),
                ValueDatum::Constant(cons) => {
                    cons.get_content(context).ty.get_pointee_type(context)
                }
                ValueDatum::Instruction(ins) => ins
                    .get_type(context)
                    .and_then(|ty| ty.get_pointee_type(context)),
            },

            // These return pointer types.
            InstOp::GetElemPtr { elem_ptr_ty, .. } => Some(*elem_ptr_ty),
            InstOp::GetLocal(local_var) => Some(local_var.get_type(context)),
            InstOp::GetGlobal(global_var) => Some(global_var.get_type(context)),
            InstOp::GetConfig(module, name) => Some(match module.get_config(context, name)? {
                crate::ConfigContent::V0 { ptr_ty, .. } => *ptr_ty,
                crate::ConfigContent::V1 { ptr_ty, .. } => *ptr_ty,
            }),

            // Use for casting between pointers and pointer-width integers.
            InstOp::IntToPtr(_, ptr_ty) => Some(*ptr_ty),
            InstOp::PtrToInt(_, int_ty) => Some(*int_ty),

            // These are all terminators which don't return, essentially.  No type.
            InstOp::Branch(_)
            | InstOp::ConditionalBranch { .. }
            | InstOp::FuelVm(
                FuelVmInstruction::Revert(..)
                | FuelVmInstruction::JmpMem
                | FuelVmInstruction::Retd { .. },
            )
            | InstOp::Ret(..) => None,

            // No-op is also no-type.
            InstOp::Nop => None,

            // State load returns a u64, other state ops return a bool.
            InstOp::FuelVm(FuelVmInstruction::StateLoadWord(_)) => Some(Type::get_uint64(context)),
            InstOp::FuelVm(FuelVmInstruction::StateClear { .. })
            | InstOp::FuelVm(FuelVmInstruction::StateLoadQuadWord { .. })
            | InstOp::FuelVm(FuelVmInstruction::StateStoreQuadWord { .. })
            | InstOp::FuelVm(FuelVmInstruction::StateStoreWord { .. }) => {
                Some(Type::get_bool(context))
            }

            // Memory writes return unit.
            InstOp::MemCopyBytes { .. } | InstOp::MemCopyVal { .. } | InstOp::Store { .. } => {
                Some(Type::get_unit(context))
            }

            // Wide Operations
            InstOp::FuelVm(FuelVmInstruction::WideUnaryOp { result, .. }) => {
                result.get_type(context)
            }
            InstOp::FuelVm(FuelVmInstruction::WideBinaryOp { result, .. }) => {
                result.get_type(context)
            }
            InstOp::FuelVm(FuelVmInstruction::WideCmpOp { .. }) => Some(Type::get_bool(context)),
            InstOp::FuelVm(FuelVmInstruction::WideModularOp { result, .. }) => {
                result.get_type(context)
            }
        }
    }

    pub fn get_operands(&self) -> Vec<Value> {
        match self {
            InstOp::AsmBlock(_, args) => args.iter().filter_map(|aa| aa.initializer).collect(),
            InstOp::BitCast(v, _) => vec![*v],
            InstOp::UnaryOp { op: _, arg } => vec![*arg],
            InstOp::BinaryOp { op: _, arg1, arg2 } => vec![*arg1, *arg2],
            InstOp::Branch(BranchToWithArgs { args, .. }) => args.clone(),
            InstOp::Call(_, vs) => vs.clone(),
            InstOp::CastPtr(val, _ty) => vec![*val],
            InstOp::Cmp(_, lhs, rhs) => vec![*lhs, *rhs],
            InstOp::ConditionalBranch {
                cond_value,
                true_block,
                false_block,
            } => {
                let mut v = vec![*cond_value];
                v.extend_from_slice(&true_block.args);
                v.extend_from_slice(&false_block.args);
                v
            }
            InstOp::ContractCall {
                return_type: _,
                name: _,
                params,
                coins,
                asset_id,
                gas,
            } => vec![*params, *coins, *asset_id, *gas],
            InstOp::GetElemPtr {
                base,
                elem_ptr_ty: _,
                indices,
            } => {
                let mut vals = indices.clone();
                vals.push(*base);
                vals
            }
            InstOp::GetLocal(_local_var) => {
                // `GetLocal` returns an SSA `Value` but does not take any as an operand.
                vec![]
            }
            InstOp::GetGlobal(_global_var) => {
                // `GetGlobal` returns an SSA `Value` but does not take any as an operand.
                vec![]
            }
            InstOp::GetConfig(_, _) => {
                // `GetConfig` returns an SSA `Value` but does not take any as an operand.
                vec![]
            }
            InstOp::IntToPtr(v, _) => vec![*v],
            InstOp::Load(v) => vec![*v],
            InstOp::MemCopyBytes {
                dst_val_ptr,
                src_val_ptr,
                byte_len: _,
            } => {
                vec![*dst_val_ptr, *src_val_ptr]
            }
            InstOp::MemCopyVal {
                dst_val_ptr,
                src_val_ptr,
            } => {
                vec![*dst_val_ptr, *src_val_ptr]
            }
            InstOp::Nop => vec![],
            InstOp::PtrToInt(v, _) => vec![*v],
            InstOp::Ret(v, _) => vec![*v],
            InstOp::Store {
                dst_val_ptr,
                stored_val,
            } => {
                vec![*dst_val_ptr, *stored_val]
            }

            InstOp::FuelVm(fuel_vm_instr) => match fuel_vm_instr {
                FuelVmInstruction::Gtf {
                    index,
                    tx_field_id: _,
                } => vec![*index],
                FuelVmInstruction::Log {
                    log_val, log_id, ..
                } => vec![*log_val, *log_id],
                FuelVmInstruction::ReadRegister(_) => vec![],
                FuelVmInstruction::Revert(v) => vec![*v],
                FuelVmInstruction::JmpMem => vec![],
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
                FuelVmInstruction::WideUnaryOp { arg, result, .. } => vec![*result, *arg],
                FuelVmInstruction::WideBinaryOp {
                    arg1, arg2, result, ..
                } => vec![*result, *arg1, *arg2],
                FuelVmInstruction::WideCmpOp { arg1, arg2, .. } => vec![*arg1, *arg2],
                FuelVmInstruction::WideModularOp {
                    result,
                    arg1,
                    arg2,
                    arg3,
                    ..
                } => vec![*result, *arg1, *arg2, *arg3],
                FuelVmInstruction::Retd { ptr, len } => {
                    vec![*ptr, *len]
                }
            },
        }
    }

    /// Set the operand at the given index to the provided value.
    /// The indices are in the same order as returned by `get_operands`.
    pub fn set_operand(&mut self, replacement: Value, idx: usize) {
        match self {
            InstOp::AsmBlock(_, args) => {
                // Because get_operand only returns operands that have an
                // initializer, we also iterate over only those, to match indices.
                let mut cur_idx = 0;
                for arg in args.iter_mut() {
                    if let Some(_asm_arg) = arg.initializer {
                        if cur_idx == idx {
                            arg.initializer = Some(replacement);
                            return;
                        }
                        cur_idx += 1;
                    }
                }
            }
            InstOp::BitCast(v, _) | InstOp::UnaryOp { arg: v, .. } => {
                if idx == 0 {
                    *v = replacement;
                } else {
                    panic!("Invalid index for Op");
                }
            }
            InstOp::BinaryOp { op: _, arg1, arg2 } => {
                if idx == 0 {
                    *arg1 = replacement;
                } else if idx == 1 {
                    *arg2 = replacement;
                } else {
                    panic!("Invalid index for BinaryOp");
                }
            }
            InstOp::Branch(BranchToWithArgs { args, .. }) => {
                if idx < args.len() {
                    args[idx] = replacement;
                } else {
                    panic!("Invalid index for Branch");
                }
            }
            InstOp::Call(_, vs) => {
                if idx < vs.len() {
                    vs[idx] = replacement;
                } else {
                    panic!("Invalid index for Call");
                }
            }
            InstOp::CastPtr(val, _ty) => {
                if idx == 0 {
                    *val = replacement;
                } else {
                    panic!("Invalid index for CastPtr");
                }
            }
            InstOp::Cmp(_, lhs, rhs) => {
                if idx == 0 {
                    *lhs = replacement;
                } else if idx == 1 {
                    *rhs = replacement;
                } else {
                    panic!("Invalid index for Cmp");
                }
            }
            InstOp::ConditionalBranch {
                cond_value,
                true_block,
                false_block,
            } => {
                if idx == 0 {
                    *cond_value = replacement;
                } else if idx - 1 < true_block.args.len() {
                    true_block.args[idx - 1] = replacement;
                } else if idx - 1 - true_block.args.len() < false_block.args.len() {
                    false_block.args[idx - 1 - true_block.args.len()] = replacement;
                } else {
                    panic!("Invalid index for ConditionalBranch");
                }
            }
            InstOp::ContractCall {
                return_type: _,
                name: _,
                params,
                coins,
                asset_id,
                gas,
            } => {
                if idx == 0 {
                    *params = replacement;
                } else if idx == 1 {
                    *coins = replacement;
                } else if idx == 2 {
                    *asset_id = replacement;
                } else if idx == 3 {
                    *gas = replacement;
                } else {
                    panic!("Invalid index for ContractCall");
                }
            }
            InstOp::GetElemPtr {
                base,
                elem_ptr_ty: _,
                indices,
            } => {
                if idx < indices.len() {
                    indices[idx] = replacement;
                } else if idx == indices.len() {
                    *base = replacement;
                } else {
                    panic!("Invalid index for GetElemPtr");
                }
            }
            InstOp::GetLocal(_local_var) => {
                // `GetLocal` returns an SSA `Value` but does not take any as an operand.
                panic!("Invalid index for GetLocal");
            }
            InstOp::GetGlobal(_global_var) => {
                // `GetGlobal` returns an SSA `Value` but does not take any as an operand.
                panic!("Invalid index for GetGlobal");
            }
            InstOp::GetConfig(_, _) => {
                // `GetConfig` returns an SSA `Value` but does not take any as an operand.
                panic!("Invalid index for GetConfig");
            }
            InstOp::IntToPtr(v, _) => {
                if idx == 0 {
                    *v = replacement;
                } else {
                    panic!("Invalid index for IntToPtr");
                }
            }
            InstOp::Load(v) => {
                if idx == 0 {
                    *v = replacement;
                } else {
                    panic!("Invalid index for Load");
                }
            }
            InstOp::MemCopyBytes {
                dst_val_ptr,
                src_val_ptr,
                byte_len: _,
            } => {
                if idx == 0 {
                    *dst_val_ptr = replacement;
                } else if idx == 1 {
                    *src_val_ptr = replacement;
                } else {
                    panic!("Invalid index for MemCopyBytes");
                }
            }
            InstOp::MemCopyVal {
                dst_val_ptr,
                src_val_ptr,
            } => {
                if idx == 0 {
                    *dst_val_ptr = replacement;
                } else if idx == 1 {
                    *src_val_ptr = replacement;
                } else {
                    panic!("Invalid index for MemCopyVal");
                }
            }
            InstOp::Nop => (),
            InstOp::PtrToInt(v, _) => {
                if idx == 0 {
                    *v = replacement;
                } else {
                    panic!("Invalid index for PtrToInt");
                }
            }
            InstOp::Ret(v, _) => {
                if idx == 0 {
                    *v = replacement;
                } else {
                    panic!("Invalid index for Ret");
                }
            }
            InstOp::Store {
                dst_val_ptr,
                stored_val,
            } => {
                if idx == 0 {
                    *dst_val_ptr = replacement;
                } else if idx == 1 {
                    *stored_val = replacement;
                } else {
                    panic!("Invalid index for Store");
                }
            }

            InstOp::FuelVm(fuel_vm_instr) => match fuel_vm_instr {
                FuelVmInstruction::Gtf {
                    index,
                    tx_field_id: _,
                } => {
                    if idx == 0 {
                        *index = replacement;
                    } else {
                        panic!("Invalid index for Gtf");
                    }
                }
                FuelVmInstruction::Log {
                    log_val, log_id, ..
                } => {
                    if idx == 0 {
                        *log_val = replacement;
                    } else if idx == 1 {
                        *log_id = replacement;
                    } else {
                        panic!("Invalid index for Log");
                    }
                }
                FuelVmInstruction::ReadRegister(_) => {
                    // `ReadRegister` returns an SSA `Value` but does not take any as an operand.
                    panic!("Invalid index for ReadRegister");
                }
                FuelVmInstruction::Revert(v) => {
                    if idx == 0 {
                        *v = replacement;
                    } else {
                        panic!("Invalid index for Revert");
                    }
                }
                FuelVmInstruction::JmpMem => {
                    // `JmpMem` does not take any operand.
                    panic!("Invalid index for JmpMem");
                }
                FuelVmInstruction::Smo {
                    recipient,
                    message,
                    message_size,
                    coins,
                } => {
                    if idx == 0 {
                        *recipient = replacement;
                    } else if idx == 1 {
                        *message = replacement;
                    } else if idx == 2 {
                        *message_size = replacement;
                    } else if idx == 3 {
                        *coins = replacement;
                    } else {
                        panic!("Invalid index for Smo");
                    }
                }
                FuelVmInstruction::StateClear {
                    key,
                    number_of_slots,
                } => {
                    if idx == 0 {
                        *key = replacement;
                    } else if idx == 1 {
                        *number_of_slots = replacement;
                    } else {
                        panic!("Invalid index for StateClear");
                    }
                }
                FuelVmInstruction::StateLoadQuadWord {
                    load_val,
                    key,
                    number_of_slots,
                } => {
                    if idx == 0 {
                        *load_val = replacement;
                    } else if idx == 1 {
                        *key = replacement;
                    } else if idx == 2 {
                        *number_of_slots = replacement;
                    } else {
                        panic!("Invalid index for StateLoadQuadWord");
                    }
                }
                FuelVmInstruction::StateLoadWord(key) => {
                    if idx == 0 {
                        *key = replacement;
                    } else {
                        panic!("Invalid index for StateLoadWord");
                    }
                }
                FuelVmInstruction::StateStoreQuadWord {
                    stored_val,
                    key,
                    number_of_slots,
                } => {
                    if idx == 0 {
                        *stored_val = replacement;
                    } else if idx == 1 {
                        *key = replacement;
                    } else if idx == 2 {
                        *number_of_slots = replacement;
                    } else {
                        panic!("Invalid index for StateStoreQuadWord");
                    }
                }
                FuelVmInstruction::StateStoreWord { stored_val, key } => {
                    if idx == 0 {
                        *stored_val = replacement;
                    } else if idx == 1 {
                        *key = replacement;
                    } else {
                        panic!("Invalid index for StateStoreWord");
                    }
                }
                FuelVmInstruction::WideUnaryOp { arg, result, .. } => {
                    if idx == 0 {
                        *result = replacement;
                    } else if idx == 1 {
                        *arg = replacement;
                    } else {
                        panic!("Invalid index for WideUnaryOp");
                    }
                }
                FuelVmInstruction::WideBinaryOp {
                    arg1, arg2, result, ..
                } => {
                    if idx == 0 {
                        *result = replacement;
                    } else if idx == 1 {
                        *arg1 = replacement;
                    } else if idx == 2 {
                        *arg2 = replacement;
                    } else {
                        panic!("Invalid index for WideBinaryOp");
                    }
                }
                FuelVmInstruction::WideCmpOp { arg1, arg2, .. } => {
                    if idx == 0 {
                        *arg1 = replacement;
                    } else if idx == 1 {
                        *arg2 = replacement;
                    } else {
                        panic!("Invalid index for WideCmpOp");
                    }
                }
                FuelVmInstruction::WideModularOp {
                    result,
                    arg1,
                    arg2,
                    arg3,
                    ..
                } => {
                    if idx == 0 {
                        *result = replacement;
                    } else if idx == 1 {
                        *arg1 = replacement;
                    } else if idx == 2 {
                        *arg2 = replacement;
                    } else if idx == 3 {
                        *arg3 = replacement;
                    } else {
                        panic!("Invalid index for WideModularOp");
                    }
                }
                FuelVmInstruction::Retd { ptr, len } => {
                    if idx == 0 {
                        *ptr = replacement;
                    } else if idx == 1 {
                        *len = replacement;
                    } else {
                        panic!("Invalid index for Retd");
                    }
                }
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
            InstOp::AsmBlock(_, args) => args
                .iter_mut()
                .for_each(|asm_arg| asm_arg.initializer.iter_mut().for_each(replace)),
            InstOp::BitCast(value, _) => replace(value),
            InstOp::UnaryOp { op: _, arg } => {
                replace(arg);
            }
            InstOp::BinaryOp { op: _, arg1, arg2 } => {
                replace(arg1);
                replace(arg2);
            }
            InstOp::Branch(block) => {
                block.args.iter_mut().for_each(replace);
            }
            InstOp::Call(_, args) => args.iter_mut().for_each(replace),
            InstOp::CastPtr(val, _ty) => replace(val),
            InstOp::Cmp(_, lhs_val, rhs_val) => {
                replace(lhs_val);
                replace(rhs_val);
            }
            InstOp::ConditionalBranch {
                cond_value,
                true_block,
                false_block,
            } => {
                replace(cond_value);
                true_block.args.iter_mut().for_each(replace);
                false_block.args.iter_mut().for_each(replace);
            }
            InstOp::ContractCall {
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
            InstOp::GetLocal(_) => (),
            InstOp::GetGlobal(_) => (),
            InstOp::GetConfig(_, _) => (),
            InstOp::GetElemPtr {
                base,
                elem_ptr_ty: _,
                indices,
            } => {
                replace(base);
                indices.iter_mut().for_each(replace);
            }
            InstOp::IntToPtr(value, _) => replace(value),
            InstOp::Load(ptr) => replace(ptr),
            InstOp::MemCopyBytes {
                dst_val_ptr,
                src_val_ptr,
                ..
            } => {
                replace(dst_val_ptr);
                replace(src_val_ptr);
            }
            InstOp::MemCopyVal {
                dst_val_ptr,
                src_val_ptr,
            } => {
                replace(dst_val_ptr);
                replace(src_val_ptr);
            }
            InstOp::Nop => (),
            InstOp::PtrToInt(value, _) => replace(value),
            InstOp::Ret(ret_val, _) => replace(ret_val),
            InstOp::Store {
                stored_val,
                dst_val_ptr,
            } => {
                replace(stored_val);
                replace(dst_val_ptr);
            }

            InstOp::FuelVm(fuel_vm_instr) => match fuel_vm_instr {
                FuelVmInstruction::Gtf { index, .. } => replace(index),
                FuelVmInstruction::Log {
                    log_val, log_id, ..
                } => {
                    replace(log_val);
                    replace(log_id);
                }
                FuelVmInstruction::ReadRegister { .. } => (),
                FuelVmInstruction::Revert(revert_val) => replace(revert_val),
                FuelVmInstruction::JmpMem => (),
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
                FuelVmInstruction::WideUnaryOp { arg, result, .. } => {
                    replace(arg);
                    replace(result);
                }
                FuelVmInstruction::WideBinaryOp {
                    arg1, arg2, result, ..
                } => {
                    replace(arg1);
                    replace(arg2);
                    replace(result);
                }
                FuelVmInstruction::WideCmpOp { arg1, arg2, .. } => {
                    replace(arg1);
                    replace(arg2);
                }
                FuelVmInstruction::WideModularOp {
                    result,
                    arg1,
                    arg2,
                    arg3,
                    ..
                } => {
                    replace(result);
                    replace(arg1);
                    replace(arg2);
                    replace(arg3);
                }
                FuelVmInstruction::Retd { ptr, len } => {
                    replace(ptr);
                    replace(len);
                }
            },
        }
    }

    pub fn may_have_side_effect(&self) -> bool {
        match self {
            InstOp::AsmBlock(asm, _) => !asm.body.is_empty(),
            InstOp::Call(..)
            | InstOp::ContractCall { .. }
            | InstOp::FuelVm(FuelVmInstruction::Log { .. })
            | InstOp::FuelVm(FuelVmInstruction::Smo { .. })
            | InstOp::FuelVm(FuelVmInstruction::StateClear { .. })
            | InstOp::FuelVm(FuelVmInstruction::StateLoadQuadWord { .. })
            | InstOp::FuelVm(FuelVmInstruction::StateStoreQuadWord { .. })
            | InstOp::FuelVm(FuelVmInstruction::StateStoreWord { .. })
            | InstOp::FuelVm(FuelVmInstruction::Revert(..))
            | InstOp::FuelVm(FuelVmInstruction::JmpMem)
            | InstOp::FuelVm(FuelVmInstruction::Retd { .. })
            | InstOp::MemCopyBytes { .. }
            | InstOp::MemCopyVal { .. }
            | InstOp::Store { .. }
            | InstOp::Ret(..)
            | InstOp::FuelVm(FuelVmInstruction::WideUnaryOp { .. })
            | InstOp::FuelVm(FuelVmInstruction::WideBinaryOp { .. })
            | InstOp::FuelVm(FuelVmInstruction::WideCmpOp { .. })
            | InstOp::FuelVm(FuelVmInstruction::WideModularOp { .. }) => true,

            InstOp::UnaryOp { .. }
            | InstOp::BinaryOp { .. }
            | InstOp::BitCast(..)
            | InstOp::Branch(_)
            | InstOp::CastPtr { .. }
            | InstOp::Cmp(..)
            | InstOp::ConditionalBranch { .. }
            | InstOp::FuelVm(FuelVmInstruction::Gtf { .. })
            | InstOp::FuelVm(FuelVmInstruction::ReadRegister(_))
            | InstOp::FuelVm(FuelVmInstruction::StateLoadWord(_))
            | InstOp::GetElemPtr { .. }
            | InstOp::GetLocal(_)
            | InstOp::GetGlobal(_)
            | InstOp::GetConfig(_, _)
            | InstOp::IntToPtr(..)
            | InstOp::Load(_)
            | InstOp::Nop
            | InstOp::PtrToInt(..) => false,
        }
    }

    pub fn is_terminator(&self) -> bool {
        matches!(
            self,
            InstOp::Branch(_)
                | InstOp::ConditionalBranch { .. }
                | InstOp::Ret(..)
                | InstOp::FuelVm(
                    FuelVmInstruction::Revert(..)
                        | FuelVmInstruction::JmpMem
                        | FuelVmInstruction::Retd { .. }
                )
        )
    }
}

/// Iterate over all [`Instruction`]s in a specific [`Block`].
pub struct InstructionIterator {
    instructions: Vec<slotmap::DefaultKey>,
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

/// Where to insert new instructions in the block.
pub enum InsertionPosition {
    // Insert at the start of the basic block.
    Start,
    // Insert at the end of the basic block (append).
    End,
    // Insert after instruction.
    After(Value),
    // Insert before instruction.
    Before(Value),
    // Insert at position / index.
    At(usize),
}

/// Provide a context for inserting new [`Instruction`]s to a [`Block`].
pub struct InstructionInserter<'a, 'eng> {
    context: &'a mut Context<'eng>,
    block: Block,
    position: InsertionPosition,
}

macro_rules! insert_instruction {
    ($self: ident, $ctor: expr) => {{
        let instruction_val = Value::new_instruction($self.context, $self.block, $ctor);
        let pos = $self.get_position_index();
        let instructions = &mut $self.context.blocks[$self.block.0].instructions;
        instructions.insert(pos, instruction_val);
        instruction_val
    }};
}

impl<'a, 'eng> InstructionInserter<'a, 'eng> {
    /// Return a new [`InstructionInserter`] context for `block`.
    pub fn new(
        context: &'a mut Context<'eng>,
        block: Block,
        position: InsertionPosition,
    ) -> InstructionInserter<'a, 'eng> {
        InstructionInserter {
            context,
            block,
            position,
        }
    }

    // Recomputes the index in the instruction vec. O(n) in the worst case.
    fn get_position_index(&self) -> usize {
        let instructions = &self.context.blocks[self.block.0].instructions;
        match self.position {
            InsertionPosition::Start => 0,
            InsertionPosition::End => instructions.len(),
            InsertionPosition::After(inst) => {
                instructions
                    .iter()
                    .position(|val| *val == inst)
                    .expect("Provided position for insertion does not exist")
                    + 1
            }
            InsertionPosition::Before(inst) => instructions
                .iter()
                .position(|val| *val == inst)
                .expect("Provided position for insertion does not exist"),
            InsertionPosition::At(pos) => pos,
        }
    }

    // Insert a slice of instructions.
    pub fn insert_slice(&mut self, slice: &[Value]) {
        let pos = self.get_position_index();
        self.context.blocks[self.block.0]
            .instructions
            .splice(pos..pos, slice.iter().cloned());
    }

    // Insert a single instruction.
    pub fn insert(&mut self, inst: Value) {
        let pos = self.get_position_index();
        self.context.blocks[self.block.0]
            .instructions
            .insert(pos, inst);
    }

    //
    // XXX Maybe these should return result, in case they get bad args?
    //

    /// Append a new [InstOp::AsmBlock] from `args` and a `body`.
    pub fn asm_block(
        self,
        args: Vec<AsmArg>,
        body: Vec<AsmInstruction>,
        return_type: Type,
        return_name: Option<Ident>,
    ) -> Value {
        let asm = AsmBlock::new(
            args.iter().map(|arg| arg.name.clone()).collect(),
            body,
            return_type,
            return_name,
        );
        self.asm_block_from_asm(asm, args)
    }

    pub fn asm_block_from_asm(self, asm: AsmBlock, args: Vec<AsmArg>) -> Value {
        insert_instruction!(self, InstOp::AsmBlock(asm, args))
    }

    pub fn bitcast(self, value: Value, ty: Type) -> Value {
        insert_instruction!(self, InstOp::BitCast(value, ty))
    }

    pub fn unary_op(self, op: UnaryOpKind, arg: Value) -> Value {
        insert_instruction!(self, InstOp::UnaryOp { op, arg })
    }

    pub fn wide_unary_op(self, op: UnaryOpKind, arg: Value, result: Value) -> Value {
        insert_instruction!(
            self,
            InstOp::FuelVm(FuelVmInstruction::WideUnaryOp { op, arg, result })
        )
    }

    pub fn wide_binary_op(
        self,
        op: BinaryOpKind,
        arg1: Value,
        arg2: Value,
        result: Value,
    ) -> Value {
        insert_instruction!(
            self,
            InstOp::FuelVm(FuelVmInstruction::WideBinaryOp {
                op,
                arg1,
                arg2,
                result
            })
        )
    }

    pub fn wide_modular_op(
        self,
        op: BinaryOpKind,
        result: Value,
        arg1: Value,
        arg2: Value,
        arg3: Value,
    ) -> Value {
        insert_instruction!(
            self,
            InstOp::FuelVm(FuelVmInstruction::WideModularOp {
                op,
                result,
                arg1,
                arg2,
                arg3,
            })
        )
    }

    pub fn wide_cmp_op(self, op: Predicate, arg1: Value, arg2: Value) -> Value {
        insert_instruction!(
            self,
            InstOp::FuelVm(FuelVmInstruction::WideCmpOp { op, arg1, arg2 })
        )
    }

    pub fn binary_op(self, op: BinaryOpKind, arg1: Value, arg2: Value) -> Value {
        insert_instruction!(self, InstOp::BinaryOp { op, arg1, arg2 })
    }

    pub fn branch(self, to_block: Block, dest_params: Vec<Value>) -> Value {
        let br_val = Value::new_instruction(
            self.context,
            self.block,
            InstOp::Branch(BranchToWithArgs {
                block: to_block,
                args: dest_params,
            }),
        );
        to_block.add_pred(self.context, &self.block);
        self.context.blocks[self.block.0].instructions.push(br_val);
        br_val
    }

    pub fn call(self, function: Function, args: &[Value]) -> Value {
        insert_instruction!(self, InstOp::Call(function, args.to_vec()))
    }

    pub fn cast_ptr(self, val: Value, ty: Type) -> Value {
        insert_instruction!(self, InstOp::CastPtr(val, ty))
    }

    pub fn cmp(self, pred: Predicate, lhs_value: Value, rhs_value: Value) -> Value {
        insert_instruction!(self, InstOp::Cmp(pred, lhs_value, rhs_value))
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
            self.block,
            InstOp::ConditionalBranch {
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
        name: Option<String>,
        params: Value,
        coins: Value,    // amount of coins to forward
        asset_id: Value, // b256 asset ID of the coint being forwarded
        gas: Value,      // amount of gas to forward
    ) -> Value {
        insert_instruction!(
            self,
            InstOp::ContractCall {
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
        insert_instruction!(
            self,
            InstOp::FuelVm(FuelVmInstruction::Gtf { index, tx_field_id })
        )
    }

    // get_elem_ptr() and get_elem_ptr_*() all take the element type and will store the pointer to
    // that type in the instruction, which is later returned by Instruction::get_type().
    pub fn get_elem_ptr(self, base: Value, elem_ty: Type, indices: Vec<Value>) -> Value {
        let elem_ptr_ty = Type::new_ptr(self.context, elem_ty);
        insert_instruction!(
            self,
            InstOp::GetElemPtr {
                base,
                elem_ptr_ty,
                indices
            }
        )
    }

    pub fn get_elem_ptr_with_idx(self, base: Value, elem_ty: Type, index: u64) -> Value {
        let idx_val = ConstantContent::get_uint(self.context, 64, index);
        self.get_elem_ptr(base, elem_ty, vec![idx_val])
    }

    pub fn get_elem_ptr_with_idcs(self, base: Value, elem_ty: Type, indices: &[u64]) -> Value {
        let idx_vals = indices
            .iter()
            .map(|idx| ConstantContent::get_uint(self.context, 64, *idx))
            .collect();
        self.get_elem_ptr(base, elem_ty, idx_vals)
    }

    pub fn get_local(self, local_var: LocalVar) -> Value {
        insert_instruction!(self, InstOp::GetLocal(local_var))
    }

    pub fn get_global(self, global_var: GlobalVar) -> Value {
        insert_instruction!(self, InstOp::GetGlobal(global_var))
    }

    pub fn get_config(self, module: Module, name: String) -> Value {
        insert_instruction!(self, InstOp::GetConfig(module, name))
    }

    pub fn int_to_ptr(self, value: Value, ty: Type) -> Value {
        insert_instruction!(self, InstOp::IntToPtr(value, ty))
    }

    pub fn load(self, src_val: Value) -> Value {
        insert_instruction!(self, InstOp::Load(src_val))
    }

    pub fn log(self, log_val: Value, log_ty: Type, log_id: Value) -> Value {
        insert_instruction!(
            self,
            InstOp::FuelVm(FuelVmInstruction::Log {
                log_val,
                log_ty,
                log_id
            })
        )
    }

    pub fn mem_copy_bytes(self, dst_val_ptr: Value, src_val_ptr: Value, byte_len: u64) -> Value {
        insert_instruction!(
            self,
            InstOp::MemCopyBytes {
                dst_val_ptr,
                src_val_ptr,
                byte_len
            }
        )
    }

    pub fn mem_copy_val(self, dst_val_ptr: Value, src_val_ptr: Value) -> Value {
        insert_instruction!(
            self,
            InstOp::MemCopyVal {
                dst_val_ptr,
                src_val_ptr,
            }
        )
    }

    pub fn nop(self) -> Value {
        insert_instruction!(self, InstOp::Nop)
    }

    pub fn ptr_to_int(self, value: Value, ty: Type) -> Value {
        insert_instruction!(self, InstOp::PtrToInt(value, ty))
    }

    pub fn read_register(self, reg: Register) -> Value {
        insert_instruction!(self, InstOp::FuelVm(FuelVmInstruction::ReadRegister(reg)))
    }

    pub fn ret(self, value: Value, ty: Type) -> Value {
        insert_instruction!(self, InstOp::Ret(value, ty))
    }

    pub fn retd(self, ptr: Value, len: Value) -> Value {
        insert_instruction!(self, InstOp::FuelVm(FuelVmInstruction::Retd { ptr, len }))
    }

    pub fn revert(self, value: Value) -> Value {
        let revert_val = Value::new_instruction(
            self.context,
            self.block,
            InstOp::FuelVm(FuelVmInstruction::Revert(value)),
        );
        self.context.blocks[self.block.0]
            .instructions
            .push(revert_val);
        revert_val
    }

    pub fn jmp_mem(self) -> Value {
        let ldc_exec = Value::new_instruction(
            self.context,
            self.block,
            InstOp::FuelVm(FuelVmInstruction::JmpMem),
        );
        self.context.blocks[self.block.0]
            .instructions
            .push(ldc_exec);
        ldc_exec
    }

    pub fn smo(self, recipient: Value, message: Value, message_size: Value, coins: Value) -> Value {
        insert_instruction!(
            self,
            InstOp::FuelVm(FuelVmInstruction::Smo {
                recipient,
                message,
                message_size,
                coins,
            })
        )
    }

    pub fn state_clear(self, key: Value, number_of_slots: Value) -> Value {
        insert_instruction!(
            self,
            InstOp::FuelVm(FuelVmInstruction::StateClear {
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
        insert_instruction!(
            self,
            InstOp::FuelVm(FuelVmInstruction::StateLoadQuadWord {
                load_val,
                key,
                number_of_slots
            })
        )
    }

    pub fn state_load_word(self, key: Value) -> Value {
        insert_instruction!(self, InstOp::FuelVm(FuelVmInstruction::StateLoadWord(key)))
    }

    pub fn state_store_quad_word(
        self,
        stored_val: Value,
        key: Value,
        number_of_slots: Value,
    ) -> Value {
        insert_instruction!(
            self,
            InstOp::FuelVm(FuelVmInstruction::StateStoreQuadWord {
                stored_val,
                key,
                number_of_slots
            })
        )
    }

    pub fn state_store_word(self, stored_val: Value, key: Value) -> Value {
        insert_instruction!(
            self,
            InstOp::FuelVm(FuelVmInstruction::StateStoreWord { stored_val, key })
        )
    }

    pub fn store(self, dst_val_ptr: Value, stored_val: Value) -> Value {
        insert_instruction!(
            self,
            InstOp::Store {
                dst_val_ptr,
                stored_val,
            }
        )
    }
}
