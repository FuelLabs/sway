use super::*;

#[derive(Debug, Clone)]
pub(crate) enum Instruction {
    AsmBlock(AsmBlock, Vec<AsmArg>),
    Branch(Block),
    Call(Function, Vec<Value>),
    ConditionalBranch {
        cond_value: Value,
        true_block: Block,
        false_block: Block,
    },
    ExtractValue {
        aggregate: Value,
        ty: Aggregate,
        indices: Vec<u64>,
    },
    GetPointer(Pointer),
    InsertValue {
        aggregate: Value,
        ty: Aggregate,
        value: Value,
        indices: Vec<u64>,
    },
    Load(Pointer),
    Phi(Vec<(Block, Value)>),
    Ret(Value, Type),
    Store {
        ptr: Pointer,
        stored_val: Value,
    },
}

impl Instruction {
    pub(crate) fn get_type(&self, context: &Context) -> Option<Type> {
        match self {
            Instruction::AsmBlock(asm_block, _) => asm_block.get_type(context),
            Instruction::Call(function, _) => Some(context.functions[function.0].return_type),
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
            Instruction::InsertValue { .. } => None,
            Instruction::Store { .. } => None,
        }
    }

    pub(crate) fn replace_value(&mut self, old_val: Value, new_val: Value) {
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
            Instruction::InsertValue {
                aggregate, value, ..
            } => {
                replace(aggregate);
                replace(value);
            }
            Instruction::ExtractValue { aggregate, .. } => replace(aggregate),
            Instruction::Load(_) => (),
            Instruction::Phi(pairs) => pairs.iter_mut().for_each(|(_, val)| replace(val)),
            Instruction::Ret(ret_val, _) => replace(ret_val),
            Instruction::Store { stored_val, .. } => {
                replace(stored_val);
            }
        }
    }
}

pub(crate) struct InstructionIterator {
    instructions: Vec<generational_arena::Index>,
    next: usize,
}

impl InstructionIterator {
    pub(crate) fn new(context: &Context, block: &Block) -> Self {
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

pub(crate) struct InstructionInserter<'a> {
    context: &'a mut Context,
    block: Block,
}

impl<'a> InstructionInserter<'a> {
    pub(crate) fn new(context: &'a mut Context, block: Block) -> InstructionInserter<'a> {
        InstructionInserter { context, block }
    }

    //
    // XXX maybe these should return result, in case they get bad args?
    //

    pub(crate) fn asm_block(
        self,
        args: Vec<AsmArg>,
        body: Vec<AsmInstruction>,
        return_name: Option<String>,
    ) -> Value {
        let asm = AsmBlock::new(
            self.context,
            args.iter().map(|arg| arg.name.clone()).collect(),
            body,
            return_name,
        );
        self.asm_block_from_asm(asm, args)
    }

    pub(crate) fn asm_block_from_asm(self, asm: AsmBlock, args: Vec<AsmArg>) -> Value {
        let asm_val = Value::new_instruction(self.context, Instruction::AsmBlock(asm, args));
        self.context.blocks[self.block.0].instructions.push(asm_val);
        asm_val
    }

    pub(crate) fn branch(self, to_block: Block, phi_value: Option<Value>) -> Value {
        let br_val = Value::new_instruction(self.context, Instruction::Branch(to_block));
        phi_value
            .into_iter()
            .for_each(|pv| to_block.add_phi(self.context, self.block, pv));
        self.context.blocks[self.block.0].instructions.push(br_val);
        br_val
    }

    pub(crate) fn call(self, function: Function, args: &[Value]) -> Value {
        let call_val =
            Value::new_instruction(self.context, Instruction::Call(function, args.to_vec()));
        self.context.blocks[self.block.0]
            .instructions
            .push(call_val);
        call_val
    }

    pub(crate) fn conditional_branch(
        self,
        cond_value: Value,
        true_block: Block,
        false_block: Block,
        phi_value: Option<Value>,
    ) -> Value {
        let cbr_val = Value::new_instruction(
            self.context,
            Instruction::ConditionalBranch {
                cond_value,
                true_block,
                false_block,
            },
        );
        phi_value.into_iter().for_each(|pv| {
            true_block.add_phi(self.context, self.block, pv);
            false_block.add_phi(self.context, self.block, pv);
        });
        self.context.blocks[self.block.0].instructions.push(cbr_val);
        cbr_val
    }

    pub(crate) fn extract_value(self, aggregate: Value, ty: Aggregate, indices: Vec<u64>) -> Value {
        let extract_value_val = Value::new_instruction(
            self.context,
            Instruction::ExtractValue {
                aggregate,
                ty,
                indices,
            },
        );
        self.context.blocks[self.block.0]
            .instructions
            .push(extract_value_val);
        extract_value_val
    }

    pub(crate) fn get_ptr(self, ptr: Pointer) -> Value {
        let get_ptr_val = Value::new_instruction(self.context, Instruction::GetPointer(ptr));
        self.context.blocks[self.block.0]
            .instructions
            .push(get_ptr_val);
        get_ptr_val
    }

    pub(crate) fn insert_value(
        self,
        aggregate: Value,
        ty: Aggregate,
        value: Value,
        indices: Vec<u64>,
    ) -> Value {
        let insert_val = Value::new_instruction(
            self.context,
            Instruction::InsertValue {
                aggregate,
                ty,
                value,
                indices,
            },
        );
        self.context.blocks[self.block.0]
            .instructions
            .push(insert_val);
        insert_val
    }

    pub(crate) fn load(self, ptr: Pointer) -> Value {
        let load_val = Value::new_instruction(self.context, Instruction::Load(ptr));
        self.context.blocks[self.block.0]
            .instructions
            .push(load_val);
        load_val
    }

    pub(crate) fn ret(self, value: Value, ty: Type) -> Value {
        let ret_val = Value::new_instruction(self.context, Instruction::Ret(value, ty));
        self.context.blocks[self.block.0].instructions.push(ret_val);
        ret_val
    }

    pub(crate) fn store(self, ptr: Pointer, stored_val: Value) -> Value {
        let store_val =
            Value::new_instruction(self.context, Instruction::Store { ptr, stored_val });
        self.context.blocks[self.block.0]
            .instructions
            .push(store_val);
        store_val
    }
}
