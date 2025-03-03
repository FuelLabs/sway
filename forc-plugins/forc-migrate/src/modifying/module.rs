use std::cmp::min;

use sway_ast::{attribute::Annotated, ItemFn, ItemKind, Module};

use sway_types::{Span, Spanned};

use super::Modifier;

#[allow(dead_code)]
impl<'a> Modifier<'a, Module> {
    /// Removes an [Annotated<ItemKind>] from `self`.
    /// The item to remove is identified by its [Span], `annotated_item_span`.
    pub(crate) fn remove_annotated_item(&mut self, annotated_item_span: &Span) -> &mut Self {
        self.element
            .items
            .retain(|annotated| annotated.span() != *annotated_item_span);
        self
    }

    /// Inserts `annotated_item` after the first already existing item whose [Span::end]
    /// is greater than or equal to `annotated_item`'s [Span::start].
    pub(crate) fn insert_annotated_item_after(
        &mut self,
        annotated_item: Annotated<ItemKind>,
    ) -> &mut Self {
        let first_existing_preceding_item_index = self
            .element
            .items
            .iter()
            .position(|annotated| annotated.span().end() >= annotated_item.span().start())
            .unwrap_or(0)
            + 1;
        let index = min(
            first_existing_preceding_item_index,
            self.element.items.len(),
        );
        self.element.items.insert(index, annotated_item);

        self
    }

    /// Appends `annotated_item` at the end of the [Module].
    pub(crate) fn append_annotated_item(
        &mut self,
        annotated_item: Annotated<ItemKind>,
    ) -> &mut Self {
        self.element.items.push(annotated_item);
        self
    }

    pub(crate) fn append_function(&mut self, function: ItemFn) -> &mut Self {
        let function = Annotated {
            attribute_list: vec![],
            value: ItemKind::Fn(function),
        };
        self.append_annotated_item(function)
    }
}
