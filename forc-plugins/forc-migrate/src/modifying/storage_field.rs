use num_bigint::BigUint;
use sway_ast::{
    keywords::{InToken, Keyword},
    Expr, LitInt, StorageField,
};
use sway_types::{Span, Spanned};

use super::Modifier;

pub(crate) trait ToInKey {
    fn to_in_key(self, span: Span) -> Expr;
}

impl ToInKey for BigUint {
    fn to_in_key(self, span: Span) -> Expr {
        Expr::Literal(sway_ast::Literal::Int(LitInt {
            span,
            parsed: self,
            ty_opt: None,
            is_generated_b256: true,
        }))
    }
}

impl ToInKey for Expr {
    fn to_in_key(self, _span: Span) -> Expr {
        // TODO: Provide infrastructure for replacing spans on the elements
        //       of a lexed tree. This will be useful in modifications in
        //       which we generate new tree elements by copying existing.
        //
        //       Until then, in this demo on how to develop `Modifier`s,
        //       just return `self`, without the spans replaced.
        self
    }
}

#[allow(dead_code)]
impl<'a> Modifier<'a, StorageField> {
    pub(crate) fn set_in_key<K: ToInKey>(&mut self, key: K) -> &mut Self {
        // If the `in` token already exists, just replace the key and leave the `in`
        // token as is. Place the key after the `in` token.
        let insert_span = if let Some(in_token) = &self.element.in_token {
            Span::empty_at_end(&in_token.span())
        } else {
            // Otherwise, place the `in` token after the name.
            Span::empty_at_end(&self.element.name.span())
        };

        if self.element.in_token.is_none() {
            self.element.in_token = Some(InToken::new(insert_span.clone()));
        }

        self.element.key_expr = Some(key.to_in_key(insert_span));

        self
    }

    pub(crate) fn remove_in_key(&mut self) -> &mut Self {
        self.element.in_token = None;
        self.element.key_expr = None;

        self
    }
}
