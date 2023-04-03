//! A value representing a function-local variable.

use crate::{
    constant::Constant,
    context::Context,
    irtype::{Type, TypeContent},
    pretty::DebugWithContext,
};

/// A wrapper around an [ECS](https://github.com/fitzgen/generational-arena) handle into the
/// [`Context`].
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, DebugWithContext)]
pub struct LocalVar(#[in_context(local_vars)] pub generational_arena::Index);

#[doc(hidden)]
#[derive(Clone, DebugWithContext)]
pub struct LocalVarContent {
    pub ptr_ty: Type,
    pub initializer: Option<Constant>,
}

impl LocalVar {
    /// Return a new local of a specific type with an optional [`Constant`] initializer.
    pub fn new(context: &mut Context, ty: Type, initializer: Option<Constant>) -> Self {
        let ptr_ty = Type::new_ptr(context, ty);
        let content = LocalVarContent {
            ptr_ty,
            initializer,
        };
        LocalVar(context.local_vars.insert(content))
    }

    /// Return the type of this local variable, which is always a pointer.
    pub fn get_type(&self, context: &Context) -> Type {
        context.local_vars[self.0].ptr_ty
    }

    /// Return the inner (pointed to) type.
    pub fn get_inner_type(&self, context: &Context) -> Type {
        let TypeContent::Pointer(inner_ty) = self.get_type(context).get_content(context) else {
            unreachable!("Local var type is always a pointer.")
        };
        *inner_ty
    }

    /// Return the initializer for this local variable.
    pub fn get_initializer<'a>(&self, context: &'a Context) -> Option<&'a Constant> {
        context.local_vars[self.0].initializer.as_ref()
    }
}
