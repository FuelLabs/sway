use crate::priv_prelude::*;

#[derive(Clone)]
struct FromFn<F> {
    func: F,
}

impl<T, E, R, F> Parser for FromFn<F>
where
    F: Fn(&Span) -> Result<(T, usize), Result<E, R>>,
{
    type Output = T;
    type Error = E;
    type FatalError = R;

    fn parse(&self, input: &Span) -> Result<(T, usize), Result<E, R>> {
        (self.func)(input)
    }
}

pub fn from_fn<T, E, R, F>(func: F) -> impl Parser<Output = T, Error = E, FatalError = R> + Clone
where
    F: Fn(&Span) -> Result<(T, usize), Result<E, R>>,
    F: Clone,
{
    FromFn { func }
}

#[derive(Clone)]
pub struct ExpectedKeywordError {
    pub position: usize,
    pub word: &'static str,
}

pub fn keyword<R>(word: &'static str) -> impl Parser<Output = (), Error = ExpectedKeywordError, FatalError = R> + Clone {
    from_fn(move |input| {
        if input.as_str().starts_with(word) {
            Ok(((), word.len()))
        } else {
            let error = ExpectedKeywordError {
                position: input.start(),
                word,
            };
            Err(Ok(error))
        }
    })
}

pub struct UnexpectedEofError;

pub fn single_char<R>() -> impl Parser<Output = char, Error = UnexpectedEofError, FatalError = R> + Clone {
    from_fn(move |input| {
        let mut char_indices = input.as_str().char_indices();
        let c = match char_indices.next() {
            Some((_, c)) => c,
            None => {
                let error = UnexpectedEofError;
                return Err(Ok(error));
            },
        };
        let len = match char_indices.next() {
            Some((i, _)) => i,
            None => input.as_str().len(),
        };
        Ok((c, len))
    })
}

pub struct ExpectedLineCommentError {
    pub position: usize,
}

