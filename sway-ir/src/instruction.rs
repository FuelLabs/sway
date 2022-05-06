//! Instructions for data manipulation, but mostly control flow.
//!
//! Since Sway abstracts most low level operations behind traits they are translated into function
//! calls which contain ASM blocks.  Therefore _at this stage_ Sway-IR doesn't need low level
//! operations such as binary arithmetic and logic operators.
//!
//! Unfortuntely, using opaque ASM blocks limits the effectiveness of certain optimizations and
//! this should be addressed in the future, perhaps by using compiler intrinsic calls instead of
//! the ASM blocks where possible.

use sway_types::ident::Ident;

use crate::{
    asm::{AsmArg, AsmBlock, AsmInstruction},
    block::Block,
    context::Context,
    function::Function,
    irtype::{Aggregate, Type},
    metadata::MetadataIndex,
    pointer::Pointer,
    value::{Value, ValueDatum},
};

#[derive(Debug, Clone)]
pub enum Instruction {
    /// An opaque list of ASM instructions passed directly to codegen.
    AsmBlock(AsmBlock, Vec<AsmArg>),
    /// Cast the type of a value without changing its actual content.
    BitCast(Value, Type),
    /// An unconditional jump.
    Branch(Block),
    /// A function call with a list of arguments.
    Call(Function, Vec<Value>),
    /// Comparison between two values using various comparators and returning a boolean.
    Cmp(Predicate, Value, Value),
    /// A conditional jump with the boolean condition value and true or false destinations.
    ConditionalBranch {
        cond_value: Value,
        true_block: Block,
        false_block: Block,
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
        ty: Aggregate,
        index_val: Value,
    },
    /// Reading a specific field from (nested) structs.
    ExtractValue {
        aggregate: Value,
        ty: Aggregate,
        indices: Vec<u64>,
    },
    /// Return a pointer as a value.
    GetPointer {
        base_ptr: Pointer,
        ptr_ty: Type,
        offset: u64,
    },
    /// Writing a specific value to an array.
    InsertElement {
        array: Value,
        ty: Aggregate,
        value: Value,
        index_val: Value,
    },
    /// Writing a specific value to a (nested) struct field.
    InsertValue {
        aggregate: Value,
        ty: Aggregate,
        value: Value,
        indices: Vec<u64>,
    },
    /// Read a value from a memory pointer.
    Load(Value),
    /// No-op, handy as a placeholder instruction.
    Nop,
    /// Choose a value from a list depending on the preceding block.
    Phi(Vec<(Block, Value)>),
    /// Reads a special register in the VM.
    ReadRegister(Register),
    /// Return from a function.
    Ret(Value, Type),
    /// Read a quad word from a storage slot. Type of `load_val` must be a B256 ptr.
    StateLoadQuadWord { load_val: Value, key: Value },
    /// Read a single word from a storage slot.
    StateLoadWord(Value),
    /// Write a value to a storage slot.  Key must be a B256, type of `stored_val` must be a
    /// Uint(256) ptr.
    StateStoreQuadWord { stored_val: Value, key: Value },
    /// Write a value to a storage slot.  Key must be a B256, type of `stored_val` must be a
    /// Uint(64) value.
    StateStoreWord { stored_val: Value, key: Value },
    /// Write a value to a memory pointer.
    Store { dst_val: Value, stored_val: Value },
}

#[derive(Debug, Clone, Copy)]
pub enum Predicate {
    /// Equivalence.
    Equal,
    // More soon.  NotEqual, LessThan, LessThanOrEqual, GreaterThan, GreaterThanOrEqual.
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
            Instruction::AsmBlock(asm_block, _) => asm_block.get_type(context),
            Instruction::BitCast(_, ty) => Some(*ty),
            Instruction::Call(function, _) => Some(context.functions[function.0].return_type),
            Instruction::Cmp(..) => Some(Type::Bool),
            Instruction::ContractCall { return_type, .. } => Some(*return_type),
            Instruction::ExtractElement { ty, .. } => ty.get_elem_type(context),
            Instruction::ExtractValue { ty, indices, .. } => ty.get_field_type(context, indices),
            Instruction::InsertElement { array, .. } => array.get_type(context),
            Instruction::InsertValue { aggregate, .. } => aggregate.get_type(context),
            Instruction::Load(ptr_val) => {
                if let ValueDatum::Instruction(ins) = &context.values[ptr_val.0].value {
                    ins.get_type(context)
                } else {
                    None
                }
            }
            Instruction::ReadRegister(_) => Some(Type::Uint(64)),
            Instruction::StateLoadWord(_) => Some(Type::Uint(64)),
            Instruction::Phi(alts) => {
                // Assuming each alt has the same type, we can take the first one. Note: `verify()`
                // confirms the types are all the same.
                alts.get(0).and_then(|(_, val)| val.get_type(context))
            }

