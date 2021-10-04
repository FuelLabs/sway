use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub(crate) struct Pointer(pub(crate) generational_arena::Index);

#[derive(Clone)]
pub(crate) struct PointerContent {
    pub(crate) ty: Type,
    pub(crate) is_mutable: bool,
    pub(crate) initializer: Option<Constant>,
}

impl Pointer {
    pub(crate) fn new(
        context: &mut Context,
        ty: Type,
        is_mutable: bool,
        initializer: Option<Constant>,
    ) -> Self {
        let content = PointerContent {
            ty,
            is_mutable,
            initializer,
        };
        Pointer(context.pointers.insert(content))
    }

    pub(crate) fn get_type<'a>(&self, context: &'a Context) -> &'a Type {
        &context.pointers[self.0].ty
    }

    pub(crate) fn is_struct_ptr(&self, context: &Context) -> bool {
        matches!(&context.pointers[self.0].ty, Type::Struct(_))
    }
}
