use crate::priv_prelude::*;

#[derive(Clone)]
pub struct Map<P, F> {
    parser: P,
    func: F,
}

impl<P, F> Map<P, F> {
    pub fn new(parser: P, func: F) -> Map<P, F> {
        Map { parser, func }
    }
}

impl<T, U, P, F> Parser for Map<P, F>
where
    P: Parser<Output = T>,
    F: Fn(T) -> U,
{
    type Output = U;
    type Error = P::Error;

    fn parse(&self, input: &Span) -> (bool, Result<(U, usize), P::Error>) {
        let (commited, res) = self.parser.parse(input);
        let (value0, len) = match res {
            Ok((value0, len)) => (value0, len),
            Err(error) => return (commited, Err(error)),
        };
        let value1 = (self.func)(value0);
        (commited, Ok((value1, len)))
    }
}

#[derive(Clone)]
pub struct MapWithSpan<P, F> {
    parser: P,
    func: F,
}

impl<P, F> MapWithSpan<P, F> {
    pub fn new(parser: P, func: F) -> MapWithSpan<P, F> {
        MapWithSpan { parser, func }
    }
}

impl<T, U, P, F> Parser for MapWithSpan<P, F>
where
    P: Parser<Output = T>,
    F: Fn(T, Span) -> U,
{
    type Output = U;
    type Error = P::Error;

    fn parse(&self, input: &Span) -> (bool, Result<(U, usize), P::Error>) {
        let (commited, res) = self.parser.parse(input);
        let (value0, len) = match res {
            Ok((value0, len)) => (value0, len),
            Err(error) => return (commited, Err(error)),
        };
        let span = input.slice(..len);
        let value1 = (self.func)(value0, span);
        (commited, Ok((value1, len)))
    }
}

#[derive(Clone)]
pub struct TryMap<P, F> {
    parser: P,
    func: F,
}

impl<P, F> TryMap<P, F> {
    pub fn new(parser: P, func: F) -> TryMap<P, F> {
        TryMap { parser, func }
    }
}

impl<P, T, F> Parser for TryMap<P, F>
where
    P: Parser,
    F: Fn(P::Output) -> Result<T, P::Error>,
{
    type Output = T;
    type Error = P::Error;

    fn parse(&self, input: &Span) -> (bool, Result<(T, usize), P::Error>) {
        let (commited, res) = self.parser.parse(input);
        let (value0, len) = match res {
            Ok((value0, len)) => (value0, len),
            Err(error) => return (commited, Err(error)),
        };
        let value1 = match (self.func)(value0) {
            Ok(value1) => value1,
            Err(error) => return (commited, Err(error)),
        };
        (commited, Ok((value1, len)))
    }
}

#[derive(Clone)]
pub struct TryMapWithSpan<P, F> {
    parser: P,
    func: F,
}

impl<P, F> TryMapWithSpan<P, F> {
    pub fn new(parser: P, func: F) -> TryMapWithSpan<P, F> {
        TryMapWithSpan { parser, func }
    }
}

impl<P, T, F> Parser for TryMapWithSpan<P, F>
where
    P: Parser,
    F: Fn(P::Output, Span) -> Result<T, P::Error>,
{
    type Output = T;
    type Error = P::Error;

    fn parse(&self, input: &Span) -> (bool, Result<(T, usize), P::Error>) {
        let (commited, res) = self.parser.parse(input);
        let (value0, len) = match res {
            Ok((value0, len)) => (value0, len),
            Err(error) => return (commited, Err(error)),
        };
        let span = input.slice(..len);
        let value1 = match (self.func)(value0, span) {
            Ok(value1) => value1,
            Err(error) => return (commited, Err(error)),
        };
        (commited, Ok((value1, len)))
    }
}

#[derive(Clone)]
pub struct Then<P0, P1> {
    parser0: P0,
    parser1: P1,
}

impl<P0, P1> Then<P0, P1> {
    pub fn new(parser0: P0, parser1: P1) -> Then<P0, P1> {
        Then { parser0, parser1 }
    }
}

