use crate::{constant::Constant, context::Context, irtype::Type};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Pointer(pub generational_arena::Index);

#[derive(Clone)]
pub struct PointerContent {
    pub ty: Type,
    pub is_mutable: bool,
    pub initializer: Option<Constant>,
}

impl Pointer {
    pub fn new(
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

    pub fn get_type<'a>(&self, context: &'a Context) -> &'a Type {
        &context.pointers[self.0].ty
    }

    pub fn is_struct_ptr(&self, context: &Context) -> bool {
        matches!(&context.pointers[self.0].ty, Type::Struct(_))
    }
}