pub fn line_comment<R>() -> impl Parser<Output = (), Error = ExpectedLineCommentError, FatalError = R> + Clone {
    from_fn(|input| {
        if !input.as_str().starts_with("//") {
            let error = ExpectedLineCommentError {
                position: input.span().start(),
            };
            return Err(Ok(error));
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

pub struct ExpectedMultilineCommentError {
    pub position: usize,
}

#[derive(Clone)]
pub struct UnclosedMultilineCommentError {
    pub start_position: usize,
}

pub fn multiline_comment()
    -> impl Parser<
        Output = (),
        Error = ExpectedMultilineCommentError,
        FatalError = UnclosedMultilineCommentError,
    > + Clone
{
    from_fn(|input| {
        if !input.as_str().starts_with("/*") {
            let error = ExpectedMultilineCommentError {
                position: input.span().start(),
            };
            return Err(Ok(error));
        }
        let mut char_indices = input.as_str().char_indices().skip(2).peekable();
        let mut depth = 1;
        let len = loop {
            let c = match char_indices.next() {
                Some((_, c)) => c,
                None => {
                    let error = UnclosedMultilineCommentError {
                        start_position: input.clone().start(),
                    };
                    return Err(Err(error));
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
        Ok(((), len))
    })
}

pub struct ExpectedSingleWhitespaceCharError {
    pub position: usize,
}

pub fn single_whitespace_char<R>()
    -> impl Parser<Output = (), Error = ExpectedSingleWhitespaceCharError, FatalError = R> + Clone
{
    single_char()
    .map_err_with_span(|UnexpectedEofError, span: Span| {
        ExpectedSingleWhitespaceCharError { position: span.end() }
    })
    .try_map_with_span(|c: char, span: Span| {
        if c.is_whitespace() {
            Ok(())
        } else {
            return Err(Ok(ExpectedSingleWhitespaceCharError {
                position: span.start()
            }));
        }
    })
}

#[derive(Clone)]
pub struct ExpectedWhitespaceError {
    pub position: usize,
}

pub fn whitespace()
    -> impl Parser<
        Output = (),
        Error = ExpectedWhitespaceError,
        FatalError = UnclosedMultilineCommentError,
    > + Clone
{
    let single_whitespace_char = {
        single_whitespace_char()
        .map_err(|ExpectedSingleWhitespaceCharError { .. }| ())
    };
    let line_comment = {
        line_comment()
        .map_err(|ExpectedLineCommentError { .. }| ())
    };
    let multiline_comment = {
        multiline_comment()
        .map_err(|ExpectedMultilineCommentError { .. }| ())
    };
    let any_whitespace = {
        or! {
            single_whitespace_char,
            line_comment,
            multiline_comment,
        }
        .map_err(|((), (), ())| ())
    };
    any_whitespace
    .clone()
    .map_err_with_span(|(), span| ExpectedWhitespaceError { position: span.start() })
    .then(any_whitespace.repeated())
    .map(|((), _vec)| ())
}

/*
pub enum WithWhitespaceError<E> {
    Whitespace(WhitespaceError),
    Parser(E),
}

pub fn leading_whitespace<P>(parser: P) -> impl Parser<Output = P::Output, Error = WithWhitespaceError<P::Error>> + Clone
where
    P: Parser + Clone,
{
    whitespace()
    .map_err(WithWhitespaceError::Whitespace)
    .then(
        parser
        .map_err(WithWhitespaceError::Parser)
    )
    .map(|((), value)| value)
}
*/

#[derive(Clone)]
pub enum PaddedFatalError<R> {
    UnclosedMultilineComment(UnclosedMultilineCommentError),
    Inner(R),
}

pub fn optional_leading_whitespace<P>(parser: P)
    -> impl Parser<Output = P::Output, Error = P::Error, FatalError = PaddedFatalError<P::FatalError>> + Clone
where
    P: Parser + Clone,
    P::Error: Clone,
{
    whitespace()
    .map_err(|ExpectedWhitespaceError { .. }| ())
    .optional()
    .map_fatal_err(PaddedFatalError::UnclosedMultilineComment)
    .then(
        parser
        .map_fatal_err(PaddedFatalError::Inner)
    )
    .map(|(_opt, value)| value)
}

pub fn padded<P>(parser: P)
    -> impl Parser<Output = P::Output, Error = P::Error, FatalError = PaddedFatalError<P::FatalError>> + Clone
where
    P: Parser + Clone,
    P::Error: Clone,
{
    optional_leading_whitespace(parser.then_optional_whitespace())
    .map_fatal_err(PaddedFatalError::flatten)
}

impl<R> PaddedFatalError<PaddedFatalError<R>> {
    pub fn flatten(self) -> PaddedFatalError<R> {
        match self {
            PaddedFatalError::UnclosedMultilineComment(error) => {
                PaddedFatalError::UnclosedMultilineComment(error)
            },
            PaddedFatalError::Inner(error) => error,
        }
    }
}

impl PaddedFatalError<Infallible> {
    pub fn to_unclosed_multiline_comment_error(self) -> UnclosedMultilineCommentError {
        match self {
            PaddedFatalError::UnclosedMultilineComment(error) => error,
            PaddedFatalError::Inner(infallible) => match infallible {},
        }
    }
}

pub struct Todo<T, E, R> {
    _phantom_ok: PhantomData<T>,
    _phantom_err: PhantomData<E>,
    _phantom_fatal_err: PhantomData<R>,
}

pub fn todo<T, E, R>() -> Todo<T, E, R> {
    Todo {
        _phantom_ok: PhantomData,
        _phantom_err: PhantomData,
        _phantom_fatal_err: PhantomData,
    }
}

impl<T, E, R> Parser for Todo<T, E, R> {
    type Output = T;
    type Error = E;
    type FatalError = R;

    fn parse(&self, _input: &Span) -> Result<(T, usize), Result<E, R>> {
        todo!()
    }
}

impl<T, E, R> Clone for Todo<T, E, R> {
    fn clone(&self) -> Todo<T, E, R> {
        Todo {
            _phantom_ok: PhantomData,
            _phantom_err: PhantomData,
            _phantom_fatal_err: PhantomData,
        }
    }
}

pub fn lazy<'a, T, E, R, P, F>(func: F) -> Rc<dyn Parser<Output = T, Error = E, FatalError = R> + 'a>
where
    F: Fn() -> P,
    F: 'a,
    P: Parser<Output = T, Error = E, FatalError = R> + 'a,
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
    type Error = P::Error;
    type FatalError = P::FatalError;

    fn parse(&self, input: &Span) -> Result<(P::Output, usize), Result<P::Error, P::FatalError>> {
        let parser = (self.func)();
        parser.parse(input)
    }
}

/*
/*
pub fn empty() -> impl Parser<Output = (), Error = Infallible> + Clone {
    from_fn(move |_input| {
        (false, Ok(((), 0)))
    })
}

pub struct ExpectedEofError {
    pub position: usize,
}

pub fn eof() -> impl Parser<Output = (), Error = ExpectedEofError> + Clone {
    from_fn(|input| {
        if input.as_str().is_empty() {
            (false, Ok(((), 0)))
        } else {
            (false, Err(ExpectedEofError { position: input.start() }))
        }
    })
}
*/
*/

/*
#[macro_export]
macro_rules! __or_inner {
    ($parsers:ident, $input:ident, ($($head_pats:pat,)*), ()) => {
        Err(Ok(()))
    };
    //($parsers:ident, $input:ident, ($($head_pats:pat,)*), (_, $($tail_pats:pat,)*)) => {{
    ($parsers:ident, $input:ident, ($($head_pats:pat,)*), ($ignore:pat, $($tail_pats:pat,)*)) => {{
        let ($($head_pats,)* this_parser, $($tail_pats,)*) = &$parsers;
        let res = Parser::parse(&this_parser, $input);
        match res {
            Ok((value, len)) => Ok((value, len)),
            Err(Ok(())) => __or_inner!($parsers, $input, (_, $($head_pats,)*), ($($tail_pats,)*)),
            Err(Err(error)) => Err(Err(error)),
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
*/


