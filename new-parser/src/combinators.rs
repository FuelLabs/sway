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

    fn parse(&self, input: &Span) -> Result<(U, usize), ParseError> {
        let (value0, len) = self.parser.parse(input)?;
        let value1 = (self.func)(value0);
        Ok((value1, len))
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

    fn parse(&self, input: &Span) -> Result<(U, usize), ParseError> {
        let (value0, len) = self.parser.parse(input)?;
        let span = input.slice(..len);
        let value1 = (self.func)(value0, span);
        Ok((value1, len))
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
    F: Fn(P::Output) -> Result<T, ParseError>,
{
    type Output = T;

    fn parse(&self, input: &Span) -> Result<(T, usize), ParseError> {
        let (value0, len) = self.parser.parse(input)?;
        let value1 = (self.func)(value0)?;
        Ok((value1, len))
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
    F: Fn(P::Output, Span) -> Result<T, ParseError>,
{
    type Output = T;

    fn parse(&self, input: &Span) -> Result<(T, usize), ParseError> {
        let (value0, len) = self.parser.parse(input)?;
        let span = input.slice(..len);
        let value1 = (self.func)(value0, span)?;
        Ok((value1, len))
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
    P1: Parser,
{
    type Output = (P0::Output, P1::Output);

    fn parse(&self, input: &Span) -> Result<((P0::Output, P1::Output), usize), ParseError> {
        let (value0, len0) = self.parser0.parse(input)?;
        let input = input.slice(len0..);
        let (value1, len1) = self.parser1.parse(&input)?;
        Ok(((value0, value1), len0 + len1))
    }
}

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

    fn parse(&self, input: &Span) -> Result<(Option<P::Output>, usize), ParseError> {
        match self.parser.parse(input) {
            Ok((value, len)) => Ok((Some(value), len)),
            Err(_error) => Ok((None, 0)),
        }
    }
}

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

    fn parse(&self, input: &Span) -> Result<(P::Output, usize), ParseError> {
        (&self.parser)
        .then(whitespace().optional())
        .map(|(value, _opt)| value)
        .parse(input)
    }
}

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

    fn parse(&self, input: &Span) -> Result<(P0::Output, usize), ParseError> {
        match self.parser0.parse(input) {
            Ok((value, len)) => Ok((value, len)),
            Err(error0) => match self.parser1.parse(input) {
                Ok((value, len)) => Ok((value, len)),
                Err(error1) => {
                    Err(ParseError::Or {
                        error0: Box::new(error0),
                        error1: Box::new(error1),
                    })
                },
            },
        }
    }
}

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

    fn parse(&self, input: &Span) -> Result<(P::Output, usize), ParseError> {
        (&self.parser)
        .then(whitespace())
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

    fn parse(&self, input: &Span) -> Result<(Vec<P::Output>, usize), ParseError> {
        let mut values = Vec::new();
        let mut total_len = 0;
        let mut remaining_input = input.clone();
        loop {
            match self.parser.parse(&remaining_input) {
                Ok((value, len)) => {
                    remaining_input = remaining_input.slice(len..);
                    total_len += len;
                    values.push(value);
                },
                Err(..) => break,
            }
        }
        Ok((values, total_len))
    }
}

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
    P1: Parser,
{
    type Output = P1::Output;

    fn parse(&self, input: &Span) -> Result<(P1::Output, usize), ParseError> {
        let (value0, len0) = self.parser.parse(input)?;
        let input = input.slice(len0..);
        let parser1 = (self.func)(value0);
        let (value1, len1) = parser1.parse(&input)?;
        Ok((value1, len0 + len1))
    }
}

