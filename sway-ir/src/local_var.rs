//! A value representing a function-local variable.

use crate::{
    constant::Constant,
    context::Context,
    irtype::{Type, TypeContent},
    pretty::DebugWithContext,
};

/// A wrapper around an [ECS](https://github.com/orlp/slotmap) handle into the
/// [`Context`].
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, DebugWithContext)]
pub struct LocalVar(#[in_context(local_vars)] pub slotmap::DefaultKey);

#[doc(hidden)]
#[derive(Clone, DebugWithContext)]
pub struct LocalVarContent {
    pub ptr_ty: Type,
    pub initializer: Option<Constant>,
    pub mutable: bool,
}

impl LocalVar {
    /// Return a new local of a specific type with an optional [`Constant`] initializer.  If a
    /// local is marked as mutable then it is guaranteed to be on the stack rather than in
    /// read-only memory.
    pub fn new(
        context: &mut Context,
        ty: Type,
        initializer: Option<Constant>,
        mutable: bool,
    ) -> Self {
        let ptr_ty = Type::new_ptr(context, ty);
        let content = LocalVarContent {
            ptr_ty,
            initializer,
            mutable,
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

    /// Return whether this local variable is mutable.
    pub fn is_mutable(&self, context: &Context) -> bool {
        context.local_vars[self.0].mutable
    }

    /// Change this local variable's mutability.
    pub fn set_mutable(&self, context: &mut Context, mutable: bool) {
        context.local_vars[self.0].mutable = mutable;
    }
}
