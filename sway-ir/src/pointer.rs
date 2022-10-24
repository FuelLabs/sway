//! A value representing a memory location, generally to a function local value.
//!
//! NOTE: much of this was hastily put together and can be streamlined or refactored altogether.

use crate::{constant::Constant, context::Context, irtype::Type, pretty::DebugWithContext};

/// A wrapper around an [ECS](https://github.com/fitzgen/generational-arena) handle into the
/// [`Context`].
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, DebugWithContext)]
pub struct Pointer(#[in_context(pointers)] pub generational_arena::Index);

#[doc(hidden)]
#[derive(Clone, DebugWithContext)]
pub struct PointerContent {
    pub ty:          Type,
    pub is_mutable:  bool,
    pub initializer: Option<Constant>,
}

impl Pointer {
    /// Return a string representation of type, used for IR printing.
    pub fn as_string(&self, context: &Context, name: Option<&str>) -> String {
        let PointerContent { ty, is_mutable, .. } = &context.pointers[self.0];
        let mut_tag = if *is_mutable { "mut " } else { "" };
        let name_tag = if name.is_some() {
            format!(" {}", name.unwrap())
        } else {
            "".to_string()
        };
        format!("{mut_tag}ptr {}{}", ty.as_string(context), name_tag)
    }
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

    /// Return the initializer for this pointer.
    pub fn get_initializer<'a>(&self, context: &'a Context) -> Option<&'a Constant> {
        context.pointers[self.0].initializer.as_ref()
    }

    /// Return whether the pointer is to a mutable value.
    pub fn is_mutable(&self, context: &Context) -> bool {
        context.pointers[self.0].is_mutable
    }

    /// Return whether this pointer is to a [`Type::Struct`] in particular.
    pub fn is_aggregate_ptr(&self, context: &Context) -> bool {
        matches!(
            &context.pointers[self.0].ty,
            Type::Array(_) | Type::Struct(_) | Type::Union(_)
        )
    }

    pub fn is_equivalent(&self, context: &Context, other: &Pointer) -> bool {
        self.get_type(context).eq(context, other.get_type(context))
    }
}
