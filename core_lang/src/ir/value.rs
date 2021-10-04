use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub(crate) struct Value(pub(crate) generational_arena::Index);

#[derive(Clone)]
pub(crate) enum ValueContent {
    Argument(Type),
    Constant(Constant),
    Instruction(Instruction),
}

impl Value {
    pub(crate) fn new_argument(context: &mut Context, ty: Type) -> Value {
        let content = ValueContent::Argument(ty);
        Value(context.values.insert(content))
    }

    pub(crate) fn new_constant(context: &mut Context, constant: Constant) -> Value {
        let content = ValueContent::Constant(constant);
        Value(context.values.insert(content))
    }

    pub(crate) fn new_instruction(context: &mut Context, instruction: Instruction) -> Value {
        let content = ValueContent::Instruction(instruction);
        Value(context.values.insert(content))
    }

    pub(crate) fn is_constant(&self, context: &Context) -> bool {
        matches!(context.values[self.0], ValueContent::Constant(_))
    }

    pub(crate) fn is_terminator(&self, context: &Context) -> bool {
        match &context.values[self.0] {
            ValueContent::Instruction(ins) => matches!(
                ins,
                Instruction::Branch(_)
                    | Instruction::ConditionalBranch { .. }
                    | Instruction::Ret(_, _)
            ),
            _ => false,
        }
    }

    //pub(crate) fn get_constant(&self, context: &Context) -> Constant {
    //    if let ValueContent::Constant(c) = &context.values[self.0] {
    //        c.clone()
    //    } else {
    //        panic!("Value is not a constant.")
    //    }
    //}

    pub(crate) fn replace_instruction_value(
        &self,
        context: &mut Context,
        old_val: Value,
        new_val: Value,
    ) {
        if let ValueContent::Instruction(instruction) = &mut context.values.get_mut(self.0).unwrap()
        {
            instruction.replace_value(old_val, new_val);
        }
    }

    pub(crate) fn get_type(&self, context: &Context) -> Option<Type> {
        match &context.values[self.0] {
            ValueContent::Argument(ty) => Some(*ty),
            ValueContent::Constant(c) => Some(c.ty),
            ValueContent::Instruction(ins) => ins.get_type(context),
        }
    }

    //pub(crate) fn is_bool_ty(&self, context: &Context) -> bool {
    //    self.get_type(context)
    //        .map(|ty| ty == Type::Bool)
    //        .unwrap_or(false)
    //}
}
