#[allow(unused_imports)] // Used in doc-comments.
use sway_ast::{
    attribute::{Annotated, Attribute},
    AttributeDecl,
};

use sway_types::{Span, Spanned};

use super::Modifier;

impl<T> Modifier<'_, Annotated<T>> {
    /// From `self`, removes [AttributeDecl] that contains an [Attribute]
    /// whose span equals `attribute_span`.
    ///
    /// Method **removes the whole [AttributeDecl]**, even if it contains
    /// other attributes, aside from the `attribute_span` matching one.
    pub(crate) fn remove_attribute_decl_containing_attribute(
        &mut self,
        attribute_span: &Span,
    ) -> &mut Self {
        self.element.attribute_list.retain(|attribute_decl| {
            attribute_decl
                .attribute
                .inner
                .iter()
                .all(|attribute| attribute.span() != *attribute_span)
        });
        self
    }
}
