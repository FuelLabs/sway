use crate::{context::Context, irtype::Type, value::Value};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct AsmBlock(pub generational_arena::Index);

#[derive(Clone, Debug)]
pub struct AsmBlockContent {
    pub args_names: Vec<String>,
    pub body: Vec<AsmInstruction>,
    pub return_name: Option<String>,
}

#[derive(Clone, Debug)]
pub struct AsmArg {
    pub name: String,
    pub initializer: Option<Value>,
}

#[derive(Clone, Debug)]
pub struct AsmInstruction {
    pub name: String,
    pub args: Vec<String>,
    pub immediate: Option<String>,
}

impl AsmBlock {
    pub fn new(
        context: &mut Context,
        args_names: Vec<String>,
        body: Vec<AsmInstruction>,
        return_name: Option<String>,
    ) -> Self {
        let content = AsmBlockContent {
            args_names,
            body,
            return_name,
        };
        AsmBlock(context.asm_blocks.insert(content))
    }

    pub fn get_type(&self, context: &Context) -> Option<Type> {
        // The type is a named register, which will be a u64.
        context.asm_blocks[self.0]
            .return_name
            .as_ref()
            .map(|_| Type::Uint(64))
    }
}
