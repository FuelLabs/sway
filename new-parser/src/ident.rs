use crate::priv_prelude::*;

#[derive(Clone, Debug)]
pub struct Ident {
    span: Span,
}

impl Ident {
    pub fn as_str(&self) -> &str {
        self.span.as_str()
    }
}

impl Spanned for Ident {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

pub struct ExpectedIdentError {
    pub position: usize,
}

pub fn ident() -> impl Parser<Output = Ident, Error = ExpectedIdentError> + Clone {
    from_fn(|input| {
        let mut char_indices = input.as_str().char_indices();
        let c = match char_indices.next() {
            Some((_, c)) => c,
            None => {
                let error = ExpectedIdentError {
                    position: input.start(),
                };
                return (false, Err(error));
            },
        };
        if !c.is_xid_start() {
            let error = ExpectedIdentError {
                position: input.start(),
            };
            return (false, Err(error));
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
        (false, Ok((Ident { span: input.slice(..len) }, len)))
    })
}

