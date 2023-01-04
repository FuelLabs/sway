//! A value representing a function-local variable.

use crate::{constant::Constant, context::Context, irtype::Type, pretty::DebugWithContext};

/// A wrapper around an [ECS](https://github.com/fitzgen/generational-arena) handle into the
/// [`Context`].
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, DebugWithContext)]
pub struct LocalVar(#[in_context(local_vars)] pub generational_arena::Index);

#[doc(hidden)]
#[derive(Clone, DebugWithContext)]
pub struct LocalVarContent {
    pub ty: Type,
    pub initializer: Option<Constant>,
}

impl LocalVar {
    /// Return a new local of a specific type with an optional [`Constant`] initializer.
    pub fn new(context: &mut Context, ty: Type, initializer: Option<Constant>) -> Self {
        let content = LocalVarContent { ty, initializer };
        LocalVar(context.local_vars.insert(content))
    }

    /// Return the type of this local variable.
    pub fn get_type<'a>(&self, context: &'a Context) -> &'a Type {
        &context.local_vars[self.0].ty
    }

    /// Return the initializer for this local variable.
    pub fn get_initializer<'a>(&self, context: &'a Context) -> Option<&'a Constant> {
        context.local_vars[self.0].initializer.as_ref()
    }
}
