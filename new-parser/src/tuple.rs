use crate::priv_prelude::*;

pub enum TupleDescriptor<T, SingletonNoCommaPermitted> {
    Unit,
    Single {
        head: Box<T>,
        singleton_no_comma_permitted: SingletonNoCommaPermitted,
    },
    Many {
        head: Box<T>,
        comma_token: CommaToken,
        tail: Box<TupleDescriptor<T, ()>>,
    },
}

impl<T, SingletonNoCommaPermitted> TupleDescriptor<T, SingletonNoCommaPermitted> {
    pub fn len(&self) -> usize {
        self.iter().len()
    }

    pub fn iter(&self) -> TupleDescriptorIter<'_, T> {
        let head_tail_opt = match self {
            TupleDescriptor::Unit => None,
            TupleDescriptor::Single { head, .. } => {
                Some((&**head, None))
            },
            TupleDescriptor::Many { head, tail, .. } => {
                Some((&**head, Some(&**tail)))
            },
        };
        TupleDescriptorIter { head_tail_opt }
    }
}

pub struct TupleDescriptorIter<'a, T> {
    head_tail_opt: Option<(&'a T, Option<&'a TupleDescriptor<T, ()>>)>,
}

impl<'a, T> Iterator for TupleDescriptorIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        let (head, tail_opt) = self.head_tail_opt?;
        let ret = head;
        self.head_tail_opt = match tail_opt {
            Some(TupleDescriptor::Unit) | None => None,
            Some(TupleDescriptor::Single { head, .. }) => Some((&**head, None)),
            Some(TupleDescriptor::Many { head, tail, .. }) => Some((&**head, Some(&**tail))),
        };
        Some(ret)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let mut tail_opt = match self.head_tail_opt {
            Some((_, tail_opt)) => tail_opt,
            None => return (0, Some(0)),
        };
        let len = {
            let mut acc = 1;
            loop {
                match tail_opt {
                    Some(TupleDescriptor::Unit) | None => break acc,
                    Some(TupleDescriptor::Single { .. }) => break acc + 1,
                    Some(TupleDescriptor::Many { tail, .. }) => {
                        acc += 1;
                        tail_opt = Some(&**tail);
                    },
                }
            }
        };
        (len, Some(len))
    }
}

impl<'a, T> ExactSizeIterator for TupleDescriptorIter<'a, T> {}
impl<'a, T> iter::FusedIterator for TupleDescriptorIter<'a, T> {}

fn tuple_descriptor_tail<E, T>(
    elem_parser: E,
) -> impl Parser<char, TupleDescriptor<T, ()>, Error = Cheap<char, Span>> + Clone
where
    T: 'static,
    E: Parser<char, T, Error = Cheap<char, Span>> + Clone + 'static,
{
    recursive(|descriptor| {
        let unit = {
            empty()
            .map(|()| TupleDescriptor::Unit)
        };
        let single = {
            elem_parser
            .clone()
            .map(|head| {
                TupleDescriptor::Single {
                    head: Box::new(head),
                    singleton_no_comma_permitted: (),
                }
            })
        };
        let many = {
            elem_parser
            .then_optional_whitespace()
            .then(comma_token())
            .then_optional_whitespace()
            .then(descriptor)
            .map(|((head, comma_token), tail)| {
                TupleDescriptor::Many {
                    head: Box::new(head),
                    comma_token,
                    tail: Box::new(tail),
                }
            })
        };

        many
        .or(single)
        .or(unit)
    })
}

pub fn tuple_descriptor<E, S, T, SingletonNoCommaPermitted>(
    elem_parser: E,
    singleton_no_comma_permitted_parser: S,
) -> impl Parser<char, TupleDescriptor<T, SingletonNoCommaPermitted>, Error = Cheap<char, Span>> + Clone
where
    T: 'static,
    E: Parser<char, T, Error = Cheap<char, Span>> + Clone + 'static,
    S: Parser<char, SingletonNoCommaPermitted, Error = Cheap<char, Span>> + Clone + 'static,
{
    let unit = {
        empty()
        .map(|()| TupleDescriptor::Unit)
    };
    let single = {
        elem_parser
        .clone()
        .then(singleton_no_comma_permitted_parser)
        .map(|(head, singleton_no_comma_permitted)| {
            TupleDescriptor::Single {
                head: Box::new(head),
                singleton_no_comma_permitted,
            }
        })
    };
    let many = {
        elem_parser
        .clone()
        .then_optional_whitespace()
        .then(comma_token())
        .then_optional_whitespace()
        .then(tuple_descriptor_tail(elem_parser))
        .map(|((head, comma_token), tail)| {
            TupleDescriptor::Many {
                head: Box::new(head),
                comma_token,
                tail: Box::new(tail),
            }
        })
    };

    many
    .or(single)
    .or(unit)
}

