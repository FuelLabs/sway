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
    value::Value,
};

#[derive(Debug, Clone)]
pub enum Instruction {
    /// An opaque list of ASM instructions passed directly to codegen.
    AsmBlock(AsmBlock, Vec<AsmArg>),
    /// An unconditional jump.
    Branch(Block),
    /// A function call with a list of arguments.
    Call(Function, Vec<Value>),
    /// A conditional jump with the boolean condition value and true or false destinations.
    ConditionalBranch {
        cond_value: Value,
        true_block: Block,
        false_block: Block,
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
    GetPointer(Pointer),
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
    Load(Pointer),
    /// No-op, handy as a placeholder instruction.
    Nop,
    /// Choose a value from a list depending on the preceding block.
    Phi(Vec<(Block, Value)>),
    /// Return from a function.
    Ret(Value, Type),
    /// Write a value to a memory pointer.
    Store { ptr: Pointer, stored_val: Value },
}

impl Instruction {
    /// Some [`Instruction`]s can return a value, but for some a return value doesn't make sense.
    ///
    /// Those which perform side effects such as writing to memory and also terminators such as
    /// `Ret` do not have a type.
    pub fn get_type(&self, context: &Context) -> Option<Type> {
        match self {
            Instruction::AsmBlock(asm_block, _) => asm_block.get_type(context),
            Instruction::Call(function, _) => Some(context.functions[function.0].return_type),
            Instruction::ExtractElement { ty, .. } => ty.get_elem_type(context),
            Instruction::ExtractValue { ty, indices, .. } => ty.get_field_type(context, indices),
            Instruction::Load(ptr) => Some(context.pointers[ptr.0].ty),
            Instruction::Phi(_alts) => {
                unimplemented!("phi get type -- I think we should put the type in the enum.")
            }

            // These are all terminators which don't return, essentially.  No type.
            Instruction::Branch(_) => None,
            Instruction::ConditionalBranch { .. } => None,
            Instruction::Ret(..) => None,

            // GetPointer returns a pointer type which we don't expose.
            Instruction::GetPointer(_) => None,

            // These write values but don't return one.  If we're explicit we could return Unit.
            Instruction::InsertElement { .. } => None,
            Instruction::InsertValue { .. } => None,
            Instruction::Store { .. } => None,

            // No-op is also no-type.
            Instruction::Nop => None,
        }
    }

    /// Some [`Instruction`]s may have struct arguments.  Return it if so for this instruction.
    pub fn get_aggregate(&self, context: &Context) -> Option<Aggregate> {
        match self {
            Instruction::GetPointer(ptr) | Instruction::Load(ptr) => match ptr.get_type(context) {
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
            Instruction::Branch(_) => (),
            Instruction::Call(_, args) => args.iter_mut().for_each(replace),
            Instruction::ConditionalBranch { cond_value, .. } => replace(cond_value),
            Instruction::GetPointer(_) => (),
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
            Instruction::Ret(ret_val, _) => replace(ret_val),
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
    // XXX maybe these should return result, in case they get bad args?
    //

    /// Append a new [`Instruction::AsmBlock`] from `args` and a `body`.
    pub fn asm_block(
        self,
        args: Vec<AsmArg>,
        body: Vec<AsmInstruction>,
        return_name: Option<Ident>,
        span_md_idx: Option<MetadataIndex>,
    ) -> Value {
        let asm = AsmBlock::new(
            self.context,
            args.iter().map(|arg| arg.name.clone()).collect(),
            body,
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

    pub fn get_ptr(self, ptr: Pointer, span_md_idx: Option<MetadataIndex>) -> Value {
        let get_ptr_val =
            Value::new_instruction(self.context, Instruction::GetPointer(ptr), span_md_idx);
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

    pub fn load(self, ptr: Pointer, span_md_idx: Option<MetadataIndex>) -> Value {
        let load_val = Value::new_instruction(self.context, Instruction::Load(ptr), span_md_idx);
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

    pub fn ret(self, value: Value, ty: Type, span_md_idx: Option<MetadataIndex>) -> Value {
        let ret_val =
            Value::new_instruction(self.context, Instruction::Ret(value, ty), span_md_idx);
        self.context.blocks[self.block.0].instructions.push(ret_val);
        ret_val
    }

    pub fn store(
        self,
        ptr: Pointer,
        stored_val: Value,
        span_md_idx: Option<MetadataIndex>,
    ) -> Value {
        let store_val = Value::new_instruction(
            self.context,
            Instruction::Store { ptr, stored_val },
            span_md_idx,
        );
        self.context.blocks[self.block.0]
            .instructions
            .push(store_val);
        store_val
    }
}
