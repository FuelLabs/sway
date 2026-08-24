//! Helpers for turning `Option::None` and `Result::Err` into [`CompileError::Internal`] ICEs.

use sway_types::Span;

use crate::error::CompileError;

pub trait OkOrIceInternal {
    type Output;

    fn ok_or_ice_internal(
        self,
        msg: &'static str,
        span: Span,
    ) -> Result<Self::Output, CompileError>;
}

impl<T> OkOrIceInternal for Option<T> {
    type Output = T;

    fn ok_or_ice_internal(
        self,
        msg: &'static str,
        span: Span,
    ) -> Result<Self::Output, CompileError> {
        self.ok_or_else(|| CompileError::Internal(msg, span))
    }
}

impl<T, E> OkOrIceInternal for Result<T, E> {
    type Output = T;

    fn ok_or_ice_internal(
        self,
        msg: &'static str,
        span: Span,
    ) -> Result<Self::Output, CompileError> {
        self.map_err(|_| CompileError::Internal(msg, span))
    }
}
