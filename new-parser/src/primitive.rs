use crate::priv_prelude::*;

#[derive(Clone)]
struct FromFn<F> {
    func: F,
}

impl<T, F> Parser for FromFn<F>
where
    F: Fn(&Span) -> Result<(T, usize), ParseError>,
{
    type Output = T;

    fn parse(&self, input: &Span) -> Result<(T, usize), ParseError> {
        (self.func)(input)
    }
}

pub fn from_fn<T, F>(func: F) -> impl Parser<Output = T> + Clone
where
    F: Fn(&Span) -> Result<(T, usize), ParseError>,
    F: Clone,
{
    FromFn { func }
}

pub fn keyword(word: &'static str) -> impl Parser<Output = ()> + Clone {
    from_fn(move |input| {
        if input.as_str().starts_with(word) {
            Ok(((), word.len()))
        } else {
            let span = input.to_start();
            Err(ParseError::ExpectedKeyword { word, span })
        }
    })
}

pub fn single_char() -> impl Parser<Output = char> + Clone {
    from_fn(move |input| {
        let mut char_indices = input.as_str().char_indices();
        let c = match char_indices.next() {
            Some((_, c)) => c,
            None => {
                return Err(ParseError::UnexpectedEof {
                    span: input.to_start(),
                });
            },
        };
        let len = match char_indices.next() {
            Some((i, _)) => i,
            None => input.as_str().len(),
        };
        Ok((c, len))
    })
}

pub fn whitespace() -> impl Parser<Output = ()> + Clone {
    from_fn(move |input| {
        let mut char_indices = input.as_str().char_indices();
        let c = match char_indices.next() {
            Some((_, c)) => c,
            None => {
                return Err(ParseError::UnexpectedEof {
                    span: input.to_start(),
                });
            },
        };
        if !c.is_whitespace() {
            return Err(ParseError::ExpectedWhitespace {
                span: input.to_start(),
            });
        }
        loop {
            let (i, c) = match char_indices.next() {
                Some((i, c)) => (i, c),
                None => {
                    return Ok(((), input.as_str().len()));
                },
            };
            if !c.is_whitespace() {
                return Ok(((), i));
            }
        }
    })
}

pub fn optional_leading_whitespace<P>(parser: P) -> impl Parser<Output = P::Output> + Clone
where
    P: Parser + Clone,
{
    whitespace()
    .optional()
    .then(parser)
    .map(|(_, value)| value)
}

pub fn padded<P>(parser: P) -> impl Parser<Output = P::Output> + Clone
where
    P: Parser + Clone,
{
    optional_leading_whitespace(parser.then_optional_whitespace())
}

pub fn todo<T>() -> Todo<T> {
    Todo {
        _phantom_data: PhantomData,
    }
}

impl<T> Parser for Todo<T>
where
    T: Spanned,
{
    type Output = T;

    fn parse(&self, _input: &Span) -> Result<(T, usize), ParseError> {
        todo!()
    }
}

pub struct Todo<T> {
    _phantom_data: PhantomData<T>,
}

impl<T> Clone for Todo<T> {
    fn clone(&self) -> Todo<T> {
        Todo {
            _phantom_data: PhantomData,
        }
    }
}

pub fn lazy<'a, T, P, F>(func: F) -> Rc<dyn Parser<Output = T> + 'a>
where
    F: Fn() -> P,
    F: 'a,
    P: Parser<Output = T> + 'a,
{
    Rc::new(Lazy { func })
}

#[derive(Clone)]
pub struct Lazy<F> {
    func: F,
}

impl<P, F> Parser for Lazy<F>
where
    F: Fn() -> P,
    P: Parser,
{
    type Output = P::Output;

    fn parse(&self, input: &Span) -> Result<(P::Output, usize), ParseError> {
        let parser = (self.func)();
        parser.parse(input)
    }
}

pub fn empty() -> impl Parser<Output = ()> {
    from_fn(move |_input| {
        Ok(((), 0))
    })
}

