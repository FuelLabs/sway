use sway_ast::PathExprSegment;
use sway_types::{Ident, Spanned};

use super::Modifier;

impl Modifier<'_, PathExprSegment> {
    pub(crate) fn set_name<S: AsRef<str> + ?Sized>(&mut self, name: &S) -> &mut Self {
        // We preserve the current span of the name.
        let insert_span = self.element.name.span();
        self.element.name = Ident::new_with_override(name.as_ref().into(), insert_span);

        self
    }
}
