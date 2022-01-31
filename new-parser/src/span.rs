use crate::priv_prelude::*;

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Span {
    src: Arc<str>,
    start: usize,
    end: usize,
}

impl Span {
    pub fn new(src: Arc<str>, start: usize, end: usize) -> Span {
        Span { src, start, end }
    }

    pub fn as_str(&self) -> &str {
        &self.src[self.start .. self.end]
    }

    pub fn join(span_0: Span, span_1: Span) -> Span {
        assert!(Arc::ptr_eq(&span_0.src, &span_1.src));
        Span {
            src: span_0.src,
            start: cmp::min(span_0.start, span_1.start),
            end: cmp::max(span_0.end, span_1.end),
        }
    }

    pub fn start(&self) -> usize {
        self.start
    }

    pub fn end(&self) -> usize {
        self.end
    }

    pub fn with_range(&self, range: impl RangeBounds<usize>) -> Span {
        let start = match range.start_bound() {
            Bound::Included(index) => *index,
            Bound::Excluded(index) => index + 1,
            Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            Bound::Included(index) => index + 1,
            Bound::Excluded(index) => *index,
            Bound::Unbounded => self.end,
        };
        assert!(start <= self.end);
        assert!(end <= self.end);
        let _check = &self.as_str()[start..end];
        Span {
            src: self.src.clone(),
            start,
            end,
        }
    }

    pub fn slice(&self, range: impl RangeBounds<usize>) -> Span {
        let start = match range.start_bound() {
            Bound::Included(index) => self.start + index,
            Bound::Excluded(index) => self.start + index + 1,
            Bound::Unbounded => self.start,
        };
        let end = match range.end_bound() {
            Bound::Included(index) => self.start + index + 1,
            Bound::Excluded(index) => self.start + index,
            Bound::Unbounded => self.end,
        };
        assert!(start <= self.src.len());
        assert!(end <= self.src.len());
        Span {
            src: self.src.clone(),
            start,
            end,
        }
    }

    pub fn to_start(&self) -> Span {
        Span {
            src: self.src.clone(),
            start: self.start,
            end: self.start,
        }
    }
}

pub trait Spanned {
    fn span(&self) -> Span;
}

impl Spanned for Span {
    fn span(&self) -> Span {
        self.clone()
    }
}

impl<T> Spanned for Box<T>
where
    T: Spanned,
{
    fn span(&self) -> Span {
        (&**self).span()
    }
}

macro_rules! spanned_for_tuple (
    ($head:ident, $($tail:ident,)*) => (
        impl<$head, $($tail,)*> Spanned for ($head, $($tail,)*)
        where
            $head: Spanned,
            $($tail: Spanned,)*
        {
            fn span(&self) -> Span {
                #[allow(non_snake_case)]
                let ($head, $($tail,)*) = self;
                #[allow(unused_mut)]
                let mut span = $head.span();
                $(
                    span = Span::join(span, $tail.span());
                )*
                span
            }
        }
    );
);

spanned_for_tuple!(T0,);
spanned_for_tuple!(T0, T1,);

impl<T, E> Spanned for Result<T, E>
where
    T: Spanned,
    E: Spanned,
{
    fn span(&self) -> Span {
        match self {
            Ok(value) => value.span(),
            Err(error) => error.span(),
        }
    }
}