            // These can be recursed to via Load, so we return the pointer type.
            Instruction::GetPointer { ptr_ty, .. } => Some(*ptr_ty),

            // These are all terminators which don't return, essentially.  No type.
            Instruction::Branch(_) => None,
            Instruction::ConditionalBranch { .. } => None,
            Instruction::Ret(..) => None,

            // These write values but don't return one.  If we're explicit we could return Unit.
            Instruction::StateLoadQuadWord { .. } => None,
            Instruction::StateStoreQuadWord { .. } => None,
            Instruction::StateStoreWord { .. } => None,
            Instruction::Store { .. } => None,

            // No-op is also no-type.
            Instruction::Nop => None,
        }
    }

    /// Some [`Instruction`]s may have struct arguments.  Return it if so for this instruction.
    pub fn get_aggregate(&self, context: &Context) -> Option<Aggregate> {
        match self {
            Instruction::GetPointer { ptr_ty, .. } => match ptr_ty {
                Type::Array(aggregate) => Some(*aggregate),
                Type::Struct(aggregate) => Some(*aggregate),
                _otherwise => None,
            },
            Instruction::ExtractElement { ty, .. } => {
                ty.get_elem_type(context).and_then(|ty| match ty {
                    Type::Array(nested_aggregate) => Some(nested_aggregate),
                    Type::Struct(nested_aggregate) => Some(nested_aggregate),
                    _otherwise => None,
                })
            }
            Instruction::ExtractValue { ty, indices, .. } => {
                // This array is a field in a struct or element in an array.
                ty.get_field_type(context, indices).and_then(|ty| match ty {
                    Type::Array(nested_aggregate) => Some(nested_aggregate),
                    Type::Struct(nested_aggregate) => Some(nested_aggregate),
                    _otherwise => None,
                })
            }

            // Unknown aggregate instruction.  Adding these as we come across them...
            _otherwise => None,
        }
    }

    /// Replace `old_val` with `new_val` if it is referenced by this instruction's arguments.
    pub fn replace_value(&mut self, old_val: Value, new_val: Value) {
        let replace = |val: &mut Value| {
            if val == &old_val {
                *val = new_val
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
            Instruction::Branch(_) => (),
            Instruction::Call(_, args) => args.iter_mut().for_each(replace),
            Instruction::Cmp(_, lhs_val, rhs_val) => {
                replace(lhs_val);
                replace(rhs_val);
            }
            Instruction::ConditionalBranch { cond_value, .. } => replace(cond_value),
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
            Instruction::GetPointer { .. } => (),
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
            Instruction::Load(_) => (),
            Instruction::Nop => (),
            Instruction::Phi(pairs) => pairs.iter_mut().for_each(|(_, val)| replace(val)),
            Instruction::ReadRegister { .. } => (),
            Instruction::Ret(ret_val, _) => replace(ret_val),
            Instruction::StateLoadQuadWord { load_val, key } => {
                replace(load_val);
                replace(key);
            }
            Instruction::StateLoadWord(key) => {
                replace(key);
            }
            Instruction::StateStoreQuadWord { stored_val, key } => {
                replace(key);
                replace(stored_val);
            }
            Instruction::StateStoreWord { stored_val, key } => {
                replace(key);
                replace(stored_val);
            }
            Instruction::Store { stored_val, .. } => {
                replace(stored_val);
            }
        }
    }
}

/// Iterate over all [`Instruction`]s in a specific [`Block`].
pub struct InstructionIterator {
    instructions: Vec<generational_arena::Index>,
    next: usize,
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

/// Provide a context for appending new [`Instruction`]s to a [`Block`].
pub struct InstructionInserter<'a> {
    context: &'a mut Context,
    block: Block,
}

