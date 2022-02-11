use crate::priv_prelude::*;

#[derive(Clone)]
struct FromFn<F> {
    func: F,
}

impl<T, F> Parser for FromFn<F>
where
    F: Fn(&Span) -> (bool, Result<(T, usize), ParseError>),
{
    type Output = T;

    fn parse(&self, input: &Span) -> (bool, Result<(T, usize), ParseError>) {
        (self.func)(input)
    }
}

pub fn from_fn<T, F>(func: F) -> impl Parser<Output = T> + Clone
where
    F: Fn(&Span) -> (bool, Result<(T, usize), ParseError>),
    F: Clone,
{
    FromFn { func }
}

pub fn keyword(word: &'static str) -> impl Parser<Output = ()> + Clone {
    from_fn(move |input| {
        if input.as_str().starts_with(word) {
            (false, Ok(((), word.len())))
        } else {
            let span = input.to_start();
            (false, Err(ParseError::ExpectedKeyword { word, span }))
        }
    })
}

pub fn single_char() -> impl Parser<Output = char> + Clone {
    from_fn(move |input| {
        let mut char_indices = input.as_str().char_indices();
        let c = match char_indices.next() {
            Some((_, c)) => c,
            None => {
                let error = ParseError::UnexpectedEof {
                    span: input.to_start(),
                };
                return (false, Err(error));
            },
        };
        let len = match char_indices.next() {
            Some((i, _)) => i,
            None => input.as_str().len(),
        };
        (false, Ok((c, len)))
    })
}

pub fn line_comment() -> impl Parser<Output = ()> + Clone {
    from_fn(|input| {
        if !input.as_str().starts_with("//") {
            let error = ParseError::ExpectedWhitespace {
                span: input.span(),
            };
            return (false, Err(error));
        };
        let mut char_indices = input.as_str().char_indices().skip(2);
        let len = loop {
            let c = match char_indices.next() {
                Some((_, c)) => c,
                None => break input.as_str().len(),
            };
            if c == '\n' {
                break match char_indices.next() {
                    Some((i, _)) => i,
                    None => input.as_str().len(),
                };
            }
        };
        (false, Ok(((), len)))
    })
}

pub fn multiline_comment() -> impl Parser<Output = ()> + Clone {
    from_fn(|input| {
        if !input.as_str().starts_with("/*") {
            let error = ParseError::ExpectedWhitespace {
                span: input.span(),
            };
            return (false, Err(error));
        }
        let mut char_indices = input.as_str().char_indices().skip(2).peekable();
        let mut depth = 1;
        let len = loop {
            let c = match char_indices.next() {
                Some((_, c)) => c,
                None => {
                    let error = ParseError::UnclosedMultilineComment {
                        span: input.clone(),
                    };
                    return (false, Err(error));
                },
            };
            match c {
                '/' => {
                    if let Some((_, '*')) = char_indices.peek() {
                        let _ = char_indices.next();
                        depth += 1;
                    }
                },
                '*' => {
                    if let Some((_, '/')) = char_indices.peek() {
                        let _ = char_indices.next();
                        depth -= 1;
                        if depth == 0 {
                            break match char_indices.next() {
                                Some((i, _)) => i,
                                None => input.as_str().len(),
                            };
                        }
                    }
                },
                _ => (),
            }
        };
        (false, Ok(((), len)))
    })
}

pub fn single_whitespace_char() -> impl Parser<Output = ()> + Clone {
    single_char()
    .try_map_with_span(|c: char, span: Span| {
        if c.is_whitespace() {
            Ok(())
        } else {
            return Err(ParseError::ExpectedWhitespace {
                span: span.to_start(),
            });
        }
    })
}

pub fn whitespace() -> impl Parser<Output = ()> + Clone {
    let any_whitespace = {
        single_whitespace_char()
        .or(line_comment())
        .or(multiline_comment())
    };
    any_whitespace
    .clone()
    .then(any_whitespace.repeated())
    .map(|(_, _vec)| ())
}

pub fn leading_whitespace<P>(parser: P) -> impl Parser<Output = P::Output> + Clone
where
    P: Parser + Clone,
{
    whitespace()
    .then(parser)
    .map(|((), value)| value)
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

impl<T> Parser for Todo<T> {
    type Output = T;

    fn parse(&self, _input: &Span) -> (bool, Result<(T, usize), ParseError>) {
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

    fn parse(&self, input: &Span) -> (bool, Result<(P::Output, usize), ParseError>) {
        let parser = (self.func)();
        parser.parse(input)
    }
}

pub fn empty() -> impl Parser<Output = ()> + Clone {
    from_fn(move |_input| {
        (false, Ok(((), 0)))
    })
}

pub fn eof() -> impl Parser<Output = ()> + Clone {
    from_fn(|input| {
        if input.as_str().is_empty() {
            (false, Ok(((), 0)))
        } else {
            (false, Err(ParseError::ExpectedEof { span: input.to_start() }))
        }
    })
}

#[macro_export]
macro_rules! __or_inner {
    ($parsers:ident, $input:ident, ($($head_pats:pat,)*), ()) => {
        (false, Ok((None, 0)))
    };
    //($parsers:ident, $input:ident, ($($head_pats:pat,)*), (_, $($tail_pats:pat,)*)) => {{
    ($parsers:ident, $input:ident, ($($head_pats:pat,)*), ($ignore:pat, $($tail_pats:pat,)*)) => {{
        let ($($head_pats,)* this_parser, $($tail_pats,)*) = &$parsers;
        let (commited, res) = Parser::parse(&this_parser, $input);
        match res {
            Ok((value, len)) => (commited, Ok((Some(value), len))),
            Err(err) => {
                if commited {
                    (true, Err(err))
                } else {
                    __or_inner!($parsers, $input, (_, $($head_pats,)*), ($($tail_pats,)*))
                }
            },
        }
    }};
}

#[macro_export]
macro_rules! __or_build_pattern {
    ($parsers:ident, $input:ident, (), ($($tail_pats:pat,)*)) => {
        __or_inner!($parsers, $input, (), ($($tail_pats,)*))
    };
    ($parsers:ident, $input:ident, ($head:expr, $($tail:expr,)*), ($($tail_pats:pat,)*)) => {
        __or_build_pattern!($parsers, $input, ($($tail,)*), (_, $($tail_pats,)*))
    };
}

#[macro_export]
macro_rules! or {
    ($($parser:expr),* $(,)?) => {{
        #[allow(unused_variables)]
        let parsers = ($($parser,)*);
        from_fn(move |input| {
            let _ = input;
            __or_build_pattern!(parsers, input, ($($parser,)*), ())
        })
    }};
}