impl<P0, P1> Parser for Then<P0, P1>
where
    P0: Parser,
    P1: Parser<Error = P0::Error>,
{
    type Output = (P0::Output, P1::Output);
    type Error = P0::Error;

    fn parse(&self, input: &Span) -> (bool, Result<((P0::Output, P1::Output), usize), P0::Error>) {
        let (commited0, res0) = self.parser0.parse(input);
        let (value0, len0) = match res0 {
            Ok((value0, len0)) => (value0, len0),
            Err(error) => return (commited0, Err(error)),
        };
        let input = input.slice(len0..);
        let (commited1, res1) = self.parser1.parse(&input);
        let commited = commited0 || commited1;
        let (value1, len1) = match res1 {
            Ok((value1, len1)) => (value1, len1),
            Err(error) => return (commited, Err(error)),
        };
        (commited, Ok(((value0, value1), len0 + len1)))
    }
}

/*
#[derive(Clone)]
pub struct Optional<P> {
    parser: P,
}

impl<P> Optional<P> {
    pub fn new(parser: P) -> Optional<P> {
        Optional { parser }
    }
}

impl<P> Parser for Optional<P>
where
    P: Parser,
{
    type Output = Option<P::Output>;

    fn parse(&self, input: &Span) -> (bool, Result<(Option<P::Output>, usize), ParseError>) {
        let (commited, res) = self.parser.parse(input);
        match res {
            Ok((value, len)) => (commited, Ok((Some(value), len))),
            Err(error) => {
                if commited {
                    (true, Err(error))
                } else {
                    (false, Ok((None, 0)))
                }
            },
        }
    }
}
*/

#[derive(Clone)]
pub struct ThenOptionalWhitespace<P> {
    parser: P,
}

impl<P> ThenOptionalWhitespace<P> {
    pub fn new(parser: P) -> ThenOptionalWhitespace<P> {
        ThenOptionalWhitespace { parser }
    }
}

impl<P> Parser for ThenOptionalWhitespace<P>
where
    P: Parser,
{
    type Output = P::Output;
    type Error = PaddedError<P::Error>;

    fn parse(&self, input: &Span) -> (bool, Result<(P::Output, usize), PaddedError<P::Error>>) {
        (&self.parser)
        .map_err(PaddedError::Parser)
        .then(
            whitespace()
            .or_else(|error| match error {
                WhitespaceError::ExpectedWhitespaceOrComment { .. } => Ok(((), 0)),
                WhitespaceError::UnclosedMultilineComment { start_position } => {
                    Err(PaddedError::UnclosedMultilineComment { start_position })
                },
            })
        )
        .map(|(value, _opt)| value)
        .parse(input)
    }
}

/*
#[derive(Clone)]
pub struct Or<P0, P1> {
    parser0: P0,
    parser1: P1,
}

impl<P0, P1> Or<P0, P1> {
    pub fn new(parser0: P0, parser1: P1) -> Or<P0, P1> {
        Or { parser0, parser1 }
    }
}

impl<P0, P1> Parser for Or<P0, P1>
where
    P0: Parser,
    P1: Parser<Output = P0::Output>,
{
    type Output = P0::Output;

    fn parse(&self, input: &Span) -> (bool, Result<(P0::Output, usize), ParseError>) {
        let (commited, res) = self.parser0.parse(input);
        match res {
            Ok((value, len)) => (commited, Ok((value, len))),
            Err(error0) => {
                if commited {
                    (true, Err(error0))
                } else {
                    let (commited, res) = self.parser1.parse(input);
                    match res {
                        Ok((value, len)) => (commited, Ok((value, len))),
                        Err(error1) => {
                            if commited {
                                (true, Err(error1))
                            } else {
                                (false, Err(ParseError::Or {
                                    error0: Box::new(error0),
                                    error1: Box::new(error1),
                                }))
                            }
                        },
                    }
                }
            },
        }
    }
}
*/

#[derive(Clone)]
pub struct ThenWhitespace<P> {
    parser: P,
}

impl<P> ThenWhitespace<P> {
    pub fn new(parser: P) -> ThenWhitespace<P> {
        ThenWhitespace { parser }
    }
}