impl<'a> InstructionInserter<'a> {
    /// Return a new [`InstructionInserter`] context for `block`.
    pub fn new(context: &'a mut Context, block: Block) -> InstructionInserter<'a> {
        InstructionInserter { context, block }
    }

    //
    // XXX Maybe these should return result, in case they get bad args?
    //
    // XXX Also, these are all the same and could probably be created with a local macro.
    //

    /// Append a new [`Instruction::AsmBlock`] from `args` and a `body`.
    pub fn asm_block(
        self,
        args: Vec<AsmArg>,
        body: Vec<AsmInstruction>,
        return_type: Type,
        return_name: Option<Ident>,
        span_md_idx: Option<MetadataIndex>,
    ) -> Value {
        let asm = AsmBlock::new(
            self.context,
            args.iter().map(|arg| arg.name.clone()).collect(),
            body,
            return_type,
            return_name,
        );
        self.asm_block_from_asm(asm, args, span_md_idx)
    }

    pub fn asm_block_from_asm(
        self,
        asm: AsmBlock,
        args: Vec<AsmArg>,
        span_md_idx: Option<MetadataIndex>,
    ) -> Value {
        let asm_val =
            Value::new_instruction(self.context, Instruction::AsmBlock(asm, args), span_md_idx);
        self.context.blocks[self.block.0].instructions.push(asm_val);
        asm_val
    }

    pub fn bitcast(self, value: Value, ty: Type, span_md_idx: Option<MetadataIndex>) -> Value {
        let bitcast_val =
            Value::new_instruction(self.context, Instruction::BitCast(value, ty), span_md_idx);
        self.context.blocks[self.block.0]
            .instructions
            .push(bitcast_val);
        bitcast_val
    }

    pub fn branch(
        self,
        to_block: Block,
        phi_value: Option<Value>,
        span_md_idx: Option<MetadataIndex>,
    ) -> Value {
        let br_val =
            Value::new_instruction(self.context, Instruction::Branch(to_block), span_md_idx);
        phi_value
            .into_iter()
            .for_each(|pv| to_block.add_phi(self.context, self.block, pv));
        self.context.blocks[self.block.0].instructions.push(br_val);
        br_val
    }

    pub fn call(
        self,
        function: Function,
        args: &[Value],
        span_md_idx: Option<MetadataIndex>,
    ) -> Value {
        let call_val = Value::new_instruction(
            self.context,
            Instruction::Call(function, args.to_vec()),
            span_md_idx,
        );
        self.context.blocks[self.block.0]
            .instructions
            .push(call_val);
        call_val
    }

    pub fn cmp(
        self,
        pred: Predicate,
        lhs_value: Value,
        rhs_value: Value,
        span_md_idx: Option<MetadataIndex>,
    ) -> Value {
        let cmp_val = Value::new_instruction(
            self.context,
            Instruction::Cmp(pred, lhs_value, rhs_value),
            span_md_idx,
        );
        self.context.blocks[self.block.0].instructions.push(cmp_val);
        cmp_val
    }

    pub fn conditional_branch(
        self,
        cond_value: Value,
        true_block: Block,
        false_block: Block,
        phi_value: Option<Value>,
        span_md_idx: Option<MetadataIndex>,
    ) -> Value {
        let cbr_val = Value::new_instruction(
            self.context,
            Instruction::ConditionalBranch {
                cond_value,
                true_block,
                false_block,
            },
            span_md_idx,
        );
        phi_value.into_iter().for_each(|pv| {
            true_block.add_phi(self.context, self.block, pv);
            false_block.add_phi(self.context, self.block, pv);
        });
        self.context.blocks[self.block.0].instructions.push(cbr_val);
        cbr_val
    }

    #[allow(clippy::too_many_arguments)]
    pub fn contract_call(
        self,
        return_type: Type,
        name: String,
        params: Value,
        coins: Value,    // amount of coins to forward
        asset_id: Value, // b256 asset ID of the coint being forwarded
        gas: Value,      // amount of gas to forward
        span_md_idx: Option<MetadataIndex>,
    ) -> Value {
        let contract_call_val = Value::new_instruction(
            self.context,
            Instruction::ContractCall {
                return_type,
                name,
                params,
                coins,
                asset_id,
                gas,
            },
            span_md_idx,
        );
        self.context.blocks[self.block.0]
            .instructions
            .push(contract_call_val);
        contract_call_val
    }

    pub fn extract_element(
        self,
        array: Value,
        ty: Aggregate,
        index_val: Value,
        span_md_idx: Option<MetadataIndex>,
    ) -> Value {
        let extract_element_val = Value::new_instruction(
            self.context,
            Instruction::ExtractElement {
                array,
                ty,
                index_val,
            },
            span_md_idx,
        );
        self.context.blocks[self.block.0]
            .instructions
            .push(extract_element_val);
        extract_element_val
    }

    pub fn extract_value(
        self,
        aggregate: Value,
        ty: Aggregate,
        indices: Vec<u64>,
        span_md_idx: Option<MetadataIndex>,
    ) -> Value {
        let extract_value_val = Value::new_instruction(
            self.context,
            Instruction::ExtractValue {
                aggregate,
                ty,
                indices,
            },
            span_md_idx,
        );
        self.context.blocks[self.block.0]
            .instructions
            .push(extract_value_val);
        extract_value_val
    }

    pub fn get_ptr(
        self,
        base_ptr: Pointer,
        ptr_ty: Type,
        offset: u64,
        span_md_idx: Option<MetadataIndex>,
    ) -> Value {
        let get_ptr_val = Value::new_instruction(
            self.context,
            Instruction::GetPointer {
                base_ptr,
                ptr_ty,
                offset,
            },
            span_md_idx,
        );
        self.context.blocks[self.block.0]
            .instructions
            .push(get_ptr_val);
        get_ptr_val
    }

    pub fn insert_element(
        self,
        array: Value,
        ty: Aggregate,
        value: Value,
        index_val: Value,
        span_md_idx: Option<MetadataIndex>,
    ) -> Value {
        let insert_val = Value::new_instruction(
            self.context,
            Instruction::InsertElement {
                array,
                ty,
                value,
                index_val,
            },
            span_md_idx,
        );
        self.context.blocks[self.block.0]
            .instructions
            .push(insert_val);
        insert_val
    }

    pub fn insert_value(
        self,
        aggregate: Value,
        ty: Aggregate,
        value: Value,
        indices: Vec<u64>,
        span_md_idx: Option<MetadataIndex>,
    ) -> Value {
        let insert_val = Value::new_instruction(
            self.context,
            Instruction::InsertValue {
                aggregate,
                ty,
                value,
                indices,
            },
            span_md_idx,
        );
        self.context.blocks[self.block.0]
            .instructions
            .push(insert_val);
        insert_val
    }

    pub fn load(self, src_val: Value, span_md_idx: Option<MetadataIndex>) -> Value {
        let load_val =
            Value::new_instruction(self.context, Instruction::Load(src_val), span_md_idx);
        self.context.blocks[self.block.0]
            .instructions
            .push(load_val);
        load_val
    }

    pub fn nop(self) -> Value {
        let nop_val = Value::new_instruction(self.context, Instruction::Nop, None);
        self.context.blocks[self.block.0].instructions.push(nop_val);
        nop_val
    }

    pub fn read_register(self, reg: Register, span_md_idx: Option<MetadataIndex>) -> Value {
        let read_register_val =
            Value::new_instruction(self.context, Instruction::ReadRegister(reg), span_md_idx);
        self.context.blocks[self.block.0]
            .instructions
            .push(read_register_val);
        read_register_val
    }

    pub fn ret(self, value: Value, ty: Type, span_md_idx: Option<MetadataIndex>) -> Value {
        let ret_val =
            Value::new_instruction(self.context, Instruction::Ret(value, ty), span_md_idx);
        self.context.blocks[self.block.0].instructions.push(ret_val);
        ret_val
    }

    pub fn state_load_quad_word(
        self,
        load_val: Value,
        key: Value,
        span_md_idx: Option<MetadataIndex>,
    ) -> Value {
        let state_load_val = Value::new_instruction(
            self.context,
            Instruction::StateLoadQuadWord { load_val, key },
            span_md_idx,
        );
        self.context.blocks[self.block.0]
            .instructions
            .push(state_load_val);
        state_load_val
    }

    pub fn state_load_word(self, key: Value, span_md_idx: Option<MetadataIndex>) -> Value {
        let state_load_val =
            Value::new_instruction(self.context, Instruction::StateLoadWord(key), span_md_idx);
        self.context.blocks[self.block.0]
            .instructions
            .push(state_load_val);
        state_load_val
    }

    pub fn state_store_quad_word(
        self,
        stored_val: Value,
        key: Value,
        span_md_idx: Option<MetadataIndex>,
    ) -> Value {
        let state_store_val = Value::new_instruction(
            self.context,
            Instruction::StateStoreQuadWord { stored_val, key },
            span_md_idx,
        );
        self.context.blocks[self.block.0]
            .instructions
            .push(state_store_val);
        state_store_val
    }

    pub fn state_store_word(
        self,
        stored_val: Value,
        key: Value,
        span_md_idx: Option<MetadataIndex>,
    ) -> Value {
        let state_store_val = Value::new_instruction(
            self.context,
            Instruction::StateStoreWord { stored_val, key },
            span_md_idx,
        );
        self.context.blocks[self.block.0]
            .instructions
            .push(state_store_val);
        state_store_val
    }

    pub fn store(
        self,
        dst_val: Value,
        stored_val: Value,
        span_md_idx: Option<MetadataIndex>,
    ) -> Value {
        let store_val = Value::new_instruction(
            self.context,
            Instruction::Store {
                dst_val,
                stored_val,
            },
            span_md_idx,
        );
        self.context.blocks[self.block.0]
            .instructions
            .push(store_val);
        store_val
    }
}
