use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub(crate) struct AsmBlock(pub(crate) generational_arena::Index);

#[derive(Clone, Debug)]
pub(crate) struct AsmBlockContent {
    pub(crate) args_names: Vec<String>,
    pub(crate) body: Vec<AsmInstruction>,
    pub(crate) return_name: Option<String>,
}

#[derive(Clone, Debug)]
pub(crate) struct AsmArg {
    pub(crate) name: String,
    pub(crate) initializer: Option<Value>,
}

#[derive(Clone, Debug)]
pub(crate) struct AsmInstruction {
    pub(crate) name: String,
    pub(crate) args: Vec<String>,
    pub(crate) immediate: Option<String>,
}

impl AsmBlock {
    pub(crate) fn new(
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

    pub(crate) fn get_type(&self, context: &Context) -> Option<Type> {
        // The type is a named register, which will be a u64.
        context.asm_blocks[self.0]
            .return_name
            .as_ref()
            .map(|_| Type::Uint(64))
    }
}
