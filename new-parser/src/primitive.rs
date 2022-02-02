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

pub fn line_comment() -> impl Parser<Output = ()> + Clone {
    from_fn(|input| {
        if !input.as_str().starts_with("//") {
            return Err(ParseError::ExpectedWhitespace {
                span: input.span(),
            });
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
        Ok(((), len))
    })
}

pub fn multiline_comment() -> impl Parser<Output = ()> + Clone {
    from_fn(|input| {
        if !input.as_str().starts_with("/*") {
            return Err(ParseError::ExpectedWhitespace {
                span: input.span(),
            });
        }
        let mut char_indices = input.as_str().char_indices().skip(2).peekable();
        let mut depth = 1;
        let len = loop {
            let c = match char_indices.next() {
                Some((_, c)) => c,
                None => return Err(ParseError::UnclosedMultilineComment {
                    span: input.clone(),
                }),
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
        Ok(((), len))
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

pub fn empty() -> impl Parser<Output = ()> + Clone {
    from_fn(move |_input| {
        Ok(((), 0))
    })
}

pub fn eof() -> impl Parser<Output = ()> + Clone {
    from_fn(|input| {
        if input.as_str().is_empty() {
            Ok(((), 0))
        } else {
            Err(ParseError::ExpectedEof { span: input.to_start() })
        }
    })
}

pub fn newline() -> impl Parser<Output = ()> + Clone {
    from_fn(|input| {
        if input.as_str().starts_with("\n") {
            Ok(((), 1))
        } else {
            Err(ParseError::ExpectedNewline { span: input.to_start() })
        }
    })
}

