//! This module contains common API for modifying elements within a lexed tree.

mod storage_field;

/// A wrapper around a lexed tree element that will be modified.
pub(crate) struct Modifier<'a, T> {
    element: &'a mut T,
}

impl<'a, T> Modifier<'a, T> {
    pub(crate) fn new(element: &'a mut T) -> Self {
        Self { element }
    }
}
