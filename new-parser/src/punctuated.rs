use crate::priv_prelude::*;

#[derive(Clone, Debug)]
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

pub fn punctuated<T, S, U, V>(value: U, separator: V) -> impl Parser<Output = Punctuated<T, S>> + Clone
where
    U: Parser<Output = T> + Clone + 'static,
    V: Parser<Output = S> + Clone + 'static,
    T: Spanned + 'static,
    S: Spanned + 'static,
{
    pre_punctuated(value, separator)
    .map(|pre_punctuated| {
        let PrePunctuated { values, separators, span } = pre_punctuated;
        Punctuated {
            values: values.into_iter().rev().collect(),
            separators: separators.into_iter().rev().collect(),
            span,
        }
    })
}

#[derive(Debug)]
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

fn pre_punctuated<T, S, U, V>(value: U, separator: V) -> impl Parser<Output = PrePunctuated<T, S>> + Clone
where
    U: Parser<Output = T> + Clone + 'static,
    V: Parser<Output = S> + Clone + 'static,
    T: Spanned + 'static,
    S: Spanned + 'static,
{
    value
    .clone()
    .then(
        separator
        .clone()
        .then(optional_leading_whitespace(lazy(move || {
            pre_punctuated(value.clone(), separator.clone())
            .optional()
        })))
        .optional()
    )
    .optional()
    .map_with_span(|item_separator_tail_opt: Option<(T, Option<(S, Option<PrePunctuated<T, S>>)>)>, span| {
        match item_separator_tail_opt {
            None => PrePunctuated {
                values: vec![],
                separators: vec![],
                span,
            },
            Some((value, separator_tail_opt)) => match separator_tail_opt {
                None => PrePunctuated {
                    values: vec![value],
                    separators: vec![],
                    span,
                },
                Some((separator, tail_opt)) => match tail_opt {
                    None => PrePunctuated {
                        values: vec![value],
                        separators: vec![separator],
                        span,
                    },
                    Some(mut tail) => {
                        tail.values.push(value);
                        tail.separators.push(separator);
                        tail.span = span;
                        tail
                    },
                },
            },
        }
    })
}

