use crate::priv_prelude::*;

pub struct Ident {
    span: Span,
}

impl Spanned for Ident {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

pub fn ident() -> impl Parser<Output = Ident> + Clone {
    from_fn(|input| {
        let mut char_indices = input.as_str().char_indices();
        let c = match char_indices.next() {
            Some((_, c)) => c,
            None => {
                return Err(ParseError::UnexpectedEof {
                    span: input.to_start(),
                })
            },
        };
        if !c.is_xid_start() {
            return Err(ParseError::ExpectedIdent {
                span: input.to_start(),
            });
        }
        let len = loop {
            let (i, c) = match char_indices.next() {
                Some((i, c)) => (i, c),
                None => break input.as_str().len(),
            };
            if !c.is_xid_continue() {
                break i;
            }
        };
        Ok((Ident { span: input.slice(..len) }, len))
    })
}