impl<P> Parser for ThenWhitespace<P>
where
    P: Parser,
{
    type Output = P::Output;
    type Error = WithWhitespaceError<P::Error>;

    fn parse(&self, input: &Span) -> (bool, Result<(P::Output, usize), WithWhitespaceError<P::Error>>) {
        (&self.parser)
        .map_err(WithWhitespaceError::Parser)
        .then(whitespace().map_err(WithWhitespaceError::Whitespace))
        .map(|(value, ())| value)
        .parse(input)
    }
}

#[derive(Clone)]
pub struct Repeated<P> {
    parser: P,
}

impl<P> Repeated<P> {
    pub fn new(parser: P) -> Repeated<P> {
        Repeated { parser }
    }
}

impl<P> Parser for Repeated<P>
where
    P: Parser,
{
    type Output = Vec<P::Output>;
    type Error = P::Error;

    fn parse(&self, input: &Span) -> (bool, Result<(Vec<P::Output>, usize), P::Error>) {
        let mut any_commited = false;
        let mut values = Vec::new();
        let mut total_len = 0;
        let mut remaining_input = input.clone();
        loop {
            let (commited, res) = self.parser.parse(&remaining_input);
            match res {
                Ok((value, len)) => {
                    remaining_input = remaining_input.slice(len..);
                    any_commited |= commited;
                    total_len += len;
                    values.push(value);
                },
                Err(error) => {
                    if commited {
                        return (true, Err(error));
                    } else {
                        return (any_commited, Ok((values, total_len)));
                    }
                },
            }
        }
    }
}

/*
#[derive(Clone)]
pub struct WhileSome<P> {
    parser: P,
}

impl<P> WhileSome<P> {
    pub fn new(parser: P) -> WhileSome<P> {
        WhileSome { parser }
    }
}

impl<P, T> Parser for WhileSome<P>
where
    P: Parser<Output = Option<T>>,
{
    type Output = Vec<T>;

    fn parse(&self, input: &Span) -> (bool, Result<(Vec<T>, usize), ParseError>) {
        let mut any_commited = false;
        let mut values = Vec::new();
        let mut total_len = 0;
        let mut remaining_input = input.clone();
        loop {
            let (commited, res) = self.parser.parse(&remaining_input);
            any_commited |= commited;
            match res {
                Ok((Some(value), len)) => {
                    remaining_input = remaining_input.slice(len..);
                    total_len += len;
                    values.push(value);
                },
                Ok((None, _)) => {
                    break;
                },
                Err(error) => return (any_commited, Err(error)),
            }
        }
        (any_commited, Ok((values, total_len)))
    }
}
*/

#[derive(Clone)]
pub struct AndThen<P, F> {
    parser: P,
    func: F,
}

impl<P, F> AndThen<P, F> {
    pub fn new(parser: P, func: F) -> AndThen<P, F> {
        AndThen { parser, func }
    }
}

impl<P0, P1, F> Parser for AndThen<P0, F>
where
    P0: Parser,
    F: Fn(P0::Output) -> P1,
    P1: Parser<Error = P0::Error>,
{
    type Output = P1::Output;
    type Error = P0::Error;

    fn parse(&self, input: &Span) -> (bool, Result<(P1::Output, usize), P0::Error>) {
        let (commited0, res) = self.parser.parse(input);
        let (value0, len0) = match res {
            Ok((value0, len0)) => (value0, len0),
            Err(error) => return (commited0, Err(error)),
        };
        let input = input.slice(len0..);
        let parser1 = (self.func)(value0);
        let (commited1, res) = parser1.parse(&input);
        let commited = commited0 || commited1;
        let (value1, len1) = match res {
            Ok((value1, len1)) => (value1, len1),
            Err(error) => return (commited, Err(error)),
        };
        (commited, Ok((value1, len0 + len1)))
    }
}

#[derive(Clone)]
pub struct Debug<P> {
    parser: P,
    text: &'static str,
}

impl<P> Debug<P> {
    pub fn new(parser: P, text: &'static str) -> Debug<P> {
        Debug { parser, text }
    }
}

