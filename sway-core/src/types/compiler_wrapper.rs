use std::{borrow::Cow, fmt};

use crate::declaration_engine::declaration_engine::DeclarationEngine;

pub(crate) struct CompileWrapper<'a, T>
where
    T: Clone,
{
    pub(crate) inner: Cow<'a, T>,
    pub(crate) declaration_engine: &'a DeclarationEngine,
}

impl<'a, T> Eq for CompileWrapper<'a, T>
where
    CompileWrapper<'a, T>: PartialEq,
    T: Clone,
{
}

pub(crate) trait ToCompileWrapper<'a, T> {
    fn wrap(self, declaration_engine: &'a DeclarationEngine) -> CompileWrapper<'a, T>
    where
        T: Clone;
    fn wrap_ref(&'a self, declaration_engine: &'a DeclarationEngine) -> CompileWrapper<'a, T>
    where
        T: Clone;
}

impl<'a, T> ToCompileWrapper<'a, T> for T {
    fn wrap(self, declaration_engine: &'a DeclarationEngine) -> CompileWrapper<'a, T>
    where
        T: Clone,
    {
        CompileWrapper {
            inner: Cow::Owned(self),
            declaration_engine,
        }
    }

    fn wrap_ref(&'a self, declaration_engine: &'a DeclarationEngine) -> CompileWrapper<'a, T>
    where
        T: Clone,
    {
        CompileWrapper {
            inner: Cow::Borrowed(self),
            declaration_engine,
        }
    }
}

impl<'a, T> ToCompileWrapper<'a, T> for &T {
    fn wrap(self, declaration_engine: &'a DeclarationEngine) -> CompileWrapper<'a, T>
    where
        T: Clone,
    {
        CompileWrapper {
            inner: Cow::Owned(self.clone()),
            declaration_engine,
        }
    }

    fn wrap_ref(&'a self, declaration_engine: &'a DeclarationEngine) -> CompileWrapper<'a, T>
    where
        T: Clone,
    {
        CompileWrapper {
            inner: Cow::Borrowed(self),
            declaration_engine,
        }
    }
}

impl<'a, T> fmt::Debug for CompileWrapper<'a, T>
where
    T: fmt::Debug + Clone,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.inner)
    }
}
