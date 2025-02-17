use sway_ast::{
    keywords::{FnToken, Keyword},
    Braces, CodeBlockContents, FnSignature, ItemFn, Parens, Punctuated,
};
use sway_types::{Ident, Span, Spanned};

use crate::assert_insert_span;

use super::{Modifier, New};

impl<'a> Modifier<'a, ItemFn> {
    pub(crate) fn set_name<S: AsRef<str> + ?Sized>(&mut self, name: &S) -> &mut Self {
        // We preserve the current span of the name.
        let insert_span = self.element.fn_signature.name.span();
        self.element.fn_signature.name =
            Ident::new_with_override(name.as_ref().into(), insert_span);

        self
    }
}

impl New {
    /// Creates an [ItemFn] representing and empty function without arguments that is named `name`.
    pub(crate) fn function<S: AsRef<str> + ?Sized>(insert_span: Span, name: &S) -> ItemFn {
        assert_insert_span!(insert_span);

        ItemFn {
            fn_signature: FnSignature {
                visibility: None,
                fn_token: FnToken::new(insert_span.clone()),
                name: Ident::new_with_override(name.as_ref().into(), insert_span.clone()),
                generics: None,
                arguments: Parens {
                    inner: sway_ast::FnArgs::Static(Punctuated {
                        value_separator_pairs: vec![],
                        final_value_opt: None,
                    }),
                    span: insert_span.clone(),
                },
                return_type_opt: None,
                where_clause_opt: None,
            },
            body: Braces {
                inner: CodeBlockContents {
                    statements: vec![],
                    final_expr_opt: None,
                    span: insert_span.clone(),
                },
                span: insert_span,
            },
        }
    }
}