impl<P> Parser for Debug<P>
where
    P: Parser,
{
    type Output = P::Output;
    type Error = P::Error;

    fn parse(&self, input: &Span) -> (bool, Result<(P::Output, usize), P::Error>) {
        let (commited, res) = self.parser.parse(input);
        match res {
            Ok(value) => {
                println!("debug: {}", self.text);
                (commited, Ok(value))
            },
            Err(error) => (commited, Err(error)),
        }
    }
}

#[derive(Clone)]
pub struct Commit<P> {
    parser: P,
}

impl<P> Commit<P> {
    pub fn new(parser: P) -> Commit<P> {
        Commit { parser }
    }
}

impl<P> Parser for Commit<P>
where
    P: Parser,
{
    type Output = P::Output;
    type Error = P::Error;

    fn parse(&self, input: &Span) -> (bool, Result<(P::Output, usize), P::Error>) {
        let (commited, res) = self.parser.parse(input);
        match res {
            Ok((value, len)) => (true, Ok((value, len))),
            Err(error) => (commited, Err(error)),
        }
    }
}

#[derive(Clone)]
pub struct Uncommit<P> {
    parser: P,
}

impl<P> Uncommit<P> {
    pub fn new(parser: P) -> Uncommit<P> {
        Uncommit { parser }
    }
}

impl<P> Parser for Uncommit<P>
where
    P: Parser,
{
    type Output = P::Output;
    type Error = P::Error;

    fn parse(&self, input: &Span) -> (bool, Result<(P::Output, usize), P::Error>) {
        let (commited, res) = self.parser.parse(input);
        match res {
            Ok((value, len)) => (false, Ok((value, len))),
            Err(error) => (commited, Err(error)),
        }
    }
}

#[derive(Clone)]
pub struct MapErrWithSpan<P, F> {
    parser: P,
    func: F,
}

impl<P, F> MapErrWithSpan<P, F> {
    pub fn new(parser: P, func: F) -> MapErrWithSpan<P, F> {
        MapErrWithSpan { parser, func }
    }
}

impl<P, F, E> Parser for MapErrWithSpan<P, F>
where
    P: Parser,
    F: Fn(P::Error, Span) -> E,
{
    type Output = P::Output;
    type Error = E;

    fn parse(&self, input: &Span) -> (bool, Result<(P::Output, usize), E>) {
        let (commited, res) = self.parser.parse(input);
        match res {
            Ok(value) => (commited, Ok(value)),
            Err(error) => (commited, Err((self.func)(error, input.clone()))),
        }
    }
}

#[derive(Clone)]
pub struct MapErr<P, F> {
    parser: P,
    func: F,
}

impl<P, F> MapErr<P, F> {
    pub fn new(parser: P, func: F) -> MapErr<P, F> {
        MapErr { parser, func }
    }
}

impl<P, F, E> Parser for MapErr<P, F>
where
    P: Parser,
    F: Fn(P::Error) -> E,
{
    type Output = P::Output;
    type Error = E;

    fn parse(&self, input: &Span) -> (bool, Result<(P::Output, usize), E>) {
        let (commited, res) = self.parser.parse(input);
        match res {
            Ok(value) => (commited, Ok(value)),
            Err(error) => (commited, Err((self.func)(error))),
        }
    }
}

#[derive(Clone)]
pub struct OrElse<P, F> {
    parser: P,
    func: F,
}

impl<P, F> OrElse<P, F> {
    pub fn new(parser: P, func: F) -> OrElse<P, F> {
        OrElse { parser, func }
    }
}

impl<P, F, E> Parser for OrElse<P, F>
where
    P: Parser,
    F: Fn(P::Error) -> Result<(P::Output, usize), E>,
{
    type Output = P::Output;
    type Error = E;

    fn parse(&self, input: &Span) -> (bool, Result<(P::Output, usize), E>) {
        let (commited, res) = self.parser.parse(input);
        match res {
            Ok(value) => (commited, Ok(value)),
            Err(error) => (commited, (self.func)(error)),
        }
    }
}

