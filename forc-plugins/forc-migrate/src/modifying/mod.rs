//! This module contains common API for building new and modifying existing
//! elements within a lexed tree.

use sway_types::Span;

mod annotated;
mod attribute;
mod function;
mod literal;
mod module;
mod storage_field;

/// A wrapper around a lexed tree element that will be modified.
pub(crate) struct Modifier<'a, T> {
    element: &'a mut T,
}

impl<'a, T> Modifier<'a, T> {
    // Private, so that we enforce creating modifiers with the
    // `modify` function.
    fn new(element: &'a mut T) -> Self {
        Self { element }
    }
}

pub(crate) fn modify<T>(element: &mut T) -> Modifier<'_, T> {
    Modifier::new(element)
}

// Empty struct for creating new lexed elements.
// Constructors for each lexed element are in separate modules,
// grouped by lexed elements they construct, and each module
// has its own `New` impl.
pub(crate) struct New {}

/// Trait for setting all spans within `Self` to the same insert span.
///
/// New elements inserted into lexed tree should have their spans set
/// to the same zero-sized [Span]. This ensures that they will always
/// be properly rendered. Sometimes, new elements are copied from existing
/// elements and modified. Such new elements might not have all spans
/// set to the same, zero-sized insert span. Implementing this trait
/// ensures proper setting of the insert span.
// TODO: Implement `SetInsertSpan` for lexed tree elements.
#[allow(dead_code)]
pub(crate) trait SetInsertSpan {
    fn set_insert_span(&mut self, insert_span: Span);
}

#[macro_export]
macro_rules! assert_insert_span {
    ($insert_span: ident) => {
        assert!(
            stringify!($insert_span) == "insert_span",
            "the insert span function argument must be called `insert_span`"
        );
        assert!($insert_span.is_empty(), "`insert_span` must be empty");
        assert!(
            !$insert_span.is_dummy(),
            "`insert_span` must not be a `Span::dummy()`"
        );
    };
}
