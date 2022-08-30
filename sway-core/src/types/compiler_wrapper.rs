use std::fmt;

use crate::declaration_engine::declaration_engine::DeclarationEngine;

pub(crate) struct CompileWrapper<'a, T> {
    pub(crate) inner: &'a T,
    pub(crate) declaration_engine: &'a DeclarationEngine,
}

impl<'a, T> Eq for CompileWrapper<'a, T> where CompileWrapper<'a, T>: PartialEq {}

pub(crate) trait ToCompileWrapper<'a, T> {
    fn wrap(&'a self, declaration_engine: &'a DeclarationEngine) -> CompileWrapper<'a, T>;
}

impl<'a, T> ToCompileWrapper<'a, T> for T {
    fn wrap(&'a self, declaration_engine: &'a DeclarationEngine) -> CompileWrapper<'a, T> {
        CompileWrapper {
            inner: self,
            declaration_engine,
        }
    }
}

impl<'a, T> ToCompileWrapper<'a, T> for &T {
    fn wrap(&'a self, declaration_engine: &'a DeclarationEngine) -> CompileWrapper<'a, T> {
        CompileWrapper {
            inner: self,
            declaration_engine,
        }
    }
}

impl<'a, T> fmt::Debug for CompileWrapper<'a, T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.inner)
    }
}
