//! A value representing a memory location, generally to a function local value.
//!
//! NOTE: much of this was hastily put together and can be streamlined or refactored altogether.

use crate::{constant::Constant, context::Context, irtype::Type};

/// A wrapper around an [ECS](https://github.com/fitzgen/generational-arena) handle into the
/// [`Context`].
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Pointer(pub generational_arena::Index);

#[doc(hidden)]
#[derive(Clone)]
pub struct PointerContent {
    pub ty: Type,
    pub is_mutable: bool,
    pub initializer: Option<Constant>,
}

impl Pointer {
    /// Return a new pointer to a specific type with an optional [`Constant`] initializer.
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

    /// Return the type pointed to by this pointer.
    pub fn get_type<'a>(&self, context: &'a Context) -> &'a Type {
        &context.pointers[self.0].ty
    }

    /// Return whether this pointer is to a [`Type::Struct`] in particular.
    pub fn is_aggregate_ptr(&self, context: &Context) -> bool {
        matches!(
            &context.pointers[self.0].ty,
            Type::Array(_) | Type::Struct(_) | Type::Union(_)
        )
    }
}
