use crate::priv_prelude::*;

pub struct Punctuated<T, S> {
    values: Vec<T>,
    separators: Vec<S>,
    span: Span,
}

impl<T, S> Punctuated<T, S> {
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.values.iter()
    }

    pub fn pairs(&self) -> impl Iterator<Item = (&T, Option<&S>)> {
        let mut values = self.values.iter();
        let mut separators = self.separators.iter();
        iter::from_fn(move || {
            let value = values.next()?;
            let separator_opt = separators.next();
            Some((value, separator_opt))
        })
    }
}

impl<T, S> Spanned for Punctuated<T, S>
where
    T: Spanned,
    S: Spanned,
{
    fn span(&self) -> Span {
        self.span.clone()
    }
}

pub fn punctuated<T, S, U, V>(item: U, separator: V) -> impl Parser<Output = Punctuated<T, S>> + Clone
where
    U: Parser<Output = T> + Clone + 'static,
    V: Parser<Output = S> + Clone + 'static,
    T: Spanned + 'static,
    S: Spanned + 'static,
{
    pre_punctuated(item, separator)
    .map(|pre_punctuated| {
        let PrePunctuated { values, separators, span } = pre_punctuated;
        Punctuated {
            values: values.into_iter().rev().collect(),
            separators: separators.into_iter().rev().collect(),
            span,
        }
    })
}

struct PrePunctuated<T, S> {
    values: Vec<T>,
    separators: Vec<S>,
    span: Span,
}

impl<T, S> Spanned for PrePunctuated<T, S> {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

fn pre_punctuated<T, S, U, V>(item: U, separator: V) -> impl Parser<Output = PrePunctuated<T, S>> + Clone
where
    U: Parser<Output = T> + Clone + 'static,
    V: Parser<Output = S> + Clone + 'static,
    T: Spanned + 'static,
    S: Spanned + 'static,
{
    item
    .clone()
    .then(
        separator
        .clone()
        .then(optional_leading_whitespace(lazy(move || {
            pre_punctuated(item.clone(), separator.clone())
            .optional()
        })))
        .optional()
    )
    .optional()
    .map(|head_separator_tail_res: Result<(T, Result<(S, Result<PrePunctuated<T, S>, _>), _>), Span>| {
        match head_separator_tail_res {
            Ok((head, separator_tail_res)) => match separator_tail_res {
                Ok((separator, tail_res)) => match tail_res {
                    Ok(mut pre_punctuated) => {
                        pre_punctuated.span = Span::join(pre_punctuated.span, separator.span());
                        pre_punctuated.values.push(head);
                        pre_punctuated.separators.push(separator);
                        pre_punctuated
                    },
                    Err(..) => {
                        let span = Span::join(head.span(), separator.span());
                        PrePunctuated {
                            values: vec![head],
                            separators: vec![separator],
                            span,
                        }
                    },
                },
                Err(..) => {
                    let span = head.span();
                    PrePunctuated {
                        values: vec![head],
                        separators: vec![],
                        span,
                    }
                },
            },
            Err(span) => {
                PrePunctuated {
                    values: vec![],
                    separators: vec![],
                    span: span.to_start(),
                }
            },
        }
    })
}

